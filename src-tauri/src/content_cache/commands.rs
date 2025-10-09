// Tauri commands for content cache operations
use crate::content_cache::{ContentCache, ChannelFilter, XtreamChannel, SyncScheduler, SyncProgress, SyncSettings};
use crate::error::Result;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::State;

/// State wrapper for ContentCache and SyncScheduler
pub struct ContentCacheState {
    pub cache: Arc<ContentCache>,
    pub sync_scheduler: Arc<SyncScheduler>,
}

impl ContentCacheState {
    pub fn new(db: Arc<Mutex<Connection>>) -> Result<Self> {
        let cache = ContentCache::new(Arc::clone(&db))?;
        let sync_scheduler = SyncScheduler::new(db);
        Ok(Self {
            cache: Arc::new(cache),
            sync_scheduler: Arc::new(sync_scheduler),
        })
    }
}

// ==================== Channel Commands ====================

/// Get cached Xtream channels for a profile with optional filtering
/// 
/// # Arguments
/// * `profile_id` - The profile ID to query
/// * `category_id` - Optional category filter
/// * `limit` - Optional limit for pagination
/// * `offset` - Optional offset for pagination
/// 
/// # Returns
/// Vector of cached channels matching the filter criteria
#[tauri::command]
pub async fn get_cached_xtream_channels(
    state: State<'_, ContentCacheState>,
    profile_id: String,
    category_id: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> std::result::Result<Vec<XtreamChannel>, String> {
    let filter = ChannelFilter {
        category_id,
        name_contains: None,
        limit,
        offset,
    };
    
    state
        .cache
        .get_channels(&profile_id, Some(filter))
        .map_err(|e| e.to_string())
}

/// Search cached Xtream channels with fuzzy matching
/// 
/// # Arguments
/// * `profile_id` - The profile ID to search within
/// * `query` - The search query string
/// * `category_id` - Optional category filter
/// * `limit` - Optional limit for pagination
/// * `offset` - Optional offset for pagination
/// 
/// # Returns
/// Vector of channels matching the search query, ordered by relevance
#[tauri::command]
pub async fn search_cached_xtream_channels(
    state: State<'_, ContentCacheState>,
    profile_id: String,
    query: String,
    category_id: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> std::result::Result<Vec<XtreamChannel>, String> {
    let filter = ChannelFilter {
        category_id,
        name_contains: None,
        limit,
        offset,
    };
    
    state
        .cache
        .search_channels(&profile_id, &query, Some(filter))
        .map_err(|e| e.to_string())
}

// ==================== Movie Commands ====================

/// Get cached Xtream movies for a profile with optional filtering
/// 
/// # Arguments
/// * `profile_id` - The profile ID to query
/// * `category_id` - Optional category filter
/// * `genre` - Optional genre filter
/// * `year` - Optional year filter
/// * `min_rating` - Optional minimum rating filter
/// * `limit` - Optional limit for pagination
/// * `offset` - Optional offset for pagination
/// 
/// # Returns
/// Vector of cached movies matching the filter criteria
#[tauri::command]
pub async fn get_cached_xtream_movies(
    state: State<'_, ContentCacheState>,
    profile_id: String,
    category_id: Option<String>,
    genre: Option<String>,
    year: Option<String>,
    min_rating: Option<f64>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> std::result::Result<Vec<crate::content_cache::XtreamMovie>, String> {
    use crate::content_cache::MovieFilter;
    
    let filter = MovieFilter {
        category_id,
        name_contains: None,
        genre,
        year,
        min_rating,
        limit,
        offset,
    };
    
    state
        .cache
        .get_movies(&profile_id, Some(filter), None, None)
        .map_err(|e| e.to_string())
}

/// Search cached Xtream movies with fuzzy matching
/// 
/// # Arguments
/// * `profile_id` - The profile ID to search within
/// * `query` - The search query string
/// * `category_id` - Optional category filter
/// * `genre` - Optional genre filter
/// * `year` - Optional year filter
/// * `min_rating` - Optional minimum rating filter
/// * `limit` - Optional limit for pagination
/// * `offset` - Optional offset for pagination
/// 
/// # Returns
/// Vector of movies matching the search query, ordered by relevance
#[tauri::command]
pub async fn search_cached_xtream_movies(
    state: State<'_, ContentCacheState>,
    profile_id: String,
    query: String,
    category_id: Option<String>,
    genre: Option<String>,
    year: Option<String>,
    min_rating: Option<f64>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> std::result::Result<Vec<crate::content_cache::XtreamMovie>, String> {
    use crate::content_cache::MovieFilter;
    
    let filter = MovieFilter {
        category_id,
        name_contains: None,
        genre,
        year,
        min_rating,
        limit,
        offset,
    };
    
    state
        .cache
        .search_movies(&profile_id, &query, Some(filter), None, None)
        .map_err(|e| e.to_string())
}

/// Filter cached Xtream movies with advanced criteria
/// 
/// This is an alias for get_cached_xtream_movies with all filter options exposed
/// 
/// # Arguments
/// * `profile_id` - The profile ID to query
/// * `category_id` - Optional category filter
/// * `genre` - Optional genre filter
/// * `year` - Optional year filter
/// * `min_rating` - Optional minimum rating filter
/// * `limit` - Optional limit for pagination
/// * `offset` - Optional offset for pagination
/// 
/// # Returns
/// Vector of cached movies matching the filter criteria
#[tauri::command]
pub async fn filter_cached_xtream_movies(
    state: State<'_, ContentCacheState>,
    profile_id: String,
    category_id: Option<String>,
    genre: Option<String>,
    year: Option<String>,
    min_rating: Option<f64>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> std::result::Result<Vec<crate::content_cache::XtreamMovie>, String> {
    // This is essentially the same as get_cached_xtream_movies
    get_cached_xtream_movies(
        state,
        profile_id,
        category_id,
        genre,
        year,
        min_rating,
        limit,
        offset,
    )
    .await
}

// ==================== Series Commands ====================

/// Get cached Xtream series for a profile with optional filtering
/// 
/// # Arguments
/// * `profile_id` - The profile ID to query
/// * `category_id` - Optional category filter
/// * `genre` - Optional genre filter
/// * `year` - Optional year filter
/// * `min_rating` - Optional minimum rating filter
/// * `limit` - Optional limit for pagination
/// * `offset` - Optional offset for pagination
/// 
/// # Returns
/// Vector of cached series matching the filter criteria
#[tauri::command]
pub async fn get_cached_xtream_series(
    state: State<'_, ContentCacheState>,
    profile_id: String,
    category_id: Option<String>,
    genre: Option<String>,
    year: Option<String>,
    min_rating: Option<f64>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> std::result::Result<Vec<crate::content_cache::XtreamSeries>, String> {
    use crate::content_cache::SeriesFilter;
    
    let filter = SeriesFilter {
        category_id,
        name_contains: None,
        genre,
        year,
        min_rating,
        limit,
        offset,
    };
    
    state
        .cache
        .get_series(&profile_id, Some(filter))
        .map_err(|e| e.to_string())
}

/// Get cached Xtream series details including seasons and episodes
/// 
/// # Arguments
/// * `profile_id` - The profile ID to query
/// * `series_id` - The series ID to get details for
/// 
/// # Returns
/// Complete series details with seasons and episodes
#[tauri::command]
pub async fn get_cached_xtream_series_details(
    state: State<'_, ContentCacheState>,
    profile_id: String,
    series_id: i64,
) -> std::result::Result<crate::content_cache::XtreamSeriesDetails, String> {
    state
        .cache
        .get_series_details(&profile_id, series_id)
        .map_err(|e| e.to_string())
}

/// Search cached Xtream series with fuzzy matching
/// 
/// # Arguments
/// * `profile_id` - The profile ID to search within
/// * `query` - The search query string
/// * `category_id` - Optional category filter
/// * `genre` - Optional genre filter
/// * `year` - Optional year filter
/// * `min_rating` - Optional minimum rating filter
/// * `limit` - Optional limit for pagination
/// * `offset` - Optional offset for pagination
/// 
/// # Returns
/// Vector of series matching the search query, ordered by relevance
#[tauri::command]
pub async fn search_cached_xtream_series(
    state: State<'_, ContentCacheState>,
    profile_id: String,
    query: String,
    category_id: Option<String>,
    genre: Option<String>,
    year: Option<String>,
    min_rating: Option<f64>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> std::result::Result<Vec<crate::content_cache::XtreamSeries>, String> {
    use crate::content_cache::SeriesFilter;
    
    let filter = SeriesFilter {
        category_id,
        name_contains: None,
        genre,
        year,
        min_rating,
        limit,
        offset,
    };
    
    state
        .cache
        .fts_search_series(&profile_id, &query, Some(filter))
        .map_err(|e| e.to_string())
}

// ==================== Sync Control Commands ====================

/// Start content synchronization for a profile
/// 
/// This command initiates either a full or incremental sync based on the `full_sync` parameter.
/// The sync runs in the background and progress can be monitored via `get_sync_progress`.
/// 
/// # Arguments
/// * `cache_state` - Content cache state containing the sync scheduler
/// * `xtream_state` - Xtream state for accessing profile credentials
/// * `profile_id` - The profile ID to sync
/// * `full_sync` - If true, performs full sync; if false, performs incremental sync
/// 
/// # Returns
/// Ok(()) if sync started successfully, error otherwise
#[tauri::command]
pub async fn start_content_sync(
    cache_state: State<'_, ContentCacheState>,
    xtream_state: State<'_, crate::xtream::XtreamState>,
    profile_id: String,
    full_sync: bool,
) -> std::result::Result<(), String> {
    use tokio::sync::mpsc;
    
    // Check if sync is already active
    if cache_state.sync_scheduler.is_sync_active(&profile_id).map_err(|e| e.to_string())? {
        return Err("Sync already in progress for this profile".to_string());
    }
    
    // Get profile credentials
    let credentials = xtream_state
        .profile_manager
        .get_profile_credentials(&profile_id)
        .map_err(|e| format!("Failed to get profile credentials: {}", e))?;
    
    // Get profile info for base URL
    let profile = xtream_state
        .profile_manager
        .get_profile(&profile_id)
        .map_err(|e| format!("Failed to get profile: {}", e))?
        .ok_or_else(|| format!("Profile not found: {}", profile_id))?;
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = mpsc::channel(100);
    
    // Create cancellation token
    let cancel_token = tokio_util::sync::CancellationToken::new();
    
    // Register the sync
    cache_state
        .sync_scheduler
        .register_sync(&profile_id, cancel_token.clone())
        .map_err(|e| e.to_string())?;
    
    // Clone necessary data for the async task
    let scheduler = Arc::clone(&cache_state.sync_scheduler);
    let cache = Arc::clone(&cache_state.cache);
    let profile_id_clone = profile_id.clone();
    let base_url = profile.url.clone();
    let username = credentials.username.clone();
    let password = credentials.password.clone();
    
    // Spawn sync task
    tokio::spawn(async move {
        let result = if full_sync {
            scheduler.run_full_sync(
                &profile_id_clone,
                &base_url,
                &username,
                &password,
                &cache,
                &progress_tx,
                &cancel_token,
            ).await
        } else {
            scheduler.run_incremental_sync(
                &profile_id_clone,
                &base_url,
                &username,
                &password,
                &cache,
                &progress_tx,
                &cancel_token,
            ).await
        };
        
        // Unregister sync when complete
        let _ = scheduler.unregister_sync(&profile_id_clone);
        
        // Log result
        match result {
            Ok(progress) => {
                println!("[INFO] Sync completed for profile {}: {:?}", profile_id_clone, progress.status);
            }
            Err(e) => {
                eprintln!("[ERROR] Sync failed for profile {}: {}", profile_id_clone, e);
            }
        }
    });
    
    // Spawn a task to consume progress updates (in a real app, you'd emit these as events)
    tokio::spawn(async move {
        while let Some(progress) = progress_rx.recv().await {
            // In a real implementation, you would emit this as a Tauri event
            // For now, just log it
            println!("[SYNC PROGRESS] Profile: {}, Status: {:?}, Progress: {}%", 
                     profile_id, progress.status, progress.progress);
        }
    });
    
    Ok(())
}

/// Cancel an active content synchronization
/// 
/// # Arguments
/// * `state` - Content cache state containing the sync scheduler
/// * `profile_id` - The profile ID to cancel sync for
/// 
/// # Returns
/// Ok(()) if sync was cancelled, error if no active sync found
#[tauri::command]
pub async fn cancel_content_sync(
    state: State<'_, ContentCacheState>,
    profile_id: String,
) -> std::result::Result<(), String> {
    state
        .sync_scheduler
        .cancel_sync(&profile_id)
        .map_err(|e| e.to_string())
}

/// Get current sync progress for a profile
/// 
/// Returns the current synchronization progress including status, percentage complete,
/// current step, and counts of synced items.
/// 
/// # Arguments
/// * `state` - Content cache state containing the sync scheduler
/// * `profile_id` - The profile ID to get progress for
/// 
/// # Returns
/// Current sync progress information
#[tauri::command]
pub async fn get_sync_progress(
    state: State<'_, ContentCacheState>,
    profile_id: String,
) -> std::result::Result<SyncProgress, String> {
    state
        .sync_scheduler
        .get_sync_status(&profile_id)
        .map_err(|e| e.to_string())
}

/// Get sync status for a profile
/// 
/// This is an alias for get_sync_progress that returns the same information.
/// 
/// # Arguments
/// * `state` - Content cache state containing the sync scheduler
/// * `profile_id` - The profile ID to get status for
/// 
/// # Returns
/// Current sync status information
#[tauri::command]
pub async fn get_sync_status(
    state: State<'_, ContentCacheState>,
    profile_id: String,
) -> std::result::Result<SyncProgress, String> {
    // This is the same as get_sync_progress
    get_sync_progress(state, profile_id).await
}

/// Get sync settings for a profile
/// 
/// # Arguments
/// * `state` - Content cache state containing the sync scheduler
/// * `profile_id` - The profile ID to get settings for
/// 
/// # Returns
/// Current sync settings
#[tauri::command]
pub async fn get_sync_settings(
    state: State<'_, ContentCacheState>,
    profile_id: String,
) -> std::result::Result<SyncSettings, String> {
    state
        .sync_scheduler
        .get_sync_settings(&profile_id)
        .map_err(|e| e.to_string())
}

/// Update sync settings for a profile
/// 
/// # Arguments
/// * `state` - Content cache state containing the sync scheduler
/// * `profile_id` - The profile ID to update settings for
/// * `settings` - New sync settings
/// 
/// # Returns
/// Ok(()) if settings were updated successfully
#[tauri::command]
pub async fn update_sync_settings(
    state: State<'_, ContentCacheState>,
    profile_id: String,
    settings: SyncSettings,
) -> std::result::Result<(), String> {
    state
        .sync_scheduler
        .update_sync_settings(&profile_id, &settings)
        .map_err(|e| e.to_string())
}

/// Clear content cache for a profile
/// 
/// # Arguments
/// * `state` - Content cache state
/// * `profile_id` - The profile ID to clear cache for
/// 
/// # Returns
/// Ok(()) if cache was cleared successfully
#[tauri::command]
pub async fn clear_content_cache(
    state: State<'_, ContentCacheState>,
    profile_id: String,
) -> std::result::Result<(), String> {
    state
        .cache
        .clear_profile_content(&profile_id)
        .map_err(|e| e.to_string())
}

/// Get content cache statistics for a profile
/// 
/// # Arguments
/// * `state` - Content cache state
/// * `profile_id` - The profile ID to get stats for
/// 
/// # Returns
/// Cache statistics including item counts (channels, movies, series)
#[tauri::command]
pub async fn get_content_cache_stats(
    state: State<'_, ContentCacheState>,
    profile_id: String,
) -> std::result::Result<(usize, usize, usize), String> {
    state
        .cache
        .get_content_counts(&profile_id)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content_cache::XtreamChannel;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    
    /// Create a test database with required dependencies
    fn create_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        
        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        
        // Create xtream_profiles table (dependency for foreign keys)
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
        
        Arc::new(Mutex::new(conn))
    }
    
    /// Insert a test profile into the database
    fn insert_test_profile(db: &Arc<Mutex<Connection>>, profile_id: &str) {
        let conn = db.lock().unwrap();
        let profile_name = format!("Test Profile {}", profile_id);
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES (?1, ?2, 'http://test.com', 'testuser', X'00')",
            rusqlite::params![profile_id, profile_name],
        )
        .unwrap();
    }
    
    fn setup_test_cache() -> ContentCache {
        let db = create_test_db();
        ContentCache::new(db).unwrap()
    }
    
    fn create_test_channel(stream_id: i64, name: &str, category_id: &str) -> XtreamChannel {
        XtreamChannel {
            stream_id,
            num: Some(stream_id),
            name: name.to_string(),
            stream_type: Some("live".to_string()),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some(category_id.to_string()),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: Some(0),
        }
    }
    
    fn create_test_movie(
        stream_id: i64,
        name: &str,
        category_id: &str,
        genre: Option<&str>,
        year: Option<&str>,
        rating: Option<f64>,
    ) -> crate::content_cache::XtreamMovie {
        crate::content_cache::XtreamMovie {
            stream_id,
            num: Some(stream_id),
            name: name.to_string(),
            title: Some(name.to_string()),
            year: year.map(|y| y.to_string()),
            stream_type: Some("movie".to_string()),
            stream_icon: None,
            rating,
            rating_5based: rating,
            genre: genre.map(|g| g.to_string()),
            added: None,
            episode_run_time: None,
            category_id: Some(category_id.to_string()),
            container_extension: Some("mp4".to_string()),
            custom_sid: None,
            direct_source: None,
            release_date: None,
            cast: None,
            director: None,
            plot: None,
            youtube_trailer: None,
        }
    }
    
    fn create_test_series(
        series_id: i64,
        name: &str,
        category_id: &str,
        genre: Option<&str>,
        year: Option<&str>,
        rating: Option<f64>,
    ) -> crate::content_cache::XtreamSeries {
        crate::content_cache::XtreamSeries {
            series_id,
            num: Some(series_id),
            name: name.to_string(),
            title: Some(name.to_string()),
            year: year.map(|y| y.to_string()),
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: genre.map(|g| g.to_string()),
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: rating,
            episode_run_time: None,
            category_id: Some(category_id.to_string()),
        }
    }
    
    #[test]
    fn test_get_cached_channels_empty() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let result = cache.get_channels("test_profile", None).unwrap();
        assert_eq!(result.len(), 0);
    }
    
    #[test]
    fn test_get_cached_channels_with_data() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let channels = vec![
            create_test_channel(1, "CNN", "news"),
            create_test_channel(2, "BBC", "news"),
            create_test_channel(3, "ESPN", "sports"),
        ];
        
        cache.save_channels("test_profile", channels).unwrap();
        
        let result = cache.get_channels("test_profile", None).unwrap();
        assert_eq!(result.len(), 3);
    }
    
    #[test]
    fn test_get_cached_channels_with_category_filter() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let channels = vec![
            create_test_channel(1, "CNN", "news"),
            create_test_channel(2, "BBC", "news"),
            create_test_channel(3, "ESPN", "sports"),
        ];
        
        cache.save_channels("test_profile", channels).unwrap();
        
        let filter = ChannelFilter {
            category_id: Some("news".to_string()),
            name_contains: None,
            limit: None,
            offset: None,
        };
        
        let result = cache.get_channels("test_profile", Some(filter)).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|c| c.category_id.as_ref().unwrap() == "news"));
    }
    
    #[test]
    fn test_get_cached_channels_with_pagination() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let channels = vec![
            create_test_channel(1, "Channel 1", "cat1"),
            create_test_channel(2, "Channel 2", "cat1"),
            create_test_channel(3, "Channel 3", "cat1"),
            create_test_channel(4, "Channel 4", "cat1"),
            create_test_channel(5, "Channel 5", "cat1"),
        ];
        
        cache.save_channels("test_profile", channels).unwrap();
        
        // Get first page
        let filter = ChannelFilter {
            category_id: None,
            name_contains: None,
            limit: Some(2),
            offset: Some(0),
        };
        
        let result = cache.get_channels("test_profile", Some(filter)).unwrap();
        assert_eq!(result.len(), 2);
        
        // Get second page
        let filter = ChannelFilter {
            category_id: None,
            name_contains: None,
            limit: Some(2),
            offset: Some(2),
        };
        
        let result = cache.get_channels("test_profile", Some(filter)).unwrap();
        assert_eq!(result.len(), 2);
    }
    
    #[test]
    fn test_search_cached_channels() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let channels = vec![
            create_test_channel(1, "CNN International", "news"),
            create_test_channel(2, "BBC World News", "news"),
            create_test_channel(3, "ESPN Sports", "sports"),
        ];
        
        cache.save_channels("test_profile", channels).unwrap();
        
        let result = cache.search_channels("test_profile", "CNN", None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "CNN International");
    }
    
    #[test]
    fn test_search_cached_channels_fuzzy() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let channels = vec![
            create_test_channel(1, "CNN International", "news"),
            create_test_channel(2, "BBC World News", "news"),
            create_test_channel(3, "ESPN Sports", "sports"),
        ];
        
        cache.save_channels("test_profile", channels).unwrap();
        
        // Fuzzy search should match partial strings
        let result = cache.search_channels("test_profile", "world", None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "BBC World News");
    }
    
    #[test]
    fn test_search_cached_channels_case_insensitive() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let channels = vec![
            create_test_channel(1, "CNN International", "news"),
            create_test_channel(2, "BBC World News", "news"),
        ];
        
        cache.save_channels("test_profile", channels).unwrap();
        
        // Search should be case-insensitive
        let result = cache.search_channels("test_profile", "cnn", None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "CNN International");
    }
    
    #[test]
    fn test_search_cached_channels_with_category_filter() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let channels = vec![
            create_test_channel(1, "CNN International", "news"),
            create_test_channel(2, "BBC World News", "news"),
            create_test_channel(3, "ESPN Sports News", "sports"),
        ];
        
        cache.save_channels("test_profile", channels).unwrap();
        
        let filter = ChannelFilter {
            category_id: Some("news".to_string()),
            name_contains: None,
            limit: None,
            offset: None,
        };
        
        // Search for "World" but only in "news" category - should find BBC World News but not ESPN Sports News
        let result = cache.search_channels("test_profile", "World", Some(filter)).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "BBC World News");
        assert!(result.iter().all(|c| c.category_id.as_ref().unwrap() == "news"));
    }
    
    #[test]
    fn test_search_cached_channels_empty_query() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let channels = vec![
            create_test_channel(1, "CNN", "news"),
            create_test_channel(2, "BBC", "news"),
        ];
        
        cache.save_channels("test_profile", channels).unwrap();
        
        // Empty query should return all channels
        let result = cache.search_channels("test_profile", "", None).unwrap();
        assert_eq!(result.len(), 2);
    }
    
    #[test]
    fn test_search_cached_channels_no_results() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let channels = vec![
            create_test_channel(1, "CNN", "news"),
            create_test_channel(2, "BBC", "news"),
        ];
        
        cache.save_channels("test_profile", channels).unwrap();
        
        let result = cache.search_channels("test_profile", "xyz123", None).unwrap();
        assert_eq!(result.len(), 0);
    }
    
    #[test]
    fn test_search_cached_channels_relevance_ordering() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let channels = vec![
            create_test_channel(1, "News Channel", "news"),
            create_test_channel(2, "CNN News", "news"),
            create_test_channel(3, "News", "news"),
        ];
        
        cache.save_channels("test_profile", channels).unwrap();
        
        let result = cache.search_channels("test_profile", "News", None).unwrap();
        assert_eq!(result.len(), 3);
        
        // Exact match should come first
        assert_eq!(result[0].name, "News");
        // Starts with should come next
        assert_eq!(result[1].name, "News Channel");
        // Contains should come last
        assert_eq!(result[2].name, "CNN News");
    }
    
    // ==================== Movie Tests ====================
    
    #[test]
    fn test_get_cached_movies_empty() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let result = cache.get_movies("test_profile", None, None, None).unwrap();
        assert_eq!(result.len(), 0);
    }
    
    #[test]
    fn test_get_cached_movies_with_data() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let movies = vec![
            create_test_movie(1, "The Matrix", "action", Some("Action, Sci-Fi"), Some("1999"), Some(4.5)),
            create_test_movie(2, "Inception", "action", Some("Action, Thriller"), Some("2010"), Some(4.8)),
            create_test_movie(3, "The Godfather", "drama", Some("Crime, Drama"), Some("1972"), Some(4.9)),
        ];
        
        cache.save_movies("test_profile", movies).unwrap();
        
        let result = cache.get_movies("test_profile", None, None, None).unwrap();
        assert_eq!(result.len(), 3);
    }
    
    #[test]
    fn test_get_cached_movies_with_category_filter() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let movies = vec![
            create_test_movie(1, "The Matrix", "action", Some("Action, Sci-Fi"), Some("1999"), Some(4.5)),
            create_test_movie(2, "Inception", "action", Some("Action, Thriller"), Some("2010"), Some(4.8)),
            create_test_movie(3, "The Godfather", "drama", Some("Crime, Drama"), Some("1972"), Some(4.9)),
        ];
        
        cache.save_movies("test_profile", movies).unwrap();
        
        use crate::content_cache::MovieFilter;
        let filter = MovieFilter {
            category_id: Some("action".to_string()),
            name_contains: None,
            genre: None,
            year: None,
            min_rating: None,
            limit: None,
            offset: None,
        };
        
        let result = cache.get_movies("test_profile", Some(filter), None, None).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|m| m.category_id.as_ref().unwrap() == "action"));
    }
    
    #[test]
    fn test_get_cached_movies_with_genre_filter() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let movies = vec![
            create_test_movie(1, "The Matrix", "action", Some("Action, Sci-Fi"), Some("1999"), Some(4.5)),
            create_test_movie(2, "Inception", "action", Some("Action, Thriller"), Some("2010"), Some(4.8)),
            create_test_movie(3, "The Godfather", "drama", Some("Crime, Drama"), Some("1972"), Some(4.9)),
        ];
        
        cache.save_movies("test_profile", movies).unwrap();
        
        use crate::content_cache::MovieFilter;
        let filter = MovieFilter {
            category_id: None,
            name_contains: None,
            genre: Some("Sci-Fi".to_string()),
            year: None,
            min_rating: None,
            limit: None,
            offset: None,
        };
        
        let result = cache.get_movies("test_profile", Some(filter), None, None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "The Matrix");
    }
    
    #[test]
    fn test_get_cached_movies_with_rating_filter() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let movies = vec![
            create_test_movie(1, "The Matrix", "action", Some("Action, Sci-Fi"), Some("1999"), Some(4.5)),
            create_test_movie(2, "Inception", "action", Some("Action, Thriller"), Some("2010"), Some(4.8)),
            create_test_movie(3, "The Godfather", "drama", Some("Crime, Drama"), Some("1972"), Some(4.9)),
        ];
        
        cache.save_movies("test_profile", movies).unwrap();
        
        use crate::content_cache::MovieFilter;
        let filter = MovieFilter {
            category_id: None,
            name_contains: None,
            genre: None,
            year: None,
            min_rating: Some(4.7),
            limit: None,
            offset: None,
        };
        
        let result = cache.get_movies("test_profile", Some(filter), None, None).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|m| m.rating.unwrap_or(0.0) >= 4.7));
    }
    
    #[test]
    fn test_search_cached_movies() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let movies = vec![
            create_test_movie(1, "The Matrix", "action", Some("Action, Sci-Fi"), Some("1999"), Some(4.5)),
            create_test_movie(2, "Inception", "action", Some("Action, Thriller"), Some("2010"), Some(4.8)),
            create_test_movie(3, "The Godfather", "drama", Some("Crime, Drama"), Some("1972"), Some(4.9)),
        ];
        
        cache.save_movies("test_profile", movies).unwrap();
        
        let result = cache.search_movies("test_profile", "Matrix", None, None, None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "The Matrix");
    }
    
    #[test]
    fn test_search_cached_movies_case_insensitive() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let movies = vec![
            create_test_movie(1, "The Matrix", "action", Some("Action, Sci-Fi"), Some("1999"), Some(4.5)),
            create_test_movie(2, "Inception", "action", Some("Action, Thriller"), Some("2010"), Some(4.8)),
        ];
        
        cache.save_movies("test_profile", movies).unwrap();
        
        let result = cache.search_movies("test_profile", "matrix", None, None, None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "The Matrix");
    }
    
    #[test]
    fn test_search_cached_movies_with_filters() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let movies = vec![
            create_test_movie(1, "The Matrix", "action", Some("Action, Sci-Fi"), Some("1999"), Some(4.5)),
            create_test_movie(2, "Inception", "action", Some("Action, Thriller"), Some("2010"), Some(4.8)),
            create_test_movie(3, "The Matrix Reloaded", "action", Some("Action, Sci-Fi"), Some("2003"), Some(4.2)),
        ];
        
        cache.save_movies("test_profile", movies).unwrap();
        
        use crate::content_cache::MovieFilter;
        let filter = MovieFilter {
            category_id: Some("action".to_string()),
            name_contains: None,
            genre: None,
            year: None,
            min_rating: Some(4.5),
            limit: None,
            offset: None,
        };
        
        let result = cache.search_movies("test_profile", "Matrix", Some(filter), None, None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "The Matrix");
    }
    
    // ==================== Series Tests ====================
    
    #[test]
    fn test_get_cached_series_empty() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let result = cache.get_series("test_profile", None).unwrap();
        assert_eq!(result.len(), 0);
    }
    
    #[test]
    fn test_get_cached_series_with_data() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let series = vec![
            create_test_series(1, "Breaking Bad", "drama", Some("Crime, Drama"), Some("2008"), Some(4.9)),
            create_test_series(2, "Game of Thrones", "fantasy", Some("Fantasy, Drama"), Some("2011"), Some(4.7)),
            create_test_series(3, "The Office", "comedy", Some("Comedy"), Some("2005"), Some(4.5)),
        ];
        
        cache.save_series("test_profile", series).unwrap();
        
        let result = cache.get_series("test_profile", None).unwrap();
        assert_eq!(result.len(), 3);
    }
    
    #[test]
    fn test_get_cached_series_with_category_filter() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let series = vec![
            create_test_series(1, "Breaking Bad", "drama", Some("Crime, Drama"), Some("2008"), Some(4.9)),
            create_test_series(2, "Game of Thrones", "fantasy", Some("Fantasy, Drama"), Some("2011"), Some(4.7)),
            create_test_series(3, "The Office", "comedy", Some("Comedy"), Some("2005"), Some(4.5)),
        ];
        
        cache.save_series("test_profile", series).unwrap();
        
        use crate::content_cache::SeriesFilter;
        let filter = SeriesFilter {
            category_id: Some("drama".to_string()),
            name_contains: None,
            genre: None,
            year: None,
            min_rating: None,
            limit: None,
            offset: None,
        };
        
        let result = cache.get_series("test_profile", Some(filter)).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Breaking Bad");
    }
    
    #[test]
    fn test_search_cached_series() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let series = vec![
            create_test_series(1, "Breaking Bad", "drama", Some("Crime, Drama"), Some("2008"), Some(4.9)),
            create_test_series(2, "Game of Thrones", "fantasy", Some("Fantasy, Drama"), Some("2011"), Some(4.7)),
            create_test_series(3, "The Office", "comedy", Some("Comedy"), Some("2005"), Some(4.5)),
        ];
        
        cache.save_series("test_profile", series).unwrap();
        
        let result = cache.fts_search_series("test_profile", "Breaking", None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Breaking Bad");
    }
    
    #[test]
    fn test_search_cached_series_case_insensitive() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        let series = vec![
            create_test_series(1, "Breaking Bad", "drama", Some("Crime, Drama"), Some("2008"), Some(4.9)),
            create_test_series(2, "Game of Thrones", "fantasy", Some("Fantasy, Drama"), Some("2011"), Some(4.7)),
        ];
        
        cache.save_series("test_profile", series).unwrap();
        
        let result = cache.fts_search_series("test_profile", "game", None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Game of Thrones");
    }
    
    #[test]
    fn test_get_cached_series_details() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        // Create and save a series
        let series = vec![
            create_test_series(1, "Breaking Bad", "drama", Some("Crime, Drama"), Some("2008"), Some(4.9)),
        ];
        cache.save_series("test_profile", series).unwrap();
        
        // Create series details with seasons and episodes
        let details = crate::content_cache::XtreamSeriesDetails {
            series: create_test_series(1, "Breaking Bad", "drama", Some("Crime, Drama"), Some("2008"), Some(4.9)),
            seasons: vec![
                crate::content_cache::XtreamSeason {
                    season_number: 1,
                    name: Some("Season 1".to_string()),
                    episode_count: Some(7),
                    overview: Some("First season".to_string()),
                    air_date: Some("2008-01-20".to_string()),
                    cover: None,
                    cover_big: None,
                    vote_average: Some(4.8),
                },
                crate::content_cache::XtreamSeason {
                    season_number: 2,
                    name: Some("Season 2".to_string()),
                    episode_count: Some(13),
                    overview: Some("Second season".to_string()),
                    air_date: Some("2009-03-08".to_string()),
                    cover: None,
                    cover_big: None,
                    vote_average: Some(4.9),
                },
            ],
            episodes: vec![
                crate::content_cache::XtreamEpisode {
                    episode_id: "1001".to_string(),
                    season_number: 1,
                    episode_num: "1".to_string(),
                    title: Some("Pilot".to_string()),
                    container_extension: Some("mp4".to_string()),
                    custom_sid: None,
                    added: None,
                    direct_source: None,
                    info_json: None,
                },
                crate::content_cache::XtreamEpisode {
                    episode_id: "1002".to_string(),
                    season_number: 1,
                    episode_num: "2".to_string(),
                    title: Some("Cat's in the Bag...".to_string()),
                    container_extension: Some("mp4".to_string()),
                    custom_sid: None,
                    added: None,
                    direct_source: None,
                    info_json: None,
                },
            ],
        };
        
        cache.save_series_details("test_profile", 1, details).unwrap();
        
        // Retrieve the details
        let result = cache.get_series_details("test_profile", 1).unwrap();
        
        assert_eq!(result.series.name, "Breaking Bad");
        assert_eq!(result.seasons.len(), 2);
        assert_eq!(result.episodes.len(), 2);
        assert_eq!(result.seasons[0].season_number, 1);
        assert_eq!(result.seasons[1].season_number, 2);
        assert_eq!(result.episodes[0].title, Some("Pilot".to_string()));
        assert_eq!(result.episodes[1].title, Some("Cat's in the Bag...".to_string()));
    }
    
    #[test]
    fn test_get_cached_series_details_not_found() {
        let db = create_test_db();
        insert_test_profile(&db, "test_profile");
        let cache = ContentCache::new(db).unwrap();
        cache.initialize_profile("test_profile").unwrap();
        
        // Try to get details for a non-existent series
        let result = cache.get_series_details("test_profile", 999);
        assert!(result.is_err());
    }
}
