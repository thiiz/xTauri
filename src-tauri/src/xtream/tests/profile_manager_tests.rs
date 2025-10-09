#[cfg(test)]
mod tests {
    use crate::xtream::{ProfileManager, CredentialManager, CreateProfileRequest, UpdateProfileRequest, ProfileCredentials};
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
        
        (Arc::new(Mutex::new(conn)), temp_dir)
    }

    fn create_test_manager() -> (ProfileManager, TempDir) {
        let (db, temp_dir) = setup_test_db();
        let credential_manager = Arc::new(CredentialManager::with_key([1u8; 32]));
        let manager = ProfileManager::new(db, credential_manager);
        (manager, temp_dir)
    }

    #[test]
    fn test_create_profile_without_validation() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        assert!(!profile_id.is_empty());
        
        // Verify profile was created
        let profile = manager.get_profile(&profile_id).unwrap().unwrap();
        assert_eq!(profile.name, "Test Profile");
        assert_eq!(profile.url, "http://example.com:8080");
        assert_eq!(profile.username, "testuser");
        assert!(!profile.is_active);
    }

    #[test]
    fn test_create_profile_duplicate_name() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        manager.create_profile_without_validation(request.clone()).unwrap();
        
        // Try to create another profile with the same name
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_profile_empty_name() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_profile_invalid_url() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "not-a-valid-url".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_profile() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        let profile = manager.get_profile(&profile_id).unwrap().unwrap();
        assert_eq!(profile.id, profile_id);
        assert_eq!(profile.name, "Test Profile");
    }

    #[test]
    fn test_get_profile_not_found() {
        let (manager, _temp_dir) = create_test_manager();
        
        let profile = manager.get_profile("nonexistent-id").unwrap();
        assert!(profile.is_none());
    }

    #[test]
    fn test_get_all_profiles() {
        let (manager, _temp_dir) = create_test_manager();
        
        // Create multiple profiles
        for i in 1..=3 {
            let request = CreateProfileRequest {
                name: format!("Profile {}", i),
                url: "http://example.com:8080".to_string(),
                username: format!("user{}", i),
                password: "testpass".to_string(),
            };
            manager.create_profile_without_validation(request).unwrap();
        }
        
        let profiles = manager.get_profiles().unwrap();
        assert_eq!(profiles.len(), 3);
    }

    #[test]
    fn test_update_profile_name() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Original Name".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        let update_request = UpdateProfileRequest {
            name: Some("Updated Name".to_string()),
            url: None,
            username: None,
            password: None,
        };
        
        manager.update_profile(&profile_id, update_request).unwrap();
        
        let profile = manager.get_profile(&profile_id).unwrap().unwrap();
        assert_eq!(profile.name, "Updated Name");
    }

    #[test]
    fn test_update_profile_credentials() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        let update_request = UpdateProfileRequest {
            name: None,
            url: Some("http://newserver.com:8080".to_string()),
            username: Some("newuser".to_string()),
            password: Some("newpass".to_string()),
        };
        
        manager.update_profile(&profile_id, update_request).unwrap();
        
        let profile = manager.get_profile(&profile_id).unwrap().unwrap();
        assert_eq!(profile.url, "http://newserver.com:8080");
        assert_eq!(profile.username, "newuser");
        
        // Verify credentials were updated
        let credentials = manager.get_profile_credentials(&profile_id).unwrap();
        assert_eq!(credentials.password, "newpass");
    }

    #[test]
    fn test_delete_profile() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        manager.delete_profile(&profile_id).unwrap();
        
        let profile = manager.get_profile(&profile_id).unwrap();
        assert!(profile.is_none());
    }

    #[test]
    fn test_delete_profile_not_found() {
        let (manager, _temp_dir) = create_test_manager();
        
        let result = manager.delete_profile("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_active_profile() {
        let (manager, _temp_dir) = create_test_manager();
        
        // Create two profiles
        let request1 = CreateProfileRequest {
            name: "Profile 1".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "user1".to_string(),
            password: "pass1".to_string(),
        };
        let profile_id1 = manager.create_profile_without_validation(request1).unwrap();
        
        let request2 = CreateProfileRequest {
            name: "Profile 2".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "user2".to_string(),
            password: "pass2".to_string(),
        };
        let profile_id2 = manager.create_profile_without_validation(request2).unwrap();
        
        // Set first profile as active
        manager.set_active_profile(&profile_id1).unwrap();
        
        let profile1 = manager.get_profile(&profile_id1).unwrap().unwrap();
        assert!(profile1.is_active);
        
        let profile2 = manager.get_profile(&profile_id2).unwrap().unwrap();
        assert!(!profile2.is_active);
        
        // Switch to second profile
        manager.set_active_profile(&profile_id2).unwrap();
        
        let profile1 = manager.get_profile(&profile_id1).unwrap().unwrap();
        assert!(!profile1.is_active);
        
        let profile2 = manager.get_profile(&profile_id2).unwrap().unwrap();
        assert!(profile2.is_active);
    }

    #[test]
    fn test_get_active_profile() {
        let (manager, _temp_dir) = create_test_manager();
        
        // Initially no active profile
        let active = manager.get_active_profile().unwrap();
        assert!(active.is_none());
        
        // Create and activate a profile
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        manager.set_active_profile(&profile_id).unwrap();
        
        let active = manager.get_active_profile().unwrap().unwrap();
        assert_eq!(active.id, profile_id);
    }

    #[test]
    fn test_get_profile_credentials() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        };
        
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        let credentials = manager.get_profile_credentials(&profile_id).unwrap();
        assert_eq!(credentials.url, "http://example.com:8080");
        assert_eq!(credentials.username, "testuser");
        assert_eq!(credentials.password, "testpass123");
    }

    #[test]
    fn test_credentials_are_encrypted() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        };
        
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        // Get raw encrypted credentials from database
        let db = manager.db.lock().unwrap();
        let encrypted: String = db.query_row(
            "SELECT encrypted_credentials FROM xtream_profiles WHERE id = ?",
            [&profile_id],
            |row| row.get(0),
        ).unwrap();
        
        // Verify the encrypted data doesn't contain the plain password
        assert!(!encrypted.contains("testpass123"));
    }

    #[test]
    fn test_profile_name_length_validation() {
        let (manager, _temp_dir) = create_test_manager();
        
        let long_name = "a".repeat(101);
        let request = CreateProfileRequest {
            name: long_name,
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_url_scheme_validation() {
        let (manager, _temp_dir) = create_test_manager();
        
        // Test invalid scheme
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "ftp://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_username_validation() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "".to_string(),
            password: "testpass".to_string(),
        };
        
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_password_validation() {
        let (manager, _temp_dir) = create_test_manager();
        
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "".to_string(),
        };
        
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_credentials_format() {
        let (manager, _temp_dir) = create_test_manager();
        
        let valid_credentials = ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        // This should not panic or error
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: valid_credentials.url.clone(),
            username: valid_credentials.username.clone(),
            password: valid_credentials.password.clone(),
        };
        
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_ok());
    }
}
