// Background scheduler for automatic content synchronization
use crate::error::{Result, XTauriError};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;
use tokio::task::JoinHandle;

/// Background scheduler that periodically checks and triggers syncs
pub struct BackgroundScheduler {
    check_interval: Duration,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl BackgroundScheduler {
    /// Create a new background scheduler
    /// 
    /// # Arguments
    /// * `check_interval_minutes` - How often to check if sync is needed (in minutes)
    pub fn new(check_interval_minutes: u64) -> Self {
        Self {
            check_interval: Duration::from_secs(check_interval_minutes * 60),
            task_handle: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Start the background scheduler
    /// 
    /// This spawns a background task that periodically checks all profiles
    /// and triggers syncs if needed based on their settings.
    /// 
    /// # Arguments
    /// * `sync_scheduler` - The sync scheduler to use for checking and triggering syncs
    /// * `profile_ids` - List of profile IDs to monitor
    /// * `on_sync_needed` - Callback function to trigger when sync is needed
    pub fn start<F>(
        &self,
        sync_scheduler: Arc<crate::content_cache::sync_scheduler::SyncScheduler>,
        profile_ids: Arc<Mutex<Vec<String>>>,
        on_sync_needed: Arc<F>,
    ) -> Result<()>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let mut task_handle = self.task_handle.lock()
            .map_err(|_| XTauriError::lock_acquisition("task handle"))?;
        
        // Stop existing task if running
        if let Some(handle) = task_handle.take() {
            handle.abort();
        }
        
        let check_interval = self.check_interval;
        
        // Spawn background task
        let handle = tokio::spawn(async move {
            let mut interval_timer = interval(check_interval);
            
            loop {
                interval_timer.tick().await;
                
                #[cfg(debug_assertions)]
                println!("[DEBUG] Background scheduler: Checking for syncs...");
                
                // Get current profile list
                let profiles = {
                    let profiles_guard = match profile_ids.lock() {
                        Ok(guard) => guard,
                        Err(e) => {
                            eprintln!("[ERROR] Failed to acquire profile list lock: {}", e);
                            continue;
                        }
                    };
                    profiles_guard.clone()
                };
                
                // Check each profile
                for profile_id in profiles {
                    // Check if sync is needed
                    match sync_scheduler.should_sync(&profile_id) {
                        Ok(true) => {
                            #[cfg(debug_assertions)]
                            println!("[DEBUG] Sync needed for profile: {}", profile_id);
                            
                            // Check if sync is already active
                            match sync_scheduler.is_sync_active(&profile_id) {
                                Ok(true) => {
                                    #[cfg(debug_assertions)]
                                    println!("[DEBUG] Sync already active for profile: {}", profile_id);
                                }
                                Ok(false) => {
                                    // Trigger sync callback
                                    on_sync_needed(profile_id.clone());
                                }
                                Err(e) => {
                                    eprintln!("[ERROR] Failed to check sync status for {}: {}", profile_id, e);
                                }
                            }
                        }
                        Ok(false) => {
                            #[cfg(debug_assertions)]
                            println!("[DEBUG] Sync not needed for profile: {}", profile_id);
                        }
                        Err(e) => {
                            eprintln!("[ERROR] Failed to check if sync needed for {}: {}", profile_id, e);
                        }
                    }
                }
            }
        });
        
        *task_handle = Some(handle);
        
        Ok(())
    }
    
    /// Stop the background scheduler
    pub fn stop(&self) -> Result<()> {
        let mut task_handle = self.task_handle.lock()
            .map_err(|_| XTauriError::lock_acquisition("task handle"))?;
        
        if let Some(handle) = task_handle.take() {
            handle.abort();
            #[cfg(debug_assertions)]
            println!("[DEBUG] Background scheduler stopped");
        }
        
        Ok(())
    }
    
    /// Check if the scheduler is running
    pub fn is_running(&self) -> Result<bool> {
        let task_handle = self.task_handle.lock()
            .map_err(|_| XTauriError::lock_acquisition("task handle"))?;
        
        Ok(task_handle.as_ref().map_or(false, |h| !h.is_finished()))
    }
}

impl Drop for BackgroundScheduler {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// WiFi detection utility
/// 
/// Note: WiFi detection is platform-specific and may not be available on all systems.
/// This is a placeholder implementation that always returns true.
pub fn is_wifi_connected() -> bool {
    // TODO: Implement platform-specific WiFi detection
    // For now, assume WiFi is always connected
    // 
    // On Windows: Use Windows API (GetAdaptersInfo, GetIfTable)
    // On Linux: Check /sys/class/net/*/wireless or use NetworkManager
    // On macOS: Use CoreWLAN framework
    
    #[cfg(debug_assertions)]
    println!("[DEBUG] WiFi detection not implemented, assuming connected");
    
    true
}

/// Notification utility for sync completion
/// 
/// Sends a system notification when sync is complete
pub fn send_sync_notification(profile_name: &str, success: bool) -> Result<()> {
    // TODO: Implement system notifications
    // Use tauri::api::notification or a notification crate
    
    let message = if success {
        format!("Content sync completed for {}", profile_name)
    } else {
        format!("Content sync failed for {}", profile_name)
    };
    
    #[cfg(debug_assertions)]
    println!("[DEBUG] Notification: {}", message);
    
    // For now, just log the notification
    // In production, this should use Tauri's notification API
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content_cache::sync_scheduler::{SyncScheduler, SyncSettings};
    use rusqlite::Connection;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    fn create_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        
        // Create xtream_profiles table
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
        )
        .unwrap();
        
        // Create sync settings table
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
        )
        .unwrap();
        
        // Create content sync table
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
        )
        .unwrap();
        
        Arc::new(Mutex::new(conn))
    }
    
    #[test]
    fn test_background_scheduler_creation() {
        let scheduler = BackgroundScheduler::new(5);
        assert!(!scheduler.is_running().unwrap());
    }
    
    #[tokio::test]
    async fn test_background_scheduler_start_stop() {
        let db = create_test_db();
        let sync_scheduler = Arc::new(SyncScheduler::new(db));
        let profile_ids = Arc::new(Mutex::new(vec![]));
        
        let callback_count = Arc::new(AtomicUsize::new(0));
        let callback_count_clone = Arc::clone(&callback_count);
        
        let on_sync_needed = Arc::new(move |_profile_id: String| {
            callback_count_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        let scheduler = BackgroundScheduler::new(1); // 1 minute interval
        
        // Start scheduler
        scheduler.start(sync_scheduler, profile_ids, on_sync_needed).unwrap();
        assert!(scheduler.is_running().unwrap());
        
        // Stop scheduler
        scheduler.stop().unwrap();
        assert!(!scheduler.is_running().unwrap());
    }
    
    #[tokio::test]
    async fn test_background_scheduler_triggers_callback() {
        let db = create_test_db();
        
        // Insert test profile
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
                 VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
                [],
            )
            .unwrap();
            
            // Set auto-sync enabled with very short interval for testing
            conn.execute(
                "INSERT INTO xtream_sync_settings (profile_id, auto_sync_enabled, sync_interval_hours) 
                 VALUES ('test-profile', 1, 6)",
                [],
            )
            .unwrap();
            
            // Initialize sync record with no last sync (should trigger sync)
            conn.execute(
                "INSERT INTO xtream_content_sync (profile_id, sync_status) 
                 VALUES ('test-profile', 'pending')",
                [],
            )
            .unwrap();
        }
        
        let sync_scheduler = Arc::new(SyncScheduler::new(db));
        let profile_ids = Arc::new(Mutex::new(vec!["test-profile".to_string()]));
        
        let callback_count = Arc::new(AtomicUsize::new(0));
        let callback_count_clone = Arc::clone(&callback_count);
        
        let on_sync_needed = Arc::new(move |profile_id: String| {
            println!("Sync callback triggered for: {}", profile_id);
            callback_count_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        // Use very short interval for testing (1 second)
        let scheduler = BackgroundScheduler {
            check_interval: Duration::from_secs(1),
            task_handle: Arc::new(Mutex::new(None)),
        };
        
        scheduler.start(sync_scheduler, profile_ids, on_sync_needed).unwrap();
        
        // Wait for at least one check cycle
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        scheduler.stop().unwrap();
        
        // Verify callback was triggered
        let count = callback_count.load(Ordering::SeqCst);
        assert!(count > 0, "Callback should have been triggered at least once");
    }
    
    #[tokio::test]
    async fn test_background_scheduler_respects_auto_sync_disabled() {
        let db = create_test_db();
        
        // Insert test profile with auto-sync disabled
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
                 VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
                [],
            )
            .unwrap();
            
            conn.execute(
                "INSERT INTO xtream_sync_settings (profile_id, auto_sync_enabled) 
                 VALUES ('test-profile', 0)",
                [],
            )
            .unwrap();
        }
        
        let sync_scheduler = Arc::new(SyncScheduler::new(db));
        let profile_ids = Arc::new(Mutex::new(vec!["test-profile".to_string()]));
        
        let callback_count = Arc::new(AtomicUsize::new(0));
        let callback_count_clone = Arc::clone(&callback_count);
        
        let on_sync_needed = Arc::new(move |_profile_id: String| {
            callback_count_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        let scheduler = BackgroundScheduler {
            check_interval: Duration::from_secs(1),
            task_handle: Arc::new(Mutex::new(None)),
        };
        
        scheduler.start(sync_scheduler, profile_ids, on_sync_needed).unwrap();
        
        // Wait for check cycles
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        scheduler.stop().unwrap();
        
        // Verify callback was NOT triggered (auto-sync disabled)
        let count = callback_count.load(Ordering::SeqCst);
        assert_eq!(count, 0, "Callback should not be triggered when auto-sync is disabled");
    }
    
    #[test]
    fn test_wifi_detection() {
        // Just verify the function doesn't panic
        let is_connected = is_wifi_connected();
        // Currently always returns true
        assert!(is_connected);
    }
    
    #[test]
    fn test_sync_notification() {
        // Verify notification doesn't panic
        let result = send_sync_notification("Test Profile", true);
        assert!(result.is_ok());
        
        let result = send_sync_notification("Test Profile", false);
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_background_scheduler_multiple_profiles() {
        let db = create_test_db();
        
        // Insert multiple test profiles
        {
            let conn = db.lock().unwrap();
            for i in 1..=3 {
                let profile_id = format!("test-profile-{}", i);
                conn.execute(
                    "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
                     VALUES (?1, ?2, 'http://test.com', 'user', X'00')",
                    [&profile_id, &format!("Test {}", i)],
                )
                .unwrap();
                
                conn.execute(
                    "INSERT INTO xtream_sync_settings (profile_id, auto_sync_enabled, sync_interval_hours) 
                     VALUES (?1, 1, 6)",
                    [&profile_id],
                )
                .unwrap();
                
                conn.execute(
                    "INSERT INTO xtream_content_sync (profile_id, sync_status) 
                     VALUES (?1, 'pending')",
                    [&profile_id],
                )
                .unwrap();
            }
        }
        
        let sync_scheduler = Arc::new(SyncScheduler::new(db));
        let profile_ids = Arc::new(Mutex::new(vec![
            "test-profile-1".to_string(),
            "test-profile-2".to_string(),
            "test-profile-3".to_string(),
        ]));
        
        let callback_count = Arc::new(AtomicUsize::new(0));
        let callback_count_clone = Arc::clone(&callback_count);
        
        let on_sync_needed = Arc::new(move |_profile_id: String| {
            callback_count_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        let scheduler = BackgroundScheduler {
            check_interval: Duration::from_secs(1),
            task_handle: Arc::new(Mutex::new(None)),
        };
        
        scheduler.start(sync_scheduler, profile_ids, on_sync_needed).unwrap();
        
        // Wait for check cycles
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        scheduler.stop().unwrap();
        
        // Verify callback was triggered for all profiles
        let count = callback_count.load(Ordering::SeqCst);
        assert!(count >= 3, "Callback should be triggered for all 3 profiles");
    }
}
