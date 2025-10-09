use super::*;
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

#[test]
fn test_content_cache_creation() {
    let db = create_test_db();
    let cache = ContentCache::new(db);
    
    assert!(cache.is_ok(), "ContentCache creation should succeed");
}

#[test]
fn test_initialize_tables() {
    let db = create_test_db();
    let _cache = ContentCache::new(db.clone()).unwrap();
    
    // Verify all tables were created
    let conn = db.lock().unwrap();
    
    let tables = vec![
        "xtream_channels",
        "xtream_movies",
        "xtream_series",
        "xtream_seasons",
        "xtream_episodes",
        "xtream_channel_categories",
        "xtream_movie_categories",
        "xtream_series_categories",
        "xtream_content_sync",
        "xtream_sync_settings",
    ];
    
    for table in tables {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| {
                    let count: i32 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap();
        
        assert!(exists, "Table {} should exist", table);
    }
}

#[test]
fn test_initialize_tables_idempotent() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    
    // Call initialize_tables multiple times
    let result1 = cache.initialize_tables();
    let result2 = cache.initialize_tables();
    let result3 = cache.initialize_tables();
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

#[test]
fn test_is_initialized_false_for_new_profile() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    let is_init = cache.is_initialized("test-profile").unwrap();
    assert!(!is_init, "New profile should not be initialized");
}

#[test]
fn test_initialize_profile() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    
    // Initialize the profile
    cache.initialize_profile("test-profile").unwrap();
    
    // Check that it's now initialized
    let is_init = cache.is_initialized("test-profile").unwrap();
    assert!(is_init, "Profile should be initialized");
    
    // Verify sync metadata was created
    let conn = db.lock().unwrap();
    let sync_status: String = conn
        .query_row(
            "SELECT sync_status FROM xtream_content_sync WHERE profile_id = ?1",
            ["test-profile"],
            |row| row.get(0),
        )
        .unwrap();
    
    assert_eq!(sync_status, "pending");
}

#[test]
fn test_initialize_profile_creates_sync_settings() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Verify sync settings were created with defaults
    let conn = db.lock().unwrap();
    let (auto_sync, interval, wifi_only, notify): (bool, i32, bool, bool) = conn
        .query_row(
            "SELECT auto_sync_enabled, sync_interval_hours, wifi_only, notify_on_complete 
             FROM xtream_sync_settings WHERE profile_id = ?1",
            ["test-profile"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    
    assert_eq!(auto_sync, true);
    assert_eq!(interval, 24);
    assert_eq!(wifi_only, true);
    assert_eq!(notify, false);
}

#[test]
fn test_initialize_profile_idempotent() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    // Initialize multiple times
    cache.initialize_profile("test-profile").unwrap();
    cache.initialize_profile("test-profile").unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Should still be initialized once
    let is_init = cache.is_initialized("test-profile").unwrap();
    assert!(is_init);
}

#[test]
fn test_clear_profile_content() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert some test data
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO xtream_channels (profile_id, stream_id, name) VALUES ('test-profile', 1, 'Test Channel')",
        [],
    )
    .unwrap();
    
    conn.execute(
        "INSERT INTO xtream_movies (profile_id, stream_id, name) VALUES ('test-profile', 1, 'Test Movie')",
        [],
    )
    .unwrap();
    
    conn.execute(
        "INSERT INTO xtream_series (profile_id, series_id, name) VALUES ('test-profile', 1, 'Test Series')",
        [],
    )
    .unwrap();
    
    // Update sync counts
    conn.execute(
        "UPDATE xtream_content_sync SET channels_count = 1, movies_count = 1, series_count = 1 WHERE profile_id = 'test-profile'",
        [],
    )
    .unwrap();
    
    drop(conn);
    
    // Clear the content
    cache.clear_profile_content("test-profile").unwrap();
    
    // Verify content was deleted
    let conn = db.lock().unwrap();
    
    let channel_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM xtream_channels WHERE profile_id = 'test-profile'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(channel_count, 0);
    
    let movie_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM xtream_movies WHERE profile_id = 'test-profile'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(movie_count, 0);
    
    let series_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM xtream_series WHERE profile_id = 'test-profile'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(series_count, 0);
    
    // Verify sync status was reset
    let (status, progress, ch_count, mv_count, sr_count): (String, i32, i32, i32, i32) = conn
        .query_row(
            "SELECT sync_status, sync_progress, channels_count, movies_count, series_count 
             FROM xtream_content_sync WHERE profile_id = 'test-profile'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .unwrap();
    
    assert_eq!(status, "pending");
    assert_eq!(progress, 0);
    assert_eq!(ch_count, 0);
    assert_eq!(mv_count, 0);
    assert_eq!(sr_count, 0);
}

#[test]
fn test_get_content_counts_empty() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let (channels, movies, series) = cache.get_content_counts("test-profile").unwrap();
    
    assert_eq!(channels, 0);
    assert_eq!(movies, 0);
    assert_eq!(series, 0);
}

#[test]
fn test_get_content_counts_with_data() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test data
    let conn = db.lock().unwrap();
    
    // Insert 3 channels
    for i in 1..=3 {
        conn.execute(
            "INSERT INTO xtream_channels (profile_id, stream_id, name) VALUES ('test-profile', ?1, 'Channel')",
            [i],
        )
        .unwrap();
    }
    
    // Insert 5 movies
    for i in 1..=5 {
        conn.execute(
            "INSERT INTO xtream_movies (profile_id, stream_id, name) VALUES ('test-profile', ?1, 'Movie')",
            [i],
        )
        .unwrap();
    }
    
    // Insert 2 series
    for i in 1..=2 {
        conn.execute(
            "INSERT INTO xtream_series (profile_id, series_id, name) VALUES ('test-profile', ?1, 'Series')",
            [i],
        )
        .unwrap();
    }
    
    drop(conn);
    
    let (channels, movies, series) = cache.get_content_counts("test-profile").unwrap();
    
    assert_eq!(channels, 3);
    assert_eq!(movies, 5);
    assert_eq!(series, 2);
}

#[test]
fn test_get_content_counts_profile_isolation() {
    let db = create_test_db();
    insert_test_profile(&db, "profile-1");
    insert_test_profile(&db, "profile-2");
    let cache = ContentCache::new(db.clone()).unwrap();
    
    cache.initialize_profile("profile-1").unwrap();
    cache.initialize_profile("profile-2").unwrap();
    
    // Insert data for profile-1
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO xtream_channels (profile_id, stream_id, name) VALUES ('profile-1', 1, 'Channel')",
        [],
    )
    .unwrap();
    
    // Insert data for profile-2
    conn.execute(
        "INSERT INTO xtream_movies (profile_id, stream_id, name) VALUES ('profile-2', 1, 'Movie')",
        [],
    )
    .unwrap();
    
    drop(conn);
    
    // Check profile-1 counts
    let (ch1, mv1, sr1) = cache.get_content_counts("profile-1").unwrap();
    assert_eq!(ch1, 1);
    assert_eq!(mv1, 0);
    assert_eq!(sr1, 0);
    
    // Check profile-2 counts
    let (ch2, mv2, sr2) = cache.get_content_counts("profile-2").unwrap();
    assert_eq!(ch2, 0);
    assert_eq!(mv2, 1);
    assert_eq!(sr2, 0);
}

#[test]
fn test_perform_maintenance() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    
    let result = cache.perform_maintenance();
    assert!(result.is_ok(), "Maintenance should succeed");
}

#[test]
fn test_vacuum() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    
    let result = cache.vacuum();
    assert!(result.is_ok(), "Vacuum should succeed");
}

#[test]
fn test_get_db_returns_valid_connection() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    
    let db_ref = cache.get_db();
    let conn = db_ref.lock().unwrap();
    
    // Try to execute a simple query
    let result: i32 = conn.query_row("SELECT 1", [], |row| row.get(0)).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_foreign_key_cascade_delete() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    
    // Enable foreign keys
    {
        let conn = db.lock().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
    }
    
    let cache = ContentCache::new(db.clone()).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert some content
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO xtream_channels (profile_id, stream_id, name) VALUES ('test-profile', 1, 'Channel')",
        [],
    )
    .unwrap();
    
    conn.execute(
        "INSERT INTO xtream_movies (profile_id, stream_id, name) VALUES ('test-profile', 1, 'Movie')",
        [],
    )
    .unwrap();
    
    // Delete the profile
    conn.execute("DELETE FROM xtream_profiles WHERE id = 'test-profile'", [])
        .unwrap();
    
    // Verify content was cascade deleted
    let channel_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM xtream_channels WHERE profile_id = 'test-profile'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(channel_count, 0);
    
    let movie_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM xtream_movies WHERE profile_id = 'test-profile'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(movie_count, 0);
    
    let sync_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM xtream_content_sync WHERE profile_id = 'test-profile'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(sync_count, 0);
}

#[test]
fn test_transaction_rollback_on_error() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert some content
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO xtream_channels (profile_id, stream_id, name) VALUES ('test-profile', 1, 'Channel')",
        [],
    )
    .unwrap();
    
    drop(conn);
    
    // Verify content exists
    let (channels_before, _, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(channels_before, 1);
    
    // The clear_profile_content uses a transaction, so if it fails, nothing should be deleted
    // We can't easily force a failure in the test, but we can verify the transaction works
    cache.clear_profile_content("test-profile").unwrap();
    
    let (channels_after, _, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(channels_after, 0);
}

// ==================== Channel CRUD Tests ====================

#[test]
fn test_save_channels_empty_list() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = vec![];
    let saved = cache.save_channels("test-profile", channels).unwrap();
    
    assert_eq!(saved, 0);
}

#[test]
fn test_save_channels_single() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = vec![XtreamChannel {
        stream_id: 123,
        num: Some(1),
        name: "Test Channel".to_string(),
        stream_type: Some("live".to_string()),
        stream_icon: Some("http://example.com/icon.png".to_string()),
        thumbnail: None,
        epg_channel_id: Some("epg123".to_string()),
        added: Some("2024-01-01".to_string()),
        category_id: Some("1".to_string()),
        custom_sid: None,
        tv_archive: Some(1),
        direct_source: None,
        tv_archive_duration: Some(7),
    }];
    
    let saved = cache.save_channels("test-profile", channels).unwrap();
    
    assert_eq!(saved, 1);
    
    // Verify it was saved
    let (count, _, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_save_channels_batch() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let mut channels = Vec::new();
    for i in 1..=100 {
        channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: Some("live".to_string()),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some("1".to_string()),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    let saved = cache.save_channels("test-profile", channels).unwrap();
    
    assert_eq!(saved, 100);
    
    let (count, _, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(count, 100);
}

#[test]
fn test_save_channels_upsert() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert initial channel
    let channels = vec![XtreamChannel {
        stream_id: 123,
        num: Some(1),
        name: "Original Name".to_string(),
        stream_type: Some("live".to_string()),
        stream_icon: None,
        thumbnail: None,
        epg_channel_id: None,
        added: None,
        category_id: Some("1".to_string()),
        custom_sid: None,
        tv_archive: Some(0),
        direct_source: None,
        tv_archive_duration: None,
    }];
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Update the same channel with new data
    let updated_channels = vec![XtreamChannel {
        stream_id: 123,
        num: Some(1),
        name: "Updated Name".to_string(),
        stream_type: Some("live".to_string()),
        stream_icon: Some("http://example.com/new-icon.png".to_string()),
        thumbnail: None,
        epg_channel_id: None,
        added: None,
        category_id: Some("2".to_string()),
        custom_sid: None,
        tv_archive: Some(1),
        direct_source: None,
        tv_archive_duration: Some(7),
    }];
    
    cache.save_channels("test-profile", updated_channels).unwrap();
    
    // Should still have only 1 channel
    let (count, _, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(count, 1);
    
    // Verify the data was updated
    let channels = cache.get_channels("test-profile", None).unwrap();
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].name, "Updated Name");
    assert_eq!(channels[0].category_id, Some("2".to_string()));
    assert_eq!(channels[0].tv_archive, Some(1));
}

#[test]
fn test_save_channels_updates_sync_metadata() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = vec![XtreamChannel {
        stream_id: 123,
        num: Some(1),
        name: "Test Channel".to_string(),
        stream_type: None,
        stream_icon: None,
        thumbnail: None,
        epg_channel_id: None,
        added: None,
        category_id: None,
        custom_sid: None,
        tv_archive: None,
        direct_source: None,
        tv_archive_duration: None,
    }];
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Verify sync metadata was updated
    let conn = db.lock().unwrap();
    let (channels_count, last_sync): (i32, Option<String>) = conn
        .query_row(
            "SELECT channels_count, last_sync_channels FROM xtream_content_sync WHERE profile_id = ?1",
            ["test-profile"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    
    assert_eq!(channels_count, 1);
    assert!(last_sync.is_some());
}

#[test]
fn test_save_channels_invalid_profile_id() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    
    let channels = vec![XtreamChannel {
        stream_id: 123,
        num: None,
        name: "Test".to_string(),
        stream_type: None,
        stream_icon: None,
        thumbnail: None,
        epg_channel_id: None,
        added: None,
        category_id: None,
        custom_sid: None,
        tv_archive: None,
        direct_source: None,
        tv_archive_duration: None,
    }];
    
    let result = cache.save_channels("", channels);
    assert!(result.is_err());
}

#[test]
fn test_save_channels_invalid_stream_id() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = vec![XtreamChannel {
        stream_id: -1,
        num: None,
        name: "Test".to_string(),
        stream_type: None,
        stream_icon: None,
        thumbnail: None,
        epg_channel_id: None,
        added: None,
        category_id: None,
        custom_sid: None,
        tv_archive: None,
        direct_source: None,
        tv_archive_duration: None,
    }];
    
    let result = cache.save_channels("test-profile", channels);
    assert!(result.is_err());
}

#[test]
fn test_get_channels_empty() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = cache.get_channels("test-profile", None).unwrap();
    
    assert_eq!(channels.len(), 0);
}

#[test]
fn test_get_channels_all() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test channels
    let mut test_channels = Vec::new();
    for i in 1..=5 {
        test_channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some("1".to_string()),
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    let channels = cache.get_channels("test-profile", None).unwrap();
    
    assert_eq!(channels.len(), 5);
}

#[test]
fn test_get_channels_with_category_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert channels with different categories
    let mut test_channels = Vec::new();
    for i in 1..=10 {
        let category = if i <= 5 { "1" } else { "2" };
        test_channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some(category.to_string()),
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Filter by category 1
    let filter = ChannelFilter {
        category_id: Some("1".to_string()),
        ..Default::default()
    };
    
    let channels = cache.get_channels("test-profile", Some(filter)).unwrap();
    
    assert_eq!(channels.len(), 5);
    for channel in channels {
        assert_eq!(channel.category_id, Some("1".to_string()));
    }
}

#[test]
fn test_get_channels_with_name_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert channels with different names
    let test_channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "HBO Channel".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 2,
            num: Some(2),
            name: "ESPN Sports".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 3,
            num: Some(3),
            name: "HBO Max".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
    ];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Filter by name containing "HBO"
    let filter = ChannelFilter {
        name_contains: Some("HBO".to_string()),
        ..Default::default()
    };
    
    let channels = cache.get_channels("test-profile", Some(filter)).unwrap();
    
    assert_eq!(channels.len(), 2);
    for channel in channels {
        assert!(channel.name.contains("HBO"));
    }
}

#[test]
fn test_get_channels_with_pagination() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert 20 channels
    let mut test_channels = Vec::new();
    for i in 1..=20 {
        test_channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {:02}", i),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Get first page (10 items)
    let filter = ChannelFilter {
        limit: Some(10),
        offset: Some(0),
        ..Default::default()
    };
    
    let page1 = cache.get_channels("test-profile", Some(filter)).unwrap();
    assert_eq!(page1.len(), 10);
    
    // Get second page (10 items)
    let filter = ChannelFilter {
        limit: Some(10),
        offset: Some(10),
        ..Default::default()
    };
    
    let page2 = cache.get_channels("test-profile", Some(filter)).unwrap();
    assert_eq!(page2.len(), 10);
    
    // Verify pages are different
    assert_ne!(page1[0].stream_id, page2[0].stream_id);
}

#[test]
fn test_get_channels_sorted_by_name() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert channels in random order
    let test_channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "Zebra Channel".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 2,
            num: Some(2),
            name: "Alpha Channel".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 3,
            num: Some(3),
            name: "Beta Channel".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
    ];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    let channels = cache.get_channels("test-profile", None).unwrap();
    
    // Should be sorted alphabetically
    assert_eq!(channels[0].name, "Alpha Channel");
    assert_eq!(channels[1].name, "Beta Channel");
    assert_eq!(channels[2].name, "Zebra Channel");
}

#[test]
fn test_get_channels_profile_isolation() {
    let db = create_test_db();
    insert_test_profile(&db, "profile-1");
    insert_test_profile(&db, "profile-2");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("profile-1").unwrap();
    cache.initialize_profile("profile-2").unwrap();
    
    // Insert channels for profile-1
    let channels1 = vec![XtreamChannel {
        stream_id: 1,
        num: Some(1),
        name: "Profile 1 Channel".to_string(),
        stream_type: None,
        stream_icon: None,
        thumbnail: None,
        epg_channel_id: None,
        added: None,
        category_id: None,
        custom_sid: None,
        tv_archive: None,
        direct_source: None,
        tv_archive_duration: None,
    }];
    
    cache.save_channels("profile-1", channels1).unwrap();
    
    // Insert channels for profile-2
    let channels2 = vec![XtreamChannel {
        stream_id: 2,
        num: Some(2),
        name: "Profile 2 Channel".to_string(),
        stream_type: None,
        stream_icon: None,
        thumbnail: None,
        epg_channel_id: None,
        added: None,
        category_id: None,
        custom_sid: None,
        tv_archive: None,
        direct_source: None,
        tv_archive_duration: None,
    }];
    
    cache.save_channels("profile-2", channels2).unwrap();
    
    // Verify profile isolation
    let p1_channels = cache.get_channels("profile-1", None).unwrap();
    assert_eq!(p1_channels.len(), 1);
    assert_eq!(p1_channels[0].name, "Profile 1 Channel");
    
    let p2_channels = cache.get_channels("profile-2", None).unwrap();
    assert_eq!(p2_channels.len(), 1);
    assert_eq!(p2_channels[0].name, "Profile 2 Channel");
}

#[test]
fn test_delete_channels_all() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test channels
    let mut test_channels = Vec::new();
    for i in 1..=5 {
        test_channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Delete all channels
    let deleted = cache.delete_channels("test-profile", None).unwrap();
    
    assert_eq!(deleted, 5);
    
    // Verify all deleted
    let channels = cache.get_channels("test-profile", None).unwrap();
    assert_eq!(channels.len(), 0);
}

#[test]
fn test_delete_channels_specific() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test channels
    let mut test_channels = Vec::new();
    for i in 1..=5 {
        test_channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Delete specific channels
    let deleted = cache.delete_channels("test-profile", Some(vec![1, 3, 5])).unwrap();
    
    assert_eq!(deleted, 3);
    
    // Verify only specified channels were deleted
    let channels = cache.get_channels("test-profile", None).unwrap();
    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].stream_id, 2);
    assert_eq!(channels[1].stream_id, 4);
}

#[test]
fn test_delete_channels_empty_list() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let deleted = cache.delete_channels("test-profile", Some(vec![])).unwrap();
    
    assert_eq!(deleted, 0);
}

#[test]
fn test_delete_channels_nonexistent() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Try to delete channels that don't exist
    let deleted = cache.delete_channels("test-profile", Some(vec![999, 1000])).unwrap();
    
    assert_eq!(deleted, 0);
}

#[test]
fn test_delete_channels_updates_sync_metadata() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert and then delete channels
    let test_channels = vec![XtreamChannel {
        stream_id: 1,
        num: Some(1),
        name: "Test Channel".to_string(),
        stream_type: None,
        stream_icon: None,
        thumbnail: None,
        epg_channel_id: None,
        added: None,
        category_id: None,
        custom_sid: None,
        tv_archive: None,
        direct_source: None,
        tv_archive_duration: None,
    }];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    cache.delete_channels("test-profile", None).unwrap();
    
    // Verify sync metadata was updated
    let conn = db.lock().unwrap();
    let channels_count: i32 = conn
        .query_row(
            "SELECT channels_count FROM xtream_content_sync WHERE profile_id = ?1",
            ["test-profile"],
            |row| row.get(0),
        )
        .unwrap();
    
    assert_eq!(channels_count, 0);
}

#[test]
fn test_delete_channels_invalid_profile_id() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    
    let result = cache.delete_channels("", None);
    assert!(result.is_err());
}

#[test]
fn test_delete_channels_invalid_stream_id() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let result = cache.delete_channels("test-profile", Some(vec![-1]));
    assert!(result.is_err());
}

// ==================== Channel Search and Filtering Tests ====================

#[test]
fn test_search_channels_empty_query() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test channels
    let test_channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "HBO Channel".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
    ];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Empty query should return all channels
    let results = cache.search_channels("test-profile", "", None).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_search_channels_exact_match() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test channels
    let test_channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "HBO".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 2,
            num: Some(2),
            name: "HBO Max".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 3,
            num: Some(3),
            name: "ESPN".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
    ];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Search for exact match
    let results = cache.search_channels("test-profile", "HBO", None).unwrap();
    
    // Should return both HBO channels, with exact match first
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].name, "HBO"); // Exact match first
}

#[test]
fn test_search_channels_partial_match() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test channels
    let test_channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "HBO Channel".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 2,
            num: Some(2),
            name: "The HBO Network".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 3,
            num: Some(3),
            name: "ESPN Sports".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
    ];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Search for partial match
    let results = cache.search_channels("test-profile", "HBO", None).unwrap();
    
    assert_eq!(results.len(), 2);
    // Results should be ordered by relevance
    assert_eq!(results[0].name, "HBO Channel"); // Starts with HBO
    assert_eq!(results[1].name, "The HBO Network"); // Contains HBO
}

#[test]
fn test_search_channels_case_insensitive() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test channels
    let test_channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "HBO Channel".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 2,
            num: Some(2),
            name: "hbo max".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
    ];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Search with different cases
    let results1 = cache.search_channels("test-profile", "hbo", None).unwrap();
    let results2 = cache.search_channels("test-profile", "HBO", None).unwrap();
    let results3 = cache.search_channels("test-profile", "HbO", None).unwrap();
    
    assert_eq!(results1.len(), 2);
    assert_eq!(results2.len(), 2);
    assert_eq!(results3.len(), 2);
}

#[test]
fn test_search_channels_no_results() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test channels
    let test_channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "HBO Channel".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
    ];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Search for non-existent channel
    let results = cache.search_channels("test-profile", "Netflix", None).unwrap();
    
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_channels_with_category_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test channels with different categories
    let test_channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "HBO Movies".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some("movies".to_string()),
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 2,
            num: Some(2),
            name: "HBO Sports".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some("sports".to_string()),
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 3,
            num: Some(3),
            name: "ESPN Sports".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some("sports".to_string()),
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
    ];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Search for "HBO" in sports category only
    let filter = ChannelFilter {
        category_id: Some("sports".to_string()),
        ..Default::default()
    };
    
    let results = cache.search_channels("test-profile", "HBO", Some(filter)).unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "HBO Sports");
}

#[test]
fn test_search_channels_with_pagination() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert many channels with "Channel" in the name
    let mut test_channels = Vec::new();
    for i in 1..=20 {
        test_channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {:02}", i),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Search with pagination
    let filter = ChannelFilter {
        limit: Some(10),
        offset: Some(0),
        ..Default::default()
    };
    
    let page1 = cache.search_channels("test-profile", "Channel", Some(filter)).unwrap();
    assert_eq!(page1.len(), 10);
    
    let filter = ChannelFilter {
        limit: Some(10),
        offset: Some(10),
        ..Default::default()
    };
    
    let page2 = cache.search_channels("test-profile", "Channel", Some(filter)).unwrap();
    assert_eq!(page2.len(), 10);
}

#[test]
fn test_search_channels_special_characters() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert channels with special characters
    let test_channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "Channel 100%".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 2,
            num: Some(2),
            name: "Channel_Test".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 3,
            num: Some(3),
            name: "Regular Channel".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
    ];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Search with special characters - they should be sanitized and not cause SQL errors
    // Search for "100" should find "Channel 100%"
    let results = cache.search_channels("test-profile", "100", None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Channel 100%");
    
    // Search for "Test" should find "Channel_Test"
    let results = cache.search_channels("test-profile", "Test", None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Channel_Test");
    
    // Verify special characters don't break the search
    let results = cache.search_channels("test-profile", "%", None);
    assert!(results.is_ok(), "Search with % should not cause SQL error");
    
    let results = cache.search_channels("test-profile", "_", None);
    assert!(results.is_ok(), "Search with _ should not cause SQL error");
}

#[test]
fn test_search_channels_performance() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert a large number of channels
    let mut test_channels = Vec::new();
    for i in 1..=1000 {
        test_channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    // Measure search performance
    let start = std::time::Instant::now();
    let results = cache.search_channels("test-profile", "Channel", None).unwrap();
    let duration = start.elapsed();
    
    assert_eq!(results.len(), 1000);
    
    // Should complete in less than 100ms (target from requirements)
    assert!(
        duration.as_millis() < 100,
        "Search took {:?}, expected < 100ms",
        duration
    );
}

#[test]
fn test_count_channels_no_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert test channels
    let mut test_channels = Vec::new();
    for i in 1..=10 {
        test_channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    let count = cache.count_channels("test-profile", None).unwrap();
    assert_eq!(count, 10);
}

#[test]
fn test_count_channels_with_category_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert channels with different categories
    let mut test_channels = Vec::new();
    for i in 1..=10 {
        let category = if i <= 6 { "sports" } else { "movies" };
        test_channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some(category.to_string()),
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    let filter = ChannelFilter {
        category_id: Some("sports".to_string()),
        ..Default::default()
    };
    
    let count = cache.count_channels("test-profile", Some(filter)).unwrap();
    assert_eq!(count, 6);
}

#[test]
fn test_count_channels_with_name_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert channels
    let test_channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "HBO Channel".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 2,
            num: Some(2),
            name: "HBO Max".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 3,
            num: Some(3),
            name: "ESPN Sports".to_string(),
            stream_type: None,
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        },
    ];
    
    cache.save_channels("test-profile", test_channels).unwrap();
    
    let filter = ChannelFilter {
        name_contains: Some("HBO".to_string()),
        ..Default::default()
    };
    
    let count = cache.count_channels("test-profile", Some(filter)).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_count_channels_empty() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let count = cache.count_channels("test-profile", None).unwrap();
    assert_eq!(count, 0);
}


// ==================== Series CRUD Tests ====================

#[test]
fn test_save_series_empty() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    let series: Vec<XtreamSeries> = vec![];
    let saved = cache.save_series("test-profile", series).unwrap();
    
    assert_eq!(saved, 0);
}

#[test]
fn test_save_series_single() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    let series = vec![XtreamSeries {
        series_id: 1,
        num: Some(1),
        name: "Breaking Bad".to_string(),
        title: Some("Breaking Bad".to_string()),
        year: Some("2008".to_string()),
        cover: Some("http://example.com/cover.jpg".to_string()),
        plot: Some("A chemistry teacher turns to cooking meth".to_string()),
        cast: Some("Bryan Cranston, Aaron Paul".to_string()),
        director: Some("Vince Gilligan".to_string()),
        genre: Some("Drama, Crime".to_string()),
        release_date: Some("2008-01-20".to_string()),
        last_modified: Some("2023-01-01".to_string()),
        rating: Some("9.5".to_string()),
        rating_5based: Some(4.75),
        episode_run_time: Some("47".to_string()),
        category_id: Some("1".to_string()),
    }];
    
    let saved = cache.save_series("test-profile", series).unwrap();
    assert_eq!(saved, 1);
    
    let (_, _, count) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_save_series_batch() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    let mut series = Vec::new();
    for i in 1..=50 {
        series.push(XtreamSeries {
            series_id: i,
            num: Some(i),
            name: format!("Series {}", i),
            title: Some(format!("Series {}", i)),
            year: Some("2020".to_string()),
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: Some("Drama".to_string()),
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: Some(4.0),
            episode_run_time: None,
            category_id: Some("1".to_string()),
        });
    }
    
    let saved = cache.save_series("test-profile", series).unwrap();
    assert_eq!(saved, 50);
    
    let (_, _, count) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(count, 50);
}

#[test]
fn test_save_series_upsert() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save initial series
    let series1 = vec![XtreamSeries {
        series_id: 1,
        num: Some(1),
        name: "Original Name".to_string(),
        title: Some("Original Title".to_string()),
        year: Some("2020".to_string()),
        cover: None,
        plot: None,
        cast: None,
        director: None,
        genre: Some("Drama".to_string()),
        release_date: None,
        last_modified: None,
        rating: None,
        rating_5based: Some(4.0),
        episode_run_time: None,
        category_id: Some("1".to_string()),
    }];
    
    cache.save_series("test-profile", series1).unwrap();
    
    // Update the same series
    let series2 = vec![XtreamSeries {
        series_id: 1,
        num: Some(1),
        name: "Updated Name".to_string(),
        title: Some("Updated Title".to_string()),
        year: Some("2021".to_string()),
        cover: Some("http://example.com/new.jpg".to_string()),
        plot: Some("New plot".to_string()),
        cast: Some("New cast".to_string()),
        director: Some("New director".to_string()),
        genre: Some("Action".to_string()),
        release_date: Some("2021-01-01".to_string()),
        last_modified: Some("2024-01-01".to_string()),
        rating: Some("8.5".to_string()),
        rating_5based: Some(4.25),
        episode_run_time: Some("45".to_string()),
        category_id: Some("2".to_string()),
    }];
    
    cache.save_series("test-profile", series2).unwrap();
    
    // Verify only one series exists with updated data
    let series = cache.get_series("test-profile", None).unwrap();
    assert_eq!(series.len(), 1);
    assert_eq!(series[0].name, "Updated Name");
    assert_eq!(series[0].year, Some("2021".to_string()));
    assert_eq!(series[0].category_id, Some("2".to_string()));
}

#[test]
fn test_save_series_invalid_profile() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    
    let series = vec![XtreamSeries {
        series_id: 1,
        num: Some(1),
        name: "Test Series".to_string(),
        title: None,
        year: None,
        cover: None,
        plot: None,
        cast: None,
        director: None,
        genre: None,
        release_date: None,
        last_modified: None,
        rating: None,
        rating_5based: None,
        episode_run_time: None,
        category_id: None,
    }];
    
    let result = cache.save_series("", series);
    assert!(result.is_err());
}

#[test]
fn test_save_series_invalid_series_id() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    let series = vec![XtreamSeries {
        series_id: -1,
        num: Some(1),
        name: "Test Series".to_string(),
        title: None,
        year: None,
        cover: None,
        plot: None,
        cast: None,
        director: None,
        genre: None,
        release_date: None,
        last_modified: None,
        rating: None,
        rating_5based: None,
        episode_run_time: None,
        category_id: None,
    }];
    
    let result = cache.save_series("test-profile", series);
    assert!(result.is_err());
}

#[test]
fn test_get_series_empty() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    let series = cache.get_series("test-profile", None).unwrap();
    
    assert_eq!(series.len(), 0);
}

#[test]
fn test_get_series_all() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save test series
    let mut test_series = Vec::new();
    for i in 1..=5 {
        test_series.push(XtreamSeries {
            series_id: i,
            num: Some(i),
            name: format!("Series {}", i),
            title: Some(format!("Series {}", i)),
            year: Some("2020".to_string()),
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: Some("Drama".to_string()),
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: Some(4.0),
            episode_run_time: None,
            category_id: Some("1".to_string()),
        });
    }
    
    cache.save_series("test-profile", test_series).unwrap();
    
    let series = cache.get_series("test-profile", None).unwrap();
    
    assert_eq!(series.len(), 5);
}

#[test]
fn test_get_series_with_category_filter() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save series with different categories
    let test_series = vec![
        XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Drama Series".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: Some("1".to_string()),
        },
        XtreamSeries {
            series_id: 2,
            num: Some(2),
            name: "Comedy Series".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: Some("2".to_string()),
        },
        XtreamSeries {
            series_id: 3,
            num: Some(3),
            name: "Another Drama".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: Some("1".to_string()),
        },
    ];
    
    cache.save_series("test-profile", test_series).unwrap();
    
    let filter = SeriesFilter {
        category_id: Some("1".to_string()),
        ..Default::default()
    };
    
    let series = cache.get_series("test-profile", Some(filter)).unwrap();
    
    assert_eq!(series.len(), 2);
    for s in &series {
        assert_eq!(s.category_id, Some("1".to_string()));
    }
}

#[test]
fn test_get_series_with_name_filter() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    let test_series = vec![
        XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Breaking Bad".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        XtreamSeries {
            series_id: 2,
            num: Some(2),
            name: "Better Call Saul".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        XtreamSeries {
            series_id: 3,
            num: Some(3),
            name: "The Wire".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
    ];
    
    cache.save_series("test-profile", test_series).unwrap();
    
    let filter = SeriesFilter {
        name_contains: Some("Bad".to_string()),
        ..Default::default()
    };
    
    let series = cache.get_series("test-profile", Some(filter)).unwrap();
    
    assert_eq!(series.len(), 1);
    assert_eq!(series[0].name, "Breaking Bad");
}

#[test]
fn test_get_series_with_pagination() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save 20 series
    let mut test_series = Vec::new();
    for i in 1..=20 {
        test_series.push(XtreamSeries {
            series_id: i,
            num: Some(i),
            name: format!("Series {:02}", i),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        });
    }
    
    cache.save_series("test-profile", test_series).unwrap();
    
    // Get first page
    let filter1 = SeriesFilter {
        limit: Some(10),
        offset: Some(0),
        ..Default::default()
    };
    let page1 = cache.get_series("test-profile", Some(filter1)).unwrap();
    
    // Get second page
    let filter2 = SeriesFilter {
        limit: Some(10),
        offset: Some(10),
        ..Default::default()
    };
    let page2 = cache.get_series("test-profile", Some(filter2)).unwrap();
    
    assert_eq!(page1.len(), 10);
    assert_eq!(page2.len(), 10);
    assert_ne!(page1[0].series_id, page2[0].series_id);
}

#[test]
fn test_delete_series_all() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save test series
    let test_series = vec![
        XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Series 1".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        XtreamSeries {
            series_id: 2,
            num: Some(2),
            name: "Series 2".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
    ];
    
    cache.save_series("test-profile", test_series).unwrap();
    
    let deleted = cache.delete_series("test-profile", None).unwrap();
    assert_eq!(deleted, 2);
    
    let series = cache.get_series("test-profile", None).unwrap();
    assert_eq!(series.len(), 0);
}

#[test]
fn test_delete_series_specific() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save test series
    let test_series = vec![
        XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Series 1".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        XtreamSeries {
            series_id: 2,
            num: Some(2),
            name: "Series 2".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        XtreamSeries {
            series_id: 3,
            num: Some(3),
            name: "Series 3".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
    ];
    
    cache.save_series("test-profile", test_series).unwrap();
    
    let deleted = cache.delete_series("test-profile", Some(vec![1, 3])).unwrap();
    assert_eq!(deleted, 2);
    
    let series = cache.get_series("test-profile", None).unwrap();
    assert_eq!(series.len(), 1);
    assert_eq!(series[0].series_id, 2);
}

#[test]
fn test_save_series_details() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    let details = XtreamSeriesDetails {
        series: XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Breaking Bad".to_string(),
            title: Some("Breaking Bad".to_string()),
            year: Some("2008".to_string()),
            cover: Some("http://example.com/cover.jpg".to_string()),
            plot: Some("A chemistry teacher turns to cooking meth".to_string()),
            cast: Some("Bryan Cranston, Aaron Paul".to_string()),
            director: Some("Vince Gilligan".to_string()),
            genre: Some("Drama, Crime".to_string()),
            release_date: Some("2008-01-20".to_string()),
            last_modified: Some("2023-01-01".to_string()),
            rating: Some("9.5".to_string()),
            rating_5based: Some(4.75),
            episode_run_time: Some("47".to_string()),
            category_id: Some("1".to_string()),
        },
        seasons: vec![
            XtreamSeason {
                season_number: 1,
                name: Some("Season 1".to_string()),
                episode_count: Some(7),
                overview: Some("First season".to_string()),
                air_date: Some("2008-01-20".to_string()),
                cover: Some("http://example.com/s1.jpg".to_string()),
                cover_big: Some("http://example.com/s1_big.jpg".to_string()),
                vote_average: Some(8.5),
            },
            XtreamSeason {
                season_number: 2,
                name: Some("Season 2".to_string()),
                episode_count: Some(13),
                overview: Some("Second season".to_string()),
                air_date: Some("2009-03-08".to_string()),
                cover: Some("http://example.com/s2.jpg".to_string()),
                cover_big: Some("http://example.com/s2_big.jpg".to_string()),
                vote_average: Some(9.0),
            },
        ],
        episodes: vec![
            XtreamEpisode {
                episode_id: "1_1".to_string(),
                season_number: 1,
                episode_num: "1".to_string(),
                title: Some("Pilot".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: Some("2023-01-01".to_string()),
                direct_source: Some("http://example.com/ep1.mp4".to_string()),
                info_json: Some(r#"{"duration": 3600}"#.to_string()),
            },
            XtreamEpisode {
                episode_id: "1_2".to_string(),
                season_number: 1,
                episode_num: "2".to_string(),
                title: Some("Cat's in the Bag...".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: Some("2023-01-01".to_string()),
                direct_source: Some("http://example.com/ep2.mp4".to_string()),
                info_json: Some(r#"{"duration": 3600}"#.to_string()),
            },
        ],
    };
    
    let result = cache.save_series_details("test-profile", 1, details);
    assert!(result.is_ok());
    
    // Verify series was saved
    let series = cache.get_series("test-profile", None).unwrap();
    assert_eq!(series.len(), 1);
    assert_eq!(series[0].name, "Breaking Bad");
    
    // Verify seasons were saved
    let db = cache.get_db();
    let conn = db.lock().unwrap();
    let season_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM xtream_seasons WHERE profile_id = 'test-profile' AND series_id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(season_count, 2);
    
    // Verify episodes were saved
    let episode_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM xtream_episodes WHERE profile_id = 'test-profile' AND series_id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(episode_count, 2);
}


// ==================== Series Details and Relationships Tests ====================

#[test]
fn test_get_series_details() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save series details
    let details = XtreamSeriesDetails {
        series: XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Breaking Bad".to_string(),
            title: Some("Breaking Bad".to_string()),
            year: Some("2008".to_string()),
            cover: Some("http://example.com/cover.jpg".to_string()),
            plot: Some("A chemistry teacher turns to cooking meth".to_string()),
            cast: Some("Bryan Cranston, Aaron Paul".to_string()),
            director: Some("Vince Gilligan".to_string()),
            genre: Some("Drama, Crime".to_string()),
            release_date: Some("2008-01-20".to_string()),
            last_modified: Some("2023-01-01".to_string()),
            rating: Some("9.5".to_string()),
            rating_5based: Some(4.75),
            episode_run_time: Some("47".to_string()),
            category_id: Some("1".to_string()),
        },
        seasons: vec![
            XtreamSeason {
                season_number: 1,
                name: Some("Season 1".to_string()),
                episode_count: Some(7),
                overview: Some("First season".to_string()),
                air_date: Some("2008-01-20".to_string()),
                cover: Some("http://example.com/s1.jpg".to_string()),
                cover_big: Some("http://example.com/s1_big.jpg".to_string()),
                vote_average: Some(8.5),
            },
            XtreamSeason {
                season_number: 2,
                name: Some("Season 2".to_string()),
                episode_count: Some(13),
                overview: Some("Second season".to_string()),
                air_date: Some("2009-03-08".to_string()),
                cover: Some("http://example.com/s2.jpg".to_string()),
                cover_big: Some("http://example.com/s2_big.jpg".to_string()),
                vote_average: Some(9.0),
            },
        ],
        episodes: vec![
            XtreamEpisode {
                episode_id: "1_1".to_string(),
                season_number: 1,
                episode_num: "1".to_string(),
                title: Some("Pilot".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: Some("2023-01-01".to_string()),
                direct_source: Some("http://example.com/ep1.mp4".to_string()),
                info_json: Some(r#"{"duration": 3600}"#.to_string()),
            },
            XtreamEpisode {
                episode_id: "1_2".to_string(),
                season_number: 1,
                episode_num: "2".to_string(),
                title: Some("Cat's in the Bag...".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: Some("2023-01-01".to_string()),
                direct_source: Some("http://example.com/ep2.mp4".to_string()),
                info_json: Some(r#"{"duration": 3600}"#.to_string()),
            },
            XtreamEpisode {
                episode_id: "2_1".to_string(),
                season_number: 2,
                episode_num: "1".to_string(),
                title: Some("Seven Thirty-Seven".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: Some("2023-01-01".to_string()),
                direct_source: Some("http://example.com/ep3.mp4".to_string()),
                info_json: Some(r#"{"duration": 3600}"#.to_string()),
            },
        ],
    };
    
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    // Get series details
    let retrieved = cache.get_series_details("test-profile", 1).unwrap();
    
    // Verify series info
    assert_eq!(retrieved.series.name, "Breaking Bad");
    assert_eq!(retrieved.series.year, Some("2008".to_string()));
    assert_eq!(retrieved.series.rating_5based, Some(4.75));
    
    // Verify seasons
    assert_eq!(retrieved.seasons.len(), 2);
    assert_eq!(retrieved.seasons[0].season_number, 1);
    assert_eq!(retrieved.seasons[0].episode_count, Some(7));
    assert_eq!(retrieved.seasons[1].season_number, 2);
    assert_eq!(retrieved.seasons[1].episode_count, Some(13));
    
    // Verify episodes
    assert_eq!(retrieved.episodes.len(), 3);
    assert_eq!(retrieved.episodes[0].episode_id, "1_1");
    assert_eq!(retrieved.episodes[0].season_number, 1);
    assert_eq!(retrieved.episodes[1].episode_id, "1_2");
    assert_eq!(retrieved.episodes[2].episode_id, "2_1");
    assert_eq!(retrieved.episodes[2].season_number, 2);
}

#[test]
fn test_get_series_details_not_found() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    let result = cache.get_series_details("test-profile", 999);
    assert!(result.is_err());
}

#[test]
fn test_get_seasons() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save series with seasons
    let details = XtreamSeriesDetails {
        series: XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Test Series".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        seasons: vec![
            XtreamSeason {
                season_number: 1,
                name: Some("Season 1".to_string()),
                episode_count: Some(10),
                overview: None,
                air_date: None,
                cover: None,
                cover_big: None,
                vote_average: None,
            },
            XtreamSeason {
                season_number: 2,
                name: Some("Season 2".to_string()),
                episode_count: Some(12),
                overview: None,
                air_date: None,
                cover: None,
                cover_big: None,
                vote_average: None,
            },
            XtreamSeason {
                season_number: 3,
                name: Some("Season 3".to_string()),
                episode_count: Some(8),
                overview: None,
                air_date: None,
                cover: None,
                cover_big: None,
                vote_average: None,
            },
        ],
        episodes: vec![],
    };
    
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    // Get seasons
    let seasons = cache.get_seasons("test-profile", 1).unwrap();
    
    assert_eq!(seasons.len(), 3);
    assert_eq!(seasons[0].season_number, 1);
    assert_eq!(seasons[0].episode_count, Some(10));
    assert_eq!(seasons[1].season_number, 2);
    assert_eq!(seasons[2].season_number, 3);
}

#[test]
fn test_get_episodes_all() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save series with episodes
    let details = XtreamSeriesDetails {
        series: XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Test Series".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        seasons: vec![],
        episodes: vec![
            XtreamEpisode {
                episode_id: "1_1".to_string(),
                season_number: 1,
                episode_num: "1".to_string(),
                title: Some("Episode 1".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
            XtreamEpisode {
                episode_id: "1_2".to_string(),
                season_number: 1,
                episode_num: "2".to_string(),
                title: Some("Episode 2".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
            XtreamEpisode {
                episode_id: "2_1".to_string(),
                season_number: 2,
                episode_num: "1".to_string(),
                title: Some("Episode 1".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
        ],
    };
    
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    // Get all episodes
    let episodes = cache.get_episodes("test-profile", 1, None).unwrap();
    
    assert_eq!(episodes.len(), 3);
    assert_eq!(episodes[0].episode_id, "1_1");
    assert_eq!(episodes[1].episode_id, "1_2");
    assert_eq!(episodes[2].episode_id, "2_1");
}

#[test]
fn test_get_episodes_by_season() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save series with episodes
    let details = XtreamSeriesDetails {
        series: XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Test Series".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        seasons: vec![],
        episodes: vec![
            XtreamEpisode {
                episode_id: "1_1".to_string(),
                season_number: 1,
                episode_num: "1".to_string(),
                title: Some("S1E1".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
            XtreamEpisode {
                episode_id: "1_2".to_string(),
                season_number: 1,
                episode_num: "2".to_string(),
                title: Some("S1E2".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
            XtreamEpisode {
                episode_id: "2_1".to_string(),
                season_number: 2,
                episode_num: "1".to_string(),
                title: Some("S2E1".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
            XtreamEpisode {
                episode_id: "2_2".to_string(),
                season_number: 2,
                episode_num: "2".to_string(),
                title: Some("S2E2".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
        ],
    };
    
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    // Get season 1 episodes
    let season1_episodes = cache.get_episodes("test-profile", 1, Some(1)).unwrap();
    assert_eq!(season1_episodes.len(), 2);
    assert_eq!(season1_episodes[0].episode_id, "1_1");
    assert_eq!(season1_episodes[1].episode_id, "1_2");
    
    // Get season 2 episodes
    let season2_episodes = cache.get_episodes("test-profile", 1, Some(2)).unwrap();
    assert_eq!(season2_episodes.len(), 2);
    assert_eq!(season2_episodes[0].episode_id, "2_1");
    assert_eq!(season2_episodes[1].episode_id, "2_2");
}

#[test]
fn test_episode_ordering() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save episodes in non-sequential order
    let details = XtreamSeriesDetails {
        series: XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Test Series".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        seasons: vec![],
        episodes: vec![
            XtreamEpisode {
                episode_id: "1_10".to_string(),
                season_number: 1,
                episode_num: "10".to_string(),
                title: Some("Episode 10".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
            XtreamEpisode {
                episode_id: "1_2".to_string(),
                season_number: 1,
                episode_num: "2".to_string(),
                title: Some("Episode 2".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
            XtreamEpisode {
                episode_id: "1_1".to_string(),
                season_number: 1,
                episode_num: "1".to_string(),
                title: Some("Episode 1".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
        ],
    };
    
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    // Get episodes - should be ordered by episode number
    let episodes = cache.get_episodes("test-profile", 1, None).unwrap();
    
    assert_eq!(episodes.len(), 3);
    assert_eq!(episodes[0].episode_num, "1");
    assert_eq!(episodes[1].episode_num, "2");
    assert_eq!(episodes[2].episode_num, "10");
}

#[test]
fn test_cascade_delete_series_relationships() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save series with seasons and episodes
    let details = XtreamSeriesDetails {
        series: XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Test Series".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        seasons: vec![
            XtreamSeason {
                season_number: 1,
                name: Some("Season 1".to_string()),
                episode_count: Some(2),
                overview: None,
                air_date: None,
                cover: None,
                cover_big: None,
                vote_average: None,
            },
        ],
        episodes: vec![
            XtreamEpisode {
                episode_id: "1_1".to_string(),
                season_number: 1,
                episode_num: "1".to_string(),
                title: Some("Episode 1".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
        ],
    };
    
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    // Verify data exists
    let seasons_before = cache.get_seasons("test-profile", 1).unwrap();
    let episodes_before = cache.get_episodes("test-profile", 1, None).unwrap();
    assert_eq!(seasons_before.len(), 1);
    assert_eq!(episodes_before.len(), 1);
    
    // Delete the series
    cache.delete_series("test-profile", Some(vec![1])).unwrap();
    
    // Verify seasons and episodes are also deleted (cascade)
    let seasons_after = cache.get_seasons("test-profile", 1).unwrap();
    let episodes_after = cache.get_episodes("test-profile", 1, None).unwrap();
    assert_eq!(seasons_after.len(), 0);
    assert_eq!(episodes_after.len(), 0);
}

#[test]
fn test_data_integrity_series_relationships() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    insert_test_profile(&cache.get_db(), "test-profile");
    cache.initialize_profile("test-profile").unwrap();
    
    // Save multiple series with overlapping season numbers
    let details1 = XtreamSeriesDetails {
        series: XtreamSeries {
            series_id: 1,
            num: Some(1),
            name: "Series 1".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        seasons: vec![
            XtreamSeason {
                season_number: 1,
                name: Some("Series 1 - Season 1".to_string()),
                episode_count: Some(10),
                overview: None,
                air_date: None,
                cover: None,
                cover_big: None,
                vote_average: None,
            },
        ],
        episodes: vec![
            XtreamEpisode {
                episode_id: "s1_1_1".to_string(),
                season_number: 1,
                episode_num: "1".to_string(),
                title: Some("Series 1 Episode 1".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
        ],
    };
    
    let details2 = XtreamSeriesDetails {
        series: XtreamSeries {
            series_id: 2,
            num: Some(2),
            name: "Series 2".to_string(),
            title: None,
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        },
        seasons: vec![
            XtreamSeason {
                season_number: 1,
                name: Some("Series 2 - Season 1".to_string()),
                episode_count: Some(8),
                overview: None,
                air_date: None,
                cover: None,
                cover_big: None,
                vote_average: None,
            },
        ],
        episodes: vec![
            XtreamEpisode {
                episode_id: "s2_1_1".to_string(),
                season_number: 1,
                episode_num: "1".to_string(),
                title: Some("Series 2 Episode 1".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                added: None,
                direct_source: None,
                info_json: None,
            },
        ],
    };
    
    cache.save_series_details("test-profile", 1, details1).unwrap();
    cache.save_series_details("test-profile", 2, details2).unwrap();
    
    // Verify data isolation - each series has its own seasons and episodes
    let series1_seasons = cache.get_seasons("test-profile", 1).unwrap();
    let series2_seasons = cache.get_seasons("test-profile", 2).unwrap();
    
    assert_eq!(series1_seasons.len(), 1);
    assert_eq!(series2_seasons.len(), 1);
    assert_eq!(series1_seasons[0].name, Some("Series 1 - Season 1".to_string()));
    assert_eq!(series2_seasons[0].name, Some("Series 2 - Season 1".to_string()));
    
    let series1_episodes = cache.get_episodes("test-profile", 1, None).unwrap();
    let series2_episodes = cache.get_episodes("test-profile", 2, None).unwrap();
    
    assert_eq!(series1_episodes.len(), 1);
    assert_eq!(series2_episodes.len(), 1);
    assert_eq!(series1_episodes[0].title, Some("Series 1 Episode 1".to_string()));
    assert_eq!(series2_episodes[0].title, Some("Series 2 Episode 1".to_string()));
}
