#[cfg(test)]
mod tests {
    use crate::xtream::{ProfileManager, CredentialManager, CreateProfileRequest, UpdateProfileRequest};
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    fn setup_integration_test() -> (ProfileManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        
        // Create all required tables
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
        
        let db = Arc::new(Mutex::new(conn));
        let credential_manager = Arc::new(CredentialManager::with_key([1u8; 32]));
        let manager = ProfileManager::new(db, credential_manager);
        
        (manager, temp_dir)
    }

    #[test]
    fn test_complete_profile_lifecycle() {
        let (manager, _temp_dir) = setup_integration_test();
        
        // 1. Create a profile
        let create_request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        };
        
        let profile_id = manager.create_profile_without_validation(create_request).unwrap();
        assert!(!profile_id.is_empty());
        
        // 2. Verify profile was created
        let profile = manager.get_profile(&profile_id).unwrap().unwrap();
        assert_eq!(profile.name, "Test Profile");
        assert!(!profile.is_active);
        
        // 3. Get credentials
        let credentials = manager.get_profile_credentials(&profile_id).unwrap();
        assert_eq!(credentials.username, "testuser");
        assert_eq!(credentials.password, "testpass123");
        
        // 4. Update profile
        let update_request = UpdateProfileRequest {
            name: Some("Updated Profile".to_string()),
            url: None,
            username: None,
            password: None,
        };
        manager.update_profile(&profile_id, update_request).unwrap();
        
        // 5. Verify update
        let updated_profile = manager.get_profile(&profile_id).unwrap().unwrap();
        assert_eq!(updated_profile.name, "Updated Profile");
        
        // 6. Set as active
        manager.set_active_profile(&profile_id).unwrap();
        let active_profile = manager.get_active_profile().unwrap().unwrap();
        assert_eq!(active_profile.id, profile_id);
        
        // 7. Delete profile
        manager.delete_profile(&profile_id).unwrap();
        let deleted_profile = manager.get_profile(&profile_id).unwrap();
        assert!(deleted_profile.is_none());
    }

    #[test]
    fn test_multiple_profiles_workflow() {
        let (manager, _temp_dir) = setup_integration_test();
        
        // Create multiple profiles
        let mut profile_ids = Vec::new();
        for i in 1..=3 {
            let request = CreateProfileRequest {
                name: format!("Profile {}", i),
                url: "http://example.com:8080".to_string(),
                username: format!("user{}", i),
                password: format!("pass{}", i),
            };
            let id = manager.create_profile_without_validation(request).unwrap();
            profile_ids.push(id);
        }
        
        // Verify all profiles exist
        let all_profiles = manager.get_profiles().unwrap();
        assert_eq!(all_profiles.len(), 3);
        
        // Set first profile as active
        manager.set_active_profile(&profile_ids[0]).unwrap();
        let active = manager.get_active_profile().unwrap().unwrap();
        assert_eq!(active.id, profile_ids[0]);
        
        // Switch to second profile
        manager.set_active_profile(&profile_ids[1]).unwrap();
        let active = manager.get_active_profile().unwrap().unwrap();
        assert_eq!(active.id, profile_ids[1]);
        
        // Verify first profile is no longer active
        let first_profile = manager.get_profile(&profile_ids[0]).unwrap().unwrap();
        assert!(!first_profile.is_active);
        
        // Delete one profile
        manager.delete_profile(&profile_ids[2]).unwrap();
        let remaining_profiles = manager.get_profiles().unwrap();
        assert_eq!(remaining_profiles.len(), 2);
    }

    #[test]
    fn test_profile_update_with_credentials() {
        let (manager, _temp_dir) = setup_integration_test();
        
        // Create profile
        let create_request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "olduser".to_string(),
            password: "oldpass".to_string(),
        };
        let profile_id = manager.create_profile_without_validation(create_request).unwrap();
        
        // Update all credentials
        let update_request = UpdateProfileRequest {
            name: None,
            url: Some("http://newserver.com:8080".to_string()),
            username: Some("newuser".to_string()),
            password: Some("newpass".to_string()),
        };
        manager.update_profile(&profile_id, update_request).unwrap();
        
        // Verify credentials were updated
        let credentials = manager.get_profile_credentials(&profile_id).unwrap();
        assert_eq!(credentials.url, "http://newserver.com:8080");
        assert_eq!(credentials.username, "newuser");
        assert_eq!(credentials.password, "newpass");
        
        // Verify profile metadata was updated
        let profile = manager.get_profile(&profile_id).unwrap().unwrap();
        assert_eq!(profile.url, "http://newserver.com:8080");
        assert_eq!(profile.username, "newuser");
    }

    #[test]
    fn test_credential_caching_workflow() {
        let (manager, _temp_dir) = setup_integration_test();
        
        // Create profile
        let create_request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        let profile_id = manager.create_profile_without_validation(create_request).unwrap();
        
        // First access - should load from database and cache
        let credentials1 = manager.get_profile_credentials(&profile_id).unwrap();
        assert_eq!(credentials1.password, "testpass");
        
        // Second access - should hit cache
        let credentials2 = manager.get_profile_credentials(&profile_id).unwrap();
        assert_eq!(credentials2.password, "testpass");
        
        // Update credentials
        let update_request = UpdateProfileRequest {
            name: None,
            url: None,
            username: None,
            password: Some("newpass".to_string()),
        };
        manager.update_profile(&profile_id, update_request).unwrap();
        
        // Should get updated credentials (cache should be updated)
        let credentials3 = manager.get_profile_credentials(&profile_id).unwrap();
        assert_eq!(credentials3.password, "newpass");
    }

    #[test]
    fn test_profile_validation_workflow() {
        let (manager, _temp_dir) = setup_integration_test();
        
        // Test empty name
        let invalid_request = CreateProfileRequest {
            name: "".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        assert!(manager.create_profile_without_validation(invalid_request).is_err());
        
        // Test invalid URL
        let invalid_request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "not-a-url".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        assert!(manager.create_profile_without_validation(invalid_request).is_err());
        
        // Test empty username
        let invalid_request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "".to_string(),
            password: "testpass".to_string(),
        };
        assert!(manager.create_profile_without_validation(invalid_request).is_err());
        
        // Test empty password
        let invalid_request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "".to_string(),
        };
        assert!(manager.create_profile_without_validation(invalid_request).is_err());
        
        // Test valid profile
        let valid_request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        assert!(manager.create_profile_without_validation(valid_request).is_ok());
    }

    #[test]
    fn test_concurrent_profile_operations() {
        use std::thread;
        
        let (manager, _temp_dir) = setup_integration_test();
        let manager = Arc::new(manager);
        
        let mut handles = vec![];
        
        // Create profiles concurrently
        for i in 0..5 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                let request = CreateProfileRequest {
                    name: format!("Profile {}", i),
                    url: "http://example.com:8080".to_string(),
                    username: format!("user{}", i),
                    password: format!("pass{}", i),
                };
                manager_clone.create_profile_without_validation(request)
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        let mut profile_ids = Vec::new();
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok());
            profile_ids.push(result.unwrap());
        }
        
        // Verify all profiles were created
        let all_profiles = manager.get_profiles().unwrap();
        assert_eq!(all_profiles.len(), 5);
    }

    #[test]
    fn test_profile_cascade_delete() {
        let (manager, _temp_dir) = setup_integration_test();
        
        // Create profile
        let create_request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        let profile_id = manager.create_profile_without_validation(create_request).unwrap();
        
        // Set as active
        manager.set_active_profile(&profile_id).unwrap();
        
        // Delete profile
        manager.delete_profile(&profile_id).unwrap();
        
        // Verify profile is gone
        assert!(manager.get_profile(&profile_id).unwrap().is_none());
        
        // Verify no active profile
        assert!(manager.get_active_profile().unwrap().is_none());
    }
}
