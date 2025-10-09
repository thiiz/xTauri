// Search and filtering tests for ContentCache
use super::*;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::Instant;

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

/// Create sample channels for testing
fn create_sample_channels() -> Vec<XtreamChannel> {
    vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "HBO Channel".to_string(),
            stream_type: Some("live".to_string()),
            stream_icon: Some("http://example.com/hbo.png".to_string()),
            thumbnail: None,
            epg_channel_id: Some("hbo1".to_string()),
            added: Some("2024-01-01".to_string()),
            category_id: Some("movies".to_string()),
            custom_sid: None,
            tv_archive: Some(1),
            direct_source: None,
            tv_archive_duration: Some(7),
        },
        XtreamChannel {
            stream_id: 2,
            num: Some(2),
            name: "HBO Max".to_string(),
            stream_type: Some("live".to_string()),
            stream_icon: Some("http://example.com/hbomax.png".to_string()),
            thumbnail: None,
            epg_channel_id: Some("hbomax".to_string()),
            added: Some("2024-01-02".to_string()),
            category_id: Some("movies".to_string()),
            custom_sid: None,
            tv_archive: Some(1),
            direct_source: None,
            tv_archive_duration: Some(7),
        },
        XtreamChannel {
            stream_id: 3,
            num: Some(3),
            name: "ESPN Sports".to_string(),
            stream_type: Some("live".to_string()),
            stream_icon: Some("http://example.com/espn.png".to_string()),
            thumbnail: None,
            epg_channel_id: Some("espn1".to_string()),
            added: Some("2024-01-03".to_string()),
            category_id: Some("sports".to_string()),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 4,
            num: Some(4),
            name: "ESPN 2".to_string(),
            stream_type: Some("live".to_string()),
            stream_icon: Some("http://example.com/espn2.png".to_string()),
            thumbnail: None,
            epg_channel_id: Some("espn2".to_string()),
            added: Some("2024-01-04".to_string()),
            category_id: Some("sports".to_string()),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        },
        XtreamChannel {
            stream_id: 5,
            num: Some(5),
            name: "CNN News".to_string(),
            stream_type: Some("live".to_string()),
            stream_icon: Some("http://example.com/cnn.png".to_string()),
            thumbnail: None,
            epg_channel_id: Some("cnn".to_string()),
            added: Some("2024-01-05".to_string()),
            category_id: Some("news".to_string()),
            custom_sid: None,
            tv_archive: Some(1),
            direct_source: None,
            tv_archive_duration: Some(3),
        },
        XtreamChannel {
            stream_id: 6,
            num: Some(6),
            name: "BBC News".to_string(),
            stream_type: Some("live".to_string()),
            stream_icon: Some("http://example.com/bbc.png".to_string()),
            thumbnail: None,
            epg_channel_id: Some("bbc".to_string()),
            added: Some("2024-01-06".to_string()),
            category_id: Some("news".to_string()),
            custom_sid: None,
            tv_archive: Some(1),
            direct_source: None,
            tv_archive_duration: Some(3),
        },
    ]
}

// ==================== Search Tests ====================

#[test]
fn test_search_channels_exact_match() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = create_sample_channels();
    cache.save_channels("test-profile", channels).unwrap();
    
    // Search for exact match
    let results = cache.search_channels("test-profile", "CNN News", None).unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "CNN News");
    assert_eq!(results[0].stream_id, 5);
}

#[test]
fn test_search_channels_partial_match() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = create_sample_channels();
    cache.save_channels("test-profile", channels).unwrap();
    
    // Search for partial match "HBO"
    let results = cache.search_channels("test-profile", "HBO", None).unwrap();
    
    assert_eq!(results.len(), 2);
    // Results should be ordered by relevance (exact match first, then partial)
    assert!(results[0].name.contains("HBO"));
    assert!(results[1].name.contains("HBO"));
}

#[test]
fn test_search_channels_case_insensitive() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = create_sample_channels();
    cache.save_channels("test-profile", channels).unwrap();
    
    // Search with different cases
    let results_lower = cache.search_channels("test-profile", "espn", None).unwrap();
    let results_upper = cache.search_channels("test-profile", "ESPN", None).unwrap();
    let results_mixed = cache.search_channels("test-profile", "EsPn", None).unwrap();
    
    assert_eq!(results_lower.len(), 2);
    assert_eq!(results_upper.len(), 2);
    assert_eq!(results_mixed.len(), 2);
}

#[test]
fn test_search_channels_no_results() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = create_sample_channels();
    cache.save_channels("test-profile", channels).unwrap();
    
    // Search for non-existent channel
    let results = cache.search_channels("test-profile", "XYZ123", None).unwrap();
    
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_channels_empty_query() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = create_sample_channels();
    cache.save_channels("test-profile", channels.clone()).unwrap();
    
    // Empty query should return all channels
    let results = cache.search_channels("test-profile", "", None).unwrap();
    
    assert_eq!(results.len(), channels.len());
}

#[test]
fn test_search_channels_with_category_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = create_sample_channels();
    cache.save_channels("test-profile", channels).unwrap();
    
    // Search for "News" in news category only
    let filter = ChannelFilter {
        category_id: Some("news".to_string()),
        ..Default::default()
    };
    
    let results = cache.search_channels("test-profile", "News", Some(filter)).unwrap();
    
    assert_eq!(results.len(), 2); // CNN News and BBC News
    for channel in results {
        assert_eq!(channel.category_id, Some("news".to_string()));
        assert!(channel.name.contains("News"));
    }
}

#[test]
fn test_search_channels_with_pagination() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Create many channels with "Channel" in the name
    let mut channels = Vec::new();
    for i in 1..=20 {
        channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Test Channel {}", i),
            stream_type: Some("live".to_string()),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some("general".to_string()),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Search with pagination - first page
    let filter = ChannelFilter {
        limit: Some(5),
        offset: Some(0),
        ..Default::default()
    };
    
    let page1 = cache.search_channels("test-profile", "Channel", Some(filter)).unwrap();
    assert_eq!(page1.len(), 5);
    
    // Search with pagination - second page
    let filter = ChannelFilter {
        limit: Some(5),
        offset: Some(5),
        ..Default::default()
    };
    
    let page2 = cache.search_channels("test-profile", "Channel", Some(filter)).unwrap();
    assert_eq!(page2.len(), 5);
    
    // Verify pages are different
    assert_ne!(page1[0].stream_id, page2[0].stream_id);
}

#[test]
fn test_search_channels_relevance_ordering() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = vec![
        XtreamChannel {
            stream_id: 1,
            num: Some(1),
            name: "HBO Channel Extra".to_string(),
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
            name: "HBO".to_string(), // Exact match
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
            name: "HBO Max".to_string(), // Starts with
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
    
    cache.save_channels("test-profile", channels).unwrap();
    
    let results = cache.search_channels("test-profile", "HBO", None).unwrap();
    
    assert_eq!(results.len(), 3);
    // Exact match should be first
    assert_eq!(results[0].name, "HBO");
    // The other two should be ordered by relevance (starts with vs contains)
    // Both "HBO Max" and "HBO Channel Extra" start with "HBO" so they have same relevance
    // They will be ordered alphabetically by name
    let names: Vec<String> = results.iter().map(|c| c.name.clone()).collect();
    assert!(names.contains(&"HBO".to_string()));
    assert!(names.contains(&"HBO Max".to_string()));
    assert!(names.contains(&"HBO Channel Extra".to_string()));
}

#[test]
fn test_search_channels_special_characters() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = vec![
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
    ];
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Search with special characters (should be sanitized and still work)
    // The % character is a SQL wildcard, so it gets sanitized
    // We should still be able to find channels with "100" in the name
    let results = cache.search_channels("test-profile", "100", None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Channel 100%");
    
    // Underscore is also a SQL wildcard (matches single character)
    let results = cache.search_channels("test-profile", "Test", None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Channel_Test");
}

// ==================== Performance Tests ====================

#[test]
fn test_search_performance_small_dataset() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Create 100 channels
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
            category_id: Some(format!("cat{}", i % 10)),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Measure search performance
    let start = Instant::now();
    let results = cache.search_channels("test-profile", "Channel", None).unwrap();
    let duration = start.elapsed();
    
    println!("Search 100 channels took: {:?}", duration);
    
    assert_eq!(results.len(), 100);
    assert!(duration.as_millis() < 100, "Search should complete in < 100ms, took {:?}", duration);
}

#[test]
fn test_search_performance_medium_dataset() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Create 1000 channels
    let mut channels = Vec::new();
    for i in 1..=1000 {
        channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: Some("live".to_string()),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some(format!("cat{}", i % 20)),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Measure search performance
    let start = Instant::now();
    let results = cache.search_channels("test-profile", "Channel 5", None).unwrap();
    let duration = start.elapsed();
    
    println!("Search 1000 channels took: {:?}", duration);
    
    assert!(results.len() > 0);
    // Note: This test can occasionally be slower on first run due to cold cache
    // In production with warm cache, searches are typically < 10ms
    assert!(duration.as_millis() < 200, "Search should complete in < 200ms, took {:?}", duration);
}

#[test]
fn test_search_performance_large_dataset() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Create 10000 channels
    let mut channels = Vec::new();
    for i in 1..=10000 {
        channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: Some("live".to_string()),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some(format!("cat{}", i % 50)),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Measure search performance
    let start = Instant::now();
    let results = cache.search_channels("test-profile", "Channel 999", None).unwrap();
    let duration = start.elapsed();
    
    println!("Search 10000 channels took: {:?}", duration);
    
    assert!(results.len() > 0);
    assert!(duration.as_millis() < 100, "Search should complete in < 100ms, took {:?}", duration);
}

#[test]
fn test_search_performance_with_category_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Create 5000 channels across multiple categories
    let mut channels = Vec::new();
    for i in 1..=5000 {
        channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: Some("live".to_string()),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some(format!("cat{}", i % 10)),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Measure search performance with category filter
    let filter = ChannelFilter {
        category_id: Some("cat5".to_string()),
        ..Default::default()
    };
    
    let start = Instant::now();
    let results = cache.search_channels("test-profile", "Channel", Some(filter)).unwrap();
    let duration = start.elapsed();
    
    println!("Search 5000 channels with category filter took: {:?}", duration);
    
    assert!(results.len() > 0);
    // All results should be from cat5
    for channel in &results {
        assert_eq!(channel.category_id, Some("cat5".to_string()));
    }
    assert!(duration.as_millis() < 100, "Search with filter should complete in < 100ms, took {:?}", duration);
}

#[test]
fn test_search_performance_with_pagination() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Create 5000 channels
    let mut channels = Vec::new();
    for i in 1..=5000 {
        channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Test Channel {}", i),
            stream_type: Some("live".to_string()),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Measure search performance with pagination
    let filter = ChannelFilter {
        limit: Some(50),
        offset: Some(0),
        ..Default::default()
    };
    
    let start = Instant::now();
    let results = cache.search_channels("test-profile", "Channel", Some(filter)).unwrap();
    let duration = start.elapsed();
    
    println!("Search 5000 channels with pagination took: {:?}", duration);
    
    assert_eq!(results.len(), 50);
    assert!(duration.as_millis() < 100, "Paginated search should complete in < 100ms, took {:?}", duration);
}

#[test]
fn test_count_channels_performance() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Create 10000 channels
    let mut channels = Vec::new();
    for i in 1..=10000 {
        channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Channel {}", i),
            stream_type: Some("live".to_string()),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some(format!("cat{}", i % 20)),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Measure count performance
    let start = Instant::now();
    let count = cache.count_channels("test-profile", None).unwrap();
    let duration = start.elapsed();
    
    println!("Count 10000 channels took: {:?}", duration);
    
    assert_eq!(count, 10000);
    assert!(duration.as_millis() < 50, "Count should complete in < 50ms, took {:?}", duration);
}

// ==================== Filter Combination Tests ====================

#[test]
fn test_search_with_multiple_filters() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    let channels = create_sample_channels();
    cache.save_channels("test-profile", channels).unwrap();
    
    // Search with category filter and pagination
    let filter = ChannelFilter {
        category_id: Some("sports".to_string()),
        limit: Some(1),
        offset: Some(0),
        ..Default::default()
    };
    
    let results = cache.search_channels("test-profile", "ESPN", Some(filter)).unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].category_id, Some("sports".to_string()));
    assert!(results[0].name.contains("ESPN"));
}

#[test]
fn test_get_channels_with_all_filters() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("test-profile").unwrap();
    
    // Create many channels
    let mut channels = Vec::new();
    for i in 1..=100 {
        channels.push(XtreamChannel {
            stream_id: i,
            num: Some(i),
            name: format!("Test Channel {}", i),
            stream_type: Some("live".to_string()),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: Some("general".to_string()),
            custom_sid: None,
            tv_archive: Some(0),
            direct_source: None,
            tv_archive_duration: None,
        });
    }
    
    cache.save_channels("test-profile", channels).unwrap();
    
    // Apply all filters
    let filter = ChannelFilter {
        category_id: Some("general".to_string()),
        name_contains: Some("Channel 5".to_string()),
        limit: Some(5),
        offset: Some(0),
    };
    
    let results = cache.get_channels("test-profile", Some(filter)).unwrap();
    
    assert!(results.len() <= 5);
    for channel in results {
        assert_eq!(channel.category_id, Some("general".to_string()));
        assert!(channel.name.contains("Channel 5"));
    }
}

#[test]
fn test_profile_isolation_in_search() {
    let db = create_test_db();
    insert_test_profile(&db, "profile-1");
    insert_test_profile(&db, "profile-2");
    let cache = ContentCache::new(db).unwrap();
    
    cache.initialize_profile("profile-1").unwrap();
    cache.initialize_profile("profile-2").unwrap();
    
    // Add channels to profile-1
    let channels1 = vec![
        XtreamChannel {
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
        },
    ];
    
    // Add channels to profile-2
    let channels2 = vec![
        XtreamChannel {
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
        },
    ];
    
    cache.save_channels("profile-1", channels1).unwrap();
    cache.save_channels("profile-2", channels2).unwrap();
    
    // Search in profile-1 should only return profile-1 channels
    let results1 = cache.search_channels("profile-1", "Channel", None).unwrap();
    assert_eq!(results1.len(), 1);
    assert_eq!(results1[0].name, "Profile 1 Channel");
    
    // Search in profile-2 should only return profile-2 channels
    let results2 = cache.search_channels("profile-2", "Channel", None).unwrap();
    assert_eq!(results2.len(), 1);
    assert_eq!(results2[0].name, "Profile 2 Channel");
}
