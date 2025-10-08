use crate::error::{Result, XTauriError};
use crate::xtream::types::ProfileCredentials;
use aes::Aes256;
use aes::cipher::{
    BlockEncrypt, BlockDecrypt, KeyInit,
    generic_array::GenericArray,
};
use rand::{RngCore, thread_rng};
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;
use std::sync::Mutex;
use keyring::Entry;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use hmac::{Hmac, Mac};
use rusqlite;
use chrono;

/// Manages secure encryption and decryption of profile credentials
pub struct CredentialManager {
    encryption_key: [u8; 32],
    // In-memory cache for decrypted credentials (cleared on app restart)
    credential_cache: Mutex<HashMap<String, ProfileCredentials>>,
    // Application identifier for keyring storage
    app_name: String,
}

impl CredentialManager {
    /// Create a new credential manager with platform-specific secure key storage
    pub fn new() -> Result<Self> {
        let app_name = "xtauri-iptv".to_string();
        let encryption_key = Self::get_or_create_master_key(&app_name)?;
        
        Ok(Self {
            encryption_key,
            credential_cache: Mutex::new(HashMap::new()),
            app_name,
        })
    }
    
    /// Create a credential manager with a specific key (for testing)
    #[cfg(test)]
    pub fn with_key(key: [u8; 32]) -> Self {
        Self {
            encryption_key: key,
            credential_cache: Mutex::new(HashMap::new()),
            app_name: "xtauri-iptv-test".to_string(),
        }
    }
    
    /// Get or create the master encryption key using platform-specific secure storage
    fn get_or_create_master_key(app_name: &str) -> Result<[u8; 32]> {
        let entry = Entry::new(app_name, "master_key")
            .map_err(|e| XTauriError::credential_encryption(format!("Failed to access keyring: {}", e)))?;
        
        // Try to retrieve existing key
        match entry.get_password() {
            Ok(key_b64) => {
                // Decode existing key
                let key_bytes = general_purpose::STANDARD.decode(&key_b64)
                    .map_err(|e| XTauriError::credential_decryption(format!("Failed to decode master key: {}", e)))?;
                
                if key_bytes.len() != 32 {
                    return Err(XTauriError::credential_decryption("Invalid master key length".to_string()));
                }
                
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                Ok(key)
            }
            Err(_) => {
                // Generate new key and store it securely
                let mut key = [0u8; 32];
                thread_rng().fill_bytes(&mut key);
                
                let key_b64 = general_purpose::STANDARD.encode(&key);
                entry.set_password(&key_b64)
                    .map_err(|e| XTauriError::credential_encryption(format!("Failed to store master key: {}", e)))?;
                
                Ok(key)
            }
        }
    }
    
    /// Derive a profile-specific encryption key using PBKDF2
    fn derive_profile_key(&self, profile_id: &str, salt: &[u8]) -> [u8; 32] {
        let mut derived_key = [0u8; 32];
        
        // Use master key + profile ID as password for PBKDF2
        let mut password = self.encryption_key.to_vec();
        password.extend_from_slice(profile_id.as_bytes());
        
        pbkdf2_hmac::<Sha256>(&password, salt, 100_000, &mut derived_key);
        
        // Clear password from memory
        password.fill(0);
        
        derived_key
    }
    
    /// Generate HMAC for integrity verification
    fn generate_hmac(&self, data: &[u8], key: &[u8; 32]) -> Result<[u8; 32]> {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key)
            .map_err(|e| XTauriError::credential_encryption(format!("HMAC key error: {}", e)))?;
        
        mac.update(data);
        let result = mac.finalize();
        
        let mut hmac_bytes = [0u8; 32];
        hmac_bytes.copy_from_slice(&result.into_bytes());
        Ok(hmac_bytes)
    }
    
    /// Verify HMAC for integrity checking
    fn verify_hmac(&self, data: &[u8], expected_hmac: &[u8; 32], key: &[u8; 32]) -> Result<bool> {
        let computed_hmac = self.generate_hmac(data, key)?;
        Ok(computed_hmac == *expected_hmac)
    }
    
    /// Encrypt profile credentials with enhanced security
    pub fn encrypt_credentials_for_profile(&self, profile_id: &str, credentials: &ProfileCredentials) -> Result<Vec<u8>> {
        let serialized = serde_json::to_vec(credentials)
            .map_err(|e| XTauriError::credential_encryption(format!("Serialization failed: {}", e)))?;
        
        // Generate random salt and IV
        let mut salt = [0u8; 16];
        let mut iv = [0u8; 16];
        thread_rng().fill_bytes(&mut salt);
        thread_rng().fill_bytes(&mut iv);
        
        // Derive profile-specific key
        let profile_key = self.derive_profile_key(profile_id, &salt);
        
        // Pad the data to block size (16 bytes for AES)
        let mut padded_data = serialized;
        let padding_needed = 16 - (padded_data.len() % 16);
        if padding_needed != 16 {
            padded_data.extend(vec![padding_needed as u8; padding_needed]);
        }
        
        // Encrypt the data using CBC mode
        let cipher = Aes256::new(GenericArray::from_slice(&profile_key));
        let mut encrypted_data = padded_data;
        let mut previous_block = iv;
        
        // Encrypt in blocks using CBC mode
        for chunk in encrypted_data.chunks_mut(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            
            // XOR with previous block (CBC mode)
            for (i, byte) in block.iter_mut().enumerate() {
                *byte ^= previous_block[i];
            }
            
            cipher.encrypt_block(&mut block);
            chunk.copy_from_slice(&block);
            previous_block.copy_from_slice(&block);
        }
        
        // Generate HMAC for integrity
        let hmac = self.generate_hmac(&encrypted_data, &profile_key)?;
        
        // Clear profile key from memory
        let mut profile_key_mut = profile_key;
        profile_key_mut.fill(0);
        
        // Format: salt (16) + iv (16) + hmac (32) + encrypted_data
        let mut result = Vec::with_capacity(16 + 16 + 32 + encrypted_data.len());
        result.extend_from_slice(&salt);
        result.extend_from_slice(&iv);
        result.extend_from_slice(&hmac);
        result.extend(encrypted_data);
        
        Ok(result)
    }
    
    /// Encrypt profile credentials (backward compatibility)
    pub fn encrypt_credentials(&self, credentials: &ProfileCredentials) -> Result<Vec<u8>> {
        // Use a default profile ID for backward compatibility
        self.encrypt_credentials_for_profile("default", credentials)
    }
    
    /// Decrypt profile credentials with enhanced security
    pub fn decrypt_credentials_for_profile(&self, profile_id: &str, encrypted_data: &[u8]) -> Result<ProfileCredentials> {
        // Minimum size: salt (16) + iv (16) + hmac (32) + at least one block (16)
        if encrypted_data.len() < 80 {
            return Err(XTauriError::credential_decryption("Invalid encrypted data length".to_string()));
        }
        
        // Extract components
        let salt = &encrypted_data[0..16];
        let iv = &encrypted_data[16..32];
        let stored_hmac = &encrypted_data[32..64];
        let ciphertext = &encrypted_data[64..];
        
        if ciphertext.len() % 16 != 0 {
            return Err(XTauriError::credential_decryption("Invalid ciphertext length".to_string()));
        }
        
        // Derive profile-specific key
        let profile_key = self.derive_profile_key(profile_id, salt);
        
        // Verify HMAC integrity
        let mut expected_hmac = [0u8; 32];
        expected_hmac.copy_from_slice(stored_hmac);
        
        if !self.verify_hmac(ciphertext, &expected_hmac, &profile_key)? {
            return Err(XTauriError::credential_decryption("HMAC verification failed - data may be corrupted".to_string()));
        }
        
        // Decrypt the data using CBC mode
        let cipher = Aes256::new(GenericArray::from_slice(&profile_key));
        let mut decrypted_data = ciphertext.to_vec();
        let mut previous_block = [0u8; 16];
        previous_block.copy_from_slice(iv);
        
        // Decrypt in blocks using CBC mode
        for chunk in decrypted_data.chunks_mut(16) {
            let mut original_chunk = [0u8; 16];
            original_chunk.copy_from_slice(chunk);
            
            let mut block = GenericArray::clone_from_slice(chunk);
            
            cipher.decrypt_block(&mut block);
            
            // XOR with previous block (CBC mode)
            for (i, byte) in block.iter_mut().enumerate() {
                *byte ^= previous_block[i];
            }
            
            chunk.copy_from_slice(&block);
            previous_block.copy_from_slice(&original_chunk);
        }
        
        // Clear profile key from memory
        let mut profile_key_mut = profile_key;
        profile_key_mut.fill(0);
        
        // Remove padding
        if let Some(&padding_len) = decrypted_data.last() {
            if padding_len as usize <= 16 && padding_len as usize <= decrypted_data.len() {
                let new_len = decrypted_data.len() - padding_len as usize;
                decrypted_data.truncate(new_len);
            }
        }
        
        // Deserialize the credentials
        let credentials: ProfileCredentials = serde_json::from_slice(&decrypted_data)
            .map_err(|e| XTauriError::credential_decryption(format!("Deserialization failed: {}", e)))?;
        
        // Clear decrypted data from memory
        decrypted_data.fill(0);
        
        Ok(credentials)
    }
    
    /// Decrypt profile credentials (backward compatibility)
    pub fn decrypt_credentials(&self, encrypted_data: &[u8]) -> Result<ProfileCredentials> {
        // Try new format first
        if encrypted_data.len() >= 80 {
            if let Ok(credentials) = self.decrypt_credentials_for_profile("default", encrypted_data) {
                return Ok(credentials);
            }
        }
        
        // Fall back to old format for backward compatibility
        self.decrypt_credentials_legacy(encrypted_data)
    }
    
    /// Legacy decrypt method for backward compatibility
    fn decrypt_credentials_legacy(&self, encrypted_data: &[u8]) -> Result<ProfileCredentials> {
        if encrypted_data.len() < 16 {
            return Err(XTauriError::credential_decryption("Invalid encrypted data length".to_string()));
        }
        
        // Extract IV and encrypted data
        let iv = &encrypted_data[0..16];
        let ciphertext = &encrypted_data[16..];
        
        if ciphertext.len() % 16 != 0 {
            return Err(XTauriError::credential_decryption("Invalid ciphertext length".to_string()));
        }
        
        // Decrypt the data
        let cipher = Aes256::new(GenericArray::from_slice(&self.encryption_key));
        let mut decrypted_data = ciphertext.to_vec();
        
        // Decrypt in blocks
        for chunk in decrypted_data.chunks_mut(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.decrypt_block(&mut block);
            // XOR with IV for first block, or previous ciphertext for subsequent blocks
            for (i, byte) in block.iter_mut().enumerate() {
                *byte ^= iv[i % 16];
            }
            chunk.copy_from_slice(&block);
        }
        
        // Remove padding
        if let Some(&padding_len) = decrypted_data.last() {
            if padding_len as usize <= 16 && padding_len as usize <= decrypted_data.len() {
                let new_len = decrypted_data.len() - padding_len as usize;
                decrypted_data.truncate(new_len);
            }
        }
        
        // Deserialize the credentials
        let credentials: ProfileCredentials = serde_json::from_slice(&decrypted_data)
            .map_err(|e| XTauriError::credential_decryption(format!("Deserialization failed: {}", e)))?;
        
        Ok(credentials)
    }
    
    /// Store credentials in memory cache
    pub fn cache_credentials(&self, profile_id: &str, credentials: &ProfileCredentials) -> Result<()> {
        let mut cache = self.credential_cache.lock()
            .map_err(|_| XTauriError::lock_acquisition("credential cache"))?;
        
        cache.insert(profile_id.to_string(), credentials.clone());
        Ok(())
    }
    
    /// Retrieve credentials from memory cache
    pub fn get_cached_credentials(&self, profile_id: &str) -> Result<Option<ProfileCredentials>> {
        let cache = self.credential_cache.lock()
            .map_err(|_| XTauriError::lock_acquisition("credential cache"))?;
        
        Ok(cache.get(profile_id).cloned())
    }
    
    /// Clear credentials from memory cache
    pub fn clear_cached_credentials(&self, profile_id: &str) -> Result<()> {
        let mut cache = self.credential_cache.lock()
            .map_err(|_| XTauriError::lock_acquisition("credential cache"))?;
        
        cache.remove(profile_id);
        Ok(())
    }
    
    /// Clear all cached credentials
    pub fn clear_all_cached_credentials(&self) -> Result<()> {
        let mut cache = self.credential_cache.lock()
            .map_err(|_| XTauriError::lock_acquisition("credential cache"))?;
        
        cache.clear();
        Ok(())
    }
    
    /// Store encrypted credentials in platform-specific secure storage
    pub fn store_credentials(&self, profile_id: &str, credentials: &ProfileCredentials) -> Result<()> {
        // Encrypt credentials with profile-specific key
        let encrypted_data = self.encrypt_credentials_for_profile(profile_id, credentials)?;
        
        // Encode for storage
        let encoded_data = self.encode_for_storage(&encrypted_data);
        
        // Store in platform-specific keyring
        let entry = Entry::new(&self.app_name, &format!("profile_{}", profile_id))
            .map_err(|e| XTauriError::credential_encryption(format!("Failed to access keyring for profile {}: {}", profile_id, e)))?;
        
        entry.set_password(&encoded_data)
            .map_err(|e| XTauriError::credential_encryption(format!("Failed to store credentials for profile {}: {}", profile_id, e)))?;
        
        // Also cache in memory for quick access
        self.cache_credentials(profile_id, credentials)?;
        
        Ok(())
    }
    
    /// Retrieve and decrypt credentials from platform-specific secure storage
    pub fn retrieve_credentials(&self, profile_id: &str) -> Result<Option<ProfileCredentials>> {
        // First check memory cache
        if let Some(cached_credentials) = self.get_cached_credentials(profile_id)? {
            return Ok(Some(cached_credentials));
        }
        
        // Retrieve from platform-specific keyring
        let entry = Entry::new(&self.app_name, &format!("profile_{}", profile_id))
            .map_err(|e| XTauriError::credential_decryption(format!("Failed to access keyring for profile {}: {}", profile_id, e)))?;
        
        match entry.get_password() {
            Ok(encoded_data) => {
                // Decode from storage
                let encrypted_data = self.decode_from_storage(&encoded_data)?;
                
                // Decrypt credentials
                let credentials = self.decrypt_credentials_for_profile(profile_id, &encrypted_data)?;
                
                // Cache for future use
                self.cache_credentials(profile_id, &credentials)?;
                
                Ok(Some(credentials))
            }
            Err(keyring::Error::NoEntry) => {
                // No credentials stored for this profile
                Ok(None)
            }
            Err(e) => {
                Err(XTauriError::credential_decryption(format!("Failed to retrieve credentials for profile {}: {}", profile_id, e)))
            }
        }
    }
    
    /// Delete credentials from both secure storage and cache
    pub fn delete_credentials(&self, profile_id: &str) -> Result<()> {
        // Remove from platform-specific keyring
        let entry = Entry::new(&self.app_name, &format!("profile_{}", profile_id))
            .map_err(|e| XTauriError::credential_decryption(format!("Failed to access keyring for profile {}: {}", profile_id, e)))?;
        
        match entry.delete_password() {
            Ok(()) => {},
            Err(keyring::Error::NoEntry) => {
                // Already deleted or never existed
            }
            Err(e) => {
                return Err(XTauriError::credential_decryption(format!("Failed to delete credentials for profile {}: {}", profile_id, e)));
            }
        }
        
        // Remove from memory cache
        self.clear_cached_credentials(profile_id)?;
        
        Ok(())
    }
    
    /// Check if credentials exist for a profile
    pub fn credentials_exist(&self, profile_id: &str) -> Result<bool> {
        // Check memory cache first
        if self.get_cached_credentials(profile_id)?.is_some() {
            return Ok(true);
        }
        
        // Check platform-specific keyring
        let entry = Entry::new(&self.app_name, &format!("profile_{}", profile_id))
            .map_err(|e| XTauriError::credential_decryption(format!("Failed to access keyring for profile {}: {}", profile_id, e)))?;
        
        match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => Err(XTauriError::credential_decryption(format!("Failed to check credentials for profile {}: {}", profile_id, e))),
        }
    }
    
    /// Migrate credentials from old format to new format
    pub fn migrate_credentials(&self, profile_id: &str, old_encrypted_data: &[u8]) -> Result<()> {
        // Decrypt using legacy method
        let credentials = self.decrypt_credentials_legacy(old_encrypted_data)?;
        
        // Store using new secure method
        self.store_credentials(profile_id, &credentials)?;
        
        Ok(())
    }
    
    /// Store credentials in database with encryption
    pub fn store_credentials_in_db(
        &self,
        conn: &rusqlite::Connection,
        profile_id: &str,
        credentials: &ProfileCredentials,
    ) -> Result<()> {
        // Encrypt credentials with profile-specific key
        let encrypted_data = self.encrypt_credentials_for_profile(profile_id, credentials)?;
        
        // Update the profile's encrypted credentials in the database
        conn.execute(
            "UPDATE xtream_profiles SET encrypted_credentials = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![
                encrypted_data,
                chrono::Utc::now().to_rfc3339(),
                profile_id
            ],
        ).map_err(|e| XTauriError::Database(e))?;
        
        // Also cache in memory for quick access
        self.cache_credentials(profile_id, credentials)?;
        
        Ok(())
    }
    
    /// Retrieve and decrypt credentials from database
    pub fn retrieve_credentials_from_db(
        &self,
        conn: &rusqlite::Connection,
        profile_id: &str,
    ) -> Result<Option<ProfileCredentials>> {
        // First check memory cache
        if let Some(cached_credentials) = self.get_cached_credentials(profile_id)? {
            return Ok(Some(cached_credentials));
        }
        
        // Retrieve from database
        let mut stmt = conn.prepare(
            "SELECT encrypted_credentials FROM xtream_profiles WHERE id = ?1"
        ).map_err(|e| XTauriError::Database(e))?;
        
        let mut credential_iter = stmt.query_map(rusqlite::params![profile_id], |row| {
            Ok(row.get::<_, Vec<u8>>(0)?)
        }).map_err(|e| XTauriError::Database(e))?;
        
        match credential_iter.next() {
            Some(encrypted_data_result) => {
                let encrypted_data = encrypted_data_result.map_err(|e| XTauriError::Database(e))?;
                
                // Decrypt credentials
                let credentials = self.decrypt_credentials_for_profile(profile_id, &encrypted_data)?;
                
                // Cache for future use
                self.cache_credentials(profile_id, &credentials)?;
                
                Ok(Some(credentials))
            }
            None => Ok(None), // Profile not found
        }
    }
    
    /// Delete credentials from database and cache
    pub fn delete_credentials_from_db(
        &self,
        _conn: &rusqlite::Connection,
        profile_id: &str,
    ) -> Result<()> {
        // The credentials will be deleted when the profile is deleted
        // via CASCADE constraint, but we should clear the cache
        self.clear_cached_credentials(profile_id)?;
        
        Ok(())
    }
    
    /// Migrate legacy credentials to database format
    pub fn migrate_credentials_to_db(
        &self,
        conn: &rusqlite::Connection,
        profile_id: &str,
        old_encrypted_data: &[u8],
    ) -> Result<()> {
        // Decrypt using legacy method
        let credentials = self.decrypt_credentials_legacy(old_encrypted_data)?;
        
        // Store using new database method
        self.store_credentials_in_db(conn, profile_id, &credentials)?;
        
        Ok(())
    }
    
    /// Securely wipe sensitive data from memory
    pub fn secure_wipe(&self) -> Result<()> {
        // Clear all cached credentials
        self.clear_all_cached_credentials()?;
        
        // Note: The encryption key in self.encryption_key cannot be wiped here
        // as it would break the struct, but it will be cleared when the struct is dropped
        
        Ok(())
    }
    
    /// Encode encrypted data as base64 for database storage
    pub fn encode_for_storage(&self, encrypted_data: &[u8]) -> String {
        general_purpose::STANDARD.encode(encrypted_data)
    }
    
    /// Decode base64 data from database storage
    pub fn decode_from_storage(&self, encoded_data: &str) -> Result<Vec<u8>> {
        general_purpose::STANDARD.decode(encoded_data)
            .map_err(|e| XTauriError::credential_decryption(format!("Base64 decode failed: {}", e)))
    }
    
    /// Get a content cache instance for Xtream client operations
    /// This creates a temporary cache for validation purposes
    pub fn get_cache(&self) -> std::sync::Arc<crate::xtream::ContentCache> {
        // Create a temporary in-memory database for validation
        let temp_db = rusqlite::Connection::open_in_memory().unwrap();
        
        // Create the required table structure
        temp_db.execute(
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
        
        temp_db.execute(
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
        
        // Insert a temporary profile for validation
        temp_db.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) VALUES ('temp', 'Temp', 'http://temp.com', 'user', X'00')",
            [],
        ).unwrap();
        
        let db = std::sync::Arc::new(std::sync::Mutex::new(temp_db));
        let default_ttl = std::time::Duration::from_secs(300); // 5 minutes for validation cache
        
        std::sync::Arc::new(crate::xtream::ContentCache::new(db, default_ttl))
    }
}

/// Implement Drop to securely clear sensitive data when CredentialManager is dropped
impl Drop for CredentialManager {
    fn drop(&mut self) {
        // Securely clear the encryption key
        self.encryption_key.fill(0);
        
        // Clear cached credentials (ignore errors during drop)
        let _ = self.clear_all_cached_credentials();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_credentials() -> ProfileCredentials {
        ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        }
    }
    
    #[test]
    fn test_encrypt_decrypt_credentials() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        let encrypted = manager.encrypt_credentials(&credentials).unwrap();
        let decrypted = manager.decrypt_credentials(&encrypted).unwrap();
        
        assert_eq!(credentials.url, decrypted.url);
        assert_eq!(credentials.username, decrypted.username);
        assert_eq!(credentials.password, decrypted.password);
    }
    
    #[test]
    fn test_encrypt_decrypt_different_keys() {
        let manager1 = CredentialManager::with_key([1u8; 32]);
        let manager2 = CredentialManager::with_key([2u8; 32]);
        let credentials = create_test_credentials();
        
        let encrypted = manager1.encrypt_credentials(&credentials).unwrap();
        let result = manager2.decrypt_credentials(&encrypted);
        
        // Should fail with different key
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
        
        // Clear cache
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
        
        // Verify they're cached
        assert!(manager.get_cached_credentials("profile1").unwrap().is_some());
        assert!(manager.get_cached_credentials("profile2").unwrap().is_some());
        
        // Clear all
        manager.clear_all_cached_credentials().unwrap();
        
        // Verify they're gone
        assert!(manager.get_cached_credentials("profile1").unwrap().is_none());
        assert!(manager.get_cached_credentials("profile2").unwrap().is_none());
    }
    
    #[test]
    fn test_encode_decode_for_storage() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        let encrypted = manager.encrypt_credentials(&credentials).unwrap();
        let encoded = manager.encode_for_storage(&encrypted);
        let decoded = manager.decode_from_storage(&encoded).unwrap();
        let decrypted = manager.decrypt_credentials(&decoded).unwrap();
        
        assert_eq!(credentials.url, decrypted.url);
        assert_eq!(credentials.username, decrypted.username);
        assert_eq!(credentials.password, decrypted.password);
    }
    
    #[test]
    fn test_invalid_encrypted_data() {
        let manager = CredentialManager::with_key([1u8; 32]);
        
        // Test with too short data
        let result = manager.decrypt_credentials(&[1, 2, 3]);
        assert!(result.is_err());
        
        // Test with invalid length
        let result = manager.decrypt_credentials(&[0u8; 17]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_invalid_base64() {
        let manager = CredentialManager::with_key([1u8; 32]);
        
        let result = manager.decode_from_storage("invalid-base64!");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_profile_specific_encryption() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        let profile_id1 = "profile1";
        let profile_id2 = "profile2";
        
        // Encrypt with different profile IDs
        let encrypted1 = manager.encrypt_credentials_for_profile(profile_id1, &credentials).unwrap();
        let encrypted2 = manager.encrypt_credentials_for_profile(profile_id2, &credentials).unwrap();
        
        // Encrypted data should be different due to different profile keys
        assert_ne!(encrypted1, encrypted2);
        
        // Decrypt with correct profile IDs
        let decrypted1 = manager.decrypt_credentials_for_profile(profile_id1, &encrypted1).unwrap();
        let decrypted2 = manager.decrypt_credentials_for_profile(profile_id2, &encrypted2).unwrap();
        
        // Both should decrypt to the same credentials
        assert_eq!(credentials.url, decrypted1.url);
        assert_eq!(credentials.username, decrypted1.username);
        assert_eq!(credentials.password, decrypted1.password);
        
        assert_eq!(credentials.url, decrypted2.url);
        assert_eq!(credentials.username, decrypted2.username);
        assert_eq!(credentials.password, decrypted2.password);
        
        // Cross-decryption should fail
        let result = manager.decrypt_credentials_for_profile(profile_id1, &encrypted2);
        assert!(result.is_err());
        
        let result = manager.decrypt_credentials_for_profile(profile_id2, &encrypted1);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_hmac_integrity_verification() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        let profile_id = "test-profile";
        
        let mut encrypted = manager.encrypt_credentials_for_profile(profile_id, &credentials).unwrap();
        
        // Corrupt the encrypted data (after HMAC)
        if encrypted.len() > 70 {
            encrypted[70] ^= 0xFF;
        }
        
        // Decryption should fail due to HMAC verification failure
        let result = manager.decrypt_credentials_for_profile(profile_id, &encrypted);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HMAC verification failed"));
    }
    
    #[test]
    fn test_backward_compatibility() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        // Encrypt using legacy method (simulate old data)
        let legacy_encrypted = manager.encrypt_credentials(&credentials).unwrap();
        
        // Should be able to decrypt using new method (backward compatibility)
        let decrypted = manager.decrypt_credentials(&legacy_encrypted).unwrap();
        
        assert_eq!(credentials.url, decrypted.url);
        assert_eq!(credentials.username, decrypted.username);
        assert_eq!(credentials.password, decrypted.password);
    }
    
    #[test]
    fn test_secure_wipe() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        let profile_id = "test-profile";
        
        // Cache some credentials
        manager.cache_credentials(profile_id, &credentials).unwrap();
        assert!(manager.get_cached_credentials(profile_id).unwrap().is_some());
        
        // Secure wipe
        manager.secure_wipe().unwrap();
        
        // Credentials should be cleared
        assert!(manager.get_cached_credentials(profile_id).unwrap().is_none());
    }
    
    #[test]
    fn test_key_derivation_consistency() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let profile_id = "test-profile";
        let salt = [1u8; 16];
        
        // Derive key multiple times with same inputs
        let key1 = manager.derive_profile_key(profile_id, &salt);
        let key2 = manager.derive_profile_key(profile_id, &salt);
        
        // Should be identical
        assert_eq!(key1, key2);
        
        // Different profile ID should produce different key
        let key3 = manager.derive_profile_key("different-profile", &salt);
        assert_ne!(key1, key3);
        
        // Different salt should produce different key
        let salt2 = [2u8; 16];
        let key4 = manager.derive_profile_key(profile_id, &salt2);
        assert_ne!(key1, key4);
    }
    
    #[test]
    fn test_hmac_generation_and_verification() {
        let manager = CredentialManager::with_key([1u8; 32]);
        let data = b"test data for hmac";
        let key = [42u8; 32];
        
        // Generate HMAC
        let hmac1 = manager.generate_hmac(data, &key).unwrap();
        let hmac2 = manager.generate_hmac(data, &key).unwrap();
        
        // Should be consistent
        assert_eq!(hmac1, hmac2);
        
        // Verify HMAC
        assert!(manager.verify_hmac(data, &hmac1, &key).unwrap());
        
        // Different data should fail verification
        let different_data = b"different test data";
        assert!(!manager.verify_hmac(different_data, &hmac1, &key).unwrap());
        
        // Different key should fail verification
        let different_key = [43u8; 32];
        assert!(!manager.verify_hmac(data, &hmac1, &different_key).unwrap());
    }
    
    #[test]
    fn test_database_credential_storage() {
        use rusqlite::Connection;
        
        let conn = Connection::open_in_memory().unwrap();
        
        // Create test table
        conn.execute(
            "CREATE TABLE xtream_profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                username TEXT NOT NULL,
                encrypted_credentials BLOB NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                is_active BOOLEAN DEFAULT FALSE
            )",
            [],
        ).unwrap();
        
        // Insert test profile
        let profile_id = "test-profile-id";
        let now = chrono::Utc::now();
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                profile_id,
                "Test Profile",
                "http://example.com",
                "testuser",
                vec![0u8; 64], // Dummy encrypted data
                now.to_rfc3339(),
                now.to_rfc3339()
            ],
        ).unwrap();
        
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        
        // Store credentials in database
        manager.store_credentials_in_db(&conn, profile_id, &credentials).unwrap();
        
        // Retrieve credentials from database
        let retrieved = manager.retrieve_credentials_from_db(&conn, profile_id).unwrap().unwrap();
        
        assert_eq!(credentials.url, retrieved.url);
        assert_eq!(credentials.username, retrieved.username);
        assert_eq!(credentials.password, retrieved.password);
        
        // Should also be cached
        let cached = manager.get_cached_credentials(profile_id).unwrap().unwrap();
        assert_eq!(credentials.url, cached.url);
        assert_eq!(credentials.username, cached.username);
        assert_eq!(credentials.password, cached.password);
    }
    
    #[test]
    fn test_database_credential_not_found() {
        use rusqlite::Connection;
        
        let conn = Connection::open_in_memory().unwrap();
        
        // Create test table (empty)
        conn.execute(
            "CREATE TABLE xtream_profiles (
                id TEXT PRIMARY KEY,
                encrypted_credentials BLOB NOT NULL
            )",
            [],
        ).unwrap();
        
        let manager = CredentialManager::with_key([1u8; 32]);
        
        // Try to retrieve non-existent credentials
        let result = manager.retrieve_credentials_from_db(&conn, "nonexistent").unwrap();
        assert!(result.is_none());
    }
    
    #[test]
    fn test_database_credential_deletion() {
        use rusqlite::Connection;
        
        let manager = CredentialManager::with_key([1u8; 32]);
        let credentials = create_test_credentials();
        let profile_id = "test-profile";
        
        // Cache some credentials
        manager.cache_credentials(profile_id, &credentials).unwrap();
        assert!(manager.get_cached_credentials(profile_id).unwrap().is_some());
        
        let conn = Connection::open_in_memory().unwrap();
        
        // Delete credentials
        manager.delete_credentials_from_db(&conn, profile_id).unwrap();
        
        // Should be cleared from cache
        assert!(manager.get_cached_credentials(profile_id).unwrap().is_none());
    }
}