#[cfg(test)]
mod tests {
    use crate::xtream::favorites::{add_favorite, remove_favorite, get_favorites, is_favorite};
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Mutex<Connection>>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        
        // Create tables
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
            "CREATE TABLE xtream_favorites (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content_id TEXT NOT NULL,
                content_data BLOB NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
                UNIQUE(profile_id, content_type, content_id)
            )",
            [],
        ).unwrap();
        
        // Insert test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        ).unwrap();
        
        (Arc::new(Mutex::new(conn)), temp_dir)
    }

    #[test]
    fn test_add_favorite() {
        let (db, _temp_dir) = setup_test_db();
        
        let content_data = serde_json::json!({
            "id": "1001",
            "name": "Test Channel"
        });
        
        let result = add_favorite(
            &db,
            "test-profile",
            "channel",
            "1001",
            &content_data
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_duplicate_favorite() {
        let (db, _temp_dir) = setup_test_db();
        
        let content_data = serde_json::json!({
            "id": "1001",
            "name": "Test Channel"
        });
        
        // Add first time
        add_favorite(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        // Try to add again - should succeed (replace)
        let result = add_favorite(&db, "test-profile", "channel", "1001", &content_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_favorite() {
        let (db, _temp_dir) = setup_test_db();
        
        let content_data = serde_json::json!({
            "id": "1001",
            "name": "Test Channel"
        });
        
        // Add favorite
        add_favorite(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        // Remove it
        let result = remove_favorite(&db, "test-profile", "channel", "1001");
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_nonexistent_favorite() {
        let (db, _temp_dir) = setup_test_db();
        
        // Try to remove a favorite that doesn't exist
        let result = remove_favorite(&db, "test-profile", "channel", "9999");
        assert!(result.is_ok()); // Should not error
    }

    #[test]
    fn test_get_favorites() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add multiple favorites
        for i in 1..=3 {
            let content_data = serde_json::json!({
                "id": format!("100{}", i),
                "name": format!("Channel {}", i)
            });
            add_favorite(&db, "test-profile", "channel", &format!("100{}", i), &content_data).unwrap();
        }
        
        let favorites = get_favorites(&db, "test-profile", Some("channel")).unwrap();
        assert_eq!(favorites.len(), 3);
    }

    #[test]
    fn test_get_favorites_by_type() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add different types of favorites
        let channel_data = serde_json::json!({"id": "1001", "name": "Channel 1"});
        let movie_data = serde_json::json!({"id": "2001", "name": "Movie 1"});
        let series_data = serde_json::json!({"id": "3001", "name": "Series 1"});
        
        add_favorite(&db, "test-profile", "channel", "1001", &channel_data).unwrap();
        add_favorite(&db, "test-profile", "movie", "2001", &movie_data).unwrap();
        add_favorite(&db, "test-profile", "series", "3001", &series_data).unwrap();
        
        // Get only channels
        let channels = get_favorites(&db, "test-profile", Some("channel")).unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].content_type, "channel");
        
        // Get only movies
        let movies = get_favorites(&db, "test-profile", Some("movie")).unwrap();
        assert_eq!(movies.len(), 1);
        assert_eq!(movies[0].content_type, "movie");
        
        // Get all
        let all = get_favorites(&db, "test-profile", None).unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_get_favorites_empty() {
        let (db, _temp_dir) = setup_test_db();
        
        let favorites = get_favorites(&db, "test-profile", None).unwrap();
        assert_eq!(favorites.len(), 0);
    }

    #[test]
    fn test_is_favorite() {
        let (db, _temp_dir) = setup_test_db();
        
        let content_data = serde_json::json!({
            "id": "1001",
            "name": "Test Channel"
        });
        
        // Initially not a favorite
        assert!(!is_favorite(&db, "test-profile", "channel", "1001").unwrap());
        
        // Add as favorite
        add_favorite(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        // Now it should be a favorite
        assert!(is_favorite(&db, "test-profile", "channel", "1001").unwrap());
    }

    #[test]
    fn test_favorites_profile_isolation() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add second profile
        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
                 VALUES ('profile2', 'Test2', 'http://test2.com', 'user2', X'00')",
                [],
            ).unwrap();
        }
        
        let content_data = serde_json::json!({
            "id": "1001",
            "name": "Test Channel"
        });
        
        // Add favorite to profile 1
        add_favorite(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        // Profile 1 should have the favorite
        assert!(is_favorite(&db, "test-profile", "channel", "1001").unwrap());
        
        // Profile 2 should not have the favorite
        assert!(!is_favorite(&db, "profile2", "channel", "1001").unwrap());
    }

    #[test]
    fn test_favorites_cascade_delete() {
        let (db, _temp_dir) = setup_test_db();
        
        let content_data = serde_json::json!({
            "id": "1001",
            "name": "Test Channel"
        });
        
        // Add favorite
        add_favorite(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        // Verify it exists
        assert!(is_favorite(&db, "test-profile", "channel", "1001").unwrap());
        
        // Delete the profile
        {
            let conn = db.lock().unwrap();
            conn.execute("DELETE FROM xtream_profiles WHERE id = 'test-profile'", []).unwrap();
        }
        
        // Favorite should be gone (cascade delete)
        let favorites = get_favorites(&db, "test-profile", None).unwrap();
        assert_eq!(favorites.len(), 0);
    }

    #[test]
    fn test_add_favorite_with_complex_data() {
        let (db, _temp_dir) = setup_test_db();
        
        let content_data = serde_json::json!({
            "id": "1001",
            "name": "Test Channel",
            "icon": "http://example.com/icon.png",
            "epg_channel_id": "test.channel",
            "metadata": {
                "quality": "HD",
                "language": "en"
            }
        });
        
        add_favorite(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        let favorites = get_favorites(&db, "test-profile", Some("channel")).unwrap();
        assert_eq!(favorites.len(), 1);
        
        let retrieved_data: serde_json::Value = serde_json::from_str(&favorites[0].content_data).unwrap();
        assert_eq!(retrieved_data["name"], "Test Channel");
        assert_eq!(retrieved_data["metadata"]["quality"], "HD");
    }

    #[test]
    fn test_favorites_ordering() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add favorites with delays to ensure different timestamps
        for i in 1..=3 {
            let content_data = serde_json::json!({
                "id": format!("100{}", i),
                "name": format!("Channel {}", i)
            });
            add_favorite(&db, "test-profile", "channel", &format!("100{}", i), &content_data).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        let favorites = get_favorites(&db, "test-profile", Some("channel")).unwrap();
        assert_eq!(favorites.len(), 3);
        
        // Should be ordered by created_at (most recent first)
        assert_eq!(favorites[0].content_id, "1003");
        assert_eq!(favorites[1].content_id, "1002");
        assert_eq!(favorites[2].content_id, "1001");
    }
}
