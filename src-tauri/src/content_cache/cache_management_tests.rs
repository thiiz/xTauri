// Integration tests for cache management commands
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
) -> XtreamMovie {
    XtreamMovie {
        stream_id,
        num: Some(stream_id),
        name: name.to_string(),
        title: Some(name.to_string()),
        year: Some("2023".to_string()),
        stream_type: Some("movie".to_string()),
        stream_icon: None,
        rating: Some(8.5),
        rating_5based: Some(4.25),
        genre: Some("Action".to_string()),
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
) -> XtreamSeries {
    XtreamSeries {
        series_id,
        num: Some(series_id),
        name: name.to_string(),
        title: Some(name.to_string()),
        year: Some("2023".to_string()),
        cover: None,
        plot: None,
        cast: None,
        director: None,
        genre: Some("Drama".to_string()),
        release_date: None,
        last_modified: None,
        rating: None,
        rating_5based: Some(4.5),
        episode_run_time: None,
        category_id: Some(category_id.to_string()),
    }
}

#[test]
fn test_get_content_cache_stats_empty() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let (channels, movies, series) = cache.get_content_counts("test-profile").unwrap();
    
    assert_eq!(channels, 0);
    assert_eq!(movies, 0);
    assert_eq!(series, 0);
}

#[test]
fn test_get_content_cache_stats_with_channels() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add some channels
    let channels = vec![
        create_test_channel(1, "Channel 1", "cat1"),
        create_test_channel(2, "Channel 2", "cat1"),
        create_test_channel(3, "Channel 3", "cat2"),
    ];
    cache.save_channels("test-profile", channels).unwrap();
    
    let (channels_count, movies_count, series_count) = cache.get_content_counts("test-profile").unwrap();
    
    assert_eq!(channels_count, 3);
    assert_eq!(movies_count, 0);
    assert_eq!(series_count, 0);
}

#[test]
fn test_get_content_cache_stats_with_movies() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add some movies
    let movies = vec![
        create_test_movie(1, "Movie 1", "cat1"),
        create_test_movie(2, "Movie 2", "cat1"),
    ];
    cache.save_movies("test-profile", movies).unwrap();
    
    let (channels_count, movies_count, series_count) = cache.get_content_counts("test-profile").unwrap();
    
    assert_eq!(channels_count, 0);
    assert_eq!(movies_count, 2);
    assert_eq!(series_count, 0);
}

#[test]
fn test_get_content_cache_stats_with_series() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add some series
    let series = vec![
        create_test_series(1, "Series 1", "cat1"),
        create_test_series(2, "Series 2", "cat1"),
        create_test_series(3, "Series 3", "cat2"),
        create_test_series(4, "Series 4", "cat2"),
    ];
    cache.save_series("test-profile", series).unwrap();
    
    let (channels_count, movies_count, series_count) = cache.get_content_counts("test-profile").unwrap();
    
    assert_eq!(channels_count, 0);
    assert_eq!(movies_count, 0);
    assert_eq!(series_count, 4);
}

#[test]
fn test_get_content_cache_stats_with_all_content_types() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add channels
    let channels = vec![
        create_test_channel(1, "Channel 1", "cat1"),
        create_test_channel(2, "Channel 2", "cat1"),
    ];
    cache.save_channels("test-profile", channels).unwrap();
    
    // Add movies
    let movies = vec![
        create_test_movie(1, "Movie 1", "cat1"),
        create_test_movie(2, "Movie 2", "cat1"),
        create_test_movie(3, "Movie 3", "cat2"),
    ];
    cache.save_movies("test-profile", movies).unwrap();
    
    // Add series
    let series = vec![
        create_test_series(1, "Series 1", "cat1"),
    ];
    cache.save_series("test-profile", series).unwrap();
    
    let (channels_count, movies_count, series_count) = cache.get_content_counts("test-profile").unwrap();
    
    assert_eq!(channels_count, 2);
    assert_eq!(movies_count, 3);
    assert_eq!(series_count, 1);
}

#[test]
fn test_get_content_cache_stats_profile_isolation() {
    let db = create_test_db();
    insert_test_profile(&db, "profile-1");
    insert_test_profile(&db, "profile-2");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("profile-1").unwrap();
    cache.initialize_profile("profile-2").unwrap();
    
    // Add content to profile-1
    let channels = vec![
        create_test_channel(1, "Channel 1", "cat1"),
        create_test_channel(2, "Channel 2", "cat1"),
    ];
    cache.save_channels("profile-1", channels).unwrap();
    
    // Add content to profile-2
    let movies = vec![
        create_test_movie(1, "Movie 1", "cat1"),
        create_test_movie(2, "Movie 2", "cat1"),
        create_test_movie(3, "Movie 3", "cat2"),
    ];
    cache.save_movies("profile-2", movies).unwrap();
    
    // Check profile-1 stats
    let (ch1, mv1, sr1) = cache.get_content_counts("profile-1").unwrap();
    assert_eq!(ch1, 2);
    assert_eq!(mv1, 0);
    assert_eq!(sr1, 0);
    
    // Check profile-2 stats
    let (ch2, mv2, sr2) = cache.get_content_counts("profile-2").unwrap();
    assert_eq!(ch2, 0);
    assert_eq!(mv2, 3);
    assert_eq!(sr2, 0);
}

#[test]
fn test_clear_content_cache_empty() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Clear empty cache should succeed
    cache.clear_profile_content("test-profile").unwrap();
    
    let (channels, movies, series) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(channels, 0);
    assert_eq!(movies, 0);
    assert_eq!(series, 0);
}

#[test]
fn test_clear_content_cache_with_channels() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add channels
    let channels = vec![
        create_test_channel(1, "Channel 1", "cat1"),
        create_test_channel(2, "Channel 2", "cat1"),
        create_test_channel(3, "Channel 3", "cat2"),
    ];
    cache.save_channels("test-profile", channels).unwrap();
    
    // Verify channels exist
    let (channels_before, _, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(channels_before, 3);
    
    // Clear cache
    cache.clear_profile_content("test-profile").unwrap();
    
    // Verify channels were deleted
    let (channels_after, _, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(channels_after, 0);
}

#[test]
fn test_clear_content_cache_with_movies() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add movies
    let movies = vec![
        create_test_movie(1, "Movie 1", "cat1"),
        create_test_movie(2, "Movie 2", "cat1"),
    ];
    cache.save_movies("test-profile", movies).unwrap();
    
    // Verify movies exist
    let (_, movies_before, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(movies_before, 2);
    
    // Clear cache
    cache.clear_profile_content("test-profile").unwrap();
    
    // Verify movies were deleted
    let (_, movies_after, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(movies_after, 0);
}

#[test]
fn test_clear_content_cache_with_series() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add series
    let series = vec![
        create_test_series(1, "Series 1", "cat1"),
        create_test_series(2, "Series 2", "cat1"),
        create_test_series(3, "Series 3", "cat2"),
    ];
    cache.save_series("test-profile", series).unwrap();
    
    // Verify series exist
    let (_, _, series_before) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(series_before, 3);
    
    // Clear cache
    cache.clear_profile_content("test-profile").unwrap();
    
    // Verify series were deleted
    let (_, _, series_after) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(series_after, 0);
}

#[test]
fn test_clear_content_cache_with_all_content_types() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add all types of content
    let channels = vec![
        create_test_channel(1, "Channel 1", "cat1"),
        create_test_channel(2, "Channel 2", "cat1"),
    ];
    cache.save_channels("test-profile", channels).unwrap();
    
    let movies = vec![
        create_test_movie(1, "Movie 1", "cat1"),
        create_test_movie(2, "Movie 2", "cat1"),
        create_test_movie(3, "Movie 3", "cat2"),
    ];
    cache.save_movies("test-profile", movies).unwrap();
    
    let series = vec![
        create_test_series(1, "Series 1", "cat1"),
    ];
    cache.save_series("test-profile", series).unwrap();
    
    // Verify all content exists
    let (ch_before, mv_before, sr_before) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(ch_before, 2);
    assert_eq!(mv_before, 3);
    assert_eq!(sr_before, 1);
    
    // Clear cache
    cache.clear_profile_content("test-profile").unwrap();
    
    // Verify all content was deleted
    let (ch_after, mv_after, sr_after) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(ch_after, 0);
    assert_eq!(mv_after, 0);
    assert_eq!(sr_after, 0);
}

#[test]
fn test_clear_content_cache_profile_isolation() {
    let db = create_test_db();
    insert_test_profile(&db, "profile-1");
    insert_test_profile(&db, "profile-2");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("profile-1").unwrap();
    cache.initialize_profile("profile-2").unwrap();
    
    // Add content to both profiles
    let channels = vec![
        create_test_channel(1, "Channel 1", "cat1"),
        create_test_channel(2, "Channel 2", "cat1"),
    ];
    cache.save_channels("profile-1", channels.clone()).unwrap();
    cache.save_channels("profile-2", channels).unwrap();
    
    // Clear only profile-1
    cache.clear_profile_content("profile-1").unwrap();
    
    // Verify profile-1 is empty
    let (ch1, mv1, sr1) = cache.get_content_counts("profile-1").unwrap();
    assert_eq!(ch1, 0);
    assert_eq!(mv1, 0);
    assert_eq!(sr1, 0);
    
    // Verify profile-2 still has content
    let (ch2, mv2, sr2) = cache.get_content_counts("profile-2").unwrap();
    assert_eq!(ch2, 2);
    assert_eq!(mv2, 0);
    assert_eq!(sr2, 0);
}

#[test]
fn test_clear_content_cache_preserves_sync_settings() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Create sync scheduler to manage settings
    let scheduler = SyncScheduler::new(Arc::clone(&db));
    
    // Set sync settings
    let settings = SyncSettings {
        auto_sync_enabled: true,
        sync_interval_hours: 12,
        wifi_only: true,
        notify_on_complete: true,
    };
    scheduler.update_sync_settings("test-profile", &settings).unwrap();
    
    // Add some content
    let channels = vec![
        create_test_channel(1, "Channel 1", "cat1"),
    ];
    cache.save_channels("test-profile", channels).unwrap();
    
    // Clear cache
    cache.clear_profile_content("test-profile").unwrap();
    
    // Verify content was deleted
    let (ch, mv, sr) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(ch, 0);
    assert_eq!(mv, 0);
    assert_eq!(sr, 0);
    
    // Verify sync settings are preserved
    let retrieved_settings = scheduler.get_sync_settings("test-profile").unwrap();
    assert_eq!(retrieved_settings.auto_sync_enabled, true);
    assert_eq!(retrieved_settings.sync_interval_hours, 12);
    assert_eq!(retrieved_settings.wifi_only, true);
    assert_eq!(retrieved_settings.notify_on_complete, true);
}

#[test]
fn test_clear_content_cache_nonexistent_profile() {
    let db = create_test_db();
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    
    // Try to clear cache for non-existent profile
    // This should not panic, but may return an error or succeed silently
    let result = cache.clear_profile_content("nonexistent-profile");
    
    // The behavior depends on implementation - it might succeed (no-op) or fail
    // Either is acceptable as long as it doesn't panic
    match result {
        Ok(_) => {
            // No-op is acceptable
        }
        Err(_) => {
            // Error is also acceptable
        }
    }
}

#[test]
fn test_get_content_cache_stats_nonexistent_profile() {
    let db = create_test_db();
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    
    // Try to get stats for non-existent profile
    let result = cache.get_content_counts("nonexistent-profile");
    
    // Should return zeros or an error
    match result {
        Ok((ch, mv, sr)) => {
            assert_eq!(ch, 0);
            assert_eq!(mv, 0);
            assert_eq!(sr, 0);
        }
        Err(_) => {
            // Error is also acceptable
        }
    }
}

#[test]
fn test_clear_content_cache_multiple_times() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add content
    let channels = vec![
        create_test_channel(1, "Channel 1", "cat1"),
    ];
    cache.save_channels("test-profile", channels).unwrap();
    
    // Clear cache first time
    cache.clear_profile_content("test-profile").unwrap();
    let (ch1, _, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(ch1, 0);
    
    // Clear cache second time (should be idempotent)
    cache.clear_profile_content("test-profile").unwrap();
    let (ch2, _, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(ch2, 0);
}

#[test]
fn test_cache_stats_after_partial_clear_and_refill() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add initial content
    let channels = vec![
        create_test_channel(1, "Channel 1", "cat1"),
        create_test_channel(2, "Channel 2", "cat1"),
    ];
    cache.save_channels("test-profile", channels).unwrap();
    
    let movies = vec![
        create_test_movie(1, "Movie 1", "cat1"),
    ];
    cache.save_movies("test-profile", movies).unwrap();
    
    // Verify initial stats
    let (ch1, mv1, sr1) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(ch1, 2);
    assert_eq!(mv1, 1);
    assert_eq!(sr1, 0);
    
    // Clear cache
    cache.clear_profile_content("test-profile").unwrap();
    
    // Add new content
    let new_channels = vec![
        create_test_channel(3, "Channel 3", "cat2"),
    ];
    cache.save_channels("test-profile", new_channels).unwrap();
    
    let new_series = vec![
        create_test_series(1, "Series 1", "cat1"),
        create_test_series(2, "Series 2", "cat1"),
    ];
    cache.save_series("test-profile", new_series).unwrap();
    
    // Verify new stats
    let (ch2, mv2, sr2) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(ch2, 1);
    assert_eq!(mv2, 0);
    assert_eq!(sr2, 2);
}

#[test]
fn test_clear_content_cache_with_large_dataset() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(Arc::clone(&db)).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Add a large number of channels
    let mut channels = Vec::new();
    for i in 1..=1000 {
        channels.push(create_test_channel(i, &format!("Channel {}", i), "cat1"));
    }
    cache.save_channels("test-profile", channels).unwrap();
    
    // Add a large number of movies
    let mut movies = Vec::new();
    for i in 1..=500 {
        movies.push(create_test_movie(i, &format!("Movie {}", i), "cat1"));
    }
    cache.save_movies("test-profile", movies).unwrap();
    
    // Verify large dataset
    let (ch_before, mv_before, _) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(ch_before, 1000);
    assert_eq!(mv_before, 500);
    
    // Clear cache (should handle large dataset efficiently)
    cache.clear_profile_content("test-profile").unwrap();
    
    // Verify all content was deleted
    let (ch_after, mv_after, sr_after) = cache.get_content_counts("test-profile").unwrap();
    assert_eq!(ch_after, 0);
    assert_eq!(mv_after, 0);
    assert_eq!(sr_after, 0);
}
