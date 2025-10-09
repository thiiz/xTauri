#[cfg(test)]
mod tests {
    use crate::xtream::{CredentialManager, ProfileCredentials};

    fn create_test_credentials() -> ProfileCredentials {
        ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        }
    }

    #[test]
    fn test_encrypt_decrypt_basic() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        let encrypted = manager.encrypt_credentials(&credentials).unwrap();
        let decrypted = manager.decrypt_credentials(&encrypted).unwrap();
        
        assert_eq!(credentials.url, decrypted.url);
        assert_eq!(credentials.username, decrypted.username);
        assert_eq!(credentials.password, decrypted.password);
    }

    #[test]
    fn test_encrypt_decrypt_with_profile_id() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        let profile_id = "test-profile-123";
        
        let encrypted = manager.encrypt_credentials_for_profile(profile_id, &credentials).unwrap();
        let decrypted = manager.decrypt_credentials_for_profile(profile_id, &encrypted).unwrap();
        
        assert_eq!(credentials.url, decrypted.url);
        assert_eq!(credentials.username, decrypted.username);
        assert_eq!(credentials.password, decrypted.password);
    }

    #[test]
    fn test_different_keys_fail_decryption() {
        let manager1 = CredentialManager::with_key([1u8; 32]);
        let manager2 = CredentialManager::with_key([2u8; 32]);
        let credentials = create_test_credentials();
        
        let encrypted = manager1.encrypt_credentials(&credentials).unwrap();
        let result = manager2.decrypt_credentials(&encrypted);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_different_profile_ids_produce_different_ciphertext() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        let encrypted1 = manager.encrypt_credentials_for_profile("profile1", &credentials).unwrap();
        let encrypted2 = manager.encrypt_credentials_for_profile("profile2", &credentials).unwrap();
        
        // Same credentials but different profile IDs should produce different ciphertext
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_wrong_profile_id_fails_decryption() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        let encrypted = manager.encrypt_credentials_for_profile("profile1", &credentials).unwrap();
        let result = manager.decrypt_credentials_for_profile("profile2", &encrypted);
        
        // Should fail HMAC verification
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_credentials() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        let profile_id = "test-profile";
        
        // Initially no cached credentials
        assert!(manager.get_cached_credentials(profile_id).unwrap().is_none());
        
        // Cache credentials
        manager.cache_credentials(profile_id, &credentials).unwrap();
        
        // Should now be cached
        let cached = manager.get_cached_credentials(profile_id).unwrap().unwrap();
        assert_eq!(credentials.url, cached.url);
        assert_eq!(credentials.username, cached.username);
        assert_eq!(credentials.password, cached.password);
    }

    #[test]
    fn test_clear_cached_credentials() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        let profile_id = "test-profile";
        
        manager.cache_credentials(profile_id, &credentials).unwrap();
        assert!(manager.get_cached_credentials(profile_id).unwrap().is_some());
        
        manager.clear_cached_credentials(profile_id).unwrap();
        assert!(manager.get_cached_credentials(profile_id).unwrap().is_none());
    }

    #[test]
    fn test_clear_all_cached_credentials() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        // Cache multiple credentials
        manager.cache_credentials("profile1", &credentials).unwrap();
        manager.cache_credentials("profile2", &credentials).unwrap();
        manager.cache_credentials("profile3", &credentials).unwrap();
        
        // Verify they're cached
        assert!(manager.get_cached_credentials("profile1").unwrap().is_some());
        assert!(manager.get_cached_credentials("profile2").unwrap().is_some());
        assert!(manager.get_cached_credentials("profile3").unwrap().is_some());
        
        // Clear all
        manager.clear_all_cached_credentials().unwrap();
        
        // Verify they're gone
        assert!(manager.get_cached_credentials("profile1").unwrap().is_none());
        assert!(manager.get_cached_credentials("profile2").unwrap().is_none());
        assert!(manager.get_cached_credentials("profile3").unwrap().is_none());
    }

    #[test]
    fn test_encode_decode_for_storage() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        let encrypted = manager.encrypt_credentials(&credentials).unwrap();
        let encoded = manager.encode_for_storage(&encrypted);
        
        // Verify it's valid base64
        assert!(!encoded.is_empty());
        
        let decoded = manager.decode_from_storage(&encoded).unwrap();
        assert_eq!(encrypted, decoded);
        
        let decrypted = manager.decrypt_credentials(&decoded).unwrap();
        assert_eq!(credentials.url, decrypted.url);
        assert_eq!(credentials.username, decrypted.username);
        assert_eq!(credentials.password, decrypted.password);
    }

    #[test]
    fn test_invalid_base64_decode() {
        let manager = CredentialManager::with_key([1u8; 32]);
        
        let result = manager.decode_from_storage("invalid-base64!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_too_short_data() {
        let manager = CredentialManager::with_key([1u8; 32]);
        
        let result = manager.decrypt_credentials(&[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_invalid_length() {
        let manager = CredentialManager::with_key([1u8; 32]);
        
        // Data that's not a multiple of block size
        let result = manager.decrypt_credentials(&[0u8; 17]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_corrupted_data() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        let mut encrypted = manager.encrypt_credentials(&credentials).unwrap();
        
        // Corrupt the data
        if encrypted.len() > 70 {
            encrypted[70] ^= 0xFF;
        }
        
        let result = manager.decrypt_credentials(&encrypted);
        // Should fail due to HMAC verification or deserialization
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_special_characters() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "user@domain.com".to_string(),
            password: "p@$$w0rd!#%&*()".to_string(),
        };
        
        let encrypted = manager.encrypt_credentials(&credentials).unwrap();
        let decrypted = manager.decrypt_credentials(&encrypted).unwrap();
        
        assert_eq!(credentials.url, decrypted.url);
        assert_eq!(credentials.username, decrypted.username);
        assert_eq!(credentials.password, decrypted.password);
    }

    #[test]
    fn test_encrypt_unicode_characters() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "用户名".to_string(),
            password: "密码🔒".to_string(),
        };
        
        let encrypted = manager.encrypt_credentials(&credentials).unwrap();
        let decrypted = manager.decrypt_credentials(&encrypted).unwrap();
        
        assert_eq!(credentials.url, decrypted.url);
        assert_eq!(credentials.username, decrypted.username);
        assert_eq!(credentials.password, decrypted.password);
    }

    #[test]
    fn test_encrypt_long_password() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let long_password = "a".repeat(1000);
        let credentials = ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: long_password.clone(),
        };
        
        let encrypted = manager.encrypt_credentials(&credentials).unwrap();
        let decrypted = manager.decrypt_credentials(&encrypted).unwrap();
        
        assert_eq!(credentials.password, decrypted.password);
    }

    #[test]
    fn test_encrypt_empty_url() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = ProfileCredentials {
            url: "".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let encrypted = manager.encrypt_credentials(&credentials).unwrap();
        let decrypted = manager.decrypt_credentials(&encrypted).unwrap();
        
        assert_eq!(credentials.url, decrypted.url);
    }

    #[test]
    fn test_multiple_encrypt_decrypt_cycles() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        // Encrypt and decrypt multiple times
        let mut current = credentials.clone();
        for _ in 0..10 {
            let encrypted = manager.encrypt_credentials(&current).unwrap();
            current = manager.decrypt_credentials(&encrypted).unwrap();
        }
        
        assert_eq!(credentials.url, current.url);
        assert_eq!(credentials.username, current.username);
        assert_eq!(credentials.password, current.password);
    }

    #[test]
    fn test_hmac_integrity_verification() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        let profile_id = "test-profile";
        
        let mut encrypted = manager.encrypt_credentials_for_profile(profile_id, &credentials).unwrap();
        
        // Tamper with the HMAC (bytes 32-64)
        if encrypted.len() > 40 {
            encrypted[40] ^= 0xFF;
        }
        
        let result = manager.decrypt_credentials_for_profile(profile_id, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_salt_randomness() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        let profile_id = "test-profile";
        
        // Encrypt the same credentials multiple times
        let encrypted1 = manager.encrypt_credentials_for_profile(profile_id, &credentials).unwrap();
        let encrypted2 = manager.encrypt_credentials_for_profile(profile_id, &credentials).unwrap();
        
        // Due to random salt and IV, ciphertext should be different
        assert_ne!(encrypted1, encrypted2);
        
        // But both should decrypt to the same credentials
        let decrypted1 = manager.decrypt_credentials_for_profile(profile_id, &encrypted1).unwrap();
        let decrypted2 = manager.decrypt_credentials_for_profile(profile_id, &encrypted2).unwrap();
        
        assert_eq!(decrypted1.password, decrypted2.password);
    }

    #[test]
    fn test_secure_wipe() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        // Cache some credentials
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
    fn test_concurrent_cache_access() {
        use std::sync::Arc;
        use std::thread;
        
        let manager = Arc::new(CredentialManager::with_key([1u8; 32]));
        let credentials = create_test_credentials();
        
        let mut handles = vec![];
        
        // Spawn multiple threads to cache credentials
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let credentials_clone = credentials.clone();
            let handle = thread::spawn(move || {
                let profile_id = format!("profile{}", i);
                manager_clone.cache_credentials(&profile_id, &credentials_clone).unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all credentials were cached
        for i in 0..10 {
            let profile_id = format!("profile{}", i);
            assert!(manager.get_cached_credentials(&profile_id).unwrap().is_some());
        }
    }
}
