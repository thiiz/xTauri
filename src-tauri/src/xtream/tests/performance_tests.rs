#[cfg(test)]
mod tests {
    use crate::xtream::{ProfileManager, CredentialManager, CreateProfileRequest, ContentCache};
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use tempfile::TempDir;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestContent {
        id: String,
        data: Vec<u8>,
    }

    fn setup_performance_test() -> (ProfileManager, ContentCache, TempDir) {
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
        
        let db = Arc::new(Mutex::new(conn));
        let credential_manager = Arc::new(CredentialManager::with_key([1u8; 32]));
        let manager = ProfileManager::new(Arc::clone(&db), credential_manager);
        let cache = ContentCache::new(db, Duration::from_secs(300));
        
        (manager, cache, temp_dir)
    }

    #[test]
    fn test_profile_creation_performance() {
        let (manager, _cache, _temp_dir) = setup_performance_test();
        
        let start = Instant::now();
        let iterations = 100;
        
        for i in 0..iterations {
            let request = CreateProfileRequest {
                name: format!("Profile {}", i),
                url: "http://example.com:8080".to_string(),
                username: format!("user{}", i),
                password: format!("pass{}", i),
            };
            manager.create_profile_without_validation(request).unwrap();
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed / iterations;
        
        println!("Created {} profiles in {:?}", iterations, elapsed);
        println!("Average time per profile: {:?}", avg_time);
        
        // Should be reasonably fast (< 50ms per profile on average)
        assert!(avg_time < Duration::from_millis(50));
    }

    #[test]
    fn test_profile_retrieval_performance() {
        let (manager, _cache, _temp_dir) = setup_performance_test();
        
        // Create profiles
        let mut profile_ids = Vec::new();
        for i in 0..100 {
            let request = CreateProfileRequest {
                name: format!("Profile {}", i),
                url: "http://example.com:8080".to_string(),
                username: format!("user{}", i),
                password: format!("pass{}", i),
            };
            let id = manager.create_profile_without_validation(request).unwrap();
            profile_ids.push(id);
        }
        
        // Measure retrieval time
        let start = Instant::now();
        let iterations = 1000;
        
        for i in 0..iterations {
            let profile_id = &profile_ids[i % profile_ids.len()];
            manager.get_profile(profile_id).unwrap();
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed / iterations;
        
        println!("Retrieved {} profiles in {:?}", iterations, elapsed);
        println!("Average time per retrieval: {:?}", avg_time);
        
        // Should be very fast (< 5ms per retrieval on average)
        assert!(avg_time < Duration::from_millis(5));
    }

    #[test]
    fn test_credential_encryption_performance() {
        let credential_manager = CredentialManager::with_key([1u8; 32]);
        
        let credentials = crate::xtream::ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        };
        
        let start = Instant::now();
        let iterations = 1000;
        
        for _ in 0..iterations {
            let encrypted = credential_manager.encrypt_credentials(&credentials).unwrap();
            let _decrypted = credential_manager.decrypt_credentials(&encrypted).unwrap();
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed / iterations;
        
        println!("Encrypted/decrypted {} times in {:?}", iterations, elapsed);
        println!("Average time per operation: {:?}", avg_time);
        
        // Should be reasonably fast (< 2ms per encrypt/decrypt cycle)
        assert!(avg_time < Duration::from_millis(2));
    }

    #[test]
    fn test_cache_performance() {
        let (_manager, cache, _temp_dir) = setup_performance_test();
        
        let test_content = TestContent {
            id: "test".to_string(),
            data: vec![0u8; 1024], // 1KB of data
        };
        
        // Measure set performance
        let start = Instant::now();
        let iterations = 1000;
        
        for i in 0..iterations {
            cache.set(&format!("key{}", i), &test_content, None).unwrap();
        }
        
        let set_elapsed = start.elapsed();
        let avg_set_time = set_elapsed / iterations;
        
        println!("Set {} cache entries in {:?}", iterations, set_elapsed);
        println!("Average time per set: {:?}", avg_set_time);
        
        // Measure get performance
        let start = Instant::now();
        
        for i in 0..iterations {
            let _retrieved: Option<TestContent> = cache.get(&format!("key{}", i)).unwrap();
        }
        
        let get_elapsed = start.elapsed();
        let avg_get_time = get_elapsed / iterations;
        
        println!("Got {} cache entries in {:?}", iterations, get_elapsed);
        println!("Average time per get: {:?}", avg_get_time);
        
        // Cache operations should be fast
        assert!(avg_set_time < Duration::from_millis(10));
        assert!(avg_get_time < Duration::from_millis(5));
    }

    #[test]
    fn test_large_data_cache_performance() {
        let (_manager, cache, _temp_dir) = setup_performance_test();
        
        // Test with 100KB of data
        let large_content = TestContent {
            id: "large".to_string(),
            data: vec![0u8; 100 * 1024],
        };
        
        let start = Instant::now();
        cache.set("large_key", &large_content, None).unwrap();
        let set_time = start.elapsed();
        
        let start = Instant::now();
        let _retrieved: Option<TestContent> = cache.get("large_key").unwrap();
        let get_time = start.elapsed();
        
        println!("Set 100KB in {:?}", set_time);
        println!("Get 100KB in {:?}", get_time);
        
        // Should handle large data reasonably well
        assert!(set_time < Duration::from_millis(100));
        assert!(get_time < Duration::from_millis(50));
    }

    #[test]
    fn test_concurrent_profile_access_performance() {
        use std::thread;
        
        let (manager, _cache, _temp_dir) = setup_performance_test();
        
        // Create a profile
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        let manager = Arc::new(manager);
        let start = Instant::now();
        
        // Spawn multiple threads to access the same profile
        let mut handles = vec![];
        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let profile_id_clone = profile_id.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    manager_clone.get_profile(&profile_id_clone).unwrap();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let elapsed = start.elapsed();
        println!("1000 concurrent profile accesses in {:?}", elapsed);
        
        // Should handle concurrent access efficiently
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn test_memory_usage_profile_creation() {
        let (manager, _cache, _temp_dir) = setup_performance_test();
        
        // Create many profiles and verify memory doesn't explode
        for i in 0..1000 {
            let request = CreateProfileRequest {
                name: format!("Profile {}", i),
                url: "http://example.com:8080".to_string(),
                username: format!("user{}", i),
                password: format!("pass{}", i),
            };
            manager.create_profile_without_validation(request).unwrap();
        }
        
        // If we got here without OOM, test passes
        let profiles = manager.get_profiles().unwrap();
        assert_eq!(profiles.len(), 1000);
    }
}
