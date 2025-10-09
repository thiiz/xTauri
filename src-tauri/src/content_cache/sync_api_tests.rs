// Integration tests for Xtream API sync functionality
use crate::content_cache::{SyncScheduler, SyncProgress, SyncStatus, RetryConfig};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path_regex, query_param};
use serde_json::json;

    /// Create a test database with all required tables
    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        
        // Create required tables manually
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
             VALUES ('test-profile', 'Test', 'http://test.com', 'testuser', X'00')",
            [],
        ).unwrap();
        
        conn
    }

    /// Create mock categories response
    fn mock_categories_response() -> serde_json::Value {
        json!([
            {
                "category_id": "1",
                "category_name": "Sports",
                "parent_id": 0
            },
            {
                "category_id": "2",
                "category_name": "Movies",
                "parent_id": 0
            },
            {
                "category_id": "3",
                "category_name": "News",
                "parent_id": 0
            }
        ])
    }

    /// Create mock channels response
    fn mock_channels_response() -> serde_json::Value {
        json!([
            {
                "stream_id": 1001,
                "num": 1,
                "name": "ESPN HD",
                "stream_type": "live",
                "stream_icon": "http://example.com/espn.png",
                "category_id": "1",
                "tv_archive": 1,
                "tv_archive_duration": 7
            },
            {
                "stream_id": 1002,
                "num": 2,
                "name": "Fox Sports",
                "stream_type": "live",
                "stream_icon": "http://example.com/fox.png",
                "category_id": "1",
                "tv_archive": 0
            }
        ])
    }

    /// Create mock movies response
    fn mock_movies_response() -> serde_json::Value {
        json!([
            {
                "stream_id": 2001,
                "num": 1,
                "name": "The Matrix",
                "title": "The Matrix",
                "year": "1999",
                "stream_type": "movie",
                "stream_icon": "http://example.com/matrix.jpg",
                "rating": "8.7",
                "rating_5based": 4.35,
                "genre": "Sci-Fi, Action",
                "category_id": "2",
                "container_extension": "mkv"
            },
            {
                "stream_id": 2002,
                "num": 2,
                "name": "Inception",
                "title": "Inception",
                "year": "2010",
                "stream_type": "movie",
                "stream_icon": "http://example.com/inception.jpg",
                "rating": "8.8",
                "rating_5based": 4.4,
                "genre": "Sci-Fi, Thriller",
                "category_id": "2",
                "container_extension": "mp4"
            }
        ])
    }

    /// Create mock series response
    fn mock_series_response() -> serde_json::Value {
        json!([
            {
                "series_id": 3001,
                "num": 1,
                "name": "Breaking Bad",
                "title": "Breaking Bad",
                "year": "2008",
                "cover": "http://example.com/bb.jpg",
                "plot": "A chemistry teacher turned meth cook",
                "genre": "Crime, Drama",
                "rating": "9.5",
                "rating_5based": 4.75,
                "category_id": "3"
            },
            {
                "series_id": 3002,
                "num": 2,
                "name": "Game of Thrones",
                "title": "Game of Thrones",
                "year": "2011",
                "cover": "http://example.com/got.jpg",
                "plot": "Noble families fight for the Iron Throne",
                "genre": "Fantasy, Drama",
                "rating": "9.3",
                "rating_5based": 4.65,
                "category_id": "3"
            }
        ])
    }

    /// Create mock series details response
    fn mock_series_details_response() -> serde_json::Value {
        json!({
            "info": {
                "series_id": 3001,
                "name": "Breaking Bad",
                "cover": "http://example.com/bb.jpg",
                "plot": "A chemistry teacher turned meth cook",
                "genre": "Crime, Drama",
                "rating": "9.5"
            },
            "seasons": [
                {
                    "season_number": 1,
                    "name": "Season 1",
                    "episode_count": 7,
                    "air_date": "2008-01-20"
                },
                {
                    "season_number": 2,
                    "name": "Season 2",
                    "episode_count": 13,
                    "air_date": "2009-03-08"
                }
            ],
            "episodes": {
                "1": [
                    {
                        "id": "30011",
                        "episode_num": 1,
                        "title": "Pilot",
                        "container_extension": "mkv"
                    }
                ],
                "2": [
                    {
                        "id": "30021",
                        "episode_num": 1,
                        "title": "Seven Thirty-Seven",
                        "container_extension": "mkv"
                    }
                ]
            }
        })
    }

    #[tokio::test]
    async fn test_fetch_categories_with_retry_success() {
        // Start mock server
        let mock_server = MockServer::start().await;
        
        // Setup mock response
        Mock::given(method("GET"))
            .and(path_regex("/player_api.php"))
            .and(query_param("action", "get_live_categories"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
            .mount(&mock_server)
            .await;
        
        // Create client
        let client = reqwest::Client::new();
        let retry_config = RetryConfig::default();
        let cancel_token = CancellationToken::new();
        
        // Fetch categories
        let result = SyncScheduler::fetch_categories_with_retry(
            &client,
            &mock_server.uri(),
            "testuser",
            "testpass",
            "channels",
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_ok());
        let categories = result.unwrap();
        assert!(categories.is_array());
        assert_eq!(categories.as_array().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_fetch_categories_with_retry_on_server_error() {
        // Start mock server
        let mock_server = MockServer::start().await;
        
        // Setup mock to fail twice then succeed
        Mock::given(method("GET"))
            .and(path_regex("/player_api.php"))
            .and(query_param("action", "get_live_categories"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;
        
        Mock::given(method("GET"))
            .and(path_regex("/player_api.php"))
            .and(query_param("action", "get_live_categories"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
            .mount(&mock_server)
            .await;
        
        // Create client with fast retry
        let client = reqwest::Client::new();
        let retry_config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };
        let cancel_token = CancellationToken::new();
        
        // Fetch categories - should succeed after retries
        let result = SyncScheduler::fetch_categories_with_retry(
            &client,
            &mock_server.uri(),
            "testuser",
            "testpass",
            "channels",
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_categories_with_retry_on_auth_failure() {
        // Start mock server
        let mock_server = MockServer::start().await;
        
        // Setup mock to return 401
        Mock::given(method("GET"))
            .and(path_regex("/player_api.php"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;
        
        // Create client
        let client = reqwest::Client::new();
        let retry_config = RetryConfig::default();
        let cancel_token = CancellationToken::new();
        
        // Fetch categories - should fail without retry
        let result = SyncScheduler::fetch_categories_with_retry(
            &client,
            &mock_server.uri(),
            "testuser",
            "wrongpass",
            "channels",
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_content_with_retry_channels() {
        // Start mock server
        let mock_server = MockServer::start().await;
        
        // Setup mock response
        Mock::given(method("GET"))
            .and(path_regex("/player_api.php"))
            .and(query_param("action", "get_live_streams"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_channels_response()))
            .mount(&mock_server)
            .await;
        
        // Create client
        let client = reqwest::Client::new();
        let retry_config = RetryConfig::default();
        let cancel_token = CancellationToken::new();
        
        // Fetch channels
        let result = SyncScheduler::fetch_content_with_retry(
            &client,
            &mock_server.uri(),
            "testuser",
            "testpass",
            "channels",
            None,
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_ok());
        let channels = result.unwrap();
        assert!(channels.is_array());
        assert_eq!(channels.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_content_with_retry_movies() {
        // Start mock server
        let mock_server = MockServer::start().await;
        
        // Setup mock response
        Mock::given(method("GET"))
            .and(path_regex("/player_api.php"))
            .and(query_param("action", "get_vod_streams"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_movies_response()))
            .mount(&mock_server)
            .await;
        
        // Create client
        let client = reqwest::Client::new();
        let retry_config = RetryConfig::default();
        let cancel_token = CancellationToken::new();
        
        // Fetch movies
        let result = SyncScheduler::fetch_content_with_retry(
            &client,
            &mock_server.uri(),
            "testuser",
            "testpass",
            "movies",
            None,
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_ok());
        let movies = result.unwrap();
        assert!(movies.is_array());
        assert_eq!(movies.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_content_with_retry_series() {
        // Start mock server
        let mock_server = MockServer::start().await;
        
        // Setup mock response
        Mock::given(method("GET"))
            .and(path_regex("/player_api.php"))
            .and(query_param("action", "get_series"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_series_response()))
            .mount(&mock_server)
            .await;
        
        // Create client
        let client = reqwest::Client::new();
        let retry_config = RetryConfig::default();
        let cancel_token = CancellationToken::new();
        
        // Fetch series
        let result = SyncScheduler::fetch_content_with_retry(
            &client,
            &mock_server.uri(),
            "testuser",
            "testpass",
            "series",
            None,
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_ok());
        let series = result.unwrap();
        assert!(series.is_array());
        assert_eq!(series.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_content_with_category_filter() {
        // Start mock server
        let mock_server = MockServer::start().await;
        
        // Setup mock response
        Mock::given(method("GET"))
            .and(path_regex("/player_api.php"))
            .and(query_param("action", "get_live_streams"))
            .and(query_param("category_id", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_channels_response()))
            .mount(&mock_server)
            .await;
        
        // Create client
        let client = reqwest::Client::new();
        let retry_config = RetryConfig::default();
        let cancel_token = CancellationToken::new();
        
        // Fetch channels with category filter
        let result = SyncScheduler::fetch_content_with_retry(
            &client,
            &mock_server.uri(),
            "testuser",
            "testpass",
            "channels",
            Some("1"),
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_series_details_with_retry() {
        // Start mock server
        let mock_server = MockServer::start().await;
        
        // Setup mock response
        Mock::given(method("GET"))
            .and(path_regex("/player_api.php"))
            .and(query_param("action", "get_series_info"))
            .and(query_param("series_id", "3001"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_series_details_response()))
            .mount(&mock_server)
            .await;
        
        // Create client
        let client = reqwest::Client::new();
        let retry_config = RetryConfig::default();
        let cancel_token = CancellationToken::new();
        
        // Fetch series details
        let result = SyncScheduler::fetch_series_details_with_retry(
            &client,
            &mock_server.uri(),
            "testuser",
            "testpass",
            3001,
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_ok());
        let details = result.unwrap();
        assert!(details.get("info").is_some());
        assert!(details.get("seasons").is_some());
        assert!(details.get("episodes").is_some());
    }

    #[tokio::test]
    async fn test_fetch_with_cancellation() {
        // Start mock server
        let mock_server = MockServer::start().await;
        
        // Setup mock with delay
        Mock::given(method("GET"))
            .and(path_regex("/player_api.php"))
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(5)))
            .mount(&mock_server)
            .await;
        
        // Create client
        let client = reqwest::Client::new();
        let retry_config = RetryConfig::default();
        let cancel_token = CancellationToken::new();
        
        // Cancel immediately
        cancel_token.cancel();
        
        // Fetch should fail with cancellation error
        let result = SyncScheduler::fetch_categories_with_retry(
            &client,
            &mock_server.uri(),
            "testuser",
            "testpass",
            "channels",
            &retry_config,
            &cancel_token,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }

    #[tokio::test]
    async fn test_calculate_progress() {
        // Test progress calculation
        assert_eq!(SyncScheduler::calculate_progress(0, 6, 0.0), 0);
        assert_eq!(SyncScheduler::calculate_progress(0, 6, 0.5), 8);
        assert_eq!(SyncScheduler::calculate_progress(1, 6, 0.0), 16);
        assert_eq!(SyncScheduler::calculate_progress(3, 6, 0.0), 50);
        assert_eq!(SyncScheduler::calculate_progress(6, 6, 0.0), 100);
        
        // Test with zero steps
        assert_eq!(SyncScheduler::calculate_progress(0, 0, 0.0), 100);
    }

    #[tokio::test]
    async fn test_parse_categories() {
        let data = mock_categories_response();
        let result = SyncScheduler::parse_categories(&data);
        
        assert!(result.is_ok());
        let categories = result.unwrap();
        assert_eq!(categories.len(), 3);
        assert_eq!(categories[0].category_id, "1");
        assert_eq!(categories[0].category_name, "Sports");
        assert_eq!(categories[1].category_id, "2");
        assert_eq!(categories[1].category_name, "Movies");
    }

    #[tokio::test]
    async fn test_parse_channels() {
        let data = mock_channels_response();
        let result = SyncScheduler::parse_channels(&data);
        
        assert!(result.is_ok());
        let channels = result.unwrap();
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0].stream_id, 1001);
        assert_eq!(channels[0].name, "ESPN HD");
        assert_eq!(channels[0].category_id, Some("1".to_string()));
        assert_eq!(channels[0].tv_archive, Some(1));
    }

    #[tokio::test]
    async fn test_parse_movies() {
        let data = mock_movies_response();
        let result = SyncScheduler::parse_movies(&data);
        
        assert!(result.is_ok());
        let movies = result.unwrap();
        assert_eq!(movies.len(), 2);
        assert_eq!(movies[0].stream_id, 2001);
        assert_eq!(movies[0].name, "The Matrix");
        assert_eq!(movies[0].year, Some("1999".to_string()));
        assert_eq!(movies[0].rating_5based, Some(4.35));
    }

    #[tokio::test]
    async fn test_parse_series() {
        let data = mock_series_response();
        let result = SyncScheduler::parse_series(&data);
        
        assert!(result.is_ok());
        let series = result.unwrap();
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].series_id, 3001);
        assert_eq!(series[0].name, "Breaking Bad");
        assert_eq!(series[0].rating_5based, Some(4.75));
    }

    #[tokio::test]
    async fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 2.0,
        };
        
        // Simulate backoff calculation
        let mut delay = config.initial_delay_ms;
        assert_eq!(delay, 100);
        
        delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay_ms);
        assert_eq!(delay, 200);
        
        delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay_ms);
        assert_eq!(delay, 400);
        
        delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay_ms);
        assert_eq!(delay, 800);
        
        delay = ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay_ms);
        assert_eq!(delay, 1000); // Capped at max_delay_ms
    }

    #[tokio::test]
    async fn test_progress_tracking_during_sync() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        // Create progress updates
        let mut progress = SyncProgress {
            status: SyncStatus::Syncing,
            progress: 0,
            current_step: "Starting".to_string(),
            channels_synced: 0,
            movies_synced: 0,
            series_synced: 0,
            errors: Vec::new(),
        };
        
        // Update progress at different stages
        scheduler.update_sync_status("test-profile", &progress).unwrap();
        
        progress.progress = 25;
        progress.current_step = "Syncing channels".to_string();
        progress.channels_synced = 100;
        scheduler.update_sync_status("test-profile", &progress).unwrap();
        
        progress.progress = 50;
        progress.current_step = "Syncing movies".to_string();
        progress.movies_synced = 50;
        scheduler.update_sync_status("test-profile", &progress).unwrap();
        
        progress.progress = 75;
        progress.current_step = "Syncing series".to_string();
        progress.series_synced = 25;
        scheduler.update_sync_status("test-profile", &progress).unwrap();
        
        progress.progress = 100;
        progress.status = SyncStatus::Completed;
        progress.current_step = "Complete".to_string();
        scheduler.update_sync_status("test-profile", &progress).unwrap();
        
        // Verify final state
        let final_progress = scheduler.get_sync_status("test-profile").unwrap();
        assert_eq!(final_progress.status, SyncStatus::Completed);
        assert_eq!(final_progress.progress, 100);
        assert_eq!(final_progress.channels_synced, 100);
        assert_eq!(final_progress.movies_synced, 50);
        assert_eq!(final_progress.series_synced, 25);
    }

    #[tokio::test]
    async fn test_error_accumulation_during_sync() {
        let mut progress = SyncProgress::default();
        
        // Add errors
        progress.errors.push("Channel sync failed: timeout".to_string());
        progress.errors.push("Movie sync failed: network error".to_string());
        
        assert_eq!(progress.errors.len(), 2);
        assert!(progress.errors[0].contains("Channel sync"));
        assert!(progress.errors[1].contains("Movie sync"));
    }

// ==================== Sync Workflow Tests ====================

/// Create a complete test database with all content cache tables
fn create_complete_test_db() -> Connection {
    let conn = create_test_db();
    
    // Add content cache tables
    conn.execute(
        "CREATE TABLE xtream_channel_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            category_id TEXT NOT NULL,
            category_name TEXT NOT NULL,
            parent_id INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(profile_id, category_id)
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE xtream_movie_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            category_id TEXT NOT NULL,
            category_name TEXT NOT NULL,
            parent_id INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(profile_id, category_id)
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE xtream_series_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            category_id TEXT NOT NULL,
            category_name TEXT NOT NULL,
            parent_id INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(profile_id, category_id)
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE xtream_channels (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            stream_id INTEGER NOT NULL,
            num INTEGER,
            name TEXT NOT NULL,
            stream_type TEXT,
            stream_icon TEXT,
            thumbnail TEXT,
            epg_channel_id TEXT,
            added TEXT,
            category_id TEXT,
            custom_sid TEXT,
            tv_archive INTEGER DEFAULT 0,
            direct_source TEXT,
            tv_archive_duration INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(profile_id, stream_id)
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE xtream_movies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            stream_id INTEGER NOT NULL,
            num INTEGER,
            name TEXT NOT NULL,
            title TEXT,
            year TEXT,
            stream_type TEXT,
            stream_icon TEXT,
            rating REAL,
            rating_5based REAL,
            genre TEXT,
            added TEXT,
            episode_run_time INTEGER,
            category_id TEXT,
            container_extension TEXT,
            custom_sid TEXT,
            direct_source TEXT,
            release_date TEXT,
            cast TEXT,
            director TEXT,
            plot TEXT,
            youtube_trailer TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(profile_id, stream_id)
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE xtream_series (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            series_id INTEGER NOT NULL,
            num INTEGER,
            name TEXT NOT NULL,
            title TEXT,
            year TEXT,
            cover TEXT,
            plot TEXT,
            cast TEXT,
            director TEXT,
            genre TEXT,
            release_date TEXT,
            last_modified TEXT,
            rating TEXT,
            rating_5based REAL,
            episode_run_time TEXT,
            category_id TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(profile_id, series_id)
        )",
        [],
    ).unwrap();
    
    conn
}

#[tokio::test]
async fn test_full_sync_workflow_success() {
    use crate::content_cache::ContentCache;
    
    // Create complete test database
    let conn = create_complete_test_db();
    let db = Arc::new(Mutex::new(conn));
    let scheduler = SyncScheduler::new(db.clone());
    let content_cache = ContentCache::new(db.clone()).unwrap();
    
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Setup mocks for all sync steps
    // Step 1: Channel categories
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_live_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
        .mount(&mock_server)
        .await;
    
    // Step 2: Channels
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_live_streams"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_channels_response()))
        .mount(&mock_server)
        .await;
    
    // Step 3: Movie categories
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_vod_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
        .mount(&mock_server)
        .await;
    
    // Step 4: Movies
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_vod_streams"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_movies_response()))
        .mount(&mock_server)
        .await;
    
    // Step 5: Series categories
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_series_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
        .mount(&mock_server)
        .await;
    
    // Step 6: Series
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_series"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_series_response()))
        .mount(&mock_server)
        .await;
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);
    let cancel_token = CancellationToken::new();
    
    // Run full sync
    let sync_handle = tokio::spawn(async move {
        scheduler.run_full_sync(
            "test-profile",
            &mock_server.uri(),
            "testuser",
            "testpass",
            &content_cache,
            &progress_tx,
            &cancel_token,
        ).await
    });
    
    // Collect progress updates
    let mut progress_updates = Vec::new();
    while let Some(progress) = progress_rx.recv().await {
        progress_updates.push(progress.clone());
        if progress.status == SyncStatus::Completed || progress.status == SyncStatus::Failed {
            break;
        }
    }
    
    // Wait for sync to complete
    let result = sync_handle.await.unwrap();
    
    // Verify sync completed successfully
    assert!(result.is_ok());
    let final_progress = result.unwrap();
    assert_eq!(final_progress.status, SyncStatus::Completed);
    assert_eq!(final_progress.progress, 100);
    assert!(final_progress.errors.is_empty());
    assert_eq!(final_progress.channels_synced, 2);
    assert_eq!(final_progress.movies_synced, 2);
    assert_eq!(final_progress.series_synced, 2);
    
    // Verify progress updates were sent
    assert!(!progress_updates.is_empty());
    assert!(progress_updates.iter().any(|p| p.current_step.contains("channel")));
    assert!(progress_updates.iter().any(|p| p.current_step.contains("movie")));
    assert!(progress_updates.iter().any(|p| p.current_step.contains("series")));
}

#[tokio::test]
async fn test_sync_workflow_with_partial_failure() {
    use crate::content_cache::ContentCache;
    
    // Create complete test database
    let conn = create_complete_test_db();
    let db = Arc::new(Mutex::new(conn));
    let scheduler = SyncScheduler::new(db.clone());
    let content_cache = ContentCache::new(db.clone()).unwrap();
    
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Setup mocks - channels will fail, but movies and series succeed
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_live_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_live_streams"))
        .respond_with(ResponseTemplate::new(500)) // Fail channels
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_vod_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_vod_streams"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_movies_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_series_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_series"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_series_response()))
        .mount(&mock_server)
        .await;
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);
    let cancel_token = CancellationToken::new();
    
    // Run full sync
    let sync_handle = tokio::spawn(async move {
        scheduler.run_full_sync(
            "test-profile",
            &mock_server.uri(),
            "testuser",
            "testpass",
            &content_cache,
            &progress_tx,
            &cancel_token,
        ).await
    });
    
    // Collect progress updates
    while let Some(progress) = progress_rx.recv().await {
        if progress.status == SyncStatus::Completed || 
           progress.status == SyncStatus::Failed || 
           progress.status == SyncStatus::Partial {
            break;
        }
    }
    
    // Wait for sync to complete
    let result = sync_handle.await.unwrap();
    
    // Verify sync completed with partial status
    assert!(result.is_ok());
    let final_progress = result.unwrap();
    assert_eq!(final_progress.status, SyncStatus::Partial);
    assert_eq!(final_progress.progress, 100);
    assert!(!final_progress.errors.is_empty());
    assert!(final_progress.errors.iter().any(|e| e.contains("Channels sync failed")));
    assert_eq!(final_progress.channels_synced, 0);
    assert_eq!(final_progress.movies_synced, 2);
    assert_eq!(final_progress.series_synced, 2);
}

#[tokio::test]
async fn test_sync_workflow_cancellation() {
    use crate::content_cache::ContentCache;
    
    // Create complete test database
    let conn = create_complete_test_db();
    let db = Arc::new(Mutex::new(conn));
    let scheduler = SyncScheduler::new(db.clone());
    let content_cache = ContentCache::new(db.clone()).unwrap();
    
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Setup mocks with delays
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(mock_categories_response())
                .set_delay(std::time::Duration::from_millis(100))
        )
        .mount(&mock_server)
        .await;
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);
    let cancel_token = CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();
    
    // Run full sync
    let sync_handle = tokio::spawn(async move {
        scheduler.run_full_sync(
            "test-profile",
            &mock_server.uri(),
            "testuser",
            "testpass",
            &content_cache,
            &progress_tx,
            &cancel_token,
        ).await
    });
    
    // Wait for sync to start, then cancel
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    cancel_token_clone.cancel();
    
    // Collect remaining progress updates
    while let Some(_progress) = progress_rx.recv().await {
        // Just drain the channel
    }
    
    // Wait for sync to complete
    let result = sync_handle.await.unwrap();
    
    // Verify sync was cancelled (should have errors)
    assert!(result.is_ok());
    let final_progress = result.unwrap();
    // Status should be Failed or Partial due to cancellation
    assert!(
        final_progress.status == SyncStatus::Failed || 
        final_progress.status == SyncStatus::Partial
    );
}

#[tokio::test]
async fn test_sync_workflow_progress_callbacks() {
    use crate::content_cache::ContentCache;
    
    // Create complete test database
    let conn = create_complete_test_db();
    let db = Arc::new(Mutex::new(conn));
    let scheduler = SyncScheduler::new(db.clone());
    let content_cache = ContentCache::new(db.clone()).unwrap();
    
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Setup mocks for all sync steps
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_channels_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_movies_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_series_response()))
        .mount(&mock_server)
        .await;
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);
    let cancel_token = CancellationToken::new();
    
    // Run full sync
    let sync_handle = tokio::spawn(async move {
        scheduler.run_full_sync(
            "test-profile",
            &mock_server.uri(),
            "testuser",
            "testpass",
            &content_cache,
            &progress_tx,
            &cancel_token,
        ).await
    });
    
    // Collect and verify progress updates
    let mut progress_updates = Vec::new();
    let mut last_progress = 0;
    
    while let Some(progress) = progress_rx.recv().await {
        // Verify progress is monotonically increasing
        assert!(progress.progress >= last_progress);
        last_progress = progress.progress;
        
        progress_updates.push(progress.clone());
        
        if progress.status == SyncStatus::Completed || progress.status == SyncStatus::Failed {
            break;
        }
    }
    
    // Wait for sync to complete
    let _ = sync_handle.await.unwrap();
    
    // Verify we received multiple progress updates
    assert!(progress_updates.len() >= 6); // At least one per step
    
    // Verify progress goes from 0 to 100
    assert_eq!(progress_updates.first().unwrap().progress, 0);
    assert_eq!(progress_updates.last().unwrap().progress, 100);
    
    // Verify we have updates for each content type
    let steps: Vec<String> = progress_updates.iter()
        .map(|p| p.current_step.clone())
        .collect();
    
    assert!(steps.iter().any(|s| s.to_lowercase().contains("channel")));
    assert!(steps.iter().any(|s| s.to_lowercase().contains("movie")));
    assert!(steps.iter().any(|s| s.to_lowercase().contains("series")));
}

#[tokio::test]
async fn test_sync_workflow_pipeline_order() {
    use crate::content_cache::ContentCache;
    
    // Create complete test database
    let conn = create_complete_test_db();
    let db = Arc::new(Mutex::new(conn));
    let scheduler = SyncScheduler::new(db.clone());
    let content_cache = ContentCache::new(db.clone()).unwrap();
    
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Setup mocks
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_channels_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_movies_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_series_response()))
        .mount(&mock_server)
        .await;
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);
    let cancel_token = CancellationToken::new();
    
    // Run full sync
    let sync_handle = tokio::spawn(async move {
        scheduler.run_full_sync(
            "test-profile",
            &mock_server.uri(),
            "testuser",
            "testpass",
            &content_cache,
            &progress_tx,
            &cancel_token,
        ).await
    });
    
    // Collect progress updates and verify order
    let mut steps = Vec::new();
    
    while let Some(progress) = progress_rx.recv().await {
        steps.push(progress.current_step.clone());
        
        if progress.status == SyncStatus::Completed || progress.status == SyncStatus::Failed {
            break;
        }
    }
    
    // Wait for sync to complete
    let _ = sync_handle.await.unwrap();
    
    // Verify the pipeline order: categories → channels → movies → series
    let channel_cat_idx = steps.iter().position(|s| s.contains("channel categories"));
    let channel_idx = steps.iter().position(|s| s.contains("channels") && !s.contains("categories"));
    let movie_cat_idx = steps.iter().position(|s| s.contains("movie categories"));
    let movie_idx = steps.iter().position(|s| s.contains("movies") && !s.contains("categories"));
    let series_cat_idx = steps.iter().position(|s| s.contains("series categories"));
    let series_idx = steps.iter().position(|s| s.contains("series") && !s.contains("categories"));
    
    // Verify order
    if let (Some(cc), Some(c), Some(mc), Some(m), Some(sc), Some(s)) = 
        (channel_cat_idx, channel_idx, movie_cat_idx, movie_idx, series_cat_idx, series_idx) {
        assert!(cc < c, "Channel categories should come before channels");
        assert!(c < mc, "Channels should come before movie categories");
        assert!(mc < m, "Movie categories should come before movies");
        assert!(m < sc, "Movies should come before series categories");
        assert!(sc < s, "Series categories should come before series");
    }
}

#[tokio::test]
async fn test_sync_workflow_error_recovery() {
    use crate::content_cache::ContentCache;
    
    // Create complete test database
    let conn = create_complete_test_db();
    let db = Arc::new(Mutex::new(conn));
    let scheduler = SyncScheduler::new(db.clone());
    let content_cache = ContentCache::new(db.clone()).unwrap();
    
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Setup mocks - first step fails, rest succeed
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_live_categories"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_live_streams"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_vod_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_vod_streams"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_movies_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_series_categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_categories_response()))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path_regex("/player_api.php"))
        .and(query_param("action", "get_series"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_series_response()))
        .mount(&mock_server)
        .await;
    
    // Create progress channel
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);
    let cancel_token = CancellationToken::new();
    
    // Run full sync
    let sync_handle = tokio::spawn(async move {
        scheduler.run_full_sync(
            "test-profile",
            &mock_server.uri(),
            "testuser",
            "testpass",
            &content_cache,
            &progress_tx,
            &cancel_token,
        ).await
    });
    
    // Drain progress updates
    while let Some(progress) = progress_rx.recv().await {
        if progress.status == SyncStatus::Completed || 
           progress.status == SyncStatus::Failed || 
           progress.status == SyncStatus::Partial {
            break;
        }
    }
    
    // Wait for sync to complete
    let result = sync_handle.await.unwrap();
    
    // Verify sync continued despite errors
    assert!(result.is_ok());
    let final_progress = result.unwrap();
    
    // Should be partial since some steps failed
    assert_eq!(final_progress.status, SyncStatus::Partial);
    
    // Should have errors for failed steps
    assert!(!final_progress.errors.is_empty());
    
    // Should have successfully synced movies and series
    assert_eq!(final_progress.movies_synced, 2);
    assert_eq!(final_progress.series_synced, 2);
    assert_eq!(final_progress.channels_synced, 0);
}
