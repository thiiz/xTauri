#[cfg(test)]
mod tests {
    use crate::xtream::ContentCache;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use serde::{Serialize, Deserialize};
    use tempfile::TempDir;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: String,
        value: String,
    }

    fn setup_test_cache() -> (ContentCache, TempDir) {
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
            "CREATE TABLE xtream_content_cache (
                cache_key TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                content_type TEXT NOT NULL,
                data BLOB NOT NULL,
                expires_at DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
            )",
            [],
        ).unwrap();
        
        // Insert a test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        ).unwrap();
        
        let db = Arc::new(Mutex::new(conn));
        let cache = ContentCache::new(db, Duration::from_secs(300));
        
        (cache, temp_dir)
    }

    #[test]
    fn test_set_and_get() {
        let (cache, _temp_dir) = setup_test_cache();
        
        let test_data = TestData {
            id: "1".to_string(),
            value: "test value".to_string(),
        };
        
        cache.set("test-key", &test_data, None).unwrap();
        
        let retrieved: Option<TestData> = cache.get("test-key").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), test_data);
    }

    #[test]
    fn test_get_nonexistent_key() {
        let (cache, _temp_dir) = setup_test_cache();
        
        let retrieved: Option<TestData> = cache.get("nonexistent-key").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_set_with_custom_ttl() {
        let (cache, _temp_dir) = setup_test_cache();
        
        let test_data = TestData {
            id: "1".to_string(),
            value: "test value".to_string(),
        };
        
        let custom_ttl = Duration::from_secs(60);
        cache.set("test-key", &test_data, Some(custom_ttl)).unwrap();
        
        let retrieved: Option<TestData> = cache.get("test-key").unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_invalidate_pattern() {
        let (cache, _temp_dir) = setup_test_cache();
        
        let test_data = TestData {
            id: "1".to_string(),
            value: "test value".to_string(),
        };
        
        // Set multiple cache entries
        cache.set("profile1:channels", &test_data, None).unwrap();
        cache.set("profile1:movies", &test_data, None).unwrap();
        cache.set("profile2:channels", &test_data, None).unwrap();
        
        // Invalidate all profile1 entries
        cache.invalidate("profile1:%").unwrap();
        
        // Verify profile1 entries are gone
        let retrieved1: Option<TestData> = cache.get("profile1:channels").unwrap();
        assert!(retrieved1.is_none());
        
        let retrieved2: Option<TestData> = cache.get("profile1:movies").unwrap();
        assert!(retrieved2.is_none());
        
        // Verify profile2 entry still exists
        let retrieved3: Option<TestData> = cache.get("profile2:channels").unwrap();
        assert!(retrieved3.is_some());
    }

    #[test]
    fn test_clear_profile_cache() {
        let (cache, _temp_dir) = setup_test_cache();
        
        let test_data = TestData {
            id: "1".to_string(),
            value: "test value".to_string(),
        };
        
        // Set multiple cache entries for different profiles
        cache.set("profile1:channels", &test_data, None).unwrap();
        cache.set("profile1:movies", &test_data, None).unwrap();
        cache.set("profile2:channels", &test_data, None).unwrap();
        
        // Clear profile1 cache
        cache.clear_profile_cache("profile1").unwrap();
        
        // Verify profile1 entries are gone
        let retrieved1: Option<TestData> = cache.get("profile1:channels").unwrap();
        assert!(retrieved1.is_none());
        
        let retrieved2: Option<TestData> = cache.get("profile1:movies").unwrap();
        assert!(retrieved2.is_none());
        
        // Verify profile2 entry still exists
        let retrieved3: Option<TestData> = cache.get("profile2:channels").unwrap();
        assert!(retrieved3.is_some());
    }

    #[test]
    fn test_overwrite_existing_key() {
        let (cache, _temp_dir) = setup_test_cache();
        
        let test_data1 = TestData {
            id: "1".to_string(),
            value: "first value".to_string(),
        };
        
        let test_data2 = TestData {
            id: "1".to_string(),
            value: "second value".to_string(),
        };
        
        cache.set("test-key", &test_data1, None).unwrap();
        cache.set("test-key", &test_data2, None).unwrap();
        
        let retrieved: Option<TestData> = cache.get("test-key").unwrap();
        assert_eq!(retrieved.unwrap().value, "second value");
    }

    #[test]
    fn test_memory_cache_hit() {
        let (cache, _temp_dir) = setup_test_cache();
        
        let test_data = TestData {
            id: "1".to_string(),
            value: "test value".to_string(),
        };
        
        cache.set("test-key", &test_data, None).unwrap();
        
        // First get should populate memory cache
        let _retrieved1: Option<TestData> = cache.get("test-key").unwrap();
        
        // Second get should hit memory cache
        let retrieved2: Option<TestData> = cache.get("test-key").unwrap();
        assert!(retrieved2.is_some());
        assert_eq!(retrieved2.unwrap(), test_data);
    }

    #[test]
    fn test_large_data() {
        let (cache, _temp_dir) = setup_test_cache();
        
        let large_value = "x".repeat(10000);
        let test_data = TestData {
            id: "1".to_string(),
            value: large_value.clone(),
        };
        
        cache.set("test-key", &test_data, None).unwrap();
        
        let retrieved: Option<TestData> = cache.get("test-key").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, large_value);
    }

    #[test]
    fn test_special_characters_in_key() {
        let (cache, _temp_dir) = setup_test_cache();
        
        let test_data = TestData {
            id: "1".to_string(),
            value: "test value".to_string(),
        };
        
        let special_key = "profile:123:channels:category-1";
        cache.set(special_key, &test_data, None).unwrap();
        
        let retrieved: Option<TestData> = cache.get(special_key).unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_invalidate_all() {
        let (cache, _temp_dir) = setup_test_cache();
        
        let test_data = TestData {
            id: "1".to_string(),
            value: "test value".to_string(),
        };
        
        // Set multiple cache entries
        cache.set("key1", &test_data, None).unwrap();
        cache.set("key2", &test_data, None).unwrap();
        cache.set("key3", &test_data, None).unwrap();
        
        // Invalidate all entries
        cache.invalidate("%").unwrap();
        
        // Verify all entries are gone
        let retrieved1: Option<TestData> = cache.get("key1").unwrap();
        assert!(retrieved1.is_none());
        
        let retrieved2: Option<TestData> = cache.get("key2").unwrap();
        assert!(retrieved2.is_none());
        
        let retrieved3: Option<TestData> = cache.get("key3").unwrap();
        assert!(retrieved3.is_none());
    }
}
