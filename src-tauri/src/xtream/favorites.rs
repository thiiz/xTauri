use crate::error::{Result, XTauriError};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

/// Favorite item for Xtream content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamFavorite {
    pub id: String,
    pub profile_id: String,
    pub content_type: String,
    pub content_id: String,
    pub content_data: serde_json::Value,
    pub created_at: String,
}

/// Request to add a favorite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFavoriteRequest {
    pub profile_id: String,
    pub content_type: String,
    pub content_id: String,
    pub content_data: serde_json::Value,
}

/// Database operations for Xtream favorites
pub struct XtreamFavoritesDb;

impl XtreamFavoritesDb {
    /// Add a favorite for a profile
    pub fn add_favorite(
        conn: &Connection,
        request: &AddFavoriteRequest,
    ) -> Result<String> {
        let favorite_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Serialize content data to JSON bytes
        let content_data_bytes = serde_json::to_vec(&request.content_data)
            .map_err(|e| XTauriError::internal(format!("Failed to serialize content data: {}", e)))?;
        
        conn.execute(
            "INSERT INTO xtream_favorites (id, profile_id, content_type, content_id, content_data, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                favorite_id,
                request.profile_id,
                request.content_type,
                request.content_id,
                content_data_bytes,
                now.to_rfc3339(),
            ],
        ).map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                XTauriError::internal("This item is already in favorites".to_string())
            } else {
                XTauriError::Database(e)
            }
        })?;
        
        Ok(favorite_id)
    }
    
    /// Remove a favorite by ID
    pub fn remove_favorite(
        conn: &Connection,
        favorite_id: &str,
    ) -> Result<()> {
        let rows_affected = conn.execute(
            "DELETE FROM xtream_favorites WHERE id = ?1",
            params![favorite_id],
        )?;
        
        if rows_affected == 0 {
            return Err(XTauriError::internal("Favorite not found".to_string()));
        }
        
        Ok(())
    }
    
    /// Remove a favorite by profile, content type, and content ID
    pub fn remove_favorite_by_content(
        conn: &Connection,
        profile_id: &str,
        content_type: &str,
        content_id: &str,
    ) -> Result<()> {
        let rows_affected = conn.execute(
            "DELETE FROM xtream_favorites WHERE profile_id = ?1 AND content_type = ?2 AND content_id = ?3",
            params![profile_id, content_type, content_id],
        )?;
        
        if rows_affected == 0 {
            return Err(XTauriError::internal("Favorite not found".to_string()));
        }
        
        Ok(())
    }
    
    /// Get all favorites for a profile
    pub fn get_favorites(
        conn: &Connection,
        profile_id: &str,
    ) -> Result<Vec<XtreamFavorite>> {
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, content_type, content_id, content_data, created_at 
             FROM xtream_favorites 
             WHERE profile_id = ?1 
             ORDER BY created_at DESC"
        )?;
        
        let favorite_iter = stmt.query_map(params![profile_id], |row| {
            let content_data_bytes: Vec<u8> = row.get(4)?;
            let content_data: serde_json::Value = serde_json::from_slice(&content_data_bytes)
                .map_err(|e| rusqlite::Error::InvalidColumnType(4, "content_data".to_string(), rusqlite::types::Type::Blob))?;
            
            Ok(XtreamFavorite {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                content_type: row.get(2)?,
                content_id: row.get(3)?,
                content_data,
                created_at: row.get(5)?,
            })
        })?;
        
        let mut favorites = Vec::new();
        for favorite in favorite_iter {
            favorites.push(favorite?);
        }
        
        Ok(favorites)
    }
    
    /// Get favorites by content type for a profile
    pub fn get_favorites_by_type(
        conn: &Connection,
        profile_id: &str,
        content_type: &str,
    ) -> Result<Vec<XtreamFavorite>> {
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, content_type, content_id, content_data, created_at 
             FROM xtream_favorites 
             WHERE profile_id = ?1 AND content_type = ?2 
             ORDER BY created_at DESC"
        )?;
        
        let favorite_iter = stmt.query_map(params![profile_id, content_type], |row| {
            let content_data_bytes: Vec<u8> = row.get(4)?;
            let content_data: serde_json::Value = serde_json::from_slice(&content_data_bytes)
                .map_err(|e| rusqlite::Error::InvalidColumnType(4, "content_data".to_string(), rusqlite::types::Type::Blob))?;
            
            Ok(XtreamFavorite {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                content_type: row.get(2)?,
                content_id: row.get(3)?,
                content_data,
                created_at: row.get(5)?,
            })
        })?;
        
        let mut favorites = Vec::new();
        for favorite in favorite_iter {
            favorites.push(favorite?);
        }
        
        Ok(favorites)
    }
    
    /// Check if an item is favorited
    pub fn is_favorite(
        conn: &Connection,
        profile_id: &str,
        content_type: &str,
        content_id: &str,
    ) -> Result<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM xtream_favorites WHERE profile_id = ?1 AND content_type = ?2 AND content_id = ?3",
            params![profile_id, content_type, content_id],
            |row| row.get(0),
        )?;
        
        Ok(count > 0)
    }
    
    /// Clear all favorites for a profile
    pub fn clear_favorites(
        conn: &Connection,
        profile_id: &str,
    ) -> Result<()> {
        conn.execute(
            "DELETE FROM xtream_favorites WHERE profile_id = ?1",
            params![profile_id],
        )?;
        
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
        
        // Create the xtream_favorites table
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
        
        // Insert a test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile-1', 'Test Profile', 'http://example.com', 'testuser', X'00')",
            [],
        ).unwrap();
        
        conn
    }
    
    fn create_test_favorite_request() -> AddFavoriteRequest {
        AddFavoriteRequest {
            profile_id: "test-profile-1".to_string(),
            content_type: "channel".to_string(),
            content_id: "123".to_string(),
            content_data: serde_json::json!({
                "name": "Test Channel",
                "stream_id": 123,
            }),
        }
    }
    
    #[test]
    fn test_add_favorite() {
        let conn = create_test_db();
        let request = create_test_favorite_request();
        
        let favorite_id = XtreamFavoritesDb::add_favorite(&conn, &request).unwrap();
        
        assert!(!favorite_id.is_empty());
        
        // Verify favorite was created
        let favorites = XtreamFavoritesDb::get_favorites(&conn, "test-profile-1").unwrap();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].content_type, "channel");
        assert_eq!(favorites[0].content_id, "123");
    }
    
    #[test]
    fn test_add_duplicate_favorite() {
        let conn = create_test_db();
        let request = create_test_favorite_request();
        
        // Add first favorite
        XtreamFavoritesDb::add_favorite(&conn, &request).unwrap();
        
        // Try to add duplicate
        let result = XtreamFavoritesDb::add_favorite(&conn, &request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already in favorites"));
    }
    
    #[test]
    fn test_remove_favorite() {
        let conn = create_test_db();
        let request = create_test_favorite_request();
        
        let favorite_id = XtreamFavoritesDb::add_favorite(&conn, &request).unwrap();
        
        // Remove favorite
        XtreamFavoritesDb::remove_favorite(&conn, &favorite_id).unwrap();
        
        // Verify it's gone
        let favorites = XtreamFavoritesDb::get_favorites(&conn, "test-profile-1").unwrap();
        assert_eq!(favorites.len(), 0);
    }
    
    #[test]
    fn test_remove_favorite_by_content() {
        let conn = create_test_db();
        let request = create_test_favorite_request();
        
        XtreamFavoritesDb::add_favorite(&conn, &request).unwrap();
        
        // Remove by content
        XtreamFavoritesDb::remove_favorite_by_content(
            &conn,
            "test-profile-1",
            "channel",
            "123",
        ).unwrap();
        
        // Verify it's gone
        let favorites = XtreamFavoritesDb::get_favorites(&conn, "test-profile-1").unwrap();
        assert_eq!(favorites.len(), 0);
    }
    
    #[test]
    fn test_get_favorites_by_type() {
        let conn = create_test_db();
        
        // Add different types of favorites
        let channel_request = AddFavoriteRequest {
            profile_id: "test-profile-1".to_string(),
            content_type: "channel".to_string(),
            content_id: "123".to_string(),
            content_data: serde_json::json!({"name": "Channel"}),
        };
        
        let movie_request = AddFavoriteRequest {
            profile_id: "test-profile-1".to_string(),
            content_type: "movie".to_string(),
            content_id: "456".to_string(),
            content_data: serde_json::json!({"name": "Movie"}),
        };
        
        XtreamFavoritesDb::add_favorite(&conn, &channel_request).unwrap();
        XtreamFavoritesDb::add_favorite(&conn, &movie_request).unwrap();
        
        // Get only channels
        let channels = XtreamFavoritesDb::get_favorites_by_type(&conn, "test-profile-1", "channel").unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].content_type, "channel");
        
        // Get only movies
        let movies = XtreamFavoritesDb::get_favorites_by_type(&conn, "test-profile-1", "movie").unwrap();
        assert_eq!(movies.len(), 1);
        assert_eq!(movies[0].content_type, "movie");
    }
    
    #[test]
    fn test_is_favorite() {
        let conn = create_test_db();
        let request = create_test_favorite_request();
        
        // Initially not a favorite
        assert!(!XtreamFavoritesDb::is_favorite(&conn, "test-profile-1", "channel", "123").unwrap());
        
        // Add favorite
        XtreamFavoritesDb::add_favorite(&conn, &request).unwrap();
        
        // Now it is a favorite
        assert!(XtreamFavoritesDb::is_favorite(&conn, "test-profile-1", "channel", "123").unwrap());
    }
    
    #[test]
    fn test_clear_favorites() {
        let conn = create_test_db();
        
        // Add multiple favorites
        for i in 0..5 {
            let request = AddFavoriteRequest {
                profile_id: "test-profile-1".to_string(),
                content_type: "channel".to_string(),
                content_id: i.to_string(),
                content_data: serde_json::json!({"name": format!("Channel {}", i)}),
            };
            XtreamFavoritesDb::add_favorite(&conn, &request).unwrap();
        }
        
        // Verify they exist
        let favorites = XtreamFavoritesDb::get_favorites(&conn, "test-profile-1").unwrap();
        assert_eq!(favorites.len(), 5);
        
        // Clear all
        XtreamFavoritesDb::clear_favorites(&conn, "test-profile-1").unwrap();
        
        // Verify they're gone
        let favorites = XtreamFavoritesDb::get_favorites(&conn, "test-profile-1").unwrap();
        assert_eq!(favorites.len(), 0);
    }
}
