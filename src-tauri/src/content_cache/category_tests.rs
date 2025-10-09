// Unit tests for category operations

#[cfg(test)]
mod tests {
    use crate::content_cache::*;
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

    fn setup_test_cache() -> (ContentCache, Arc<Mutex<Connection>>) {
        let db = create_test_db();
        let cache = ContentCache::new(Arc::clone(&db)).expect("Failed to create ContentCache");
        (cache, db)
    }

    fn create_test_categories() -> Vec<XtreamCategory> {
        vec![
            XtreamCategory {
                category_id: "1".to_string(),
                category_name: "Action".to_string(),
                parent_id: None,
            },
            XtreamCategory {
                category_id: "2".to_string(),
                category_name: "Comedy".to_string(),
                parent_id: None,
            },
            XtreamCategory {
                category_id: "3".to_string(),
                category_name: "Drama".to_string(),
                parent_id: None,
            },
            XtreamCategory {
                category_id: "4".to_string(),
                category_name: "Action Thriller".to_string(),
                parent_id: Some(1),
            },
        ]
    }

    #[test]
    fn test_save_categories_channels() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        let saved = cache
            .save_categories(profile_id, ContentType::Channels, categories.clone())
            .unwrap();

        assert_eq!(saved, 4, "Should save all 4 categories");
    }

    #[test]
    fn test_save_categories_movies() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        let saved = cache
            .save_categories(profile_id, ContentType::Movies, categories.clone())
            .unwrap();

        assert_eq!(saved, 4, "Should save all 4 categories");
    }

    #[test]
    fn test_save_categories_series() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        let saved = cache
            .save_categories(profile_id, ContentType::Series, categories.clone())
            .unwrap();

        assert_eq!(saved, 4, "Should save all 4 categories");
    }

    #[test]
    fn test_save_categories_empty() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories: Vec<XtreamCategory> = vec![];
        let saved = cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        assert_eq!(saved, 0, "Should return 0 for empty list");
    }

    #[test]
    fn test_save_categories_upsert() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        // Save initial categories
        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        // Update one category
        let updated = vec![XtreamCategory {
            category_id: "1".to_string(),
            category_name: "Action Movies".to_string(), // Changed name
            parent_id: None,
        }];

        let saved = cache
            .save_categories(profile_id, ContentType::Channels, updated)
            .unwrap();

        assert_eq!(saved, 1, "Should update 1 category");

        // Verify the update
        let retrieved = cache
            .get_categories(profile_id, ContentType::Channels, None)
            .unwrap();

        let action_cat = retrieved
            .iter()
            .find(|c| c.category_id == "1")
            .expect("Should find category 1");

        assert_eq!(
            action_cat.category_name, "Action Movies",
            "Category name should be updated"
        );
    }

    #[test]
    fn test_get_categories_all() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        let retrieved = cache
            .get_categories(profile_id, ContentType::Channels, None)
            .unwrap();

        assert_eq!(retrieved.len(), 4, "Should retrieve all 4 categories");
    }

    #[test]
    fn test_get_categories_by_parent() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        // Get root categories (parent_id = None or 0)
        let filter = CategoryFilter {
            parent_id: Some(1),
            name_contains: None,
        };

        let retrieved = cache
            .get_categories(profile_id, ContentType::Channels, Some(filter))
            .unwrap();

        assert_eq!(
            retrieved.len(),
            1,
            "Should retrieve 1 category with parent_id = 1"
        );
        assert_eq!(retrieved[0].category_name, "Action Thriller");
    }

    #[test]
    fn test_get_categories_by_name() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        let filter = CategoryFilter {
            parent_id: None,
            name_contains: Some("Action".to_string()),
        };

        let retrieved = cache
            .get_categories(profile_id, ContentType::Channels, Some(filter))
            .unwrap();

        assert_eq!(
            retrieved.len(),
            2,
            "Should retrieve 2 categories containing 'Action'"
        );
    }

    #[test]
    fn test_get_categories_sorted() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        let retrieved = cache
            .get_categories(profile_id, ContentType::Channels, None)
            .unwrap();

        // Should be sorted alphabetically (case-insensitive)
        assert_eq!(retrieved[0].category_name, "Action");
        assert_eq!(retrieved[1].category_name, "Action Thriller");
        assert_eq!(retrieved[2].category_name, "Comedy");
        assert_eq!(retrieved[3].category_name, "Drama");
    }

    #[test]
    fn test_get_categories_with_counts() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        // Save categories
        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        // Save some channels in category 1
        let channels = vec![
            XtreamChannel {
                stream_id: 1,
                num: Some(1),
                name: "Channel 1".to_string(),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 2,
                num: Some(2),
                name: "Channel 2".to_string(),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 3,
                num: Some(3),
                name: "Channel 3".to_string(),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: Some("2".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
        ];

        cache.save_channels(profile_id, channels).unwrap();

        // Get categories with counts
        let categories_with_counts = cache
            .get_categories_with_counts(profile_id, ContentType::Channels, None)
            .unwrap();

        assert_eq!(
            categories_with_counts.len(),
            4,
            "Should retrieve all 4 categories"
        );

        // Find category 1 and check count
        let cat1 = categories_with_counts
            .iter()
            .find(|c| c.category_id == "1")
            .expect("Should find category 1");
        assert_eq!(cat1.item_count, 2, "Category 1 should have 2 channels");

        // Find category 2 and check count
        let cat2 = categories_with_counts
            .iter()
            .find(|c| c.category_id == "2")
            .expect("Should find category 2");
        assert_eq!(cat2.item_count, 1, "Category 2 should have 1 channel");

        // Find category 3 and check count
        let cat3 = categories_with_counts
            .iter()
            .find(|c| c.category_id == "3")
            .expect("Should find category 3");
        assert_eq!(cat3.item_count, 0, "Category 3 should have 0 channels");
    }

    #[test]
    fn test_get_categories_with_counts_movies() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        // Save categories
        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Movies, categories)
            .unwrap();

        // Save some movies in category 1
        let movies = vec![
            XtreamMovie {
                stream_id: 1,
                num: Some(1),
                name: "Movie 1".to_string(),
                title: Some("Movie 1".to_string()),
                year: Some("2023".to_string()),
                stream_type: Some("movie".to_string()),
                stream_icon: None,
                rating: Some(8.5),
                rating_5based: Some(4.25),
                genre: Some("Action".to_string()),
                added: None,
                episode_run_time: Some(120),
                category_id: Some("1".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: None,
                cast: None,
                director: None,
                plot: None,
                youtube_trailer: None,
            },
            XtreamMovie {
                stream_id: 2,
                num: Some(2),
                name: "Movie 2".to_string(),
                title: Some("Movie 2".to_string()),
                year: Some("2023".to_string()),
                stream_type: Some("movie".to_string()),
                stream_icon: None,
                rating: Some(7.5),
                rating_5based: Some(3.75),
                genre: Some("Action".to_string()),
                added: None,
                episode_run_time: Some(110),
                category_id: Some("1".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: None,
                cast: None,
                director: None,
                plot: None,
                youtube_trailer: None,
            },
        ];

        cache.save_movies(profile_id, movies).unwrap();

        // Get categories with counts
        let categories_with_counts = cache
            .get_categories_with_counts(profile_id, ContentType::Movies, None)
            .unwrap();

        // Find category 1 and check count
        let cat1 = categories_with_counts
            .iter()
            .find(|c| c.category_id == "1")
            .expect("Should find category 1");
        assert_eq!(cat1.item_count, 2, "Category 1 should have 2 movies");
    }

    #[test]
    fn test_delete_categories_specific() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        // Delete specific categories
        let deleted = cache
            .delete_categories(
                profile_id,
                ContentType::Channels,
                Some(vec!["1".to_string(), "2".to_string()]),
            )
            .unwrap();

        assert_eq!(deleted, 2, "Should delete 2 categories");

        // Verify remaining categories
        let remaining = cache
            .get_categories(profile_id, ContentType::Channels, None)
            .unwrap();

        assert_eq!(remaining.len(), 2, "Should have 2 categories remaining");
        assert!(remaining.iter().all(|c| c.category_id != "1" && c.category_id != "2"));
    }

    #[test]
    fn test_delete_categories_all() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        // Delete all categories
        let deleted = cache
            .delete_categories(profile_id, ContentType::Channels, None)
            .unwrap();

        assert_eq!(deleted, 4, "Should delete all 4 categories");

        // Verify no categories remain
        let remaining = cache
            .get_categories(profile_id, ContentType::Channels, None)
            .unwrap();

        assert_eq!(remaining.len(), 0, "Should have no categories remaining");
    }

    #[test]
    fn test_delete_categories_empty_list() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        // Delete with empty list
        let deleted = cache
            .delete_categories(profile_id, ContentType::Channels, Some(vec![]))
            .unwrap();

        assert_eq!(deleted, 0, "Should delete 0 categories");

        // Verify all categories remain
        let remaining = cache
            .get_categories(profile_id, ContentType::Channels, None)
            .unwrap();

        assert_eq!(remaining.len(), 4, "Should have all 4 categories remaining");
    }

    #[test]
    fn test_count_categories() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        let count = cache
            .count_categories(profile_id, ContentType::Channels, None)
            .unwrap();

        assert_eq!(count, 4, "Should count 4 categories");
    }

    #[test]
    fn test_count_categories_with_filter() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();
        cache
            .save_categories(profile_id, ContentType::Channels, categories)
            .unwrap();

        let filter = CategoryFilter {
            parent_id: None,
            name_contains: Some("Action".to_string()),
        };

        let count = cache
            .count_categories(profile_id, ContentType::Channels, Some(filter))
            .unwrap();

        assert_eq!(count, 2, "Should count 2 categories containing 'Action'");
    }

    #[test]
    fn test_categories_profile_isolation() {
        let (cache, db) = setup_test_cache();
        let profile1 = "profile1";
        let profile2 = "profile2";

        insert_test_profile(&db, profile1);
        insert_test_profile(&db, profile2);
        cache.initialize_profile(profile1).unwrap();
        cache.initialize_profile(profile2).unwrap();

        let categories = create_test_categories();

        // Save categories for profile1
        cache
            .save_categories(profile1, ContentType::Channels, categories.clone())
            .unwrap();

        // Save different categories for profile2
        let profile2_categories = vec![XtreamCategory {
            category_id: "100".to_string(),
            category_name: "Profile 2 Category".to_string(),
            parent_id: None,
        }];

        cache
            .save_categories(profile2, ContentType::Channels, profile2_categories)
            .unwrap();

        // Verify profile1 has 4 categories
        let profile1_cats = cache
            .get_categories(profile1, ContentType::Channels, None)
            .unwrap();
        assert_eq!(profile1_cats.len(), 4);

        // Verify profile2 has 1 category
        let profile2_cats = cache
            .get_categories(profile2, ContentType::Channels, None)
            .unwrap();
        assert_eq!(profile2_cats.len(), 1);
        assert_eq!(profile2_cats[0].category_id, "100");
    }

    #[test]
    fn test_categories_content_type_isolation() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        let categories = create_test_categories();

        // Save to all three content types
        cache
            .save_categories(profile_id, ContentType::Channels, categories.clone())
            .unwrap();
        cache
            .save_categories(profile_id, ContentType::Movies, categories.clone())
            .unwrap();
        cache
            .save_categories(profile_id, ContentType::Series, categories.clone())
            .unwrap();

        // Verify each content type has its own categories
        let channel_cats = cache
            .get_categories(profile_id, ContentType::Channels, None)
            .unwrap();
        let movie_cats = cache
            .get_categories(profile_id, ContentType::Movies, None)
            .unwrap();
        let series_cats = cache
            .get_categories(profile_id, ContentType::Series, None)
            .unwrap();

        assert_eq!(channel_cats.len(), 4);
        assert_eq!(movie_cats.len(), 4);
        assert_eq!(series_cats.len(), 4);

        // Delete channels categories
        cache
            .delete_categories(profile_id, ContentType::Channels, None)
            .unwrap();

        // Verify only channel categories are deleted
        let channel_cats = cache
            .get_categories(profile_id, ContentType::Channels, None)
            .unwrap();
        let movie_cats = cache
            .get_categories(profile_id, ContentType::Movies, None)
            .unwrap();
        let series_cats = cache
            .get_categories(profile_id, ContentType::Series, None)
            .unwrap();

        assert_eq!(channel_cats.len(), 0);
        assert_eq!(movie_cats.len(), 4);
        assert_eq!(series_cats.len(), 4);
    }

    #[test]
    fn test_save_categories_validation() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        // Try to save category with empty category_id
        let invalid_categories = vec![XtreamCategory {
            category_id: "".to_string(),
            category_name: "Invalid".to_string(),
            parent_id: None,
        }];

        let result = cache.save_categories(profile_id, ContentType::Channels, invalid_categories);

        assert!(result.is_err(), "Should fail with empty category_id");
    }

    #[test]
    fn test_delete_categories_validation() {
        let (cache, db) = setup_test_cache();
        let profile_id = "test_profile";
        insert_test_profile(&db, profile_id);
        cache.initialize_profile(profile_id).unwrap();

        // Try to delete with empty category_id
        let result = cache.delete_categories(
            profile_id,
            ContentType::Channels,
            Some(vec!["".to_string()]),
        );

        assert!(result.is_err(), "Should fail with empty category_id");
    }
}

