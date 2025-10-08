use crate::error::{Result, XTauriError};
use crate::xtream::types::{ProfileCredentials, StreamURLRequest, ContentType};
use crate::xtream::content_cache::ContentCache;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use url::Url;
use chrono;

/// Client for interacting with Xtream Codes API
pub struct XtreamClient {
    client: Client,
    base_url: String,
    credentials: ProfileCredentials,
    cache: Arc<ContentCache>,
}

impl XtreamClient {
    /// Create a new Xtream client
    pub fn new(credentials: ProfileCredentials, cache: Arc<ContentCache>) -> Result<Self> {
        Self::new_with_timeout(credentials, cache, Duration::from_secs(30))
    }
    
    /// Create a new Xtream client with custom timeout
    pub fn new_with_timeout(credentials: ProfileCredentials, cache: Arc<ContentCache>, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| XTauriError::internal(format!("Failed to create HTTP client: {}", e)))?;
        
        // Validate and normalize the base URL
        let base_url = Self::normalize_base_url(&credentials.url)?;
        
        Ok(Self {
            client,
            base_url,
            credentials,
            cache,
        })
    }
    
    /// Authenticate with the Xtream server and get profile information
    pub async fn authenticate(&self) -> Result<Value> {
        self.authenticate_with_retry(3).await
    }
    
    /// Authenticate with retry logic for network failures
    pub async fn authenticate_with_retry(&self, max_retries: u32) -> Result<Value> {
        let url = format!(
            "{}/player_api.php?username={}&password={}",
            self.base_url, self.credentials.username, self.credentials.password
        );
        
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match self.try_authenticate(&url).await {
                Ok(profile_data) => return Ok(profile_data),
                Err(e) => {
                    last_error = Some(e);
                    
                    // Don't retry for authentication failures or invalid credentials
                    if let Some(ref err) = last_error {
                        match err {
                            XTauriError::XtreamInvalidCredentials => break,
                            XTauriError::XtreamAuthenticationFailed { .. } => {
                                // Only retry network-related auth failures
                                if !err.to_string().contains("Network error") {
                                    break;
                                }
                            }
                            XTauriError::XtreamApiError { status, .. } => {
                                // Don't retry client errors (4xx), but retry server errors (5xx)
                                if *status < 500 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    
                    // Wait before retrying (exponential backoff)
                    if attempt < max_retries {
                        let delay = Duration::from_millis(1000 * (2_u64.pow(attempt)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| XTauriError::xtream_auth_failed("Authentication failed after retries".to_string())))
    }
    
    /// Single authentication attempt
    async fn try_authenticate(&self, url: &str) -> Result<Value> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    XTauriError::timeout("authentication request")
                } else if e.is_connect() {
                    XTauriError::xtream_auth_failed(format!("Connection failed: {}", e))
                } else {
                    XTauriError::xtream_auth_failed(format!("Network error: {}", e))
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
            
            return Err(XTauriError::xtream_api_error(status.as_u16(), error_message));
        }
        
        let profile_data: Value = response
            .json()
            .await
            .map_err(|e| XTauriError::xtream_auth_failed(format!("Invalid response format: {}", e)))?;
        
        // Check if authentication was successful
        self.validate_auth_response(&profile_data)?;
        
        Ok(profile_data)
    }
    
    /// Validate the authentication response structure
    fn validate_auth_response(&self, profile_data: &Value) -> Result<()> {
        // Check for error messages in response
        if let Some(error) = profile_data.get("error") {
            if let Some(error_str) = error.as_str() {
                return Err(XTauriError::xtream_auth_failed(format!("Server error: {}", error_str)));
            }
        }
        
        // Check user_info structure
        if let Some(user_info) = profile_data.get("user_info") {
            // Check authentication status
            if let Some(auth) = user_info.get("auth") {
                match auth.as_i64() {
                    Some(1) => {
                        // Authentication successful
                        return Ok(());
                    }
                    Some(0) => {
                        return Err(XTauriError::XtreamInvalidCredentials);
                    }
                    _ => {
                        return Err(XTauriError::xtream_auth_failed("Invalid auth status in response".to_string()));
                    }
                }
            }
            
            // Check for account status
            if let Some(status) = user_info.get("status") {
                if let Some(status_str) = status.as_str() {
                    match status_str {
                        "Active" => {}, // Good
                        "Banned" => return Err(XTauriError::xtream_auth_failed("Account is banned".to_string())),
                        "Disabled" => return Err(XTauriError::xtream_auth_failed("Account is disabled".to_string())),
                        "Expired" => return Err(XTauriError::xtream_auth_failed("Account has expired".to_string())),
                        _ => return Err(XTauriError::xtream_auth_failed(format!("Account status: {}", status_str))),
                    }
                }
            }
            
            // Check expiration date
            if let Some(exp_date) = user_info.get("exp_date") {
                if let Some(exp_timestamp) = exp_date.as_i64() {
                    if exp_timestamp > 0 {
                        let exp_datetime = chrono::DateTime::from_timestamp(exp_timestamp, 0);
                        if let Some(exp_dt) = exp_datetime {
                            if exp_dt < chrono::Utc::now() {
                                return Err(XTauriError::xtream_auth_failed("Account has expired".to_string()));
                            }
                        }
                    }
                }
            }
        } else {
            return Err(XTauriError::xtream_auth_failed("Missing user_info in response".to_string()));
        }
        
        Ok(())
    }
    
    /// Get live channel categories
    pub async fn get_channel_categories(&self) -> Result<Value> {
        // Check cache first for categories
        let cache_key = format!("channel_categories_{}", self.credentials.username);
        
        if let Ok(Some(cached_data)) = self.cache.get::<Value>(&cache_key) {
            return Ok(cached_data);
        }
        
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_live_categories",
            self.base_url, self.credentials.username, self.credentials.password
        );
        
        let categories_data = self.make_api_request(&url).await?;
        
        // Parse and enhance category data
        let enhanced_categories = self.parse_and_enhance_categories(&categories_data)?;
        
        // Cache categories for 30 minutes (categories don't change often)
        let cache_ttl = std::time::Duration::from_secs(30 * 60);
        let _ = self.cache.set(&cache_key, &enhanced_categories, Some(cache_ttl));
        
        Ok(enhanced_categories)
    }
    
    /// Get live channels
    pub async fn get_channels(&self, category_id: Option<&str>) -> Result<Value> {
        self.get_channels_with_pagination(category_id, None, None).await
    }
    
    /// Get live channels with pagination support
    pub async fn get_channels_with_pagination(
        &self, 
        category_id: Option<&str>, 
        limit: Option<u32>, 
        offset: Option<u32>
    ) -> Result<Value> {
        // Check cache first for channel data
        let cache_key = format!(
            "channels_{}_{}_{}_{}",
            self.credentials.username,
            category_id.unwrap_or("all"),
            limit.unwrap_or(0),
            offset.unwrap_or(0)
        );
        
        if let Ok(Some(cached_data)) = self.cache.get::<Value>(&cache_key) {
            return Ok(cached_data);
        }
        
        let mut url = format!(
            "{}/player_api.php?username={}&password={}&action=get_live_streams",
            self.base_url, self.credentials.username, self.credentials.password
        );
        
        if let Some(cat_id) = category_id {
            url.push_str(&format!("&category_id={}", cat_id));
        }
        
        if let Some(limit_val) = limit {
            url.push_str(&format!("&limit={}", limit_val));
        }
        
        if let Some(offset_val) = offset {
            url.push_str(&format!("&offset={}", offset_val));
        }
        
        let channels_data = self.make_api_request(&url).await?;
        
        // Parse and enhance channel data with streaming URLs
        let enhanced_channels = self.parse_and_enhance_channels(&channels_data)?;
        
        // Cache channel data for 10 minutes
        let cache_ttl = std::time::Duration::from_secs(10 * 60);
        let _ = self.cache.set(&cache_key, &enhanced_channels, Some(cache_ttl));
        
        Ok(enhanced_channels)
    }
    
    /// Get VOD (movie) categories
    pub async fn get_movie_categories(&self) -> Result<Value> {
        // Check cache first for categories
        let cache_key = format!("movie_categories_{}", self.credentials.username);
        
        if let Ok(Some(cached_data)) = self.cache.get::<Value>(&cache_key) {
            return Ok(cached_data);
        }
        
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_categories",
            self.base_url, self.credentials.username, self.credentials.password
        );
        
        let categories_data = self.make_api_request(&url).await?;
        
        // Parse and enhance category data
        let enhanced_categories = self.parse_and_enhance_categories(&categories_data)?;
        
        // Cache categories for 30 minutes (categories don't change often)
        let cache_ttl = std::time::Duration::from_secs(30 * 60);
        let _ = self.cache.set(&cache_key, &enhanced_categories, Some(cache_ttl));
        
        Ok(enhanced_categories)
    }
    
    /// Get VOD (movies)
    pub async fn get_movies(&self, category_id: Option<&str>) -> Result<Value> {
        self.get_movies_with_pagination(category_id, None, None).await
    }
    
    /// Get VOD (movies) with pagination support
    pub async fn get_movies_with_pagination(
        &self, 
        category_id: Option<&str>, 
        limit: Option<u32>, 
        offset: Option<u32>
    ) -> Result<Value> {
        // Check cache first for movie data
        let cache_key = format!(
            "movies_{}_{}_{}_{}",
            self.credentials.username,
            category_id.unwrap_or("all"),
            limit.unwrap_or(0),
            offset.unwrap_or(0)
        );
        
        if let Ok(Some(cached_data)) = self.cache.get::<Value>(&cache_key) {
            return Ok(cached_data);
        }
        
        let mut url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_streams",
            self.base_url, self.credentials.username, self.credentials.password
        );
        
        if let Some(cat_id) = category_id {
            url.push_str(&format!("&category_id={}", cat_id));
        }
        
        if let Some(limit_val) = limit {
            url.push_str(&format!("&limit={}", limit_val));
        }
        
        if let Some(offset_val) = offset {
            url.push_str(&format!("&offset={}", offset_val));
        }
        
        let movies_data = self.make_api_request(&url).await?;
        
        // Parse and enhance movie data with streaming URLs
        let enhanced_movies = self.parse_and_enhance_movies(&movies_data)?;
        
        // Cache movie data for 15 minutes
        let cache_ttl = std::time::Duration::from_secs(15 * 60);
        let _ = self.cache.set(&cache_key, &enhanced_movies, Some(cache_ttl));
        
        Ok(enhanced_movies)
    }
    
    /// Get movie information with enhanced metadata parsing
    pub async fn get_movie_info(&self, movie_id: &str) -> Result<Value> {
        // Check cache first for movie details
        let cache_key = format!("movie_info_{}_{}", self.credentials.username, movie_id);
        
        if let Ok(Some(cached_data)) = self.cache.get::<Value>(&cache_key) {
            return Ok(cached_data);
        }
        
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_info&vod_id={}",
            self.base_url, self.credentials.username, self.credentials.password, movie_id
        );
        
        let movie_data = self.make_api_request(&url).await?;
        
        // Parse and enhance movie detail data
        let enhanced_movie = self.parse_and_enhance_movie_details(&movie_data, movie_id)?;
        
        // Cache movie details for 60 minutes (details don't change often)
        let cache_ttl = std::time::Duration::from_secs(60 * 60);
        let _ = self.cache.set(&cache_key, &enhanced_movie, Some(cache_ttl));
        
        Ok(enhanced_movie)
    }
    
    /// Get TV series categories
    pub async fn get_series_categories(&self) -> Result<Value> {
        // Check cache first for categories
        let cache_key = format!("series_categories_{}", self.credentials.username);
        
        if let Ok(Some(cached_data)) = self.cache.get::<Value>(&cache_key) {
            return Ok(cached_data);
        }
        
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series_categories",
            self.base_url, self.credentials.username, self.credentials.password
        );
        
        let categories_data = self.make_api_request(&url).await?;
        
        // Parse and enhance category data
        let enhanced_categories = self.parse_and_enhance_categories(&categories_data)?;
        
        // Cache categories for 30 minutes (categories don't change often)
        let cache_ttl = std::time::Duration::from_secs(30 * 60);
        let _ = self.cache.set(&cache_key, &enhanced_categories, Some(cache_ttl));
        
        Ok(enhanced_categories)
    }
    
    /// Get TV series
    pub async fn get_series(&self, category_id: Option<&str>) -> Result<Value> {
        self.get_series_with_pagination(category_id, None, None).await
    }
    
    /// Get TV series with pagination support
    pub async fn get_series_with_pagination(
        &self, 
        category_id: Option<&str>, 
        limit: Option<u32>, 
        offset: Option<u32>
    ) -> Result<Value> {
        // Check cache first for series data
        let cache_key = format!(
            "series_{}_{}_{}_{}",
            self.credentials.username,
            category_id.unwrap_or("all"),
            limit.unwrap_or(0),
            offset.unwrap_or(0)
        );
        
        if let Ok(Some(cached_data)) = self.cache.get::<Value>(&cache_key) {
            return Ok(cached_data);
        }
        
        let mut url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series",
            self.base_url, self.credentials.username, self.credentials.password
        );
        
        if let Some(cat_id) = category_id {
            url.push_str(&format!("&category_id={}", cat_id));
        }
        
        if let Some(limit_val) = limit {
            url.push_str(&format!("&limit={}", limit_val));
        }
        
        if let Some(offset_val) = offset {
            url.push_str(&format!("&offset={}", offset_val));
        }
        
        let series_data = self.make_api_request(&url).await?;
        
        // Parse and enhance series data with streaming URLs
        let enhanced_series = self.parse_and_enhance_series(&series_data)?;
        
        // Cache series data for 15 minutes
        let cache_ttl = std::time::Duration::from_secs(15 * 60);
        let _ = self.cache.set(&cache_key, &enhanced_series, Some(cache_ttl));
        
        Ok(enhanced_series)
    }
    
    /// Get series information with enhanced metadata parsing
    pub async fn get_series_info(&self, series_id: &str) -> Result<Value> {
        // Check cache first for series details
        let cache_key = format!("series_info_{}_{}", self.credentials.username, series_id);
        
        if let Ok(Some(cached_data)) = self.cache.get::<Value>(&cache_key) {
            return Ok(cached_data);
        }
        
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
            self.base_url, self.credentials.username, self.credentials.password, series_id
        );
        
        let series_data = self.make_api_request(&url).await?;
        
        // Parse and enhance series detail data with episode URLs
        let enhanced_series = self.parse_and_enhance_series_details(&series_data, series_id)?;
        
        // Cache series details for 60 minutes (details don't change often)
        let cache_ttl = std::time::Duration::from_secs(60 * 60);
        let _ = self.cache.set(&cache_key, &enhanced_series, Some(cache_ttl));
        
        Ok(enhanced_series)
    }
    
    /// Get short EPG for a channel
    pub async fn get_short_epg(&self, channel_id: &str) -> Result<Value> {
        // Check cache first (EPG data changes frequently, so use shorter TTL)
        let cache_key = format!("epg_short_{}_{}", self.credentials.username, channel_id);
        
        if let Ok(Some(cached_data)) = self.cache.get::<Value>(&cache_key) {
            return Ok(cached_data);
        }
        
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_short_epg&stream_id={}",
            self.base_url, self.credentials.username, self.credentials.password, channel_id
        );
        
        let epg_data = self.make_api_request(&url).await?;
        
        // Cache EPG data for 15 minutes (EPG changes frequently)
        let epg_ttl = std::time::Duration::from_secs(15 * 60);
        let _ = self.cache.set(&cache_key, &epg_data, Some(epg_ttl));
        
        Ok(epg_data)
    }
    
    /// Get full EPG for a channel with date range
    pub async fn get_full_epg(&self, channel_id: &str, start_date: Option<&str>, end_date: Option<&str>) -> Result<Value> {
        // Create cache key including date range
        let date_key = match (start_date, end_date) {
            (Some(start), Some(end)) => format!("_{}_{}", start, end),
            (Some(start), None) => format!("_{}_", start),
            (None, Some(end)) => format!("__{}", end),
            (None, None) => String::new(),
        };
        let cache_key = format!("epg_full_{}_{}_{}", self.credentials.username, channel_id, date_key);
        
        if let Ok(Some(cached_data)) = self.cache.get::<Value>(&cache_key) {
            return Ok(cached_data);
        }
        
        let mut url = format!(
            "{}/player_api.php?username={}&password={}&action=get_simple_data_table&stream_id={}",
            self.base_url, self.credentials.username, self.credentials.password, channel_id
        );
        
        if let Some(start) = start_date {
            url.push_str(&format!("&start={}", start));
        }
        
        if let Some(end) = end_date {
            url.push_str(&format!("&end={}", end));
        }
        
        let epg_data = self.make_api_request(&url).await?;
        
        // Cache full EPG data for 30 minutes
        let epg_ttl = std::time::Duration::from_secs(30 * 60);
        let _ = self.cache.set(&cache_key, &epg_data, Some(epg_ttl));
        
        Ok(epg_data)
    }
    
    /// Get EPG for multiple channels
    pub async fn get_epg_for_channels(&self, channel_ids: &[&str]) -> Result<Value> {
        let channel_list = channel_ids.join(",");
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_short_epg&stream_id={}",
            self.base_url, self.credentials.username, self.credentials.password, channel_list
        );
        
        self.make_api_request(&url).await
    }
    
    /// Get EPG for a specific date range
    pub async fn get_epg_by_date_range(
        &self, 
        channel_id: &str, 
        start_timestamp: u64, 
        end_timestamp: u64
    ) -> Result<Value> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_simple_data_table&stream_id={}&start={}&end={}",
            self.base_url, 
            self.credentials.username, 
            self.credentials.password, 
            channel_id,
            start_timestamp,
            end_timestamp
        );
        
        self.make_api_request(&url).await
    }
    
    /// Format EPG time for display
    pub fn format_epg_time(timestamp: i64, timezone: Option<&str>) -> String {
        use chrono::{DateTime, Utc};
        
        let dt = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now());
        
        // If timezone is provided, try to convert
        if let Some(_tz_str) = timezone {
            // For now, just return UTC time formatted nicely
            // In a full implementation, you'd use a timezone library
            dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
        } else {
            dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
        }
    }
    
    /// Get current timestamp for EPG queries
    pub fn get_current_timestamp() -> u64 {
        use chrono::Utc;
        Utc::now().timestamp() as u64
    }
    
    /// Get timestamp for a specific number of hours from now
    pub fn get_timestamp_hours_from_now(hours: i64) -> u64 {
        use chrono::{Utc, Duration};
        (Utc::now() + Duration::hours(hours)).timestamp() as u64
    }
    
    /// Parse EPG data and extract program information
    pub fn parse_epg_programs(epg_data: &Value) -> Result<Vec<Value>> {
        if let Some(epg_listings) = epg_data.get("epg_listings") {
            if let Some(listings_array) = epg_listings.as_array() {
                return Ok(listings_array.clone());
            }
        }
        
        // If no epg_listings field, return the data as-is if it's an array
        if let Some(array) = epg_data.as_array() {
            Ok(array.clone())
        } else {
            Ok(vec![])
        }
    }
    
    /// Parse and enhance EPG data with formatted times and additional metadata
    pub fn parse_and_enhance_epg_data(epg_data: &Value, timezone: Option<&str>) -> Result<Value> {
        let programs = Self::parse_epg_programs(epg_data)?;
        
        let enhanced_programs: Vec<Value> = programs
            .into_iter()
            .map(|mut program| {
                // Enhance program with formatted times
                if let Some(start_time) = program.get("start").and_then(|s| s.as_str()) {
                    if let Ok(timestamp) = start_time.parse::<i64>() {
                        let formatted_start = Self::format_epg_time(timestamp, timezone);
                        program["formatted_start"] = Value::String(formatted_start);
                        program["start_timestamp"] = Value::Number(serde_json::Number::from(timestamp));
                    }
                }
                
                if let Some(stop_time) = program.get("stop").and_then(|s| s.as_str()) {
                    if let Ok(timestamp) = stop_time.parse::<i64>() {
                        let formatted_stop = Self::format_epg_time(timestamp, timezone);
                        program["formatted_stop"] = Value::String(formatted_stop);
                        program["stop_timestamp"] = Value::Number(serde_json::Number::from(timestamp));
                    }
                }
                
                // Calculate duration if both start and stop are available
                if let (Some(start), Some(stop)) = (
                    program.get("start_timestamp").and_then(|s| s.as_i64()),
                    program.get("stop_timestamp").and_then(|s| s.as_i64())
                ) {
                    let duration_minutes = (stop - start) / 60;
                    program["duration_minutes"] = Value::Number(serde_json::Number::from(duration_minutes));
                    
                    // Format duration as human-readable string
                    let hours = duration_minutes / 60;
                    let minutes = duration_minutes % 60;
                    let duration_str = if hours > 0 {
                        format!("{}h {}m", hours, minutes)
                    } else {
                        format!("{}m", minutes)
                    };
                    program["formatted_duration"] = Value::String(duration_str);
                }
                
                // Add current program indicator
                let now = chrono::Utc::now().timestamp();
                if let (Some(start), Some(stop)) = (
                    program.get("start_timestamp").and_then(|s| s.as_i64()),
                    program.get("stop_timestamp").and_then(|s| s.as_i64())
                ) {
                    program["is_current"] = Value::Bool(now >= start && now <= stop);
                    program["is_past"] = Value::Bool(now > stop);
                    program["is_future"] = Value::Bool(now < start);
                    
                    // Calculate progress percentage for current programs
                    if now >= start && now <= stop {
                        let progress = ((now - start) as f64 / (stop - start) as f64 * 100.0).round() as u64;
                        program["progress_percent"] = Value::Number(serde_json::Number::from(progress));
                    }
                }
                
                // Ensure required fields have default values
                if !program.as_object().unwrap().contains_key("title") {
                    program["title"] = Value::String("Unknown Program".to_string());
                }
                
                if !program.as_object().unwrap().contains_key("description") {
                    program["description"] = Value::String("".to_string());
                }
                
                program
            })
            .collect();
        
        Ok(Value::Array(enhanced_programs))
    }
    
    /// Get EPG data for current and next programs on a channel
    pub async fn get_current_and_next_epg(&self, channel_id: &str) -> Result<Value> {
        let now = Self::get_current_timestamp();
        let next_6_hours = Self::get_timestamp_hours_from_now(6);
        
        let epg_data = self.get_epg_by_date_range(channel_id, now, next_6_hours).await?;
        let enhanced_epg = Self::parse_and_enhance_epg_data(&epg_data, None)?;
        
        if let Some(programs) = enhanced_epg.as_array() {
            let mut current_program = None;
            let mut next_program = None;
            
            for program in programs {
                if let Some(is_current) = program.get("is_current").and_then(|c| c.as_bool()) {
                    if is_current && current_program.is_none() {
                        current_program = Some(program.clone());
                    }
                }
                
                if let Some(is_future) = program.get("is_future").and_then(|f| f.as_bool()) {
                    if is_future && next_program.is_none() {
                        next_program = Some(program.clone());
                    }
                }
            }
            
            let result = serde_json::json!({
                "current": current_program,
                "next": next_program,
                "all_programs": programs
            });
            
            Ok(result)
        } else {
            Ok(serde_json::json!({
                "current": null,
                "next": null,
                "all_programs": []
            }))
        }
    }
    
    /// Filter EPG programs by time range
    pub fn filter_epg_by_time_range(
        epg_data: &Value,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>
    ) -> Result<Value> {
        if let Some(programs) = epg_data.as_array() {
            let filtered_programs: Vec<Value> = programs
                .iter()
                .filter(|program| {
                    let program_start = program
                        .get("start_timestamp")
                        .and_then(|s| s.as_i64())
                        .or_else(|| {
                            program.get("start")
                                .and_then(|s| s.as_str())
                                .and_then(|s| s.parse::<i64>().ok())
                        });
                    
                    let program_stop = program
                        .get("stop_timestamp")
                        .and_then(|s| s.as_i64())
                        .or_else(|| {
                            program.get("stop")
                                .and_then(|s| s.as_str())
                                .and_then(|s| s.parse::<i64>().ok())
                        });
                    
                    // Check if program overlaps with the requested time range
                    match (program_start, program_stop, start_timestamp, end_timestamp) {
                        (Some(p_start), Some(p_stop), Some(range_start), Some(range_end)) => {
                            // Program overlaps if it starts before range ends and ends after range starts
                            p_start < range_end && p_stop > range_start
                        }
                        (Some(p_start), Some(_p_stop), Some(range_start), None) => {
                            p_start >= range_start
                        }
                        (Some(_p_start), Some(p_stop), None, Some(range_end)) => {
                            p_stop <= range_end
                        }
                        _ => true, // Include if we can't determine times
                    }
                })
                .cloned()
                .collect();
            
            Ok(Value::Array(filtered_programs))
        } else {
            Ok(epg_data.clone())
        }
    }
    
    /// Search EPG programs by title or description
    pub fn search_epg_programs(epg_data: &Value, search_query: &str) -> Result<Value> {
        if let Some(programs) = epg_data.as_array() {
            let search_lower = search_query.to_lowercase();
            
            let matching_programs: Vec<Value> = programs
                .iter()
                .filter(|program| {
                    let title_match = program
                        .get("title")
                        .and_then(|t| t.as_str())
                        .map(|title| title.to_lowercase().contains(&search_lower))
                        .unwrap_or(false);
                    
                    let desc_match = program
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|desc| desc.to_lowercase().contains(&search_lower))
                        .unwrap_or(false);
                    
                    title_match || desc_match
                })
                .cloned()
                .collect();
            
            Ok(Value::Array(matching_programs))
        } else {
            Ok(epg_data.clone())
        }
    }
    
    /// Parse and enhance channel data with streaming URLs and additional metadata
    pub fn parse_and_enhance_channels(&self, channels_data: &Value) -> Result<Value> {
        if let Some(channels_array) = channels_data.as_array() {
            let enhanced_channels: Result<Vec<Value>> = channels_array
                .iter()
                .map(|channel| self.enhance_channel_data(channel))
                .collect();
            
            Ok(Value::Array(enhanced_channels?))
        } else {
            // Return as-is if not an array
            Ok(channels_data.clone())
        }
    }
    
    /// Enhance individual channel data with streaming URL and parsed metadata
    fn enhance_channel_data(&self, channel: &Value) -> Result<Value> {
        let mut enhanced_channel = channel.clone();
        
        // Extract stream_id for URL generation
        if let Some(stream_id) = channel.get("stream_id") {
            if let Some(id_num) = stream_id.as_u64() {
                // Generate streaming URL for the channel
                // Use m3u8 for live channels to ensure browser compatibility
                let stream_request = StreamURLRequest {
                    content_type: ContentType::Channel,
                    content_id: id_num.to_string(),
                    extension: Some("m3u8".to_string()),
                };
                
                if let Ok(stream_url) = self.generate_stream_url(&stream_request) {
                    enhanced_channel["url"] = Value::String(stream_url);
                }
            }
        }
        
        // Normalize and validate channel metadata
        if let Some(channel_obj) = enhanced_channel.as_object_mut() {
            // Ensure required fields have default values
            if !channel_obj.contains_key("stream_icon") {
                channel_obj.insert("stream_icon".to_string(), Value::String("".to_string()));
            }
            
            if !channel_obj.contains_key("epg_channel_id") {
                channel_obj.insert("epg_channel_id".to_string(), Value::String("".to_string()));
            }
            
            if !channel_obj.contains_key("category_ids") {
                if let Some(category_id) = channel_obj.get("category_id") {
                    if let Some(cat_id_str) = category_id.as_str() {
                        if let Ok(cat_id_num) = cat_id_str.parse::<u64>() {
                            channel_obj.insert("category_ids".to_string(), Value::Array(vec![Value::Number(serde_json::Number::from(cat_id_num))]));
                        }
                    }
                }
            }
            
            // Ensure tv_archive is a number
            if let Some(tv_archive) = channel_obj.get("tv_archive") {
                if tv_archive.is_string() {
                    if let Some(archive_str) = tv_archive.as_str() {
                        if let Ok(archive_num) = archive_str.parse::<u64>() {
                            channel_obj.insert("tv_archive".to_string(), Value::Number(serde_json::Number::from(archive_num)));
                        }
                    }
                }
            }
        }
        
        Ok(enhanced_channel)
    }
    
    /// Filter channels by various criteria
    pub fn filter_channels(
        channels: &Value,
        name_filter: Option<&str>,
        category_filter: Option<&str>,
        has_epg: Option<bool>,
        has_archive: Option<bool>,
    ) -> Result<Value> {
        if let Some(channels_array) = channels.as_array() {
            let filtered_channels: Vec<Value> = channels_array
                .iter()
                .filter(|channel| {
                    // Filter by name (case-insensitive partial match)
                    if let Some(name_query) = name_filter {
                        if let Some(channel_name) = channel.get("name").and_then(|n| n.as_str()) {
                            if !channel_name.to_lowercase().contains(&name_query.to_lowercase()) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    
                    // Filter by category
                    if let Some(category_query) = category_filter {
                        if let Some(category_id) = channel.get("category_id").and_then(|c| c.as_str()) {
                            if category_id != category_query {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    
                    // Filter by EPG availability
                    if let Some(epg_required) = has_epg {
                        let has_epg_data = channel
                            .get("epg_channel_id")
                            .and_then(|epg| epg.as_str())
                            .map(|epg_id| !epg_id.is_empty())
                            .unwrap_or(false);
                        
                        if epg_required && !has_epg_data {
                            return false;
                        }
                        if !epg_required && has_epg_data {
                            return false;
                        }
                    }
                    
                    // Filter by archive availability
                    if let Some(archive_required) = has_archive {
                        let has_archive_data = channel
                            .get("tv_archive")
                            .and_then(|archive| archive.as_u64())
                            .map(|archive_val| archive_val > 0)
                            .unwrap_or(false);
                        
                        if archive_required && !has_archive_data {
                            return false;
                        }
                        if !archive_required && has_archive_data {
                            return false;
                        }
                    }
                    
                    true
                })
                .cloned()
                .collect();
            
            Ok(Value::Array(filtered_channels))
        } else {
            Ok(channels.clone())
        }
    }
    
    /// Sort channels by various criteria
    pub fn sort_channels(channels: &Value, sort_by: &str, ascending: bool) -> Result<Value> {
        if let Some(channels_array) = channels.as_array() {
            let mut sorted_channels = channels_array.clone();
            
            sorted_channels.sort_by(|a, b| {
                let comparison = match sort_by {
                    "name" => {
                        let name_a = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        let name_b = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        name_a.cmp(name_b)
                    }
                    "num" => {
                        let num_a = a.get("num").and_then(|n| n.as_u64()).unwrap_or(0);
                        let num_b = b.get("num").and_then(|n| n.as_u64()).unwrap_or(0);
                        num_a.cmp(&num_b)
                    }
                    "category_id" => {
                        let cat_a = a.get("category_id").and_then(|c| c.as_str()).unwrap_or("");
                        let cat_b = b.get("category_id").and_then(|c| c.as_str()).unwrap_or("");
                        cat_a.cmp(cat_b)
                    }
                    "added" => {
                        let added_a = a.get("added").and_then(|d| d.as_str()).unwrap_or("");
                        let added_b = b.get("added").and_then(|d| d.as_str()).unwrap_or("");
                        added_a.cmp(added_b)
                    }
                    _ => std::cmp::Ordering::Equal,
                };
                
                if ascending {
                    comparison
                } else {
                    comparison.reverse()
                }
            });
            
            Ok(Value::Array(sorted_channels))
        } else {
            Ok(channels.clone())
        }
    }
    
    /// Parse and enhance category data with additional metadata
    pub fn parse_and_enhance_categories(&self, categories_data: &Value) -> Result<Value> {
        if let Some(categories_array) = categories_data.as_array() {
            let enhanced_categories: Result<Vec<Value>> = categories_array
                .iter()
                .map(|category| self.enhance_category_data(category))
                .collect();
            
            Ok(Value::Array(enhanced_categories?))
        } else {
            // Return as-is if not an array
            Ok(categories_data.clone())
        }
    }
    
    /// Enhance individual category data with normalized metadata
    fn enhance_category_data(&self, category: &Value) -> Result<Value> {
        let mut enhanced_category = category.clone();
        
        // Normalize and validate category metadata
        if let Some(category_obj) = enhanced_category.as_object_mut() {
            // Ensure required fields have default values
            if !category_obj.contains_key("category_name") {
                category_obj.insert("category_name".to_string(), Value::String("Unknown Category".to_string()));
            }
            
            // Ensure parent_id is a number
            if let Some(parent_id) = category_obj.get("parent_id") {
                if parent_id.is_string() {
                    if let Some(parent_str) = parent_id.as_str() {
                        if let Ok(parent_num) = parent_str.parse::<u64>() {
                            category_obj.insert("parent_id".to_string(), Value::Number(serde_json::Number::from(parent_num)));
                        }
                    }
                }
            } else {
                // Default parent_id to 0 if not present
                category_obj.insert("parent_id".to_string(), Value::Number(serde_json::Number::from(0u64)));
            }
        }
        
        Ok(enhanced_category)
    }
    
    /// Get channel count for each category
    pub async fn get_channel_counts_by_category(&self) -> Result<Value> {
        // This would require fetching all channels and counting by category
        // For now, we'll return an empty object and let the frontend handle counting
        Ok(Value::Object(serde_json::Map::new()))
    }
    
    /// Validate channel data structure
    pub fn validate_channel_data(channel: &Value) -> bool {
        // Check for required fields
        let required_fields = ["stream_id", "name"];
        
        for field in &required_fields {
            if !channel.get(field).is_some() {
                return false;
            }
        }
        
        // Validate stream_id is a number
        if let Some(stream_id) = channel.get("stream_id") {
            if !stream_id.is_number() && !stream_id.is_string() {
                return false;
            }
        }
        
        true
    }
    
    /// Parse and enhance movie data with streaming URLs and additional metadata
    pub fn parse_and_enhance_movies(&self, movies_data: &Value) -> Result<Value> {
        if let Some(movies_array) = movies_data.as_array() {
            let enhanced_movies: Result<Vec<Value>> = movies_array
                .iter()
                .map(|movie| self.enhance_movie_data(movie))
                .collect();
            
            Ok(Value::Array(enhanced_movies?))
        } else {
            // Return as-is if not an array
            Ok(movies_data.clone())
        }
    }
    
    /// Enhance individual movie data with streaming URL and parsed metadata
    fn enhance_movie_data(&self, movie: &Value) -> Result<Value> {
        let mut enhanced_movie = movie.clone();
        
        // Extract stream_id for URL generation
        if let Some(stream_id) = movie.get("stream_id") {
            if let Some(id_num) = stream_id.as_u64() {
                // Generate streaming URL for the movie
                let stream_request = StreamURLRequest {
                    content_type: ContentType::Movie,
                    content_id: id_num.to_string(),
                    extension: Some("mp4".to_string()),
                };
                
                if let Ok(stream_url) = self.generate_stream_url(&stream_request) {
                    enhanced_movie["url"] = Value::String(stream_url);
                }
            }
        }
        
        // Normalize and validate movie metadata
        if let Some(movie_obj) = enhanced_movie.as_object_mut() {
            // Ensure required fields have default values
            if !movie_obj.contains_key("stream_icon") {
                movie_obj.insert("stream_icon".to_string(), Value::String("".to_string()));
            }
            
            if !movie_obj.contains_key("rating") {
                movie_obj.insert("rating".to_string(), Value::String("0".to_string()));
            }
            
            if !movie_obj.contains_key("rating_5based") {
                movie_obj.insert("rating_5based".to_string(), Value::Number(serde_json::Number::from_f64(0.0).unwrap_or(serde_json::Number::from(0))));
            }
            
            if !movie_obj.contains_key("genre") {
                movie_obj.insert("genre".to_string(), Value::String("".to_string()));
            }
            
            if !movie_obj.contains_key("plot") {
                movie_obj.insert("plot".to_string(), Value::String("".to_string()));
            }
            
            if !movie_obj.contains_key("duration_secs") {
                movie_obj.insert("duration_secs".to_string(), Value::Number(serde_json::Number::from(0)));
            }
            
            // Ensure category_ids is an array
            if !movie_obj.contains_key("category_ids") {
                if let Some(category_id) = movie_obj.get("category_id") {
                    if let Some(cat_id_str) = category_id.as_str() {
                        if let Ok(cat_id_num) = cat_id_str.parse::<u64>() {
                            movie_obj.insert("category_ids".to_string(), Value::Array(vec![Value::Number(serde_json::Number::from(cat_id_num))]));
                        }
                    }
                }
            }
            
            // Parse and normalize rating
            if let Some(rating) = movie_obj.get("rating") {
                if let Some(rating_str) = rating.as_str() {
                    if let Ok(rating_float) = rating_str.parse::<f64>() {
                        // Convert to 5-based rating if it's out of 10
                        let rating_5based = if rating_float > 5.0 { rating_float / 2.0 } else { rating_float };
                        movie_obj.insert("rating_5based".to_string(), Value::Number(serde_json::Number::from_f64(rating_5based).unwrap_or(serde_json::Number::from(0))));
                    }
                }
            }
        }
        
        Ok(enhanced_movie)
    }
    
    /// Parse and enhance movie detail data with comprehensive metadata
    pub fn parse_and_enhance_movie_details(&self, movie_data: &Value, movie_id: &str) -> Result<Value> {
        let mut enhanced_movie = movie_data.clone();
        
        // Generate streaming URL for the movie
        let stream_request = StreamURLRequest {
            content_type: ContentType::Movie,
            content_id: movie_id.to_string(),
            extension: Some("mp4".to_string()),
        };
        
        if let Ok(stream_url) = self.generate_stream_url(&stream_request) {
            enhanced_movie["url"] = Value::String(stream_url);
        }
        
        // Extract and enhance movie info if it's nested
        if let Some(movie_info) = movie_data.get("info") {
            if let Some(movie_obj) = enhanced_movie.as_object_mut() {
                // Merge info fields into the main object
                if let Some(info_obj) = movie_info.as_object() {
                    for (key, value) in info_obj {
                        movie_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        
        // Extract and enhance movie data if it's nested
        if let Some(movie_data_nested) = movie_data.get("movie_data") {
            if let Some(movie_obj) = enhanced_movie.as_object_mut() {
                // Merge movie_data fields into the main object
                if let Some(data_obj) = movie_data_nested.as_object() {
                    for (key, value) in data_obj {
                        movie_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        
        // Normalize and validate comprehensive movie metadata
        if let Some(movie_obj) = enhanced_movie.as_object_mut() {
            // Ensure all metadata fields have proper defaults
            let default_fields = [
                ("name", Value::String("Unknown Movie".to_string())),
                ("stream_icon", Value::String("".to_string())),
                ("rating", Value::String("0".to_string())),
                ("rating_5based", Value::Number(serde_json::Number::from_f64(0.0).unwrap_or(serde_json::Number::from(0)))),
                ("genre", Value::String("".to_string())),
                ("plot", Value::String("".to_string())),
                ("cast", Value::String("".to_string())),
                ("director", Value::String("".to_string())),
                ("releasedate", Value::String("".to_string())),
                ("country", Value::String("".to_string())),
                ("duration_secs", Value::Number(serde_json::Number::from(0))),
                ("duration", Value::String("".to_string())),
                ("youtube_trailer", Value::String("".to_string())),
                ("quality", Value::String("".to_string())),
                ("container_extension", Value::String("mp4".to_string())),
            ];
            
            for (field, default_value) in &default_fields {
                if !movie_obj.contains_key(*field) {
                    movie_obj.insert(field.to_string(), default_value.clone());
                }
            }
            
            // Parse and enhance rating information
            if let Some(rating) = movie_obj.get("rating") {
                if let Some(rating_str) = rating.as_str() {
                    if let Ok(rating_float) = rating_str.parse::<f64>() {
                        // Convert to 5-based rating if it's out of 10
                        let rating_5based = if rating_float > 5.0 { rating_float / 2.0 } else { rating_float };
                        movie_obj.insert("rating_5based".to_string(), Value::Number(serde_json::Number::from_f64(rating_5based).unwrap_or(serde_json::Number::from(0))));
                    }
                }
            }
            
            // Parse duration from seconds to human readable format
            if let Some(duration_secs) = movie_obj.get("duration_secs") {
                if let Some(secs) = duration_secs.as_u64() {
                    let hours = secs / 3600;
                    let minutes = (secs % 3600) / 60;
                    let duration_str = if hours > 0 {
                        format!("{}h {}m", hours, minutes)
                    } else {
                        format!("{}m", minutes)
                    };
                    movie_obj.insert("duration".to_string(), Value::String(duration_str));
                }
            }
            
            // Parse release date and format it properly
            if let Some(release_date) = movie_obj.get("releasedate") {
                if let Some(date_str) = release_date.as_str() {
                    // Try to parse and reformat the date
                    if let Ok(timestamp) = date_str.parse::<i64>() {
                        if let Some(dt) = chrono::DateTime::from_timestamp(timestamp, 0) {
                            let formatted_date = dt.format("%Y-%m-%d").to_string();
                            movie_obj.insert("releasedate".to_string(), Value::String(formatted_date));
                        }
                    }
                }
            }
        }
        
        Ok(enhanced_movie)
    }
    
    /// Filter movies by various criteria
    pub fn filter_movies(
        movies: &Value,
        name_filter: Option<&str>,
        category_filter: Option<&str>,
        genre_filter: Option<&str>,
        rating_min: Option<f64>,
        year_filter: Option<&str>,
    ) -> Result<Value> {
        if let Some(movies_array) = movies.as_array() {
            let filtered_movies: Vec<Value> = movies_array
                .iter()
                .filter(|movie| {
                    // Filter by name (case-insensitive partial match)
                    if let Some(name_query) = name_filter {
                        if let Some(movie_name) = movie.get("name").and_then(|n| n.as_str()) {
                            if !movie_name.to_lowercase().contains(&name_query.to_lowercase()) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    
                    // Filter by category
                    if let Some(category_query) = category_filter {
                        if let Some(category_id) = movie.get("category_id").and_then(|c| c.as_str()) {
                            if category_id != category_query {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    
                    // Filter by genre (case-insensitive partial match)
                    if let Some(genre_query) = genre_filter {
                        if let Some(movie_genre) = movie.get("genre").and_then(|g| g.as_str()) {
                            if !movie_genre.to_lowercase().contains(&genre_query.to_lowercase()) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    
                    // Filter by minimum rating
                    if let Some(min_rating) = rating_min {
                        let movie_rating = movie
                            .get("rating_5based")
                            .and_then(|r| r.as_f64())
                            .unwrap_or(0.0);
                        
                        if movie_rating < min_rating {
                            return false;
                        }
                    }
                    
                    // Filter by release year
                    if let Some(year_query) = year_filter {
                        if let Some(release_date) = movie.get("releasedate").and_then(|d| d.as_str()) {
                            if !release_date.starts_with(year_query) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    
                    true
                })
                .cloned()
                .collect();
            
            Ok(Value::Array(filtered_movies))
        } else {
            Ok(movies.clone())
        }
    }
    
    /// Sort movies by various criteria
    pub fn sort_movies(movies: &Value, sort_by: &str, ascending: bool) -> Result<Value> {
        if let Some(movies_array) = movies.as_array() {
            let mut sorted_movies = movies_array.clone();
            
            sorted_movies.sort_by(|a, b| {
                let comparison = match sort_by {
                    "name" => {
                        let name_a = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        let name_b = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        name_a.cmp(name_b)
                    }
                    "rating" => {
                        let rating_a = a.get("rating_5based").and_then(|r| r.as_f64()).unwrap_or(0.0);
                        let rating_b = b.get("rating_5based").and_then(|r| r.as_f64()).unwrap_or(0.0);
                        rating_a.partial_cmp(&rating_b).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    "releasedate" => {
                        let date_a = a.get("releasedate").and_then(|d| d.as_str()).unwrap_or("");
                        let date_b = b.get("releasedate").and_then(|d| d.as_str()).unwrap_or("");
                        date_a.cmp(date_b)
                    }
                    "duration" => {
                        let duration_a = a.get("duration_secs").and_then(|d| d.as_u64()).unwrap_or(0);
                        let duration_b = b.get("duration_secs").and_then(|d| d.as_u64()).unwrap_or(0);
                        duration_a.cmp(&duration_b)
                    }
                    "added" => {
                        let added_a = a.get("added").and_then(|d| d.as_str()).unwrap_or("");
                        let added_b = b.get("added").and_then(|d| d.as_str()).unwrap_or("");
                        added_a.cmp(added_b)
                    }
                    _ => std::cmp::Ordering::Equal,
                };
                
                if ascending {
                    comparison
                } else {
                    comparison.reverse()
                }
            });
            
            Ok(Value::Array(sorted_movies))
        } else {
            Ok(movies.clone())
        }
    }
    
    /// Validate movie data structure
    pub fn validate_movie_data(movie: &Value) -> bool {
        // Check for required fields
        let required_fields = ["stream_id", "name"];
        
        for field in &required_fields {
            if !movie.get(field).is_some() {
                return false;
            }
        }
        
        // Validate stream_id is a number
        if let Some(stream_id) = movie.get("stream_id") {
            if !stream_id.is_number() && !stream_id.is_string() {
                return false;
            }
        }
        
        true
    }
    
    /// Parse and enhance series data with streaming URLs and additional metadata
    pub fn parse_and_enhance_series(&self, series_data: &Value) -> Result<Value> {
        if let Some(series_array) = series_data.as_array() {
            let enhanced_series: Result<Vec<Value>> = series_array
                .iter()
                .map(|series| self.enhance_series_data(series))
                .collect();
            
            Ok(Value::Array(enhanced_series?))
        } else {
            // Return as-is if not an array
            Ok(series_data.clone())
        }
    }
    
    /// Enhance individual series data with streaming URL and parsed metadata
    fn enhance_series_data(&self, series: &Value) -> Result<Value> {
        let mut enhanced_series = series.clone();
        
        // Extract series_id for URL generation
        if let Some(series_id) = series.get("series_id") {
            if let Some(id_num) = series_id.as_u64() {
                // Generate base streaming URL for the series
                let stream_request = StreamURLRequest {
                    content_type: ContentType::Series,
                    content_id: id_num.to_string(),
                    extension: Some("mp4".to_string()),
                };
                
                if let Ok(stream_url) = self.generate_stream_url(&stream_request) {
                    enhanced_series["base_url"] = Value::String(stream_url);
                }
            }
        }
        
        // Normalize and validate series metadata
        if let Some(series_obj) = enhanced_series.as_object_mut() {
            // Ensure required fields have default values
            if !series_obj.contains_key("cover") {
                series_obj.insert("cover".to_string(), Value::String("".to_string()));
            }
            
            if !series_obj.contains_key("plot") {
                series_obj.insert("plot".to_string(), Value::String("".to_string()));
            }
            
            if !series_obj.contains_key("cast") {
                series_obj.insert("cast".to_string(), Value::String("".to_string()));
            }
            
            if !series_obj.contains_key("director") {
                series_obj.insert("director".to_string(), Value::String("".to_string()));
            }
            
            if !series_obj.contains_key("genre") {
                series_obj.insert("genre".to_string(), Value::String("".to_string()));
            }
            
            if !series_obj.contains_key("releaseDate") {
                series_obj.insert("releaseDate".to_string(), Value::String("".to_string()));
            }
            
            if !series_obj.contains_key("rating") {
                series_obj.insert("rating".to_string(), Value::String("0".to_string()));
            }
            
            if !series_obj.contains_key("rating_5based") {
                series_obj.insert("rating_5based".to_string(), Value::Number(serde_json::Number::from_f64(0.0).unwrap_or(serde_json::Number::from(0))));
            }
            
            // Ensure category_ids is an array
            if !series_obj.contains_key("category_ids") {
                if let Some(category_id) = series_obj.get("category_id") {
                    if let Some(cat_id_str) = category_id.as_str() {
                        if let Ok(cat_id_num) = cat_id_str.parse::<u64>() {
                            series_obj.insert("category_ids".to_string(), Value::Array(vec![Value::Number(serde_json::Number::from(cat_id_num))]));
                        }
                    }
                }
            }
            
            // Parse and normalize rating
            if let Some(rating) = series_obj.get("rating") {
                if let Some(rating_str) = rating.as_str() {
                    if let Ok(rating_float) = rating_str.parse::<f64>() {
                        // Convert to 5-based rating if it's out of 10
                        let rating_5based = if rating_float > 5.0 { rating_float / 2.0 } else { rating_float };
                        series_obj.insert("rating_5based".to_string(), Value::Number(serde_json::Number::from_f64(rating_5based).unwrap_or(serde_json::Number::from(0))));
                    }
                }
            }
        }
        
        Ok(enhanced_series)
    }
    
    /// Parse and enhance series detail data with comprehensive metadata and episode information
    pub fn parse_and_enhance_series_details(&self, series_data: &Value, series_id: &str) -> Result<Value> {
        let mut enhanced_series = series_data.clone();
        
        // Extract and enhance series info if it's nested
        if let Some(info) = series_data.get("info") {
            if let Some(series_obj) = enhanced_series.as_object_mut() {
                // Merge info fields into the main object
                if let Some(info_obj) = info.as_object() {
                    for (key, value) in info_obj {
                        series_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        
        // Process seasons and episodes data
        if let Some(seasons) = series_data.get("seasons") {
            if let Some(series_obj) = enhanced_series.as_object_mut() {
                let enhanced_seasons = self.enhance_seasons_data(seasons, series_id)?;
                series_obj.insert("seasons".to_string(), enhanced_seasons);
            }
        }
        
        // Process episodes data if present at root level
        if let Some(episodes) = series_data.get("episodes") {
            if let Some(series_obj) = enhanced_series.as_object_mut() {
                let enhanced_episodes = self.enhance_episodes_data(episodes, series_id)?;
                series_obj.insert("episodes".to_string(), enhanced_episodes);
            }
        }
        
        // Normalize and validate comprehensive series metadata
        if let Some(series_obj) = enhanced_series.as_object_mut() {
            // Ensure all metadata fields have proper defaults
            let default_fields = [
                ("name", Value::String("Unknown Series".to_string())),
                ("cover", Value::String("".to_string())),
                ("plot", Value::String("".to_string())),
                ("cast", Value::String("".to_string())),
                ("director", Value::String("".to_string())),
                ("genre", Value::String("".to_string())),
                ("releaseDate", Value::String("".to_string())),
                ("last_modified", Value::String("".to_string())),
                ("rating", Value::String("0".to_string())),
                ("rating_5based", Value::Number(serde_json::Number::from_f64(0.0).unwrap_or(serde_json::Number::from(0)))),
                ("youtube_trailer", Value::String("".to_string())),
                ("episode_run_time", Value::String("".to_string())),
                ("category_id", Value::String("0".to_string())),
            ];
            
            for (field, default_value) in &default_fields {
                if !series_obj.contains_key(*field) {
                    series_obj.insert(field.to_string(), default_value.clone());
                }
            }
            
            // Parse and enhance rating information
            if let Some(rating) = series_obj.get("rating") {
                if let Some(rating_str) = rating.as_str() {
                    if let Ok(rating_float) = rating_str.parse::<f64>() {
                        // Convert to 5-based rating if it's out of 10
                        let rating_5based = if rating_float > 5.0 { rating_float / 2.0 } else { rating_float };
                        series_obj.insert("rating_5based".to_string(), Value::Number(serde_json::Number::from_f64(rating_5based).unwrap_or(serde_json::Number::from(0))));
                    }
                }
            }
            
            // Parse release date and format it properly
            if let Some(release_date) = series_obj.get("releaseDate") {
                if let Some(date_str) = release_date.as_str() {
                    // Try to parse and reformat the date
                    if let Ok(timestamp) = date_str.parse::<i64>() {
                        if let Some(dt) = chrono::DateTime::from_timestamp(timestamp, 0) {
                            let formatted_date = dt.format("%Y-%m-%d").to_string();
                            series_obj.insert("releaseDate".to_string(), Value::String(formatted_date));
                        }
                    }
                }
            }
        }
        
        Ok(enhanced_series)
    }
    
    /// Enhance seasons data with episode information and streaming URLs
    fn enhance_seasons_data(&self, seasons: &Value, series_id: &str) -> Result<Value> {
        if let Some(seasons_array) = seasons.as_array() {
            let enhanced_seasons: Result<Vec<Value>> = seasons_array
                .iter()
                .map(|season| self.enhance_season_data(season, series_id))
                .collect();
            
            Ok(Value::Array(enhanced_seasons?))
        } else if let Some(seasons_obj) = seasons.as_object() {
            // Handle seasons as object with season numbers as keys
            let mut enhanced_seasons_obj = serde_json::Map::new();
            
            for (season_num, episodes) in seasons_obj {
                let enhanced_episodes = self.enhance_episodes_data(episodes, series_id)?;
                enhanced_seasons_obj.insert(season_num.clone(), enhanced_episodes);
            }
            
            Ok(Value::Object(enhanced_seasons_obj))
        } else {
            Ok(seasons.clone())
        }
    }
    
    /// Enhance individual season data
    fn enhance_season_data(&self, season: &Value, series_id: &str) -> Result<Value> {
        let mut enhanced_season = season.clone();
        
        // Process episodes within the season
        if let Some(episodes) = season.get("episodes") {
            if let Some(season_obj) = enhanced_season.as_object_mut() {
                let enhanced_episodes = self.enhance_episodes_data(episodes, series_id)?;
                season_obj.insert("episodes".to_string(), enhanced_episodes);
            }
        }
        
        Ok(enhanced_season)
    }
    
    /// Enhance episodes data with streaming URLs and metadata
    fn enhance_episodes_data(&self, episodes: &Value, series_id: &str) -> Result<Value> {
        if let Some(episodes_array) = episodes.as_array() {
            let enhanced_episodes: Result<Vec<Value>> = episodes_array
                .iter()
                .map(|episode| self.enhance_episode_data(episode, series_id))
                .collect();
            
            Ok(Value::Array(enhanced_episodes?))
        } else if let Some(episodes_obj) = episodes.as_object() {
            // Handle episodes as object with episode IDs as keys
            let mut enhanced_episodes_obj = serde_json::Map::new();
            
            for (episode_id, episode_data) in episodes_obj {
                let enhanced_episode = self.enhance_episode_data(episode_data, series_id)?;
                enhanced_episodes_obj.insert(episode_id.clone(), enhanced_episode);
            }
            
            Ok(Value::Object(enhanced_episodes_obj))
        } else {
            Ok(episodes.clone())
        }
    }
    
    /// Enhance individual episode data with streaming URL and metadata
    fn enhance_episode_data(&self, episode: &Value, _series_id: &str) -> Result<Value> {
        let mut enhanced_episode = episode.clone();
        
        // Generate streaming URL for the episode
        if let Some(id) = episode.get("id") {
            if let Some(id_str) = id.as_str() {
                let stream_request = StreamURLRequest {
                    content_type: ContentType::Series,
                    content_id: id_str.to_string(),
                    extension: Some("mp4".to_string()),
                };
                
                if let Ok(stream_url) = self.generate_stream_url(&stream_request) {
                    enhanced_episode["url"] = Value::String(stream_url);
                }
            }
        }
        
        // Normalize and validate episode metadata
        if let Some(episode_obj) = enhanced_episode.as_object_mut() {
            // Ensure required fields have default values
            let default_fields = [
                ("title", Value::String("Unknown Episode".to_string())),
                ("info", Value::String("".to_string())),
                ("plot", Value::String("".to_string())),
                ("duration_secs", Value::Number(serde_json::Number::from(0))),
                ("duration", Value::String("".to_string())),
                ("movie_image", Value::String("".to_string())),
                ("releasedate", Value::String("".to_string())),
                ("rating", Value::String("0".to_string())),
                ("season", Value::Number(serde_json::Number::from(1))),
                ("episode_num", Value::Number(serde_json::Number::from(1))),
                ("container_extension", Value::String("mp4".to_string())),
            ];
            
            for (field, default_value) in &default_fields {
                if !episode_obj.contains_key(*field) {
                    episode_obj.insert(field.to_string(), default_value.clone());
                }
            }
            
            // Parse duration from seconds to human readable format
            if let Some(duration_secs) = episode_obj.get("duration_secs") {
                if let Some(secs) = duration_secs.as_u64() {
                    let hours = secs / 3600;
                    let minutes = (secs % 3600) / 60;
                    let duration_str = if hours > 0 {
                        format!("{}h {}m", hours, minutes)
                    } else {
                        format!("{}m", minutes)
                    };
                    episode_obj.insert("duration".to_string(), Value::String(duration_str));
                }
            }
            
            // Parse release date and format it properly
            if let Some(release_date) = episode_obj.get("releasedate") {
                if let Some(date_str) = release_date.as_str() {
                    // Try to parse and reformat the date
                    if let Ok(timestamp) = date_str.parse::<i64>() {
                        if let Some(dt) = chrono::DateTime::from_timestamp(timestamp, 0) {
                            let formatted_date = dt.format("%Y-%m-%d").to_string();
                            episode_obj.insert("releasedate".to_string(), Value::String(formatted_date));
                        }
                    }
                }
            }
        }
        
        Ok(enhanced_episode)
    }
    
    /// Generate episode streaming URL with specific episode ID
    pub fn generate_episode_stream_url(&self, _series_id: &str, episode_id: &str, extension: Option<&str>) -> Result<String> {
        let url = format!(
            "{}/series/{}/{}/{}.{}",
            self.base_url,
            self.credentials.username,
            self.credentials.password,
            episode_id,
            extension.unwrap_or("mp4")
        );
        
        Ok(url)
    }
    
    /// Filter series by various criteria
    pub fn filter_series(
        series: &Value,
        name_filter: Option<&str>,
        category_filter: Option<&str>,
        genre_filter: Option<&str>,
        rating_min: Option<f64>,
        year_filter: Option<&str>,
    ) -> Result<Value> {
        if let Some(series_array) = series.as_array() {
            let filtered_series: Vec<Value> = series_array
                .iter()
                .filter(|series| {
                    // Filter by name (case-insensitive partial match)
                    if let Some(name_query) = name_filter {
                        if let Some(series_name) = series.get("name").and_then(|n| n.as_str()) {
                            if !series_name.to_lowercase().contains(&name_query.to_lowercase()) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    
                    // Filter by category
                    if let Some(category_query) = category_filter {
                        if let Some(category_id) = series.get("category_id").and_then(|c| c.as_str()) {
                            if category_id != category_query {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    
                    // Filter by genre (case-insensitive partial match)
                    if let Some(genre_query) = genre_filter {
                        if let Some(series_genre) = series.get("genre").and_then(|g| g.as_str()) {
                            if !series_genre.to_lowercase().contains(&genre_query.to_lowercase()) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    
                    // Filter by minimum rating
                    if let Some(min_rating) = rating_min {
                        let series_rating = series
                            .get("rating_5based")
                            .and_then(|r| r.as_f64())
                            .unwrap_or(0.0);
                        
                        if series_rating < min_rating {
                            return false;
                        }
                    }
                    
                    // Filter by release year
                    if let Some(year_query) = year_filter {
                        if let Some(release_date) = series.get("releaseDate").and_then(|d| d.as_str()) {
                            if !release_date.starts_with(year_query) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    
                    true
                })
                .cloned()
                .collect();
            
            Ok(Value::Array(filtered_series))
        } else {
            Ok(series.clone())
        }
    }
    
    /// Sort series by various criteria
    pub fn sort_series(series: &Value, sort_by: &str, ascending: bool) -> Result<Value> {
        if let Some(series_array) = series.as_array() {
            let mut sorted_series = series_array.clone();
            
            sorted_series.sort_by(|a, b| {
                let comparison = match sort_by {
                    "name" => {
                        let name_a = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        let name_b = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        name_a.cmp(name_b)
                    }
                    "rating" => {
                        let rating_a = a.get("rating_5based").and_then(|r| r.as_f64()).unwrap_or(0.0);
                        let rating_b = b.get("rating_5based").and_then(|r| r.as_f64()).unwrap_or(0.0);
                        rating_a.partial_cmp(&rating_b).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    "releaseDate" => {
                        let date_a = a.get("releaseDate").and_then(|d| d.as_str()).unwrap_or("");
                        let date_b = b.get("releaseDate").and_then(|d| d.as_str()).unwrap_or("");
                        date_a.cmp(date_b)
                    }
                    "last_modified" => {
                        let modified_a = a.get("last_modified").and_then(|d| d.as_str()).unwrap_or("");
                        let modified_b = b.get("last_modified").and_then(|d| d.as_str()).unwrap_or("");
                        modified_a.cmp(modified_b)
                    }
                    _ => std::cmp::Ordering::Equal,
                };
                
                if ascending {
                    comparison
                } else {
                    comparison.reverse()
                }
            });
            
            Ok(Value::Array(sorted_series))
        } else {
            Ok(series.clone())
        }
    }
    
    /// Validate series data structure
    pub fn validate_series_data(series: &Value) -> bool {
        // Check for required fields
        let required_fields = ["series_id", "name"];
        
        for field in &required_fields {
            if !series.get(field).is_some() {
                return false;
            }
        }
        
        // Validate series_id is a number
        if let Some(series_id) = series.get("series_id") {
            if !series_id.is_number() && !series_id.is_string() {
                return false;
            }
        }
        
        true
    }
    
    /// Generate streaming URL for content
    pub fn generate_stream_url(&self, request: &StreamURLRequest) -> Result<String> {
        let url = match request.content_type {
            ContentType::Channel => {
                // Always use m3u8 for live channels to ensure browser compatibility
                // .ts streams are not natively supported by browsers
                let extension = request.extension.as_deref().unwrap_or("m3u8");
                let extension = if extension == "ts" { "m3u8" } else { extension };
                format!(
                    "{}/live/{}/{}/{}.{}",
                    self.base_url,
                    self.credentials.username,
                    self.credentials.password,
                    request.content_id,
                    extension
                )
            }
            ContentType::Movie => {
                format!(
                    "{}/movie/{}/{}/{}.{}",
                    self.base_url,
                    self.credentials.username,
                    self.credentials.password,
                    request.content_id,
                    request.extension.as_deref().unwrap_or("mp4")
                )
            }
            ContentType::Series => {
                format!(
                    "{}/series/{}/{}/{}.{}",
                    self.base_url,
                    self.credentials.username,
                    self.credentials.password,
                    request.content_id,
                    request.extension.as_deref().unwrap_or("mp4")
                )
            }
        };
        
        Ok(url)
    }
    
    /// Make an API request and handle common errors
    async fn make_api_request(&self, url: &str) -> Result<Value> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| XTauriError::Network(e))?;
        
        if !response.status().is_success() {
            return Err(XTauriError::xtream_api_error(
                response.status().as_u16(),
                format!("API request failed: {}", response.status()),
            ));
        }
        
        let data: Value = response
            .json()
            .await
            .map_err(|e| XTauriError::xtream_api_error(500, format!("Invalid JSON response: {}", e)))?;
        
        Ok(data)
    }
    
    /// Normalize and validate base URL
    fn normalize_base_url(url: &str) -> Result<String> {
        let parsed_url = Url::parse(url)
            .map_err(|e| XTauriError::internal(format!("Invalid URL format: {}", e)))?;
        
        // Validate scheme
        match parsed_url.scheme() {
            "http" | "https" => {}
            _ => return Err(XTauriError::internal("URL must use HTTP or HTTPS scheme".to_string())),
        }
        
        // Build base URL without path
        let base_url = format!(
            "{}://{}{}",
            parsed_url.scheme(),
            parsed_url.host_str().ok_or_else(|| XTauriError::internal("Invalid host in URL".to_string()))?,
            if let Some(port) = parsed_url.port() {
                format!(":{}", port)
            } else {
                String::new()
            }
        );
        
        Ok(base_url)
    }
}