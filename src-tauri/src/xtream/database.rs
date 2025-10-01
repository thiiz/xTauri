use crate::error::{Result, TolloError};
use crate::xtream::types::{XtreamProfile, CreateProfileRequest, UpdateProfileRequest};
use rusqlite::{Connection, params};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Database operations for Xtream profiles
pub struct XtreamDatabase;

impl XtreamDatabase {
    /// Create a new Xtream profile in the database
    pub fn create_profile(
        conn: &Connection,
        request: &CreateProfileRequest,
        encrypted_credentials: &[u8],
    ) -> Result<String> {
        let profile_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials, created_at, updated_at, is_active) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                profile_id,
                request.name,
                request.url,
                request.username,
                encrypted_credentials,
                now.to_rfc3339(),
                now.to_rfc3339(),
                false
            ],
        ).map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TolloError::profile_validation(format!("Profile name '{}' already exists", request.name))
            } else {
                TolloError::Database(e)
            }
        })?;
        
        Ok(profile_id)
    }
    
    /// Update an existing Xtream profile
    pub fn update_profile(
        conn: &Connection,
        profile_id: &str,
        request: &UpdateProfileRequest,
        encrypted_credentials: Option<&[u8]>,
    ) -> Result<()> {
        let now = Utc::now();
        
        // Build dynamic query based on what fields are being updated
        let mut query_parts = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        if let Some(name) = &request.name {
            query_parts.push("name = ?");
            params.push(Box::new(name.clone()));
        }
        
        if let Some(url) = &request.url {
            query_parts.push("url = ?");
            params.push(Box::new(url.clone()));
        }
        
        if let Some(username) = &request.username {
            query_parts.push("username = ?");
            params.push(Box::new(username.clone()));
        }
        
        if let Some(credentials) = encrypted_credentials {
            query_parts.push("encrypted_credentials = ?");
            params.push(Box::new(credentials.to_vec()));
        }
        
        if query_parts.is_empty() {
            return Ok(()); // Nothing to update
        }
        
        // Always update the updated_at timestamp
        query_parts.push("updated_at = ?");
        params.push(Box::new(now.to_rfc3339()));
        
        // Add profile_id for WHERE clause
        params.push(Box::new(profile_id.to_string()));
        
        let query = format!(
            "UPDATE xtream_profiles SET {} WHERE id = ?",
            query_parts.join(", ")
        );
        
        let rows_affected = conn.execute(&query, rusqlite::params_from_iter(params.iter()))?;
        
        if rows_affected == 0 {
            return Err(TolloError::xtream_profile_not_found(profile_id));
        }
        
        Ok(())
    }
    
    /// Delete an Xtream profile and all associated data
    pub fn delete_profile(conn: &Connection, profile_id: &str) -> Result<()> {
        let rows_affected = conn.execute(
            "DELETE FROM xtream_profiles WHERE id = ?1",
            params![profile_id],
        )?;
        
        if rows_affected == 0 {
            return Err(TolloError::xtream_profile_not_found(profile_id));
        }
        
        Ok(())
    }
    
    /// Get all Xtream profiles
    pub fn get_profiles(conn: &Connection) -> Result<Vec<XtreamProfile>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, url, username, created_at, updated_at, last_used, is_active 
             FROM xtream_profiles 
             ORDER BY name"
        )?;
        
        let profile_iter = stmt.query_map([], |row| {
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            let last_used_str: Option<String> = row.get(6)?;
            
            Ok(XtreamProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(5, "updated_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                last_used: match last_used_str {
                    Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(6, "last_used".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc)),
                    None => None,
                },
                is_active: row.get(7)?,
            })
        })?;
        
        let mut profiles = Vec::new();
        for profile in profile_iter {
            profiles.push(profile?);
        }
        
        Ok(profiles)
    }
    
    /// Get a specific Xtream profile by ID
    pub fn get_profile(conn: &Connection, profile_id: &str) -> Result<Option<XtreamProfile>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, url, username, created_at, updated_at, last_used, is_active 
             FROM xtream_profiles 
             WHERE id = ?1"
        )?;
        
        let mut profile_iter = stmt.query_map(params![profile_id], |row| {
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            let last_used_str: Option<String> = row.get(6)?;
            
            Ok(XtreamProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(5, "updated_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                last_used: match last_used_str {
                    Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(6, "last_used".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc)),
                    None => None,
                },
                is_active: row.get(7)?,
            })
        })?;
        
        match profile_iter.next() {
            Some(profile) => Ok(Some(profile?)),
            None => Ok(None),
        }
    }
    
    /// Get encrypted credentials for a profile
    pub fn get_encrypted_credentials(conn: &Connection, profile_id: &str) -> Result<Option<Vec<u8>>> {
        let mut stmt = conn.prepare(
            "SELECT encrypted_credentials FROM xtream_profiles WHERE id = ?1"
        )?;
        
        let mut credential_iter = stmt.query_map(params![profile_id], |row| {
            Ok(row.get::<_, Vec<u8>>(0)?)
        })?;
        
        match credential_iter.next() {
            Some(credentials) => Ok(Some(credentials?)),
            None => Ok(None),
        }
    }
    
    /// Set a profile as active (and deactivate all others)
    pub fn set_active_profile(conn: &Connection, profile_id: &str) -> Result<()> {
        let tx = conn.unchecked_transaction()?;
        
        // Deactivate all profiles
        tx.execute("UPDATE xtream_profiles SET is_active = FALSE", [])?;
        
        // Activate the specified profile and update last_used
        let now = Utc::now();
        let rows_affected = tx.execute(
            "UPDATE xtream_profiles SET is_active = TRUE, last_used = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), profile_id],
        )?;
        
        if rows_affected == 0 {
            tx.rollback()?;
            return Err(TolloError::xtream_profile_not_found(profile_id));
        }
        
        tx.commit()?;
        Ok(())
    }
    
    /// Get the currently active profile
    pub fn get_active_profile(conn: &Connection) -> Result<Option<XtreamProfile>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, url, username, created_at, updated_at, last_used, is_active 
             FROM xtream_profiles 
             WHERE is_active = TRUE 
             LIMIT 1"
        )?;
        
        let mut profile_iter = stmt.query_map([], |row| {
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            let last_used_str: Option<String> = row.get(6)?;
            
            Ok(XtreamProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(4, "created_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(5, "updated_at".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
                last_used: match last_used_str {
                    Some(s) => Some(DateTime::parse_from_rfc3339(&s)
                        .map_err(|_| rusqlite::Error::InvalidColumnType(6, "last_used".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc)),
                    None => None,
                },
                is_active: row.get(7)?,
            })
        })?;
        
        match profile_iter.next() {
            Some(profile) => Ok(Some(profile?)),
            None => Ok(None),
        }
    }
    
    /// Deactivate all profiles
    pub fn deactivate_all_profiles(conn: &Connection) -> Result<()> {
        conn.execute("UPDATE xtream_profiles SET is_active = FALSE", [])?;
        Ok(())
    }
    
    /// Check if a profile name already exists (for validation)
    pub fn profile_name_exists(conn: &Connection, name: &str, exclude_id: Option<&str>) -> Result<bool> {
        let (query, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = match exclude_id {
            Some(id) => (
                "SELECT COUNT(*) FROM xtream_profiles WHERE name = ? AND id != ?".to_string(),
                vec![Box::new(name.to_string()), Box::new(id.to_string())],
            ),
            None => (
                "SELECT COUNT(*) FROM xtream_profiles WHERE name = ?".to_string(),
                vec![Box::new(name.to_string())],
            ),
        };
        
        let count: i64 = conn.query_row(&query, rusqlite::params_from_iter(params.iter()), |row| {
            row.get(0)
        })?;
        
        Ok(count > 0)
    }
    
    /// Update profile last used timestamp
    pub fn update_last_used(conn: &Connection, profile_id: &str) -> Result<()> {
        let now = Utc::now();
        let rows_affected = conn.execute(
            "UPDATE xtream_profiles SET last_used = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), profile_id],
        )?;
        
        if rows_affected == 0 {
            return Err(TolloError::xtream_profile_not_found(profile_id));
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
        
        // Create the xtream_profiles table
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
    
    fn create_test_profile_request() -> CreateProfileRequest {
        CreateProfileRequest {
            name: "Test Profile".to_string(),
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        }
    }
    
    #[test]
    fn test_create_profile() {
        let conn = create_test_db();
        let request = create_test_profile_request();
        let encrypted_credentials = b"encrypted_data";
        
        let profile_id = XtreamDatabase::create_profile(&conn, &request, encrypted_credentials).unwrap();
        
        // Verify profile was created
        assert!(!profile_id.is_empty());
        
        let profile = XtreamDatabase::get_profile(&conn, &profile_id).unwrap().unwrap();
        assert_eq!(profile.name, request.name);
        assert_eq!(profile.url, request.url);
        assert_eq!(profile.username, request.username);
        assert!(!profile.is_active);
    }
    
    #[test]
    fn test_create_profile_duplicate_name() {
        let conn = create_test_db();
        let request = create_test_profile_request();
        let encrypted_credentials = b"encrypted_data";
        
        // Create first profile
        XtreamDatabase::create_profile(&conn, &request, encrypted_credentials).unwrap();
        
        // Try to create second profile with same name
        let result = XtreamDatabase::create_profile(&conn, &request, encrypted_credentials);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }
    
    #[test]
    fn test_update_profile() {
        let conn = create_test_db();
        let request = create_test_profile_request();
        let encrypted_credentials = b"encrypted_data";
        
        let profile_id = XtreamDatabase::create_profile(&conn, &request, encrypted_credentials).unwrap();
        
        // Update profile
        let update_request = UpdateProfileRequest {
            name: Some("Updated Profile".to_string()),
            url: Some("http://updated.com:8080".to_string()),
            username: None,
            password: None,
        };
        
        XtreamDatabase::update_profile(&conn, &profile_id, &update_request, None).unwrap();
        
        // Verify update
        let profile = XtreamDatabase::get_profile(&conn, &profile_id).unwrap().unwrap();
        assert_eq!(profile.name, "Updated Profile");
        assert_eq!(profile.url, "http://updated.com:8080");
        assert_eq!(profile.username, request.username); // Should remain unchanged
    }
    
    #[test]
    fn test_update_nonexistent_profile() {
        let conn = create_test_db();
        let update_request = UpdateProfileRequest {
            name: Some("Updated Profile".to_string()),
            url: None,
            username: None,
            password: None,
        };
        
        let result = XtreamDatabase::update_profile(&conn, "nonexistent", &update_request, None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_delete_profile() {
        let conn = create_test_db();
        let request = create_test_profile_request();
        let encrypted_credentials = b"encrypted_data";
        
        let profile_id = XtreamDatabase::create_profile(&conn, &request, encrypted_credentials).unwrap();
        
        // Verify profile exists
        assert!(XtreamDatabase::get_profile(&conn, &profile_id).unwrap().is_some());
        
        // Delete profile
        XtreamDatabase::delete_profile(&conn, &profile_id).unwrap();
        
        // Verify profile is gone
        assert!(XtreamDatabase::get_profile(&conn, &profile_id).unwrap().is_none());
    }
    
    #[test]
    fn test_delete_nonexistent_profile() {
        let conn = create_test_db();
        
        let result = XtreamDatabase::delete_profile(&conn, "nonexistent");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_profiles() {
        let conn = create_test_db();
        let encrypted_credentials = b"encrypted_data";
        
        // Create multiple profiles
        let request1 = CreateProfileRequest {
            name: "Profile A".to_string(),
            url: "http://a.com".to_string(),
            username: "user_a".to_string(),
            password: "pass_a".to_string(),
        };
        let request2 = CreateProfileRequest {
            name: "Profile B".to_string(),
            url: "http://b.com".to_string(),
            username: "user_b".to_string(),
            password: "pass_b".to_string(),
        };
        
        XtreamDatabase::create_profile(&conn, &request1, encrypted_credentials).unwrap();
        XtreamDatabase::create_profile(&conn, &request2, encrypted_credentials).unwrap();
        
        let profiles = XtreamDatabase::get_profiles(&conn).unwrap();
        assert_eq!(profiles.len(), 2);
        
        // Should be ordered by name
        assert_eq!(profiles[0].name, "Profile A");
        assert_eq!(profiles[1].name, "Profile B");
    }
    
    #[test]
    fn test_get_encrypted_credentials() {
        let conn = create_test_db();
        let request = create_test_profile_request();
        let encrypted_credentials = b"encrypted_test_data";
        
        let profile_id = XtreamDatabase::create_profile(&conn, &request, encrypted_credentials).unwrap();
        
        let retrieved_credentials = XtreamDatabase::get_encrypted_credentials(&conn, &profile_id)
            .unwrap()
            .unwrap();
        
        assert_eq!(retrieved_credentials, encrypted_credentials);
    }
    
    #[test]
    fn test_set_active_profile() {
        let conn = create_test_db();
        let encrypted_credentials = b"encrypted_data";
        
        // Create two profiles
        let request1 = create_test_profile_request();
        let request2 = CreateProfileRequest {
            name: "Profile 2".to_string(),
            url: "http://example2.com".to_string(),
            username: "user2".to_string(),
            password: "pass2".to_string(),
        };
        
        let profile_id1 = XtreamDatabase::create_profile(&conn, &request1, encrypted_credentials).unwrap();
        let profile_id2 = XtreamDatabase::create_profile(&conn, &request2, encrypted_credentials).unwrap();
        
        // Set first profile as active
        XtreamDatabase::set_active_profile(&conn, &profile_id1).unwrap();
        
        let active_profile = XtreamDatabase::get_active_profile(&conn).unwrap().unwrap();
        assert_eq!(active_profile.id, profile_id1);
        assert!(active_profile.is_active);
        assert!(active_profile.last_used.is_some());
        
        // Set second profile as active
        XtreamDatabase::set_active_profile(&conn, &profile_id2).unwrap();
        
        let active_profile = XtreamDatabase::get_active_profile(&conn).unwrap().unwrap();
        assert_eq!(active_profile.id, profile_id2);
        
        // First profile should no longer be active
        let profile1 = XtreamDatabase::get_profile(&conn, &profile_id1).unwrap().unwrap();
        assert!(!profile1.is_active);
    }
    
    #[test]
    fn test_deactivate_all_profiles() {
        let conn = create_test_db();
        let request = create_test_profile_request();
        let encrypted_credentials = b"encrypted_data";
        
        let profile_id = XtreamDatabase::create_profile(&conn, &request, encrypted_credentials).unwrap();
        XtreamDatabase::set_active_profile(&conn, &profile_id).unwrap();
        
        // Verify profile is active
        assert!(XtreamDatabase::get_active_profile(&conn).unwrap().is_some());
        
        // Deactivate all profiles
        XtreamDatabase::deactivate_all_profiles(&conn).unwrap();
        
        // Verify no active profile
        assert!(XtreamDatabase::get_active_profile(&conn).unwrap().is_none());
    }
    
    #[test]
    fn test_profile_name_exists() {
        let conn = create_test_db();
        let request = create_test_profile_request();
        let encrypted_credentials = b"encrypted_data";
        
        // Initially name doesn't exist
        assert!(!XtreamDatabase::profile_name_exists(&conn, &request.name, None).unwrap());
        
        let profile_id = XtreamDatabase::create_profile(&conn, &request, encrypted_credentials).unwrap();
        
        // Now name exists
        assert!(XtreamDatabase::profile_name_exists(&conn, &request.name, None).unwrap());
        
        // But not when excluding the same profile
        assert!(!XtreamDatabase::profile_name_exists(&conn, &request.name, Some(&profile_id)).unwrap());
    }
    
    #[test]
    fn test_update_last_used() {
        let conn = create_test_db();
        let request = create_test_profile_request();
        let encrypted_credentials = b"encrypted_data";
        
        let profile_id = XtreamDatabase::create_profile(&conn, &request, encrypted_credentials).unwrap();
        
        // Initially last_used should be None
        let profile = XtreamDatabase::get_profile(&conn, &profile_id).unwrap().unwrap();
        assert!(profile.last_used.is_none());
        
        // Update last_used
        XtreamDatabase::update_last_used(&conn, &profile_id).unwrap();
        
        // Verify last_used is now set
        let profile = XtreamDatabase::get_profile(&conn, &profile_id).unwrap().unwrap();
        assert!(profile.last_used.is_some());
    }
}