// Sync scheduler module for managing content synchronization
use crate::error::{Result, XTauriError};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Synchronization status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    Pending,
    Syncing,
    Completed,
    Failed,
    Partial,
}

impl SyncStatus {
    /// Convert to database string representation
    pub fn to_db_string(&self) -> &'static str {
        match self {
            SyncStatus::Pending => "pending",
            SyncStatus::Syncing => "syncing",
            SyncStatus::Completed => "completed",
            SyncStatus::Failed => "failed",
            SyncStatus::Partial => "partial",
        }
    }
    
    /// Parse from database string
    pub fn from_db_string(s: &str) -> Self {
        match s {
            "pending" => SyncStatus::Pending,
            "syncing" => SyncStatus::Syncing,
            "completed" => SyncStatus::Completed,
            "failed" => SyncStatus::Failed,
            "partial" => SyncStatus::Partial,
            _ => SyncStatus::Pending,
        }
    }
}

/// Synchronization progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub status: SyncStatus,
    pub progress: u8, // 0-100
    pub current_step: String,
    pub channels_synced: usize,
    pub movies_synced: usize,
    pub series_synced: usize,
    pub errors: Vec<String>,
}

impl Default for SyncProgress {
    fn default() -> Self {
        Self {
            status: SyncStatus::Pending,
            progress: 0,
            current_step: String::new(),
            channels_synced: 0,
            movies_synced: 0,
            series_synced: 0,
            errors: Vec::new(),
        }
    }
}

/// Synchronization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub auto_sync_enabled: bool,
    pub sync_interval_hours: u32,
    pub wifi_only: bool,
    pub notify_on_complete: bool,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            auto_sync_enabled: true,
            sync_interval_hours: 24,
            wifi_only: true,
            notify_on_complete: false,
        }
    }
}

/// Handle for managing an active sync operation
pub struct SyncHandle {
    pub profile_id: String,
    pub cancel_token: CancellationToken,
    pub progress_rx: mpsc::Receiver<SyncProgress>,
}

impl SyncHandle {
    /// Create a new sync handle
    pub fn new(profile_id: String) -> (Self, mpsc::Sender<SyncProgress>, CancellationToken) {
        let (progress_tx, progress_rx) = mpsc::channel(100);
        let cancel_token = CancellationToken::new();
        
        let handle = Self {
            profile_id,
            cancel_token: cancel_token.clone(),
            progress_rx,
        };
        
        (handle, progress_tx, cancel_token)
    }
    
    /// Check if sync has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }
    
    /// Cancel the sync operation
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
}

/// Manages synchronization scheduling and execution
pub struct SyncScheduler {
    db: Arc<Mutex<Connection>>,
    active_syncs: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

/// Configuration for retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

impl SyncScheduler {
    /// Create a new SyncScheduler
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self {
            db,
            active_syncs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get sync status for a profile from the database
    pub fn get_sync_status(&self, profile_id: &str) -> Result<SyncProgress> {
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        let result = conn.query_row(
            "SELECT sync_status, sync_progress, sync_message, 
                    channels_count, movies_count, series_count
             FROM xtream_content_sync
             WHERE profile_id = ?1",
            [profile_id],
            |row| {
                let status_str: String = row.get(0)?;
                let progress: i32 = row.get(1)?;
                let message: Option<String> = row.get(2)?;
                let channels_count: i32 = row.get(3)?;
                let movies_count: i32 = row.get(4)?;
                let series_count: i32 = row.get(5)?;
                
                Ok(SyncProgress {
                    status: SyncStatus::from_db_string(&status_str),
                    progress: progress.clamp(0, 100) as u8,
                    current_step: message.unwrap_or_default(),
                    channels_synced: channels_count as usize,
                    movies_synced: movies_count as usize,
                    series_synced: series_count as usize,
                    errors: Vec::new(),
                })
            },
        );
        
        match result {
            Ok(progress) => Ok(progress),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Profile not initialized, return default pending status
                Ok(SyncProgress::default())
            }
            Err(e) => Err(e.into()),
        }
    }
    
    /// Update sync status in the database
    pub fn update_sync_status(&self, profile_id: &str, progress: &SyncProgress) -> Result<()> {
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        // Ensure the profile has a sync record
        conn.execute(
            "INSERT OR IGNORE INTO xtream_content_sync (profile_id, sync_status) 
             VALUES (?1, 'pending')",
            [profile_id],
        )?;
        
        // Update the sync status
        conn.execute(
            "UPDATE xtream_content_sync 
             SET sync_status = ?1,
                 sync_progress = ?2,
                 sync_message = ?3,
                 channels_count = ?4,
                 movies_count = ?5,
                 series_count = ?6,
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?7",
            rusqlite::params![
                progress.status.to_db_string(),
                progress.progress as i32,
                progress.current_step.clone(),
                progress.channels_synced as i32,
                progress.movies_synced as i32,
                progress.series_synced as i32,
                profile_id,
            ],
        )?;
        
        Ok(())
    }
    
    /// Update last sync timestamp for a specific content type
    pub fn update_last_sync_timestamp(&self, profile_id: &str, content_type: &str) -> Result<()> {
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        let column = match content_type {
            "channels" => "last_sync_channels",
            "movies" => "last_sync_movies",
            "series" => "last_sync_series",
            _ => return Err(XTauriError::internal(format!("Invalid content type: {}", content_type))),
        };
        
        let query = format!(
            "UPDATE xtream_content_sync 
             SET {} = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?1",
            column
        );
        
        conn.execute(&query, [profile_id])?;
        
        Ok(())
    }
    
    /// Get sync settings for a profile
    pub fn get_sync_settings(&self, profile_id: &str) -> Result<SyncSettings> {
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        let result = conn.query_row(
            "SELECT auto_sync_enabled, sync_interval_hours, wifi_only, notify_on_complete
             FROM xtream_sync_settings
             WHERE profile_id = ?1",
            [profile_id],
            |row| {
                Ok(SyncSettings {
                    auto_sync_enabled: row.get(0)?,
                    sync_interval_hours: row.get::<_, i32>(1)? as u32,
                    wifi_only: row.get(2)?,
                    notify_on_complete: row.get(3)?,
                })
            },
        );
        
        match result {
            Ok(settings) => Ok(settings),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // No settings found, return defaults
                Ok(SyncSettings::default())
            }
            Err(e) => Err(e.into()),
        }
    }
    
    /// Update sync settings for a profile
    pub fn update_sync_settings(&self, profile_id: &str, settings: &SyncSettings) -> Result<()> {
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        // Validate settings
        if settings.sync_interval_hours < 6 {
            return Err(XTauriError::internal("Sync interval must be at least 6 hours".to_string()));
        }
        
        // Ensure the profile has a settings record
        conn.execute(
            "INSERT OR IGNORE INTO xtream_sync_settings (profile_id) VALUES (?1)",
            [profile_id],
        )?;
        
        // Update the settings
        conn.execute(
            "UPDATE xtream_sync_settings 
             SET auto_sync_enabled = ?1,
                 sync_interval_hours = ?2,
                 wifi_only = ?3,
                 notify_on_complete = ?4,
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?5",
            rusqlite::params![
                settings.auto_sync_enabled,
                settings.sync_interval_hours as i32,
                settings.wifi_only,
                settings.notify_on_complete,
                profile_id,
            ],
        )?;
        
        Ok(())
    }
    
    /// Check if a sync is currently active for a profile
    pub fn is_sync_active(&self, profile_id: &str) -> Result<bool> {
        let active_syncs = self.active_syncs.lock()
            .map_err(|_| XTauriError::lock_acquisition("active syncs"))?;
        
        Ok(active_syncs.contains_key(profile_id))
    }
    
    /// Register an active sync operation
    pub fn register_sync(&self, profile_id: &str, cancel_token: CancellationToken) -> Result<()> {
        let mut active_syncs = self.active_syncs.lock()
            .map_err(|_| XTauriError::lock_acquisition("active syncs"))?;
        
        if active_syncs.contains_key(profile_id) {
            return Err(XTauriError::internal(format!("Sync already in progress for profile {}", profile_id)));
        }
        
        active_syncs.insert(profile_id.to_string(), cancel_token);
        
        Ok(())
    }
    
    /// Unregister an active sync operation
    pub fn unregister_sync(&self, profile_id: &str) -> Result<()> {
        let mut active_syncs = self.active_syncs.lock()
            .map_err(|_| XTauriError::lock_acquisition("active syncs"))?;
        
        active_syncs.remove(profile_id);
        
        Ok(())
    }
    
    /// Cancel an active sync operation
    pub fn cancel_sync(&self, profile_id: &str) -> Result<()> {
        let active_syncs = self.active_syncs.lock()
            .map_err(|_| XTauriError::lock_acquisition("active syncs"))?;
        
        if let Some(cancel_token) = active_syncs.get(profile_id) {
            cancel_token.cancel();
            Ok(())
        } else {
            Err(XTauriError::NotFound {
                resource: format!("No active sync for profile {}", profile_id),
            })
        }
    }
    
    /// Get the number of active syncs
    pub fn active_sync_count(&self) -> Result<usize> {
        let active_syncs = self.active_syncs.lock()
            .map_err(|_| XTauriError::lock_acquisition("active syncs"))?;
        
        Ok(active_syncs.len())
    }
    
    /// Check if sync is needed based on settings and last sync time
    pub fn should_sync(&self, profile_id: &str) -> Result<bool> {
        // Get sync settings
        let settings = self.get_sync_settings(profile_id)?;
        
        // If auto-sync is disabled, don't sync
        if !settings.auto_sync_enabled {
            return Ok(false);
        }
        
        // Check last sync time
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        let last_sync: Option<String> = conn.query_row(
            "SELECT MAX(last_sync) FROM (
                SELECT last_sync_channels as last_sync FROM xtream_content_sync WHERE profile_id = ?1
                UNION ALL
                SELECT last_sync_movies FROM xtream_content_sync WHERE profile_id = ?1
                UNION ALL
                SELECT last_sync_series FROM xtream_content_sync WHERE profile_id = ?1
            )",
            [profile_id],
            |row| row.get(0),
        ).ok();
        
        // If never synced, should sync
        if last_sync.is_none() {
            return Ok(true);
        }
        
        // Parse last sync timestamp
        if let Some(last_sync_str) = last_sync {
            use chrono::{DateTime, Utc, Duration};
            
            if let Ok(last_sync_dt) = DateTime::parse_from_rfc3339(&last_sync_str) {
                let now = Utc::now();
                let elapsed = now.signed_duration_since(last_sync_dt.with_timezone(&Utc));
                let interval = Duration::hours(settings.sync_interval_hours as i64);
                
                return Ok(elapsed >= interval);
            }
        }
        
        // If we can't parse the timestamp, assume we should sync
        Ok(true)
    }
    
    // ==================== Sync API Integration Methods ====================
    
    /// Fetch categories from Xtream API with retry logic
    /// 
    /// # Arguments
    /// * `client` - HTTP client for making requests
    /// * `base_url` - Base URL of the Xtream server
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    /// * `content_type` - Type of content (channels, movies, series)
    /// * `retry_config` - Configuration for retry behavior
    /// * `cancel_token` - Token to check for cancellation
    /// 
    /// # Returns
    /// JSON value containing the categories
    pub async fn fetch_categories_with_retry(
        client: &reqwest::Client,
        base_url: &str,
        username: &str,
        password: &str,
        content_type: &str,
        retry_config: &RetryConfig,
        cancel_token: &CancellationToken,
    ) -> Result<serde_json::Value> {
        let action = match content_type {
            "channels" => "get_live_categories",
            "movies" => "get_vod_categories",
            "series" => "get_series_categories",
            _ => return Err(XTauriError::internal(format!("Invalid content type: {}", content_type))),
        };
        
        let url = format!(
            "{}/player_api.php?username={}&password={}&action={}",
            base_url, username, password, action
        );
        
        Self::fetch_with_retry(client, &url, retry_config, cancel_token).await
    }
    
    /// Fetch content list from Xtream API with retry logic
    /// 
    /// # Arguments
    /// * `client` - HTTP client for making requests
    /// * `base_url` - Base URL of the Xtream server
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    /// * `content_type` - Type of content (channels, movies, series)
    /// * `category_id` - Optional category ID to filter by
    /// * `retry_config` - Configuration for retry behavior
    /// * `cancel_token` - Token to check for cancellation
    /// 
    /// # Returns
    /// JSON value containing the content list
    pub async fn fetch_content_with_retry(
        client: &reqwest::Client,
        base_url: &str,
        username: &str,
        password: &str,
        content_type: &str,
        category_id: Option<&str>,
        retry_config: &RetryConfig,
        cancel_token: &CancellationToken,
    ) -> Result<serde_json::Value> {
        let action = match content_type {
            "channels" => "get_live_streams",
            "movies" => "get_vod_streams",
            "series" => "get_series",
            _ => return Err(XTauriError::internal(format!("Invalid content type: {}", content_type))),
        };
        
        let mut url = format!(
            "{}/player_api.php?username={}&password={}&action={}",
            base_url, username, password, action
        );
        
        if let Some(cat_id) = category_id {
            url.push_str(&format!("&category_id={}", cat_id));
        }
        
        Self::fetch_with_retry(client, &url, retry_config, cancel_token).await
    }
    
    /// Fetch series details from Xtream API with retry logic
    /// 
    /// # Arguments
    /// * `client` - HTTP client for making requests
    /// * `base_url` - Base URL of the Xtream server
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    /// * `series_id` - ID of the series to fetch details for
    /// * `retry_config` - Configuration for retry behavior
    /// * `cancel_token` - Token to check for cancellation
    /// 
    /// # Returns
    /// JSON value containing the series details with seasons and episodes
    pub async fn fetch_series_details_with_retry(
        client: &reqwest::Client,
        base_url: &str,
        username: &str,
        password: &str,
        series_id: i64,
        retry_config: &RetryConfig,
        cancel_token: &CancellationToken,
    ) -> Result<serde_json::Value> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
            base_url, username, password, series_id
        );
        
        Self::fetch_with_retry(client, &url, retry_config, cancel_token).await
    }
    
    /// Generic fetch with retry logic and exponential backoff
    /// 
    /// # Arguments
    /// * `client` - HTTP client for making requests
    /// * `url` - URL to fetch
    /// * `retry_config` - Configuration for retry behavior
    /// * `cancel_token` - Token to check for cancellation
    /// 
    /// # Returns
    /// JSON value containing the response
    async fn fetch_with_retry(
        client: &reqwest::Client,
        url: &str,
        retry_config: &RetryConfig,
        cancel_token: &CancellationToken,
    ) -> Result<serde_json::Value> {
        let mut last_error = None;
        let mut delay_ms = retry_config.initial_delay_ms;
        
        for attempt in 0..=retry_config.max_retries {
            // Check for cancellation
            if cancel_token.is_cancelled() {
                return Err(XTauriError::internal("Sync cancelled by user".to_string()));
            }
            
            match Self::try_fetch(client, url).await {
                Ok(data) => {
                    #[cfg(debug_assertions)]
                    if attempt > 0 {
                        println!("[DEBUG] Fetch succeeded on attempt {}", attempt + 1);
                    }
                    return Ok(data);
                }
                Err(e) => {
                    last_error = Some(e);
                    
                    // Check if we should retry
                    if let Some(ref err) = last_error {
                        let should_retry = match err {
                            // Don't retry authentication failures
                            XTauriError::XtreamInvalidCredentials => false,
                            XTauriError::XtreamAuthenticationFailed { .. } => {
                                // Only retry network-related auth failures
                                err.to_string().contains("Network") || 
                                err.to_string().contains("timeout")
                            }
                            // Don't retry client errors (4xx), but retry server errors (5xx)
                            XTauriError::XtreamApiError { status, .. } => *status >= 500,
                            // Retry network errors and timeouts
                            XTauriError::Timeout { .. } => true,
                            XTauriError::Network(_) => true,
                            _ => false,
                        };
                        
                        if !should_retry {
                            break;
                        }
                    }
                    
                    // Wait before retrying (exponential backoff)
                    if attempt < retry_config.max_retries {
                        #[cfg(debug_assertions)]
                        println!(
                            "[DEBUG] Fetch failed on attempt {}, retrying in {}ms: {:?}",
                            attempt + 1,
                            delay_ms,
                            last_error
                        );
                        
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                        
                        // Exponential backoff with max delay cap
                        delay_ms = ((delay_ms as f64 * retry_config.backoff_multiplier) as u64)
                            .min(retry_config.max_delay_ms);
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            XTauriError::internal("Fetch failed after all retries".to_string())
        }))
    }
    
    /// Single fetch attempt
    async fn try_fetch(
        client: &reqwest::Client,
        url: &str,
    ) -> Result<serde_json::Value> {
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    XTauriError::timeout("API request")
                } else {
                    // Network error is a tuple variant wrapping reqwest::Error
                    XTauriError::Network(e)
                }
            })?;
        
        let status = response.status();
        
        if !status.is_success() {
            let error_message = match status.as_u16() {
                401 => "Invalid username or password".to_string(),
                403 => "Access forbidden - account may be suspended".to_string(),
                404 => "Server endpoint not found - check URL".to_string(),
                500..=599 => "Server error - please try again later".to_string(),
                _ => format!("HTTP error: {}", status),
            };
            
            return Err(XTauriError::XtreamApiError {
                status: status.as_u16(),
                message: error_message,
            });
        }
        
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| XTauriError::internal(format!("Invalid response format: {}", e)))?;
        
        Ok(data)
    }
    
    /// Calculate progress percentage based on completed steps
    /// 
    /// # Arguments
    /// * `current_step` - Current step number (0-based)
    /// * `total_steps` - Total number of steps
    /// * `step_progress` - Progress within the current step (0.0 to 1.0)
    /// 
    /// # Returns
    /// Progress percentage (0-100)
    pub fn calculate_progress(current_step: usize, total_steps: usize, step_progress: f64) -> u8 {
        if total_steps == 0 {
            return 100;
        }
        
        let step_weight = 100.0 / total_steps as f64;
        let completed_steps_progress = current_step as f64 * step_weight;
        let current_step_progress = step_progress * step_weight;
        
        let total_progress = completed_steps_progress + current_step_progress;
        
        total_progress.clamp(0.0, 100.0) as u8
    }
    
    // ==================== Sync Workflow Methods ====================
    
    /// Start a full synchronization for a profile
    /// 
    /// This orchestrates the complete sync pipeline:
    /// 1. Fetch and save channel categories
    /// 2. Fetch and save channels
    /// 3. Fetch and save movie categories
    /// 4. Fetch and save movies
    /// 5. Fetch and save series categories
    /// 6. Fetch and save series (with details)
    /// 
    /// # Arguments
    /// * `profile_id` - The profile ID to sync
    /// * `base_url` - Base URL of the Xtream server
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    /// * `content_cache` - Reference to the content cache for saving data
    /// * `progress_tx` - Channel to send progress updates
    /// * `cancel_token` - Token to check for cancellation
    /// 
    /// # Returns
    /// Final sync progress with status
    pub async fn run_full_sync(
        &self,
        profile_id: &str,
        base_url: &str,
        username: &str,
        password: &str,
        content_cache: &crate::content_cache::ContentCache,
        progress_tx: &mpsc::Sender<SyncProgress>,
        cancel_token: &CancellationToken,
    ) -> Result<SyncProgress> {
        use std::time::Duration;
        
        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| XTauriError::internal(format!("Failed to create HTTP client: {}", e)))?;
        
        let retry_config = RetryConfig::default();
        
        // Initialize progress
        let mut progress = SyncProgress {
            status: SyncStatus::Syncing,
            progress: 0,
            current_step: "Starting sync...".to_string(),
            channels_synced: 0,
            movies_synced: 0,
            series_synced: 0,
            errors: Vec::new(),
        };
        
        // Update initial status
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        // Total steps: 6 (categories + content for each type)
        let total_steps = 6;
        let mut current_step = 0;
        
        // Step 1: Sync channel categories
        progress.current_step = "Syncing channel categories...".to_string();
        progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        match Self::sync_categories(
            &client,
            base_url,
            username,
            password,
            "channels",
            profile_id,
            content_cache,
            &retry_config,
            cancel_token,
        ).await {
            Ok(_) => {
                current_step += 1;
                progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
            }
            Err(e) => {
                progress.errors.push(format!("Channel categories sync failed: {}", e));
                eprintln!("[ERROR] Channel categories sync failed: {}", e);
            }
        }
        
        // Step 2: Sync channels
        progress.current_step = "Syncing channels...".to_string();
        progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        match Self::sync_content(
            &client,
            base_url,
            username,
            password,
            "channels",
            profile_id,
            content_cache,
            &retry_config,
            cancel_token,
        ).await {
            Ok(count) => {
                progress.channels_synced = count;
                current_step += 1;
                progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
                self.update_last_sync_timestamp(profile_id, "channels")?;
            }
            Err(e) => {
                progress.errors.push(format!("Channels sync failed: {}", e));
                eprintln!("[ERROR] Channels sync failed: {}", e);
            }
        }
        
        // Step 3: Sync movie categories
        progress.current_step = "Syncing movie categories...".to_string();
        progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        match Self::sync_categories(
            &client,
            base_url,
            username,
            password,
            "movies",
            profile_id,
            content_cache,
            &retry_config,
            cancel_token,
        ).await {
            Ok(_) => {
                current_step += 1;
                progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
            }
            Err(e) => {
                progress.errors.push(format!("Movie categories sync failed: {}", e));
                eprintln!("[ERROR] Movie categories sync failed: {}", e);
            }
        }
        
        // Step 4: Sync movies
        progress.current_step = "Syncing movies...".to_string();
        progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        match Self::sync_content(
            &client,
            base_url,
            username,
            password,
            "movies",
            profile_id,
            content_cache,
            &retry_config,
            cancel_token,
        ).await {
            Ok(count) => {
                progress.movies_synced = count;
                current_step += 1;
                progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
                self.update_last_sync_timestamp(profile_id, "movies")?;
            }
            Err(e) => {
                progress.errors.push(format!("Movies sync failed: {}", e));
                eprintln!("[ERROR] Movies sync failed: {}", e);
            }
        }
        
        // Step 5: Sync series categories
        progress.current_step = "Syncing series categories...".to_string();
        progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        match Self::sync_categories(
            &client,
            base_url,
            username,
            password,
            "series",
            profile_id,
            content_cache,
            &retry_config,
            cancel_token,
        ).await {
            Ok(_) => {
                current_step += 1;
                progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
            }
            Err(e) => {
                progress.errors.push(format!("Series categories sync failed: {}", e));
                eprintln!("[ERROR] Series categories sync failed: {}", e);
            }
        }
        
        // Step 6: Sync series
        progress.current_step = "Syncing series...".to_string();
        progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        match Self::sync_content(
            &client,
            base_url,
            username,
            password,
            "series",
            profile_id,
            content_cache,
            &retry_config,
            cancel_token,
        ).await {
            Ok(count) => {
                progress.series_synced = count;
                current_step += 1;
                progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
                self.update_last_sync_timestamp(profile_id, "series")?;
            }
            Err(e) => {
                progress.errors.push(format!("Series sync failed: {}", e));
                eprintln!("[ERROR] Series sync failed: {}", e);
            }
        }
        
        // Determine final status
        progress.progress = 100;
        progress.status = if progress.errors.is_empty() {
            progress.current_step = "Sync completed successfully".to_string();
            SyncStatus::Completed
        } else if progress.channels_synced > 0 || progress.movies_synced > 0 || progress.series_synced > 0 {
            progress.current_step = format!("Sync completed with {} errors", progress.errors.len());
            SyncStatus::Partial
        } else {
            progress.current_step = "Sync failed".to_string();
            SyncStatus::Failed
        };
        
        // Update final status
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        Ok(progress)
    }
    
    /// Sync categories for a specific content type
    async fn sync_categories(
        client: &reqwest::Client,
        base_url: &str,
        username: &str,
        password: &str,
        content_type: &str,
        profile_id: &str,
        content_cache: &crate::content_cache::ContentCache,
        retry_config: &RetryConfig,
        cancel_token: &CancellationToken,
    ) -> Result<usize> {
        // Fetch categories from API
        let categories_data = Self::fetch_categories_with_retry(
            client,
            base_url,
            username,
            password,
            content_type,
            retry_config,
            cancel_token,
        ).await?;
        
        // Parse categories
        let categories = Self::parse_categories(&categories_data)?;
        
        // Save to cache
        let content_type_enum = match content_type {
            "channels" => crate::content_cache::ContentType::Channels,
            "movies" => crate::content_cache::ContentType::Movies,
            "series" => crate::content_cache::ContentType::Series,
            _ => return Err(XTauriError::internal(format!("Invalid content type: {}", content_type))),
        };
        
        let count = content_cache.save_categories(profile_id, content_type_enum, categories)?;
        
        Ok(count)
    }
    
    /// Sync content for a specific content type
    async fn sync_content(
        client: &reqwest::Client,
        base_url: &str,
        username: &str,
        password: &str,
        content_type: &str,
        profile_id: &str,
        content_cache: &crate::content_cache::ContentCache,
        retry_config: &RetryConfig,
        cancel_token: &CancellationToken,
    ) -> Result<usize> {
        // Fetch content from API
        let content_data = Self::fetch_content_with_retry(
            client,
            base_url,
            username,
            password,
            content_type,
            None, // Fetch all categories
            retry_config,
            cancel_token,
        ).await?;
        
        // Parse and save based on content type
        let count = match content_type {
            "channels" => {
                let channels = Self::parse_channels(&content_data)?;
                content_cache.save_channels(profile_id, channels)?
            }
            "movies" => {
                let movies = Self::parse_movies(&content_data)?;
                content_cache.save_movies(profile_id, movies)?
            }
            "series" => {
                let series = Self::parse_series(&content_data)?;
                content_cache.save_series(profile_id, series)?
            }
            _ => return Err(XTauriError::internal(format!("Invalid content type: {}", content_type))),
        };
        
        Ok(count)
    }
    
    /// Parse categories from JSON response
    pub fn parse_categories(data: &serde_json::Value) -> Result<Vec<crate::content_cache::XtreamCategory>> {
        let array = data.as_array()
            .ok_or_else(|| XTauriError::internal("Categories response is not an array".to_string()))?;
        
        let mut categories = Vec::new();
        
        for item in array {
            let category_id = item.get("category_id")
                .and_then(|v| v.as_str().or_else(|| v.as_i64().map(|i| Box::leak(i.to_string().into_boxed_str()) as &str)))
                .unwrap_or("0")
                .to_string();
            
            let category_name = item.get("category_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            
            let parent_id = item.get("parent_id")
                .and_then(|v| v.as_i64());
            
            categories.push(crate::content_cache::XtreamCategory {
                category_id,
                category_name,
                parent_id,
            });
        }
        
        Ok(categories)
    }
    
    /// Parse channels from JSON response
    pub fn parse_channels(data: &serde_json::Value) -> Result<Vec<crate::content_cache::XtreamChannel>> {
        let array = data.as_array()
            .ok_or_else(|| XTauriError::internal("Channels response is not an array".to_string()))?;
        
        let mut channels = Vec::new();
        
        for item in array {
            let stream_id = item.get("stream_id")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            
            let name = item.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            
            channels.push(crate::content_cache::XtreamChannel {
                stream_id,
                num: item.get("num").and_then(|v| v.as_i64()),
                name,
                stream_type: item.get("stream_type").and_then(|v| v.as_str()).map(String::from),
                stream_icon: item.get("stream_icon").and_then(|v| v.as_str()).map(String::from),
                thumbnail: item.get("thumbnail").and_then(|v| v.as_str()).map(String::from),
                epg_channel_id: item.get("epg_channel_id").and_then(|v| v.as_str()).map(String::from),
                added: item.get("added").and_then(|v| v.as_str()).map(String::from),
                category_id: item.get("category_id").and_then(|v| v.as_str().or_else(|| v.as_i64().map(|i| Box::leak(i.to_string().into_boxed_str()) as &str))).map(String::from),
                custom_sid: item.get("custom_sid").and_then(|v| v.as_str()).map(String::from),
                tv_archive: item.get("tv_archive").and_then(|v| v.as_i64()),
                direct_source: item.get("direct_source").and_then(|v| v.as_str()).map(String::from),
                tv_archive_duration: item.get("tv_archive_duration").and_then(|v| v.as_i64()),
            });
        }
        
        Ok(channels)
    }
    
    /// Parse movies from JSON response
    pub fn parse_movies(data: &serde_json::Value) -> Result<Vec<crate::content_cache::XtreamMovie>> {
        let array = data.as_array()
            .ok_or_else(|| XTauriError::internal("Movies response is not an array".to_string()))?;
        
        let mut movies = Vec::new();
        
        for item in array {
            let stream_id = item.get("stream_id")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            
            let name = item.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            
            movies.push(crate::content_cache::XtreamMovie {
                stream_id,
                num: item.get("num").and_then(|v| v.as_i64()),
                name,
                title: item.get("title").and_then(|v| v.as_str()).map(String::from),
                year: item.get("year").and_then(|v| v.as_str()).map(String::from),
                stream_type: item.get("stream_type").and_then(|v| v.as_str()).map(String::from),
                stream_icon: item.get("stream_icon").and_then(|v| v.as_str()).map(String::from),
                rating: item.get("rating").and_then(|v| v.as_f64()),
                rating_5based: item.get("rating_5based").and_then(|v| v.as_f64()),
                genre: item.get("genre").and_then(|v| v.as_str()).map(String::from),
                added: item.get("added").and_then(|v| v.as_str()).map(String::from),
                episode_run_time: item.get("episode_run_time").and_then(|v| v.as_i64()),
                category_id: item.get("category_id").and_then(|v| v.as_str().or_else(|| v.as_i64().map(|i| Box::leak(i.to_string().into_boxed_str()) as &str))).map(String::from),
                container_extension: item.get("container_extension").and_then(|v| v.as_str()).map(String::from),
                custom_sid: item.get("custom_sid").and_then(|v| v.as_str()).map(String::from),
                direct_source: item.get("direct_source").and_then(|v| v.as_str()).map(String::from),
                release_date: item.get("release_date").and_then(|v| v.as_str()).map(String::from),
                cast: item.get("cast").and_then(|v| v.as_str()).map(String::from),
                director: item.get("director").and_then(|v| v.as_str()).map(String::from),
                plot: item.get("plot").and_then(|v| v.as_str()).map(String::from),
                youtube_trailer: item.get("youtube_trailer").and_then(|v| v.as_str()).map(String::from),
            });
        }
        
        Ok(movies)
    }
    
    /// Parse series from JSON response
    pub fn parse_series(data: &serde_json::Value) -> Result<Vec<crate::content_cache::XtreamSeries>> {
        let array = data.as_array()
            .ok_or_else(|| XTauriError::internal("Series response is not an array".to_string()))?;
        
        let mut series = Vec::new();
        
        for item in array {
            let series_id = item.get("series_id")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            
            let name = item.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            
            series.push(crate::content_cache::XtreamSeries {
                series_id,
                num: item.get("num").and_then(|v| v.as_i64()),
                name,
                title: item.get("title").and_then(|v| v.as_str()).map(String::from),
                year: item.get("year").and_then(|v| v.as_str()).map(String::from),
                cover: item.get("cover").and_then(|v| v.as_str()).map(String::from),
                plot: item.get("plot").and_then(|v| v.as_str()).map(String::from),
                cast: item.get("cast").and_then(|v| v.as_str()).map(String::from),
                director: item.get("director").and_then(|v| v.as_str()).map(String::from),
                genre: item.get("genre").and_then(|v| v.as_str()).map(String::from),
                release_date: item.get("release_date").and_then(|v| v.as_str()).map(String::from),
                last_modified: item.get("last_modified").and_then(|v| v.as_str()).map(String::from),
                rating: item.get("rating").and_then(|v| v.as_str()).map(String::from),
                rating_5based: item.get("rating_5based").and_then(|v| v.as_f64()),
                episode_run_time: item.get("episode_run_time").and_then(|v| v.as_str()).map(String::from),
                category_id: item.get("category_id").and_then(|v| v.as_str().or_else(|| v.as_i64().map(|i| Box::leak(i.to_string().into_boxed_str()) as &str))).map(String::from),
            });
        }
        
        Ok(series)
    }
    
    // ==================== Incremental Sync Methods ====================
    
    /// Start an incremental synchronization for a profile
    /// 
    /// This performs a differential sync that only updates changed content:
    /// 1. Compares timestamps with the server
    /// 2. Downloads only new or modified content
    /// 3. Detects and removes deleted content
    /// 
    /// # Arguments
    /// * `profile_id` - The profile ID to sync
    /// * `base_url` - Base URL of the Xtream server
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    /// * `content_cache` - Reference to the content cache for saving data
    /// * `progress_tx` - Channel to send progress updates
    /// * `cancel_token` - Token to check for cancellation
    /// 
    /// # Returns
    /// Final sync progress with status
    pub async fn run_incremental_sync(
        &self,
        profile_id: &str,
        base_url: &str,
        username: &str,
        password: &str,
        content_cache: &crate::content_cache::ContentCache,
        progress_tx: &mpsc::Sender<SyncProgress>,
        cancel_token: &CancellationToken,
    ) -> Result<SyncProgress> {
        use std::time::Duration;
        
        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| XTauriError::internal(format!("Failed to create HTTP client: {}", e)))?;
        
        let retry_config = RetryConfig::default();
        
        // Initialize progress
        let mut progress = SyncProgress {
            status: SyncStatus::Syncing,
            progress: 0,
            current_step: "Starting incremental sync...".to_string(),
            channels_synced: 0,
            movies_synced: 0,
            series_synced: 0,
            errors: Vec::new(),
        };
        
        // Update initial status
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        // Get last sync timestamps
        let last_sync_times = self.get_last_sync_timestamps(profile_id)?;
        
        // Total steps: 3 (one for each content type)
        let total_steps = 3;
        let mut current_step = 0;
        
        // Step 1: Incremental sync channels
        progress.current_step = "Syncing channels (incremental)...".to_string();
        progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        match Self::sync_content_incremental(
            &client,
            base_url,
            username,
            password,
            "channels",
            profile_id,
            content_cache,
            last_sync_times.channels,
            &retry_config,
            cancel_token,
        ).await {
            Ok(count) => {
                progress.channels_synced = count;
                current_step += 1;
                progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
                self.update_last_sync_timestamp(profile_id, "channels")?;
            }
            Err(e) => {
                progress.errors.push(format!("Channels incremental sync failed: {}", e));
                eprintln!("[ERROR] Channels incremental sync failed: {}", e);
            }
        }
        
        // Step 2: Incremental sync movies
        progress.current_step = "Syncing movies (incremental)...".to_string();
        progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        match Self::sync_content_incremental(
            &client,
            base_url,
            username,
            password,
            "movies",
            profile_id,
            content_cache,
            last_sync_times.movies,
            &retry_config,
            cancel_token,
        ).await {
            Ok(count) => {
                progress.movies_synced = count;
                current_step += 1;
                progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
                self.update_last_sync_timestamp(profile_id, "movies")?;
            }
            Err(e) => {
                progress.errors.push(format!("Movies incremental sync failed: {}", e));
                eprintln!("[ERROR] Movies incremental sync failed: {}", e);
            }
        }
        
        // Step 3: Incremental sync series
        progress.current_step = "Syncing series (incremental)...".to_string();
        progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        match Self::sync_content_incremental(
            &client,
            base_url,
            username,
            password,
            "series",
            profile_id,
            content_cache,
            last_sync_times.series,
            &retry_config,
            cancel_token,
        ).await {
            Ok(count) => {
                progress.series_synced = count;
                current_step += 1;
                progress.progress = Self::calculate_progress(current_step, total_steps, 0.0);
                self.update_last_sync_timestamp(profile_id, "series")?;
            }
            Err(e) => {
                progress.errors.push(format!("Series incremental sync failed: {}", e));
                eprintln!("[ERROR] Series incremental sync failed: {}", e);
            }
        }
        
        // Determine final status
        progress.progress = 100;
        progress.status = if progress.errors.is_empty() {
            progress.current_step = "Incremental sync completed successfully".to_string();
            SyncStatus::Completed
        } else if progress.channels_synced > 0 || progress.movies_synced > 0 || progress.series_synced > 0 {
            progress.current_step = format!("Incremental sync completed with {} errors", progress.errors.len());
            SyncStatus::Partial
        } else {
            progress.current_step = "Incremental sync failed".to_string();
            SyncStatus::Failed
        };
        
        // Update final status
        self.update_sync_status(profile_id, &progress)?;
        let _ = progress_tx.send(progress.clone()).await;
        
        Ok(progress)
    }
    
    /// Get last sync timestamps for all content types
    pub fn get_last_sync_timestamps(&self, profile_id: &str) -> Result<LastSyncTimestamps> {
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        let result = conn.query_row(
            "SELECT last_sync_channels, last_sync_movies, last_sync_series
             FROM xtream_content_sync
             WHERE profile_id = ?1",
            [profile_id],
            |row| {
                Ok(LastSyncTimestamps {
                    channels: row.get(0)?,
                    movies: row.get(1)?,
                    series: row.get(2)?,
                })
            },
        );
        
        match result {
            Ok(timestamps) => Ok(timestamps),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // No sync record yet, return None for all
                Ok(LastSyncTimestamps {
                    channels: None,
                    movies: None,
                    series: None,
                })
            }
            Err(e) => Err(e.into()),
        }
    }
    
    /// Sync content incrementally for a specific content type
    /// 
    /// This compares the server content with local cache and:
    /// 1. Adds new items
    /// 2. Updates modified items (based on timestamps)
    /// 3. Removes deleted items
    async fn sync_content_incremental(
        client: &reqwest::Client,
        base_url: &str,
        username: &str,
        password: &str,
        content_type: &str,
        profile_id: &str,
        content_cache: &crate::content_cache::ContentCache,
        last_sync: Option<String>,
        retry_config: &RetryConfig,
        cancel_token: &CancellationToken,
    ) -> Result<usize> {
        // Fetch all content from API
        let content_data = Self::fetch_content_with_retry(
            client,
            base_url,
            username,
            password,
            content_type,
            None, // Fetch all categories
            retry_config,
            cancel_token,
        ).await?;
        
        // Get current content IDs from cache
        let cached_ids = content_cache.get_content_ids(profile_id, content_type)?;
        
        // Parse server content and compare with cache
        let (new_items, updated_items, server_ids) = match content_type {
            "channels" => {
                let channels = Self::parse_channels(&content_data)?;
                Self::compare_channels(&channels, &cached_ids, last_sync.as_deref())
            }
            "movies" => {
                let movies = Self::parse_movies(&content_data)?;
                Self::compare_movies(&movies, &cached_ids, last_sync.as_deref())
            }
            "series" => {
                let series = Self::parse_series(&content_data)?;
                Self::compare_series(&series, &cached_ids, last_sync.as_deref())
            }
            _ => return Err(XTauriError::internal(format!("Invalid content type: {}", content_type))),
        };
        
        // Find deleted items (in cache but not on server)
        let deleted_ids: Vec<i64> = cached_ids
            .into_iter()
            .filter(|id| !server_ids.contains(id))
            .collect();
        
        // Apply changes
        let mut total_changes = 0;
        
        // Add/update new and modified items
        if !new_items.is_empty() || !updated_items.is_empty() {
            let items_to_save = [new_items, updated_items].concat();
            
            let count = match content_type {
                "channels" => {
                    let channels: Vec<crate::content_cache::XtreamChannel> = 
                        serde_json::from_value(serde_json::Value::Array(items_to_save))
                            .map_err(|e| XTauriError::internal(format!("Failed to deserialize channels: {}", e)))?;
                    content_cache.save_channels(profile_id, channels)?
                }
                "movies" => {
                    let movies: Vec<crate::content_cache::XtreamMovie> = 
                        serde_json::from_value(serde_json::Value::Array(items_to_save))
                            .map_err(|e| XTauriError::internal(format!("Failed to deserialize movies: {}", e)))?;
                    content_cache.save_movies(profile_id, movies)?
                }
                "series" => {
                    let series: Vec<crate::content_cache::XtreamSeries> = 
                        serde_json::from_value(serde_json::Value::Array(items_to_save))
                            .map_err(|e| XTauriError::internal(format!("Failed to deserialize series: {}", e)))?;
                    content_cache.save_series(profile_id, series)?
                }
                _ => 0,
            };
            
            total_changes += count;
        }
        
        // Remove deleted items
        if !deleted_ids.is_empty() {
            let deleted_count = content_cache.delete_content_by_ids(profile_id, content_type, &deleted_ids)?;
            total_changes += deleted_count;
            
            #[cfg(debug_assertions)]
            println!("[DEBUG] Removed {} deleted {} items", deleted_count, content_type);
        }
        
        Ok(total_changes)
    }
    
    /// Compare channels and identify new/updated items
    pub fn compare_channels(
        server_channels: &[crate::content_cache::XtreamChannel],
        cached_ids: &[i64],
        last_sync: Option<&str>,
    ) -> (Vec<serde_json::Value>, Vec<serde_json::Value>, Vec<i64>) {
        let mut new_items = Vec::new();
        let mut updated_items = Vec::new();
        let mut server_ids = Vec::new();
        
        for channel in server_channels {
            server_ids.push(channel.stream_id);
            
            let is_new = !cached_ids.contains(&channel.stream_id);
            let is_updated = if let Some(last_sync_time) = last_sync {
                Self::is_item_updated(&channel.added, last_sync_time)
            } else {
                false
            };
            
            if is_new {
                if let Ok(value) = serde_json::to_value(channel) {
                    new_items.push(value);
                }
            } else if is_updated {
                if let Ok(value) = serde_json::to_value(channel) {
                    updated_items.push(value);
                }
            }
        }
        
        (new_items, updated_items, server_ids)
    }
    
    /// Compare movies and identify new/updated items
    pub fn compare_movies(
        server_movies: &[crate::content_cache::XtreamMovie],
        cached_ids: &[i64],
        last_sync: Option<&str>,
    ) -> (Vec<serde_json::Value>, Vec<serde_json::Value>, Vec<i64>) {
        let mut new_items = Vec::new();
        let mut updated_items = Vec::new();
        let mut server_ids = Vec::new();
        
        for movie in server_movies {
            server_ids.push(movie.stream_id);
            
            let is_new = !cached_ids.contains(&movie.stream_id);
            let is_updated = if let Some(last_sync_time) = last_sync {
                Self::is_item_updated(&movie.added, last_sync_time)
            } else {
                false
            };
            
            if is_new {
                if let Ok(value) = serde_json::to_value(movie) {
                    new_items.push(value);
                }
            } else if is_updated {
                if let Ok(value) = serde_json::to_value(movie) {
                    updated_items.push(value);
                }
            }
        }
        
        (new_items, updated_items, server_ids)
    }
    
    /// Compare series and identify new/updated items
    pub fn compare_series(
        server_series: &[crate::content_cache::XtreamSeries],
        cached_ids: &[i64],
        last_sync: Option<&str>,
    ) -> (Vec<serde_json::Value>, Vec<serde_json::Value>, Vec<i64>) {
        let mut new_items = Vec::new();
        let mut updated_items = Vec::new();
        let mut server_ids = Vec::new();
        
        for series in server_series {
            server_ids.push(series.series_id);
            
            let is_new = !cached_ids.contains(&series.series_id);
            let is_updated = if let Some(last_sync_time) = last_sync {
                Self::is_item_updated(&series.last_modified, last_sync_time)
            } else {
                false
            };
            
            if is_new {
                if let Ok(value) = serde_json::to_value(series) {
                    new_items.push(value);
                }
            } else if is_updated {
                if let Ok(value) = serde_json::to_value(series) {
                    updated_items.push(value);
                }
            }
        }
        
        (new_items, updated_items, server_ids)
    }
    
    /// Check if an item has been updated since last sync
    /// 
    /// Compares the item's timestamp with the last sync timestamp
    pub fn is_item_updated(item_timestamp: &Option<String>, last_sync: &str) -> bool {
        if let Some(item_time) = item_timestamp {
            // Try to parse both timestamps and compare
            // Xtream API typically uses Unix timestamps as strings
            if let (Ok(item_ts), Ok(sync_ts)) = (
                item_time.parse::<i64>(),
                last_sync.parse::<i64>()
            ) {
                return item_ts > sync_ts;
            }
            
            // Fallback: try ISO 8601 format
            use chrono::{DateTime, Utc};
            if let (Ok(item_dt), Ok(sync_dt)) = (
                DateTime::parse_from_rfc3339(item_time),
                DateTime::parse_from_rfc3339(last_sync)
            ) {
                return item_dt.with_timezone(&Utc) > sync_dt.with_timezone(&Utc);
            }
        }
        
        // If we can't parse timestamps, assume not updated
        false
    }
}

/// Last sync timestamps for all content types
#[derive(Debug, Clone)]
pub struct LastSyncTimestamps {
    pub channels: Option<String>,
    pub movies: Option<String>,
    pub series: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    
    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        
        // Create required tables
        conn.execute(
            "CREATE TABLE xtream_profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                url TEXT NOT NULL,
                username TEXT NOT NULL,
                encrypted_credentials BLOB NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_used DATETIME,
                is_active BOOLEAN DEFAULT FALSE
            )",
            [],
        ).unwrap();
        
        conn.execute(
            "CREATE TABLE xtream_content_sync (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id TEXT NOT NULL UNIQUE,
                last_sync_channels TIMESTAMP,
                last_sync_movies TIMESTAMP,
                last_sync_series TIMESTAMP,
                sync_status TEXT DEFAULT 'pending',
                sync_progress INTEGER DEFAULT 0,
                sync_message TEXT,
                channels_count INTEGER DEFAULT 0,
                movies_count INTEGER DEFAULT 0,
                series_count INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
            )",
            [],
        ).unwrap();
        
        conn.execute(
            "CREATE TABLE xtream_sync_settings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id TEXT NOT NULL UNIQUE,
                auto_sync_enabled BOOLEAN DEFAULT 1,
                sync_interval_hours INTEGER DEFAULT 24,
                wifi_only BOOLEAN DEFAULT 1,
                notify_on_complete BOOLEAN DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
            )",
            [],
        ).unwrap();
        
        // Insert test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        ).unwrap();
        
        conn
    }
    
    #[test]
    fn test_sync_scheduler_initialization() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        
        let scheduler = SyncScheduler::new(db);
        
        // Verify scheduler is created
        assert_eq!(scheduler.active_sync_count().unwrap(), 0);
    }
    
    #[test]
    fn test_sync_status_conversion() {
        assert_eq!(SyncStatus::Pending.to_db_string(), "pending");
        assert_eq!(SyncStatus::Syncing.to_db_string(), "syncing");
        assert_eq!(SyncStatus::Completed.to_db_string(), "completed");
        assert_eq!(SyncStatus::Failed.to_db_string(), "failed");
        assert_eq!(SyncStatus::Partial.to_db_string(), "partial");
        
        assert_eq!(SyncStatus::from_db_string("pending"), SyncStatus::Pending);
        assert_eq!(SyncStatus::from_db_string("syncing"), SyncStatus::Syncing);
        assert_eq!(SyncStatus::from_db_string("completed"), SyncStatus::Completed);
        assert_eq!(SyncStatus::from_db_string("failed"), SyncStatus::Failed);
        assert_eq!(SyncStatus::from_db_string("partial"), SyncStatus::Partial);
        assert_eq!(SyncStatus::from_db_string("unknown"), SyncStatus::Pending);
    }
    
    #[test]
    fn test_get_sync_status_default() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        // Get status for profile without sync record
        let status = scheduler.get_sync_status("test-profile").unwrap();
        
        assert_eq!(status.status, SyncStatus::Pending);
        assert_eq!(status.progress, 0);
        assert_eq!(status.channels_synced, 0);
        assert_eq!(status.movies_synced, 0);
        assert_eq!(status.series_synced, 0);
    }
    
    #[test]
    fn test_update_sync_status() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        let mut progress = SyncProgress {
            status: SyncStatus::Syncing,
            progress: 50,
            current_step: "Syncing channels".to_string(),
            channels_synced: 100,
            movies_synced: 50,
            series_synced: 25,
            errors: Vec::new(),
        };
        
        // Update status
        scheduler.update_sync_status("test-profile", &progress).unwrap();
        
        // Retrieve and verify
        let retrieved = scheduler.get_sync_status("test-profile").unwrap();
        assert_eq!(retrieved.status, SyncStatus::Syncing);
        assert_eq!(retrieved.progress, 50);
        assert_eq!(retrieved.current_step, "Syncing channels");
        assert_eq!(retrieved.channels_synced, 100);
        assert_eq!(retrieved.movies_synced, 50);
        assert_eq!(retrieved.series_synced, 25);
        
        // Update again with different values
        progress.status = SyncStatus::Completed;
        progress.progress = 100;
        progress.current_step = "Sync complete".to_string();
        
        scheduler.update_sync_status("test-profile", &progress).unwrap();
        
        let retrieved = scheduler.get_sync_status("test-profile").unwrap();
        assert_eq!(retrieved.status, SyncStatus::Completed);
        assert_eq!(retrieved.progress, 100);
        assert_eq!(retrieved.current_step, "Sync complete");
    }
    
    #[test]
    fn test_update_last_sync_timestamp() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db.clone());
        
        // Initialize sync record
        let progress = SyncProgress::default();
        scheduler.update_sync_status("test-profile", &progress).unwrap();
        
        // Update timestamps
        scheduler.update_last_sync_timestamp("test-profile", "channels").unwrap();
        scheduler.update_last_sync_timestamp("test-profile", "movies").unwrap();
        scheduler.update_last_sync_timestamp("test-profile", "series").unwrap();
        
        // Verify timestamps are set
        let conn = db.lock().unwrap();
        let (channels, movies, series): (Option<String>, Option<String>, Option<String>) = conn.query_row(
            "SELECT last_sync_channels, last_sync_movies, last_sync_series 
             FROM xtream_content_sync WHERE profile_id = 'test-profile'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();
        
        assert!(channels.is_some());
        assert!(movies.is_some());
        assert!(series.is_some());
    }
    
    #[test]
    fn test_get_sync_settings_default() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        // Get settings for profile without settings record
        let settings = scheduler.get_sync_settings("test-profile").unwrap();
        
        assert_eq!(settings.auto_sync_enabled, true);
        assert_eq!(settings.sync_interval_hours, 24);
        assert_eq!(settings.wifi_only, true);
        assert_eq!(settings.notify_on_complete, false);
    }
    
    #[test]
    fn test_update_sync_settings() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        let settings = SyncSettings {
            auto_sync_enabled: false,
            sync_interval_hours: 12,
            wifi_only: false,
            notify_on_complete: true,
        };
        
        // Update settings
        scheduler.update_sync_settings("test-profile", &settings).unwrap();
        
        // Retrieve and verify
        let retrieved = scheduler.get_sync_settings("test-profile").unwrap();
        assert_eq!(retrieved.auto_sync_enabled, false);
        assert_eq!(retrieved.sync_interval_hours, 12);
        assert_eq!(retrieved.wifi_only, false);
        assert_eq!(retrieved.notify_on_complete, true);
    }
    
    #[test]
    fn test_sync_settings_validation() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        let invalid_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 3, // Too low
            wifi_only: true,
            notify_on_complete: false,
        };
        
        // Should fail validation
        let result = scheduler.update_sync_settings("test-profile", &invalid_settings);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_sync_handle_creation() {
        let (handle, _tx, cancel_token) = SyncHandle::new("test-profile".to_string());
        
        assert_eq!(handle.profile_id, "test-profile");
        assert!(!handle.is_cancelled());
        
        // Cancel and verify
        cancel_token.cancel();
        assert!(handle.is_cancelled());
    }
    
    #[test]
    fn test_register_and_unregister_sync() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        let cancel_token = CancellationToken::new();
        
        // Register sync
        scheduler.register_sync("test-profile", cancel_token.clone()).unwrap();
        assert!(scheduler.is_sync_active("test-profile").unwrap());
        assert_eq!(scheduler.active_sync_count().unwrap(), 1);
        
        // Try to register again (should fail)
        let result = scheduler.register_sync("test-profile", cancel_token.clone());
        assert!(result.is_err());
        
        // Unregister sync
        scheduler.unregister_sync("test-profile").unwrap();
        assert!(!scheduler.is_sync_active("test-profile").unwrap());
        assert_eq!(scheduler.active_sync_count().unwrap(), 0);
    }
    
    #[test]
    fn test_cancel_sync() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        let cancel_token = CancellationToken::new();
        
        // Register sync
        scheduler.register_sync("test-profile", cancel_token.clone()).unwrap();
        
        // Cancel sync
        scheduler.cancel_sync("test-profile").unwrap();
        assert!(cancel_token.is_cancelled());
        
        // Try to cancel non-existent sync
        let result = scheduler.cancel_sync("non-existent");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_should_sync_auto_disabled() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        // Set auto-sync to disabled
        let settings = SyncSettings {
            auto_sync_enabled: false,
            sync_interval_hours: 24,
            wifi_only: true,
            notify_on_complete: false,
        };
        scheduler.update_sync_settings("test-profile", &settings).unwrap();
        
        // Should not sync
        assert!(!scheduler.should_sync("test-profile").unwrap());
    }
    
    #[test]
    fn test_should_sync_never_synced() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        // Enable auto-sync
        let settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 24,
            wifi_only: false,
            notify_on_complete: false,
        };
        scheduler.update_sync_settings("test-profile", &settings).unwrap();
        
        // Should sync (never synced before)
        assert!(scheduler.should_sync("test-profile").unwrap());
    }
    
    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }
    
    #[test]
    fn test_calculate_progress() {
        // Test with 4 total steps
        assert_eq!(SyncScheduler::calculate_progress(0, 4, 0.0), 0);
        assert_eq!(SyncScheduler::calculate_progress(0, 4, 0.5), 12);
        assert_eq!(SyncScheduler::calculate_progress(0, 4, 1.0), 25);
        assert_eq!(SyncScheduler::calculate_progress(1, 4, 0.0), 25);
        assert_eq!(SyncScheduler::calculate_progress(1, 4, 0.5), 37);
        assert_eq!(SyncScheduler::calculate_progress(2, 4, 0.0), 50);
        assert_eq!(SyncScheduler::calculate_progress(3, 4, 0.0), 75);
        assert_eq!(SyncScheduler::calculate_progress(3, 4, 1.0), 100);
        assert_eq!(SyncScheduler::calculate_progress(4, 4, 0.0), 100);
        
        // Test edge cases
        assert_eq!(SyncScheduler::calculate_progress(0, 0, 0.0), 100);
        assert_eq!(SyncScheduler::calculate_progress(0, 1, 0.5), 50);
        assert_eq!(SyncScheduler::calculate_progress(0, 1, 1.0), 100);
    }
    
    #[tokio::test]
    async fn test_fetch_with_retry_cancellation() {
        use std::time::Duration;
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        
        let retry_config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 2.0,
        };
        
        let cancel_token = CancellationToken::new();
        
        // Cancel immediately
        cancel_token.cancel();
        
        // Should fail with cancellation error
        let result = SyncScheduler::fetch_with_retry(
            &client,
            "http://invalid-url-that-will-fail.test/api",
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("cancelled"));
    }
    
    #[tokio::test]
    async fn test_fetch_with_retry_invalid_url() {
        use std::time::Duration;
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();
        
        let retry_config = RetryConfig {
            max_retries: 2,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 2.0,
        };
        
        let cancel_token = CancellationToken::new();
        
        // Should fail after retries
        let result = SyncScheduler::fetch_with_retry(
            &client,
            "http://invalid-url-that-will-fail.test/api",
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_categories() {
        use serde_json::json;
        
        let data = json!([
            {
                "category_id": "1",
                "category_name": "Sports",
                "parent_id": null
            },
            {
                "category_id": "2",
                "category_name": "Movies",
                "parent_id": 0
            }
        ]);
        
        let categories = SyncScheduler::parse_categories(&data).unwrap();
        
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].category_id, "1");
        assert_eq!(categories[0].category_name, "Sports");
        assert_eq!(categories[0].parent_id, None);
        assert_eq!(categories[1].category_id, "2");
        assert_eq!(categories[1].category_name, "Movies");
        assert_eq!(categories[1].parent_id, Some(0));
    }
    
    #[test]
    fn test_parse_channels() {
        use serde_json::json;
        
        let data = json!([
            {
                "stream_id": 123,
                "num": 1,
                "name": "Test Channel",
                "stream_type": "live",
                "category_id": "1"
            },
            {
                "stream_id": 456,
                "name": "Another Channel"
            }
        ]);
        
        let channels = SyncScheduler::parse_channels(&data).unwrap();
        
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0].stream_id, 123);
        assert_eq!(channels[0].name, "Test Channel");
        assert_eq!(channels[0].num, Some(1));
        assert_eq!(channels[0].stream_type, Some("live".to_string()));
        assert_eq!(channels[1].stream_id, 456);
        assert_eq!(channels[1].name, "Another Channel");
        assert_eq!(channels[1].num, None);
    }
    
    #[test]
    fn test_parse_movies() {
        use serde_json::json;
        
        let data = json!([
            {
                "stream_id": 789,
                "name": "Test Movie",
                "title": "Test Movie Title",
                "year": "2023",
                "rating": 8.5,
                "genre": "Action"
            }
        ]);
        
        let movies = SyncScheduler::parse_movies(&data).unwrap();
        
        assert_eq!(movies.len(), 1);
        assert_eq!(movies[0].stream_id, 789);
        assert_eq!(movies[0].name, "Test Movie");
        assert_eq!(movies[0].title, Some("Test Movie Title".to_string()));
        assert_eq!(movies[0].year, Some("2023".to_string()));
        assert_eq!(movies[0].rating, Some(8.5));
        assert_eq!(movies[0].genre, Some("Action".to_string()));
    }
    
    #[test]
    fn test_parse_series() {
        use serde_json::json;
        
        let data = json!([
            {
                "series_id": 999,
                "name": "Test Series",
                "year": "2022",
                "rating_5based": 4.5,
                "genre": "Drama"
            }
        ]);
        
        let series = SyncScheduler::parse_series(&data).unwrap();
        
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].series_id, 999);
        assert_eq!(series[0].name, "Test Series");
        assert_eq!(series[0].year, Some("2022".to_string()));
        assert_eq!(series[0].rating_5based, Some(4.5));
        assert_eq!(series[0].genre, Some("Drama".to_string()));
    }
    
    #[test]
    fn test_parse_empty_arrays() {
        use serde_json::json;
        
        let empty_data = json!([]);
        
        assert_eq!(SyncScheduler::parse_categories(&empty_data).unwrap().len(), 0);
        assert_eq!(SyncScheduler::parse_channels(&empty_data).unwrap().len(), 0);
        assert_eq!(SyncScheduler::parse_movies(&empty_data).unwrap().len(), 0);
        assert_eq!(SyncScheduler::parse_series(&empty_data).unwrap().len(), 0);
    }
    
    #[test]
    fn test_parse_invalid_data() {
        use serde_json::json;
        
        let invalid_data = json!({"error": "not an array"});
        
        assert!(SyncScheduler::parse_categories(&invalid_data).is_err());
        assert!(SyncScheduler::parse_channels(&invalid_data).is_err());
        assert!(SyncScheduler::parse_movies(&invalid_data).is_err());
        assert!(SyncScheduler::parse_series(&invalid_data).is_err());
    }
}
