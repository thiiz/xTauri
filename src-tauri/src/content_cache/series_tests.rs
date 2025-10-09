use super::*;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Create a test database with required dependencies
fn create_test_db() -> Arc<Mutex<Connection>> {
    let conn = Connection::open_in_memory().unwrap();
    
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

/// Create sample series data for testing
fn create_sample_series(series_id: i64, name: &str, category_id: Option<&str>) -> XtreamSeries {
    XtreamSeries {
        series_id,
        num: Some(series_id),
        name: name.to_string(),
        title: Some(name.to_string()),
        year: Some("2023".to_string()),
        cover: Some("http://example.com/cover.jpg".to_string()),
        plot: Some("An exciting series".to_string()),
        cast: Some("Actor 1, Actor 2".to_string()),
        director: Some("Director Name".to_string()),
        genre: Some("Drama".to_string()),
        release_date: Some("2023-01-01".to_string()),
        last_modified: Some("2023-12-01".to_string()),
        rating: Some("8.5".to_string()),
        rating_5based: Some(4.25),
        episode_run_time: Some("45".to_string()),
        category_id: category_id.map(|s| s.to_string()),
    }
}

/// Create sample season data for testing
fn create_sample_season(season_number: i64) -> XtreamSeason {
    XtreamSeason {
        season_number,
        name: Some(format!("Season {}", season_number)),
        episode_count: Some(10),
        overview: Some(format!("Season {} overview", season_number)),
        air_date: Some("2023-01-01".to_string()),
        cover: Some("http://example.com/season_cover.jpg".to_string()),
        cover_big: Some("http://example.com/season_cover_big.jpg".to_string()),
        vote_average: Some(8.5),
    }
}

/// Create sample episode data for testing
fn create_sample_episode(episode_id: &str, season_number: i64, episode_num: &str) -> XtreamEpisode {
    XtreamEpisode {
        episode_id: episode_id.to_string(),
        season_number,
        episode_num: episode_num.to_string(),
        title: Some(format!("Episode {}", episode_num)),
        container_extension: Some("mp4".to_string()),
        custom_sid: None,
        added: Some("2023-01-01".to_string()),
        direct_source: None,
        info_json: Some(r#"{"duration": 2700}"#.to_string()),
    }
}

/// Create complete series details for testing
fn create_sample_series_details(series_id: i64, name: &str) -> XtreamSeriesDetails {
    let series = create_sample_series(series_id, name, Some("1"));
    let seasons = vec![
        create_sample_season(1),
        create_sample_season(2),
    ];
    let episodes = vec![
        create_sample_episode(&format!("{}_1_1", series_id), 1, "1"),
        create_sample_episode(&format!("{}_1_2", series_id), 1, "2"),
        create_sample_episode(&format!("{}_2_1", series_id), 2, "1"),
        create_sample_episode(&format!("{}_2_2", series_id), 2, "2"),
    ];
    
    XtreamSeriesDetails {
        series,
        seasons,
        episodes,
    }
}

// ==================== Series CRUD Tests ====================

#[test]
fn test_save_series_empty_list() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let result = cache.save_series("test-profile", vec![]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_save_series_single() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let series = create_sample_series(1, "Test Series", Some("1"));
    let result = cache.save_series("test-profile", vec![series.clone()]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
    
    // Verify series was saved
    let series_list = cache.get_series("test-profile", None).unwrap();
    assert_eq!(series_list.len(), 1);
    assert_eq!(series_list[0].series_id, 1);
    assert_eq!(series_list[0].name, "Test Series");
}

#[test]
fn test_save_series_batch() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let series = vec![
        create_sample_series(1, "Series 1", Some("1")),
        create_sample_series(2, "Series 2", Some("1")),
        create_sample_series(3, "Series 3", Some("2")),
    ];
    
    let result = cache.save_series("test-profile", series);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);
    
    let saved_series = cache.get_series("test-profile", None).unwrap();
    assert_eq!(saved_series.len(), 3);
}

#[test]
fn test_save_series_upsert() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Save initial series
    let series1 = create_sample_series(1, "Original Title", Some("1"));
    cache.save_series("test-profile", vec![series1]).unwrap();
    
    // Update the same series with new data
    let mut series2 = create_sample_series(1, "Updated Title", Some("1"));
    series2.rating_5based = Some(4.8);
    cache.save_series("test-profile", vec![series2]).unwrap();
    
    // Verify only one series exists with updated data
    let series_list = cache.get_series("test-profile", None).unwrap();
    assert_eq!(series_list.len(), 1);
    assert_eq!(series_list[0].name, "Updated Title");
    assert_eq!(series_list[0].rating_5based, Some(4.8));
}

#[test]
fn test_save_series_updates_sync_metadata() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let series = vec![
        create_sample_series(1, "Series 1", Some("1")),
        create_sample_series(2, "Series 2", Some("1")),
    ];
    
    cache.save_series("test-profile", series).unwrap();
    
    // Verify sync metadata was updated
    let conn = db.lock().unwrap();
    let (count, last_sync): (i32, Option<String>) = conn
        .query_row(
            "SELECT series_count, last_sync_series FROM xtream_content_sync WHERE profile_id = ?1",
            ["test-profile"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    
    assert_eq!(count, 2);
    assert!(last_sync.is_some());
}

#[test]
fn test_get_series_empty() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let series = cache.get_series("test-profile", None).unwrap();
    assert_eq!(series.len(), 0);
}

#[test]
fn test_get_series_all() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let series = vec![
        create_sample_series(1, "Series A", Some("1")),
        create_sample_series(2, "Series B", Some("1")),
        create_sample_series(3, "Series C", Some("2")),
    ];
    
    cache.save_series("test-profile", series).unwrap();
    
    let retrieved = cache.get_series("test-profile", None).unwrap();
    assert_eq!(retrieved.len(), 3);
}

#[test]
fn test_get_series_with_category_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let series = vec![
        create_sample_series(1, "Drama Series 1", Some("drama")),
        create_sample_series(2, "Drama Series 2", Some("drama")),
        create_sample_series(3, "Comedy Series", Some("comedy")),
    ];
    
    cache.save_series("test-profile", series).unwrap();
    
    let filter = SeriesFilter {
        category_id: Some("drama".to_string()),
        ..Default::default()
    };
    
    let filtered = cache.get_series("test-profile", Some(filter)).unwrap();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|s| s.category_id == Some("drama".to_string())));
}

#[test]
fn test_get_series_with_genre_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let mut series1 = create_sample_series(1, "Drama Series", Some("1"));
    series1.genre = Some("Drama".to_string());
    
    let mut series2 = create_sample_series(2, "Action Series", Some("1"));
    series2.genre = Some("Action".to_string());
    
    cache.save_series("test-profile", vec![series1, series2]).unwrap();
    
    let filter = SeriesFilter {
        genre: Some("Drama".to_string()),
        ..Default::default()
    };
    
    let filtered = cache.get_series("test-profile", Some(filter)).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "Drama Series");
}

#[test]
fn test_get_series_with_rating_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let mut series1 = create_sample_series(1, "Great Series", Some("1"));
    series1.rating_5based = Some(4.5);
    
    let mut series2 = create_sample_series(2, "Good Series", Some("1"));
    series2.rating_5based = Some(3.8);
    
    let mut series3 = create_sample_series(3, "Average Series", Some("1"));
    series3.rating_5based = Some(2.5);
    
    cache.save_series("test-profile", vec![series1, series2, series3]).unwrap();
    
    let filter = SeriesFilter {
        min_rating: Some(3.5),
        ..Default::default()
    };
    
    let filtered = cache.get_series("test-profile", Some(filter)).unwrap();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|s| s.rating_5based.unwrap_or(0.0) >= 3.5));
}

#[test]
fn test_delete_series_all() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let series = vec![
        create_sample_series(1, "Series 1", Some("1")),
        create_sample_series(2, "Series 2", Some("1")),
    ];
    
    cache.save_series("test-profile", series).unwrap();
    
    let deleted = cache.delete_series("test-profile", None).unwrap();
    assert_eq!(deleted, 2);
    
    let remaining = cache.get_series("test-profile", None).unwrap();
    assert_eq!(remaining.len(), 0);
}

#[test]
fn test_delete_series_specific() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let series = vec![
        create_sample_series(1, "Series 1", Some("1")),
        create_sample_series(2, "Series 2", Some("1")),
        create_sample_series(3, "Series 3", Some("1")),
    ];
    
    cache.save_series("test-profile", series).unwrap();
    
    let deleted = cache.delete_series("test-profile", Some(vec![1, 3])).unwrap();
    assert_eq!(deleted, 2);
    
    let remaining = cache.get_series("test-profile", None).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].series_id, 2);
}

// ==================== Series Details Tests ====================

#[test]
fn test_save_series_details() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let details = create_sample_series_details(1, "Test Series");
    let result = cache.save_series_details("test-profile", 1, details);
    
    assert!(result.is_ok());
    
    // Verify series was saved
    let series_list = cache.get_series("test-profile", None).unwrap();
    assert_eq!(series_list.len(), 1);
    
    // Verify seasons were saved
    let seasons = cache.get_seasons("test-profile", 1).unwrap();
    assert_eq!(seasons.len(), 2);
    
    // Verify episodes were saved
    let episodes = cache.get_episodes("test-profile", 1, None).unwrap();
    assert_eq!(episodes.len(), 4);
}

#[test]
fn test_get_series_details() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let details = create_sample_series_details(1, "Test Series");
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    let retrieved = cache.get_series_details("test-profile", 1).unwrap();
    
    assert_eq!(retrieved.series.series_id, 1);
    assert_eq!(retrieved.series.name, "Test Series");
    assert_eq!(retrieved.seasons.len(), 2);
    assert_eq!(retrieved.episodes.len(), 4);
}

#[test]
fn test_get_series_details_not_found() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let result = cache.get_series_details("test-profile", 999);
    assert!(result.is_err());
}

#[test]
fn test_get_seasons() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let details = create_sample_series_details(1, "Test Series");
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    let seasons = cache.get_seasons("test-profile", 1).unwrap();
    
    assert_eq!(seasons.len(), 2);
    assert_eq!(seasons[0].season_number, 1);
    assert_eq!(seasons[1].season_number, 2);
}

#[test]
fn test_get_episodes_all() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let details = create_sample_series_details(1, "Test Series");
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    let episodes = cache.get_episodes("test-profile", 1, None).unwrap();
    
    assert_eq!(episodes.len(), 4);
}

#[test]
fn test_get_episodes_by_season() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let details = create_sample_series_details(1, "Test Series");
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    let season1_episodes = cache.get_episodes("test-profile", 1, Some(1)).unwrap();
    let season2_episodes = cache.get_episodes("test-profile", 1, Some(2)).unwrap();
    
    assert_eq!(season1_episodes.len(), 2);
    assert_eq!(season2_episodes.len(), 2);
    assert!(season1_episodes.iter().all(|e| e.season_number == 1));
    assert!(season2_episodes.iter().all(|e| e.season_number == 2));
}

#[test]
fn test_delete_series_cascades_to_seasons_and_episodes() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let details = create_sample_series_details(1, "Test Series");
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    // Verify data exists
    assert_eq!(cache.get_series("test-profile", None).unwrap().len(), 1);
    assert_eq!(cache.get_seasons("test-profile", 1).unwrap().len(), 2);
    assert_eq!(cache.get_episodes("test-profile", 1, None).unwrap().len(), 4);
    
    // Delete series
    cache.delete_series("test-profile", Some(vec![1])).unwrap();
    
    // Verify all related data was deleted
    assert_eq!(cache.get_series("test-profile", None).unwrap().len(), 0);
    assert_eq!(cache.get_seasons("test-profile", 1).unwrap().len(), 0);
    assert_eq!(cache.get_episodes("test-profile", 1, None).unwrap().len(), 0);
}

#[test]
fn test_save_series_details_upsert() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Save initial details
    let details1 = create_sample_series_details(1, "Original Series");
    cache.save_series_details("test-profile", 1, details1).unwrap();
    
    // Update with new details (more episodes)
    let mut details2 = create_sample_series_details(1, "Updated Series");
    details2.episodes.push(create_sample_episode("1_2_3", 2, "3"));
    cache.save_series_details("test-profile", 1, details2).unwrap();
    
    // Verify updated data
    let retrieved = cache.get_series_details("test-profile", 1).unwrap();
    assert_eq!(retrieved.series.name, "Updated Series");
    assert_eq!(retrieved.episodes.len(), 5); // 4 original + 1 new
}

// ==================== Series Performance Tests ====================

#[test]
fn test_get_series_performance() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert a large number of series
    let mut series = Vec::new();
    for i in 1..=1000 {
        let s = create_sample_series(i, &format!("Series {}", i), Some("1"));
        series.push(s);
    }
    
    cache.save_series("test-profile", series).unwrap();
    
    // Measure query performance
    let start = std::time::Instant::now();
    let results = cache.get_series("test-profile", None).unwrap();
    let duration = start.elapsed();
    
    println!("Get series took {:?} for {} results", duration, results.len());
    
    // Target: < 100ms for query
    assert!(duration.as_millis() < 100, "Query took too long: {:?}", duration);
    assert_eq!(results.len(), 1000);
}

#[test]
fn test_get_series_with_filters_performance() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert series with various attributes
    let mut series = Vec::new();
    for i in 1..=1000 {
        let mut s = create_sample_series(i, &format!("Series {}", i), Some("drama"));
        s.genre = Some("Drama".to_string());
        s.year = Some(format!("{}", 2020 + (i % 5)));
        s.rating_5based = Some(2.5 + (i % 3) as f64);
        series.push(s);
    }
    
    cache.save_series("test-profile", series).unwrap();
    
    // Measure filter performance
    let filter = SeriesFilter {
        category_id: Some("drama".to_string()),
        genre: Some("Drama".to_string()),
        min_rating: Some(3.5),
        ..Default::default()
    };
    
    let start = std::time::Instant::now();
    let results = cache.get_series("test-profile", Some(filter)).unwrap();
    let duration = start.elapsed();
    
    println!("Filtered query took {:?} for {} results", duration, results.len());
    
    // Target: < 100ms for filtered query
    assert!(duration.as_millis() < 100, "Filtered query took too long: {:?}", duration);
    assert!(results.len() > 0);
}

#[test]
fn test_get_series_details_performance() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Create a series with many seasons and episodes
    let mut details = create_sample_series_details(1, "Big Series");
    
    // Add more seasons and episodes
    for season_num in 3..=10 {
        details.seasons.push(create_sample_season(season_num));
        for ep_num in 1..=20 {
            details.episodes.push(create_sample_episode(
                &format!("1_{}_{}", season_num, ep_num),
                season_num,
                &ep_num.to_string(),
            ));
        }
    }
    
    cache.save_series_details("test-profile", 1, details).unwrap();
    
    // Measure retrieval performance
    let start = std::time::Instant::now();
    let retrieved = cache.get_series_details("test-profile", 1).unwrap();
    let duration = start.elapsed();
    
    println!(
        "Get series details took {:?} for {} seasons and {} episodes",
        duration,
        retrieved.seasons.len(),
        retrieved.episodes.len()
    );
    
    // Target: < 100ms for detailed query
    assert!(duration.as_millis() < 100, "Detailed query took too long: {:?}", duration);
    assert_eq!(retrieved.seasons.len(), 10);
    assert_eq!(retrieved.episodes.len(), 164); // 4 from first 2 seasons + 160 from 8 more seasons
}

#[test]
fn test_batch_save_series_performance() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Create a large batch of series
    let mut series = Vec::new();
    for i in 1..=1000 {
        series.push(create_sample_series(i, &format!("Series {}", i), Some("1")));
    }
    
    // Measure batch insert performance
    let start = std::time::Instant::now();
    let saved = cache.save_series("test-profile", series).unwrap();
    let duration = start.elapsed();
    
    println!("Batch save of {} series took {:?}", saved, duration);
    
    // Target: < 1 second for 1000 series
    assert!(duration.as_secs() < 1, "Batch save took too long: {:?}", duration);
    assert_eq!(saved, 1000);
}

// ==================== Data Integrity Tests ====================

#[test]
fn test_series_profile_isolation() {
    let db = create_test_db();
    insert_test_profile(&db, "profile1");
    insert_test_profile(&db, "profile2");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("profile1").unwrap();
    cache.initialize_profile("profile2").unwrap();
    
    // Save series for profile1
    let series1 = vec![create_sample_series(1, "Profile 1 Series", Some("1"))];
    cache.save_series("profile1", series1).unwrap();
    
    // Save series for profile2
    let series2 = vec![create_sample_series(2, "Profile 2 Series", Some("1"))];
    cache.save_series("profile2", series2).unwrap();
    
    // Verify isolation
    let profile1_series = cache.get_series("profile1", None).unwrap();
    let profile2_series = cache.get_series("profile2", None).unwrap();
    
    assert_eq!(profile1_series.len(), 1);
    assert_eq!(profile2_series.len(), 1);
    assert_eq!(profile1_series[0].name, "Profile 1 Series");
    assert_eq!(profile2_series[0].name, "Profile 2 Series");
}

#[test]
fn test_series_invalid_profile_id() {
    let db = create_test_db();
    let cache = ContentCache::new(db).unwrap();
    
    let series = vec![create_sample_series(1, "Test Series", Some("1"))];
    let result = cache.save_series("", series);
    
    assert!(result.is_err());
}

#[test]
fn test_series_invalid_series_id() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let mut series = create_sample_series(1, "Test Series", Some("1"));
    series.series_id = -1; // Invalid ID
    
    let result = cache.save_series("test-profile", vec![series]);
    assert!(result.is_err());
}
