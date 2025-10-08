use crate::error::{Result, XTauriError};
use crate::xtream::types::{XtreamProfile, CreateProfileRequest, UpdateProfileRequest, ProfileCredentials, AuthenticationResult, AuthenticationErrorType};
use crate::xtream::credential_manager::CredentialManager;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Manages Xtream profiles including CRUD operations and credential handling
pub struct ProfileManager {
    db: Arc<Mutex<Connection>>,
    credential_manager: Arc<CredentialManager>,
}

impl ProfileManager {
    /// Create a new profile manager
    pub fn new(db: Arc<Mutex<Connection>>, credential_manager: Arc<CredentialManager>) -> Self {
        Self {
            db,
            credential_manager,
        }
    }
    
    /// Create a new profile (synchronous version for backward compatibility)
    pub fn create_profile(&self, request: CreateProfileRequest) -> Result<String> {
        // Use tokio runtime to run async validation
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| XTauriError::internal(format!("Failed to create async runtime: {}", e)))?;
        
        rt.block_on(self.create_profile_async(request))
    }
    
    /// Create a new profile without credential validation (for testing)
    #[cfg(test)]
    pub fn create_profile_without_validation(&self, request: CreateProfileRequest) -> Result<String> {
        // Validate the request
        self.validate_create_request(&request)?;
        
        // Check if profile name already exists
        if self.profile_name_exists(&request.name)? {
            return Err(XTauriError::profile_validation(format!("Profile name '{}' already exists", request.name)));
        }
        
        let profile_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Create credentials object
        let credentials = ProfileCredentials {
            url: request.url.clone(),
            username: request.username.clone(),
            password: request.password,
        };
        
        // Encrypt credentials
        let encrypted_credentials = self.credential_manager.encrypt_credentials(&credentials)?;
        let encoded_credentials = self.credential_manager.encode_for_storage(&encrypted_credentials);
        
        // Insert profile into database
        let now_str = now.to_rfc3339();
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        db.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials, created_at, updated_at, is_active) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                &profile_id,
                &request.name,
                &request.url,
                &request.username,
                &encoded_credentials,
                &now_str,
                &now_str,
                false,
            ),
        )?;
        
        // Cache the credentials
        self.credential_manager.cache_credentials(&profile_id, &credentials)?;
        
        Ok(profile_id)
    }
    
    /// Create a new profile with async credential validation
    pub async fn create_profile_async(&self, request: CreateProfileRequest) -> Result<String> {
        // Validate the request
        self.validate_create_request(&request)?;
        
        // Check if profile name already exists
        if self.profile_name_exists(&request.name)? {
            return Err(XTauriError::profile_validation(format!("Profile name '{}' already exists", request.name)));
        }
        
        // Validate credentials against Xtream server
        let credentials = ProfileCredentials {
            url: request.url.clone(),
            username: request.username.clone(),
            password: request.password.clone(),
        };
        
        if !self.validate_credentials(&credentials).await? {
            return Err(XTauriError::XtreamInvalidCredentials);
        }
        
        let profile_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Create credentials object
        let credentials = ProfileCredentials {
            url: request.url.clone(),
            username: request.username.clone(),
            password: request.password,
        };
        
        // Encrypt credentials
        let encrypted_credentials = self.credential_manager.encrypt_credentials(&credentials)?;
        let encoded_credentials = self.credential_manager.encode_for_storage(&encrypted_credentials);
        
        // Insert profile into database
        let now_str = now.to_rfc3339();
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        db.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials, created_at, updated_at, is_active) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                &profile_id,
                &request.name,
                &request.url,
                &request.username,
                &encoded_credentials,
                &now_str,
                &now_str,
                false,
            ),
        )?;
        
        // Cache the credentials
        self.credential_manager.cache_credentials(&profile_id, &credentials)?;
        
        Ok(profile_id)
    }
    
    /// Update an existing profile
    pub fn update_profile(&self, id: &str, request: UpdateProfileRequest) -> Result<()> {
        // Check if profile exists
        let existing_profile = self.get_profile(id)?
            .ok_or_else(|| XTauriError::xtream_profile_not_found(id.to_string()))?;
        
        // Validate the request
        self.validate_update_request(&request)?;
        
        // Check if new name conflicts with existing profiles (if name is being changed)
        if let Some(ref new_name) = request.name {
            if new_name != &existing_profile.name && self.profile_name_exists(new_name)? {
                return Err(XTauriError::profile_validation(format!("Profile name '{}' already exists", new_name)));
            }
        }
        
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        
        // Handle credential updates
        let encoded_credentials = if request.url.is_some() || request.username.is_some() || request.password.is_some() {
            // Get current credentials
            let current_credentials = self.get_profile_credentials(id)?;
            
            // Create updated credentials
            let updated_credentials = ProfileCredentials {
                url: request.url.clone().unwrap_or(current_credentials.url),
                username: request.username.clone().unwrap_or(current_credentials.username),
                password: request.password.clone().unwrap_or(current_credentials.password),
            };
            
            // Encrypt and encode new credentials
            let encrypted = self.credential_manager.encrypt_credentials(&updated_credentials)?;
            let encoded = self.credential_manager.encode_for_storage(&encrypted);
            
            // Update cached credentials
            self.credential_manager.cache_credentials(id, &updated_credentials)?;
            
            Some(encoded)
        } else {
            None
        };
        
        // Update profile in database
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        // Build update query based on what fields are being updated
        if let Some(name) = &request.name {
            db.execute(
                "UPDATE xtream_profiles SET name = ?, updated_at = ? WHERE id = ?",
                (name, &now_str, id),
            )?;
        }
        
        if let Some(url) = &request.url {
            db.execute(
                "UPDATE xtream_profiles SET url = ?, updated_at = ? WHERE id = ?",
                (url, &now_str, id),
            )?;
        }
        
        if let Some(username) = &request.username {
            db.execute(
                "UPDATE xtream_profiles SET username = ?, updated_at = ? WHERE id = ?",
                (username, &now_str, id),
            )?;
        }
        
        if let Some(encoded_creds) = &encoded_credentials {
            db.execute(
                "UPDATE xtream_profiles SET encrypted_credentials = ?, updated_at = ? WHERE id = ?",
                (encoded_creds, &now_str, id),
            )?;
        }
        
        // Always update the timestamp
        db.execute(
            "UPDATE xtream_profiles SET updated_at = ? WHERE id = ?",
            (&now_str, id),
        )?;
        
        Ok(())
    }
    
    /// Delete a profile
    pub fn delete_profile(&self, id: &str) -> Result<()> {
        // Check if profile exists
        if self.get_profile(id)?.is_none() {
            return Err(XTauriError::xtream_profile_not_found(id.to_string()));
        }
        
        // Clear cached credentials
        self.credential_manager.clear_cached_credentials(id)?;
        
        // Delete from database (cascade will handle related data)
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        db.execute("DELETE FROM xtream_profiles WHERE id = ?", [id])?;
        
        Ok(())
    }
    
    /// Get all profiles
    pub fn get_profiles(&self) -> Result<Vec<XtreamProfile>> {
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let mut stmt = db.prepare(
            "SELECT id, name, url, username, created_at, updated_at, last_used, is_active 
             FROM xtream_profiles ORDER BY name"
        )?;
        
        let profile_iter = stmt.query_map([], |row| {
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            let last_used_str: Option<String> = row.get(6)?;
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let last_used = if let Some(last_used_str) = last_used_str {
                Some(DateTime::parse_from_rfc3339(&last_used_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(6, "last_used".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc))
            } else {
                None
            };
            
            Ok(XtreamProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                created_at,
                updated_at,
                last_used,
                is_active: row.get(7)?,
            })
        })?;
        
        let mut profiles = Vec::new();
        for profile in profile_iter {
            profiles.push(profile?);
        }
        
        Ok(profiles)
    }
    
    /// Get a specific profile by ID
    pub fn get_profile(&self, id: &str) -> Result<Option<XtreamProfile>> {
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let mut stmt = db.prepare(
            "SELECT id, name, url, username, created_at, updated_at, last_used, is_active 
             FROM xtream_profiles WHERE id = ?"
        )?;
        
        let result = stmt.query_row([id], |row| {
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            let last_used_str: Option<String> = row.get(6)?;
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let last_used = if let Some(last_used_str) = last_used_str {
                Some(DateTime::parse_from_rfc3339(&last_used_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(6, "last_used".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc))
            } else {
                None
            };
            
            Ok(XtreamProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                created_at,
                updated_at,
                last_used,
                is_active: row.get(7)?,
            })
        });
        
        match result {
            Ok(profile) => Ok(Some(profile)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(XTauriError::Database(e)),
        }
    }
    
    /// Get profile credentials (decrypted)
    pub fn get_profile_credentials(&self, id: &str) -> Result<ProfileCredentials> {
        // First check cache
        if let Some(cached) = self.credential_manager.get_cached_credentials(id)? {
            return Ok(cached);
        }
        
        // Get from database
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let mut stmt = db.prepare("SELECT encrypted_credentials FROM xtream_profiles WHERE id = ?")?;
        let result = stmt.query_row([id], |row| {
            let encoded: String = row.get(0)?;
            Ok(encoded)
        });
        
        let encoded_credentials = match result {
            Ok(encoded) => encoded,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(XTauriError::xtream_profile_not_found(id.to_string())),
            Err(e) => return Err(XTauriError::Database(e)),
        };
        
        // Decrypt credentials
        let encrypted_data = self.credential_manager.decode_from_storage(&encoded_credentials)?;
        let credentials = self.credential_manager.decrypt_credentials(&encrypted_data)?;
        
        // Cache for future use
        self.credential_manager.cache_credentials(id, &credentials)?;
        
        Ok(credentials)
    }
    
    /// Set profile as active (and deactivate others)
    pub fn set_active_profile(&self, id: &str) -> Result<()> {
        // Check if profile exists
        if self.get_profile(id)?.is_none() {
            return Err(XTauriError::xtream_profile_not_found(id.to_string()));
        }
        
        let mut db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        let now_str = Utc::now().to_rfc3339();
        
        let tx = db.transaction()?;
        
        // Deactivate all profiles
        tx.execute("UPDATE xtream_profiles SET is_active = FALSE", [])?;
        
        // Activate the specified profile and update last_used
        tx.execute(
            "UPDATE xtream_profiles SET is_active = TRUE, last_used = ? WHERE id = ?",
            (&now_str, id),
        )?;
        
        tx.commit()?;
        
        Ok(())
    }
    
    /// Test authentication with Xtream server without creating a profile
    pub async fn test_authentication(&self, credentials: &ProfileCredentials) -> Result<bool> {
        self.validate_credentials(credentials).await
    }
    
    /// Test authentication and return detailed result information
    pub async fn test_authentication_detailed(&self, credentials: &ProfileCredentials) -> Result<AuthenticationResult> {
        // First validate the format
        if let Err(e) = self.validate_credentials_format(credentials) {
            return Ok(AuthenticationResult {
                success: false,
                error_message: Some(e.to_string()),
                error_type: AuthenticationErrorType::ValidationError,
                server_info: None,
            });
        }
        
        // Create a temporary Xtream client to test authentication with shorter timeout
        let timeout = std::time::Duration::from_secs(5);
        let client = match crate::xtream::XtreamClient::new_with_timeout(credentials.clone(), self.credential_manager.get_cache(), timeout) {
            Ok(client) => client,
            Err(e) => {
                return Ok(AuthenticationResult {
                    success: false,
                    error_message: Some(e.to_string()),
                    error_type: AuthenticationErrorType::ClientError,
                    server_info: None,
                });
            }
        };
        
        // Attempt authentication with retry
        match client.authenticate_with_retry(2).await {
            Ok(server_info) => {
                Ok(AuthenticationResult {
                    success: true,
                    error_message: None,
                    error_type: AuthenticationErrorType::None,
                    server_info: Some(server_info),
                })
            }
            Err(e) => {
                let error_type = match &e {
                    XTauriError::XtreamInvalidCredentials => AuthenticationErrorType::InvalidCredentials,
                    XTauriError::XtreamAuthenticationFailed { .. } => AuthenticationErrorType::AuthenticationFailed,
                    XTauriError::XtreamApiError { status, .. } => {
                        if *status >= 500 {
                            AuthenticationErrorType::ServerError
                        } else {
                            AuthenticationErrorType::ClientError
                        }
                    }
                    XTauriError::Network(_) => AuthenticationErrorType::NetworkError,
                    XTauriError::Timeout { .. } => AuthenticationErrorType::TimeoutError,
                    _ => AuthenticationErrorType::UnknownError,
                };
                
                Ok(AuthenticationResult {
                    success: false,
                    error_message: Some(e.user_message()),
                    error_type,
                    server_info: None,
                })
            }
        }
    }
    
    /// Update profile with async credential validation (if credentials are being updated)
    pub async fn update_profile_async(&self, id: &str, request: UpdateProfileRequest) -> Result<()> {
        // Check if profile exists
        let existing_profile = self.get_profile(id)?
            .ok_or_else(|| XTauriError::xtream_profile_not_found(id.to_string()))?;
        
        // Validate the request
        self.validate_update_request(&request)?;
        
        // Check if new name conflicts with existing profiles (if name is being changed)
        if let Some(ref new_name) = request.name {
            if new_name != &existing_profile.name && self.profile_name_exists(new_name)? {
                return Err(XTauriError::profile_validation(format!("Profile name '{}' already exists", new_name)));
            }
        }
        
        // If credentials are being updated, validate them
        if request.url.is_some() || request.username.is_some() || request.password.is_some() {
            // Get current credentials
            let current_credentials = self.get_profile_credentials(id)?;
            
            // Create updated credentials
            let updated_credentials = ProfileCredentials {
                url: request.url.clone().unwrap_or(current_credentials.url),
                username: request.username.clone().unwrap_or(current_credentials.username),
                password: request.password.clone().unwrap_or(current_credentials.password),
            };
            
            // Validate new credentials
            if !self.validate_credentials(&updated_credentials).await? {
                return Err(XTauriError::XtreamInvalidCredentials);
            }
        }
        
        // Proceed with the update using the synchronous method
        self.update_profile(id, request)
    }
    
    /// Get the currently active profile
    pub fn get_active_profile(&self) -> Result<Option<XtreamProfile>> {
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let mut stmt = db.prepare(
            "SELECT id, name, url, username, created_at, updated_at, last_used, is_active 
             FROM xtream_profiles WHERE is_active = TRUE LIMIT 1"
        )?;
        
        let result = stmt.query_row([], |row| {
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            let last_used_str: Option<String> = row.get(6)?;
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let last_used = if let Some(last_used_str) = last_used_str {
                Some(DateTime::parse_from_rfc3339(&last_used_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(6, "last_used".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc))
            } else {
                None
            };
            
            Ok(XtreamProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                created_at,
                updated_at,
                last_used,
                is_active: row.get(7)?,
            })
        });
        
        match result {
            Ok(profile) => Ok(Some(profile)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(XTauriError::Database(e)),
        }
    }
    
    /// Validate profile credentials by attempting authentication with Xtream server
    pub async fn validate_credentials(&self, credentials: &ProfileCredentials) -> Result<bool> {
        // First validate the format
        self.validate_credentials_format(credentials)?;
        
        // Create a temporary Xtream client to test authentication with shorter timeout
        let timeout = std::time::Duration::from_secs(5);
        let client = crate::xtream::XtreamClient::new_with_timeout(credentials.clone(), self.credential_manager.get_cache(), timeout)?;
        
        // Attempt authentication
        match client.authenticate().await {
            Ok(_) => Ok(true),
            Err(XTauriError::XtreamInvalidCredentials) => Ok(false),
            Err(XTauriError::XtreamAuthenticationFailed { .. }) => Ok(false),
            Err(XTauriError::XtreamApiError { .. }) => Ok(false),
            Err(e) => Err(e), // Network or other errors should be propagated
        }
    }
    
    /// Check if a profile name already exists
    fn profile_name_exists(&self, name: &str) -> Result<bool> {
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let count: i64 = db.query_row(
            "SELECT COUNT(*) FROM xtream_profiles WHERE name = ?",
            [name],
            |row| row.get(0),
        )?;
        
        Ok(count > 0)
    }
    
    /// Validate create profile request
    fn validate_create_request(&self, request: &CreateProfileRequest) -> Result<()> {
        if request.name.trim().is_empty() {
            return Err(XTauriError::profile_validation("Profile name cannot be empty".to_string()));
        }
        
        if request.name.len() > 100 {
            return Err(XTauriError::profile_validation("Profile name cannot exceed 100 characters".to_string()));
        }
        
        let credentials = ProfileCredentials {
            url: request.url.clone(),
            username: request.username.clone(),
            password: request.password.clone(),
        };
        
        self.validate_credentials_format(&credentials)?;
        
        Ok(())
    }
    
    /// Validate update profile request
    fn validate_update_request(&self, request: &UpdateProfileRequest) -> Result<()> {
        if let Some(ref name) = request.name {
            if name.trim().is_empty() {
                return Err(XTauriError::profile_validation("Profile name cannot be empty".to_string()));
            }
            
            if name.len() > 100 {
                return Err(XTauriError::profile_validation("Profile name cannot exceed 100 characters".to_string()));
            }
        }
        
        // If any credential fields are provided, validate them
        if request.url.is_some() || request.username.is_some() || request.password.is_some() {
            // We can't fully validate without all fields, but we can check basic format
            if let Some(ref url) = request.url {
                self.validate_url_format(url)?;
            }
            
            if let Some(ref username) = request.username {
                if username.trim().is_empty() {
                    return Err(XTauriError::profile_validation("Username cannot be empty".to_string()));
                }
            }
            
            if let Some(ref password) = request.password {
                if password.is_empty() {
                    return Err(XTauriError::profile_validation("Password cannot be empty".to_string()));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate credentials format
    fn validate_credentials_format(&self, credentials: &ProfileCredentials) -> Result<()> {
        self.validate_url_format(&credentials.url)?;
        
        if credentials.username.trim().is_empty() {
            return Err(XTauriError::profile_validation("Username cannot be empty".to_string()));
        }
        
        if credentials.password.is_empty() {
            return Err(XTauriError::profile_validation("Password cannot be empty".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate URL format
    fn validate_url_format(&self, url: &str) -> Result<()> {
        use url::Url;
        
        let parsed_url = Url::parse(url)
            .map_err(|_| XTauriError::profile_validation("Invalid URL format".to_string()))?;
        
        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(XTauriError::profile_validation("URL must use http or https scheme".to_string()));
        }
        
        if parsed_url.host().is_none() {
            return Err(XTauriError::profile_validation("URL must have a valid host".to_string()));
        }
        
        Ok(())
    }
    
    /// Update the last used timestamp for a profile
    pub async fn update_last_used(&self, id: &str) -> Result<()> {
        // Check if profile exists
        if self.get_profile(id)?.is_none() {
            return Err(XTauriError::xtream_profile_not_found(id.to_string()));
        }
        
        let now_str = Utc::now().to_rfc3339();
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        db.execute(
            "UPDATE xtream_profiles SET last_used = ? WHERE id = ?",
            (&now_str, id),
        )?;
        
        Ok(())
    }
    
    /// Async versions of CRUD operations for better integration with Tauri commands
    pub async fn create_profile_async_wrapper(&self, request: CreateProfileRequest) -> Result<String> {
        self.create_profile_async(request).await
    }
    
    pub async fn update_profile_async_wrapper(&self, id: &str, request: UpdateProfileRequest) -> Result<()> {
        self.update_profile_async(id, request).await
    }
    
    pub async fn delete_profile_async_wrapper(&self, id: &str) -> Result<()> {
        // Use tokio task to run the synchronous delete in a blocking context
        let db = Arc::clone(&self.db);
        let credential_manager = Arc::clone(&self.credential_manager);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            Self::delete_profile_sync_static(&db, &credential_manager, &id)
        }).await.map_err(|e| XTauriError::internal(format!("Task join error: {}", e)))?
    }
    
    pub async fn get_profiles_async_wrapper(&self) -> Result<Vec<XtreamProfile>> {
        // Use tokio task to run the synchronous get in a blocking context
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || {
            Self::get_profiles_sync_static(&db)
        }).await.map_err(|e| XTauriError::internal(format!("Task join error: {}", e)))?
    }
    
    pub async fn get_profile_async_wrapper(&self, id: &str) -> Result<Option<XtreamProfile>> {
        // Use tokio task to run the synchronous get in a blocking context
        let db = Arc::clone(&self.db);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            Self::get_profile_sync_static(&db, &id)
        }).await.map_err(|e| XTauriError::internal(format!("Task join error: {}", e)))?
    }
    
    pub async fn get_profile_credentials_async_wrapper(&self, id: &str) -> Result<ProfileCredentials> {
        // Use tokio task to run the synchronous get in a blocking context
        let db = Arc::clone(&self.db);
        let credential_manager = Arc::clone(&self.credential_manager);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            Self::get_profile_credentials_sync_static(&db, &credential_manager, &id)
        }).await.map_err(|e| XTauriError::internal(format!("Task join error: {}", e)))?
    }
    
    /// Static synchronous versions for use in async contexts
    fn delete_profile_sync_static(
        db: &Arc<Mutex<Connection>>, 
        credential_manager: &Arc<CredentialManager>, 
        id: &str
    ) -> Result<()> {
        // Check if profile exists
        if Self::get_profile_sync_static(db, id)?.is_none() {
            return Err(XTauriError::xtream_profile_not_found(id.to_string()));
        }
        
        // Clear cached credentials
        credential_manager.clear_cached_credentials(id)?;
        
        // Delete from database (cascade will handle related data)
        let db_conn = db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        db_conn.execute("DELETE FROM xtream_profiles WHERE id = ?", [id])?;
        
        Ok(())
    }
    
    fn get_profiles_sync_static(db: &Arc<Mutex<Connection>>) -> Result<Vec<XtreamProfile>> {
        let db_conn = db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let mut stmt = db_conn.prepare(
            "SELECT id, name, url, username, created_at, updated_at, last_used, is_active 
             FROM xtream_profiles ORDER BY name"
        )?;
        
        let profile_iter = stmt.query_map([], |row| {
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            let last_used_str: Option<String> = row.get(6)?;
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let last_used = if let Some(last_used_str) = last_used_str {
                Some(DateTime::parse_from_rfc3339(&last_used_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(6, "last_used".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc))
            } else {
                None
            };
            
            Ok(XtreamProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                created_at,
                updated_at,
                last_used,
                is_active: row.get(7)?,
            })
        })?;
        
        let mut profiles = Vec::new();
        for profile in profile_iter {
            profiles.push(profile?);
        }
        
        Ok(profiles)
    }
    
    fn get_profile_sync_static(db: &Arc<Mutex<Connection>>, id: &str) -> Result<Option<XtreamProfile>> {
        let db_conn = db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let mut stmt = db_conn.prepare(
            "SELECT id, name, url, username, created_at, updated_at, last_used, is_active 
             FROM xtream_profiles WHERE id = ?"
        )?;
        
        let result = stmt.query_row([id], |row| {
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            let last_used_str: Option<String> = row.get(6)?;
            
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            let last_used = if let Some(last_used_str) = last_used_str {
                Some(DateTime::parse_from_rfc3339(&last_used_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(6, "last_used".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc))
            } else {
                None
            };
            
            Ok(XtreamProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                created_at,
                updated_at,
                last_used,
                is_active: row.get(7)?,
            })
        });
        
        match result {
            Ok(profile) => Ok(Some(profile)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(XTauriError::Database(e)),
        }
    }
    
    fn get_profile_credentials_sync_static(
        db: &Arc<Mutex<Connection>>, 
        credential_manager: &Arc<CredentialManager>, 
        id: &str
    ) -> Result<ProfileCredentials> {
        // First check cache
        if let Some(cached) = credential_manager.get_cached_credentials(id)? {
            return Ok(cached);
        }
        
        // Get from database
        let db_conn = db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let mut stmt = db_conn.prepare("SELECT encrypted_credentials FROM xtream_profiles WHERE id = ?")?;
        let result = stmt.query_row([id], |row| {
            let encoded: String = row.get(0)?;
            Ok(encoded)
        });
        
        let encoded_credentials = match result {
            Ok(encoded) => encoded,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(XTauriError::xtream_profile_not_found(id.to_string())),
            Err(e) => return Err(XTauriError::Database(e)),
        };
        
        // Decrypt credentials
        let encrypted_data = credential_manager.decode_from_storage(&encoded_credentials)?;
        let credentials = credential_manager.decrypt_credentials(&encrypted_data)?;
        
        // Cache for future use
        credential_manager.cache_credentials(id, &credentials)?;
        
        Ok(credentials)
    }

    /// Get playback history for a profile
    pub async fn get_playback_history(&self, profile_id: &str) -> Result<serde_json::Value> {
        let db = self.db.lock().unwrap();
        
        let mut stmt = db.prepare(
            "SELECT id, content_type, content_id, content_data, watched_at, position, duration 
             FROM xtream_history 
             WHERE profile_id = ? 
             ORDER BY watched_at DESC 
             LIMIT 100"
        )?;
        
        let history_iter = stmt.query_map([profile_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "profileId": profile_id,
                "contentType": row.get::<_, String>(1)?,
                "contentId": row.get::<_, String>(2)?,
                "contentData": serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(3)?).unwrap_or(serde_json::Value::Null),
                "watchedAt": row.get::<_, String>(4)?,
                "position": row.get::<_, Option<f64>>(5)?,
                "duration": row.get::<_, Option<f64>>(6)?
            }))
        })?;
        
        let mut history = Vec::new();
        for item in history_iter {
            history.push(item?);
        }
        
        Ok(serde_json::Value::Array(history))
    }

    /// Add content to playback history
    pub async fn add_to_playback_history(
        &self,
        profile_id: &str,
        content_type: &str,
        content_id: &str,
        content_data: &serde_json::Value,
        position: Option<f64>,
        duration: Option<f64>,
    ) -> Result<()> {
        let history_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let content_data_str = serde_json::to_string(content_data)
            .map_err(|e| XTauriError::internal(format!("Failed to serialize content data: {}", e)))?;
        
        let db = self.db.lock().unwrap();
        // Insert or update existing history entry
        db.execute(
            "INSERT OR REPLACE INTO xtream_history 
             (id, profile_id, content_type, content_id, content_data, watched_at, position, duration)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                history_id,
                profile_id,
                content_type,
                content_id,
                content_data_str,
                now,
                position,
                duration
            ],
        )?;
        
        Ok(())
    }

    /// Update playback position for resume functionality
    pub async fn update_playback_position(
        &self,
        profile_id: &str,
        content_type: &str,
        content_id: &str,
        position: f64,
        duration: Option<f64>,
    ) -> Result<()> {
        let rows_affected = {
            let db = self.db.lock().unwrap();
            db.execute(
                "UPDATE xtream_history 
                 SET position = ?1, duration = COALESCE(?2, duration), watched_at = ?3
                 WHERE profile_id = ?4 AND content_type = ?5 AND content_id = ?6",
                rusqlite::params![
                    position,
                    duration,
                    Utc::now().to_rfc3339(),
                    profile_id,
                    content_type,
                    content_id
                ],
            )?
        };
        
        // If no existing entry was updated, create a new one
        if rows_affected == 0 {
            self.add_to_playback_history(profile_id, content_type, content_id, &serde_json::Value::Null, Some(position), duration).await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    
    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        
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
        
        conn
    }
    
    fn create_test_request() -> CreateProfileRequest {
        CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        }
    }
    
    #[test]
    fn test_create_profile() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        let request = create_test_request();
        
        // For testing, we'll use the synchronous method that skips validation
        // In a real scenario, you'd mock the XtreamClient or use a test server
        let profile_id = manager.create_profile_without_validation(request.clone()).unwrap();
        
        // Verify profile was created
        let profile = manager.get_profile(&profile_id).unwrap().unwrap();
        assert_eq!(profile.name, request.name);
        assert_eq!(profile.url, request.url);
        assert_eq!(profile.username, request.username);
        assert!(!profile.is_active);
        
        // Verify credentials were stored and can be retrieved
        let credentials = manager.get_profile_credentials(&profile_id).unwrap();
        assert_eq!(credentials.url, request.url);
        assert_eq!(credentials.username, request.username);
        assert_eq!(credentials.password, request.password);
    }
    
    #[test]
    fn test_create_duplicate_name() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        let request = create_test_request();
        
        // Create first profile
        manager.create_profile_without_validation(request.clone()).unwrap();
        
        // Try to create second profile with same name
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_update_profile() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        let request = create_test_request();
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        // Update profile
        let update_request = UpdateProfileRequest {
            name: Some("Updated Profile".to_string()),
            url: Some("https://newserver.com:8080".to_string()),
            username: Some("newuser".to_string()),
            password: Some("newpass456".to_string()),
        };
        
        manager.update_profile(&profile_id, update_request.clone()).unwrap();
        
        // Verify updates
        let profile = manager.get_profile(&profile_id).unwrap().unwrap();
        assert_eq!(profile.name, update_request.name.unwrap());
        assert_eq!(profile.url, update_request.url.unwrap());
        assert_eq!(profile.username, update_request.username.unwrap());
        
        let credentials = manager.get_profile_credentials(&profile_id).unwrap();
        assert_eq!(credentials.password, update_request.password.unwrap());
    }
    
    #[test]
    fn test_delete_profile() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        let request = create_test_request();
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        // Verify profile exists
        assert!(manager.get_profile(&profile_id).unwrap().is_some());
        
        // Delete profile
        manager.delete_profile(&profile_id).unwrap();
        
        // Verify profile is gone
        assert!(manager.get_profile(&profile_id).unwrap().is_none());
    }
    
    #[test]
    fn test_set_active_profile() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        let request1 = create_test_request();
        let mut request2 = create_test_request();
        request2.name = "Profile 2".to_string();
        
        let profile_id1 = manager.create_profile_without_validation(request1).unwrap();
        let profile_id2 = manager.create_profile_without_validation(request2).unwrap();
        
        // Set first profile as active
        manager.set_active_profile(&profile_id1).unwrap();
        
        let active = manager.get_active_profile().unwrap().unwrap();
        assert_eq!(active.id, profile_id1);
        assert!(active.is_active);
        
        // Set second profile as active
        manager.set_active_profile(&profile_id2).unwrap();
        
        let active = manager.get_active_profile().unwrap().unwrap();
        assert_eq!(active.id, profile_id2);
        
        // First profile should no longer be active
        let profile1 = manager.get_profile(&profile_id1).unwrap().unwrap();
        assert!(!profile1.is_active);
    }
    
    #[test]
    fn test_get_profiles() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        // Initially empty
        let profiles = manager.get_profiles().unwrap();
        assert_eq!(profiles.len(), 0);
        
        // Create some profiles
        let request1 = create_test_request();
        let mut request2 = create_test_request();
        request2.name = "Profile 2".to_string();
        
        manager.create_profile_without_validation(request1).unwrap();
        manager.create_profile_without_validation(request2).unwrap();
        
        // Should have 2 profiles
        let profiles = manager.get_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
    }
    
    #[tokio::test]
    async fn test_credential_validation() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        let credentials = ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        };
        
        // This will fail because it's not a real Xtream server
        // Use timeout to prevent test from hanging
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            manager.validate_credentials(&credentials)
        ).await;
        
        match result {
            Ok(validation_result) => {
                // We expect this to fail with a network error or authentication failure
                assert!(validation_result.is_err() || !validation_result.unwrap());
            }
            Err(_) => {
                // Timeout occurred, which is acceptable for this test
            }
        }
        
        // Test with invalid URL format (should fail quickly)
        let invalid_credentials = ProfileCredentials {
            url: "not-a-url".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        };
        
        let result = manager.validate_credentials(&invalid_credentials).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_detailed_authentication() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        // Test with invalid URL format
        let invalid_credentials = ProfileCredentials {
            url: "not-a-url".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        };
        
        let result = manager.test_authentication_detailed(&invalid_credentials).await.unwrap();
        assert!(!result.success);
        assert!(matches!(result.error_type, AuthenticationErrorType::ValidationError));
        assert!(result.error_message.is_some());
        
        // Test with valid format but non-existent server (with timeout)
        let credentials = ProfileCredentials {
            url: "http://nonexistent.example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass123".to_string(),
        };
        
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            manager.test_authentication_detailed(&credentials)
        ).await;
        
        match result {
            Ok(auth_result) => {
                let auth_result = auth_result.unwrap();
                assert!(!auth_result.success);
                // Should be either network error or timeout
                assert!(matches!(auth_result.error_type, AuthenticationErrorType::NetworkError | AuthenticationErrorType::TimeoutError | AuthenticationErrorType::AuthenticationFailed));
            }
            Err(_) => {
                // Timeout occurred, which is acceptable for this test
            }
        }
    }
    
    #[tokio::test]
    async fn test_async_profile_creation() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        let request = create_test_request();
        
        // This should fail because the credentials can't be validated against a real server
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            manager.create_profile_async(request)
        ).await;
        
        match result {
            Ok(create_result) => {
                assert!(create_result.is_err());
                // The error should be related to invalid credentials or network failure
                let error = create_result.unwrap_err();
                assert!(matches!(error, XTauriError::XtreamInvalidCredentials | XTauriError::XtreamAuthenticationFailed { .. } | XTauriError::Network(_) | XTauriError::Timeout { .. }));
            }
            Err(_) => {
                // Timeout occurred, which is acceptable for this test
            }
        }
    }
    
    #[tokio::test]
    async fn test_async_profile_update() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        // Create a profile without validation first
        let request = create_test_request();
        let profile_id = manager.create_profile_without_validation(request).unwrap();
        
        // Try to update with new credentials (should fail validation)
        let update_request = UpdateProfileRequest {
            name: None,
            url: Some("http://newserver.example.com:8080".to_string()),
            username: Some("newuser".to_string()),
            password: Some("newpass".to_string()),
        };
        
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            manager.update_profile_async(&profile_id, update_request)
        ).await;
        
        match result {
            Ok(update_result) => {
                assert!(update_result.is_err());
            }
            Err(_) => {
                // Timeout occurred, which is acceptable for this test
            }
        }
        
        // Update without changing credentials should work
        let update_request = UpdateProfileRequest {
            name: Some("Updated Profile Name".to_string()),
            url: None,
            username: None,
            password: None,
        };
        
        let result = manager.update_profile_async(&profile_id, update_request).await;
        assert!(result.is_ok());
        
        // Verify the name was updated
        let profile = manager.get_profile(&profile_id).unwrap().unwrap();
        assert_eq!(profile.name, "Updated Profile Name");
    }
    
    #[test]
    fn test_validation() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let credential_manager = Arc::new(CredentialManager::new().unwrap());
        let manager = ProfileManager::new(db, credential_manager);
        
        // Test empty name
        let mut request = create_test_request();
        request.name = "".to_string();
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
        
        // Test invalid URL
        let mut request = create_test_request();
        request.url = "not-a-url".to_string();
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
        
        // Test empty username
        let mut request = create_test_request();
        request.username = "".to_string();
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
        
        // Test empty password
        let mut request = create_test_request();
        request.password = "".to_string();
        let result = manager.create_profile_without_validation(request);
        assert!(result.is_err());
    }
}