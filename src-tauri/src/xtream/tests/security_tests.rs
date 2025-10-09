#[cfg(test)]
mod tests {
    use crate::xtream::{ProfileManager, CredentialManager, CreateProfileRequest};
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    fn setup_security_test() -> (ProfileManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        
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
        
        let db = Arc::new(Mutex::new(conn));
        let credential_manager = Arc::new(CredentialManager::with_key([1u8; 32]));
        let manager = ProfileManager::new(db, credential_manager);
        
        (manager, temp_dir)
    }

    #[test]
    fn test_credentials_are_encrypted_in_database() {
        let (manager, _temp_dir) = setup_security_test();
        
        let password = "super_secret_password_123!@#";
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: password.to_string(),
        };
        
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        // Read raw data from database
        let db = manager.db.lock().unwrap();
        let encrypted_data: Vec<u8> = db.query_row(
            "SELECT encrypted_credentials FROM xtream_profiles WHERE id = ?",
            [&profile_id],
            |row| row.get(0),
        ).unwrap();
        
        // Convert to string to check if password is visible
        let encrypted_str = String::from_utf8_lossy(&encrypted_data);
        
        // Password should NOT be visible in encrypted data
        assert!(!encrypted_str.contains(password));
        assert!(!encrypted_str.contains("testuser"));
    }

    #[test]
    fn test_sql_injection_prevention_in_profile_name() {
        let (manager, _temp_dir) = setup_security_test();
        
        // Try SQL injection in profile name
        let malicious_name = "Test'; DROP TABLE xtream_profiles; --";
        let request = CreateProfileRequest {
            name: malicious_name.to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        // Should either reject or safely handle the malicious input
        let result = manager.create_profile_without_validation(request);
        
        // If it succeeds, verify the table still exists
        if result.is_ok() {
            let profiles = manager.get_profiles();
            assert!(profiles.is_ok(), "Table should still exist");
        }
    }

    #[test]
    fn test_sql_injection_prevention_in_username() {
        let (manager, _temp_dir) = setup_security_test();
        
        let malicious_username = "admin' OR '1'='1";
        let request = CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: malicious_username.to_string(),
            password: "testpass".to_string(),
        };
        
        let result = manager.create_profile_without_validation(request);
        
        // Should handle safely
        if result.is_ok() {
            let profiles = manager.get_profiles().unwrap();
            assert!(profiles.len() > 0);
        }
    }

    #[test]
    fn test_different_encryption_keys_produce_different_ciphertext() {
        let manager1 = CredentialManager::with_key([1u8; 32]);
        let manager2 = CredentialManager::with_key([2u8; 32]);
        
        let credentials = crate::xtream::ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let encrypted1 = manager1.encrypt_credentials(&credentials).unwrap();
        let encrypted2 = manager2.encrypt_credentials(&credentials).unwrap();
        
        // Different keys should produce different ciphertext
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_tampering_detection() {
        let manager = CredentialManager::with_key([1u8; 32]);
        
        let credentials = crate::xtream::ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let mut encrypted = manager.encrypt_credentials(&credentials).unwrap();
        
        // Tamper with the encrypted data
        if encrypted.len() > 50 {
            encrypted[50] ^= 0xFF;
        }
        
        // Decryption should fail due to HMAC verification
        let result = manager.decrypt_credentials(&encrypted);
        assert!(result.is_err(), "Tampered data should fail decryption");
    }

    #[test]
    fn test_credential_cache_isolation() {
        let manager = CredentialManager::with_key([1u8; 32]);
        
        let credentials1 = crate::xtream::ProfileCredentials {
            url: "http://example1.com:8080".to_string(),
            username: "user1".to_string(),
            password: "pass1".to_string(),
        };
        
        let credentials2 = crate::xtream::ProfileCredentials {
            url: "http://example2.com:8080".to_string(),
            username: "user2".to_string(),
            password: "pass2".to_string(),
        };
        
        // Cache credentials for different profiles
        manager.cache_credentials("profile1", &credentials1).unwrap();
        manager.cache_credentials("profile2", &credentials2).unwrap();
        
        // Verify isolation
        let cached1 = manager.get_cached_credentials("profile1").unwrap().unwrap();
        let cached2 = manager.get_cached_credentials("profile2").unwrap().unwrap();
        
        assert_eq!(cached1.password, "pass1");
        assert_eq!(cached2.password, "pass2");
        assert_ne!(cached1.password, cached2.password);
    }

    #[test]
    fn test_secure_wipe_clears_sensitive_data() {
        let manager = CredentialManager::with_key([1u8; 32]);
        
        let credentials = crate::xtream::ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        // Cache credentials
        manager.cache_credentials("profile1", &credentials).unwrap();
        manager.cache_credentials("profile2", &credentials).unwrap();
        
        // Verify they're cached
        assert!(manager.get_cached_credentials("profile1").unwrap().is_some());
        assert!(manager.get_cached_credentials("profile2").unwrap().is_some());
        
        // Secure wipe
        manager.secure_wipe().unwrap();
        
        // Verify cache is cleared
        assert!(manager.get_cached_credentials("profile1").unwrap().is_none());
        assert!(manager.get_cached_credentials("profile2").unwrap().is_none());
    }

    #[test]
    fn test_url_validation_prevents_malicious_urls() {
        let (manager, _temp_dir) = setup_security_test();
        
        // Test various malicious URL patterns
        let malicious_urls = vec![
            "javascript:alert('xss')",
            "file:///etc/passwd",
            "data:text/html,<script>alert('xss')</script>",
            "ftp://malicious.com",
        ];
        
        for url in malicious_urls {
            let request = CreateProfileRequest {
                name: "Test Profile".to_string(),
                url: url.to_string(),
                username: "testuser".to_string(),
                password: "testpass".to_string(),
            };
            
            let result = manager.create_profile_without_validation(request);
            assert!(result.is_err(), "Malicious URL should be rejected: {}", url);
        }
    }

    #[test]
    fn test_valid_urls_are_accepted() {
        let (manager, _temp_dir) = setup_security_test();
        
        let valid_urls = vec![
            "http://example.com:8080",
            "https://secure.example.com:8443",
            "http://192.168.1.1:8080",
            "https://example.com",
        ];
        
        for (i, url) in valid_urls.iter().enumerate() {
            let request = CreateProfileRequest {
                name: format!("Profile {}", i),
                url: url.to_string(),
                username: "testuser".to_string(),
                password: "testpass".to_string(),
            };
            
            let result = manager.create_profile_without_validation(request);
            assert!(result.is_ok(), "Valid URL should be accepted: {}", url);
        }
    }

    #[test]
    fn test_profile_name_length_limit() {
        let (manager, _temp_dir) = setup_security_test();
        
        // Test extremely long name (potential buffer overflow attempt)
        let long_name = "a".repeat(10000);
        let request = CreateProfileRequest {
            name: long_name,
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err(), "Extremely long name should be rejected");
    }

    #[test]
    fn test_special_characters_in_credentials() {
        let (manager, _temp_dir) = setup_security_test();
        
        // Test with special characters that might cause issues
        let special_chars = vec![
            ("user<script>", "pass'\""),
            ("user\0null", "pass\nnewline"),
            ("user\ttab", "pass\rreturn"),
        ];
        
        for (i, (username, password)) in special_chars.iter().enumerate() {
            let request = CreateProfileRequest {
                name: format!("Profile {}", i),
                url: "http://example.com:8080".to_string(),
                username: username.to_string(),
                password: password.to_string(),
            };
            
            let result = manager.create_profile_without_validation(request);
            
            // Should either reject or handle safely
            if result.is_ok() {
                let profile_id = result.unwrap();
                let credentials = manager.get_profile_credentials(&profile_id).unwrap();
                // Credentials should be stored and retrieved correctly
                assert_eq!(credentials.username, *username);
                assert_eq!(credentials.password, *password);
            }
        }
    }
}
