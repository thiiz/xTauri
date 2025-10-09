#[cfg(test)]
mod tests {
    use crate::xtream::history::{add_to_history, get_history, clear_history};
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
            "CREATE TABLE xtream_history (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content_id TEXT NOT NULL,
                content_data BLOB NOT NULL,
                watched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
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
    fn test_add_to_history() {
        let (db, _temp_dir) = setup_test_db();
        
        let content_data = serde_json::json!({
            "id": "1001",
            "name": "Test Channel"
        });
        
        let result = add_to_history(
            &db,
            "test-profile",
            "channel",
            "1001",
            &content_data
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_duplicate_to_history() {
        let (db, _temp_dir) = setup_test_db();
        
        let content_data = serde_json::json!({
            "id": "1001",
            "name": "Test Channel"
        });
        
        // Add first time
        add_to_history(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        // Add again - should create a new entry
        add_to_history(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        let history = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_get_history() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add multiple history entries
        for i in 1..=5 {
            let content_data = serde_json::json!({
                "id": format!("100{}", i),
                "name": format!("Channel {}", i)
            });
            add_to_history(&db, "test-profile", "channel", &format!("100{}", i), &content_data).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        let history = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(history.len(), 5);
    }

    #[test]
    fn test_get_history_with_limit() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add multiple history entries
        for i in 1..=10 {
            let content_data = serde_json::json!({
                "id": format!("100{}", i),
                "name": format!("Channel {}", i)
            });
            add_to_history(&db, "test-profile", "channel", &format!("100{}", i), &content_data).unwrap();
        }
        
        let history = get_history(&db, "test-profile", None, Some(5)).unwrap();
        assert_eq!(history.len(), 5);
    }

    #[test]
    fn test_get_history_by_type() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add different types of history
        let channel_data = serde_json::json!({"id": "1001", "name": "Channel 1"});
        let movie_data = serde_json::json!({"id": "2001", "name": "Movie 1"});
        let series_data = serde_json::json!({"id": "3001", "name": "Series 1"});
        
        add_to_history(&db, "test-profile", "channel", "1001", &channel_data).unwrap();
        add_to_history(&db, "test-profile", "movie", "2001", &movie_data).unwrap();
        add_to_history(&db, "test-profile", "series", "3001", &series_data).unwrap();
        
        // Get only channels
        let channels = get_history(&db, "test-profile", Some("channel"), None).unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].content_type, "channel");
        
        // Get only movies
        let movies = get_history(&db, "test-profile", Some("movie"), None).unwrap();
        assert_eq!(movies.len(), 1);
        assert_eq!(movies[0].content_type, "movie");
        
        // Get all
        let all = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_get_history_empty() {
        let (db, _temp_dir) = setup_test_db();
        
        let history = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_clear_history() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add history entries
        for i in 1..=3 {
            let content_data = serde_json::json!({
                "id": format!("100{}", i),
                "name": format!("Channel {}", i)
            });
            add_to_history(&db, "test-profile", "channel", &format!("100{}", i), &content_data).unwrap();
        }
        
        // Verify history exists
        let history = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(history.len(), 3);
        
        // Clear history
        clear_history(&db, "test-profile").unwrap();
        
        // Verify history is empty
        let history = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_history_profile_isolation() {
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
        
        // Add history to profile 1
        add_to_history(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        // Profile 1 should have history
        let history1 = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(history1.len(), 1);
        
        // Profile 2 should not have history
        let history2 = get_history(&db, "profile2", None, None).unwrap();
        assert_eq!(history2.len(), 0);
    }

    #[test]
    fn test_history_cascade_delete() {
        let (db, _temp_dir) = setup_test_db();
        
        let content_data = serde_json::json!({
            "id": "1001",
            "name": "Test Channel"
        });
        
        // Add history
        add_to_history(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        // Verify it exists
        let history = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(history.len(), 1);
        
        // Delete the profile
        {
            let conn = db.lock().unwrap();
            conn.execute("DELETE FROM xtream_profiles WHERE id = 'test-profile'", []).unwrap();
        }
        
        // History should be gone (cascade delete)
        let history = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_history_ordering() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add history entries with delays to ensure different timestamps
        for i in 1..=3 {
            let content_data = serde_json::json!({
                "id": format!("100{}", i),
                "name": format!("Channel {}", i)
            });
            add_to_history(&db, "test-profile", "channel", &format!("100{}", i), &content_data).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        let history = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(history.len(), 3);
        
        // Should be ordered by watched_at (most recent first)
        assert_eq!(history[0].content_id, "1003");
        assert_eq!(history[1].content_id, "1002");
        assert_eq!(history[2].content_id, "1001");
    }

    #[test]
    fn test_add_history_with_complex_data() {
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
        
        add_to_history(&db, "test-profile", "channel", "1001", &content_data).unwrap();
        
        let history = get_history(&db, "test-profile", Some("channel"), None).unwrap();
        assert_eq!(history.len(), 1);
        
        let retrieved_data: serde_json::Value = serde_json::from_str(&history[0].content_data).unwrap();
        assert_eq!(retrieved_data["name"], "Test Channel");
        assert_eq!(retrieved_data["metadata"]["quality"], "HD");
    }

    #[test]
    fn test_history_limit_boundary() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add 10 history entries
        for i in 1..=10 {
            let content_data = serde_json::json!({
                "id": format!("100{}", i),
                "name": format!("Channel {}", i)
            });
            add_to_history(&db, "test-profile", "channel", &format!("100{}", i), &content_data).unwrap();
        }
        
        // Test limit of 0 (should return empty)
        let history = get_history(&db, "test-profile", None, Some(0)).unwrap();
        assert_eq!(history.len(), 0);
        
        // Test limit of 1
        let history = get_history(&db, "test-profile", None, Some(1)).unwrap();
        assert_eq!(history.len(), 1);
        
        // Test limit greater than available
        let history = get_history(&db, "test-profile", None, Some(20)).unwrap();
        assert_eq!(history.len(), 10);
    }

    #[test]
    fn test_clear_history_by_type() {
        let (db, _temp_dir) = setup_test_db();
        
        // Add different types of history
        let channel_data = serde_json::json!({"id": "1001", "name": "Channel 1"});
        let movie_data = serde_json::json!({"id": "2001", "name": "Movie 1"});
        
        add_to_history(&db, "test-profile", "channel", "1001", &channel_data).unwrap();
        add_to_history(&db, "test-profile", "movie", "2001", &movie_data).unwrap();
        
        // Clear all history
        clear_history(&db, "test-profile").unwrap();
        
        // Verify all history is cleared
        let history = get_history(&db, "test-profile", None, None).unwrap();
        assert_eq!(history.len(), 0);
    }
}
