use crate::error::{Result, XTauriError};
use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

/// History item for Xtream content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamHistory {
    pub id: String,
    pub profile_id: String,
    pub content_type: String,
    pub content_id: String,
    pub content_data: serde_json::Value,
    pub watched_at: String,
    pub position: Option<f64>,
    pub duration: Option<f64>,
}

/// Request to add a history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddHistoryRequest {
    pub profile_id: String,
    pub content_type: String,
    pub content_id: String,
    pub content_data: serde_json::Value,
    pub position: Option<f64>,
    pub duration: Option<f64>,
}

/// Request to update playback position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePositionRequest {
    pub profile_id: String,
    pub content_type: String,
    pub content_id: String,
    pub position: f64,
    pub duration: Option<f64>,
}

/// Database operations for Xtream history
pub struct XtreamHistoryDb;

impl XtreamHistoryDb {
    /// Add or update a history item for a profile
    pub fn add_history(
        conn: &Connection,
        request: &AddHistoryRequest,
    ) -> Result<String> {
        // Check if history item already exists
        let existing_id: Option<String> = conn.query_row(
            "SELECT id FROM xtream_history WHERE profile_id = ?1 AND content_type = ?2 AND content_id = ?3",
            params![request.profile_id, request.content_type, request.content_id],
            |row| row.get(0),
        ).optional()?;
        
        let now = Utc::now();
        let content_data_bytes = serde_json::to_vec(&request.content_data)
            .map_err(|e| XTauriError::internal(format!("Failed to serialize content data: {}", e)))?;
        
        if let Some(id) = existing_id {
            // Update existing history item
            conn.execute(
                "UPDATE xtream_history 
                 SET content_data = ?1, watched_at = ?2, position = ?3, duration = ?4 
                 WHERE id = ?5",
                params![
                    content_data_bytes,
                    now.to_rfc3339(),
                    request.position,
                    request.duration,
                    id,
                ],
            )?;
            Ok(id)
        } else {
            // Insert new history item
            let history_id = Uuid::new_v4().to_string();
            
            conn.execute(
                "INSERT INTO xtream_history (id, profile_id, content_type, content_id, content_data, watched_at, position, duration) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    history_id,
                    request.profile_id,
                    request.content_type,
                    request.content_id,
                    content_data_bytes,
                    now.to_rfc3339(),
                    request.position,
                    request.duration,
                ],
            )?;
            
            Ok(history_id)
        }
    }
    
    /// Update playback position for a history item
    pub fn update_position(
        conn: &Connection,
        request: &UpdatePositionRequest,
    ) -> Result<()> {
        let now = Utc::now();
        
        let rows_affected = conn.execute(
            "UPDATE xtream_history 
             SET position = ?1, duration = ?2, watched_at = ?3 
             WHERE profile_id = ?4 AND content_type = ?5 AND content_id = ?6",
            params![
                request.position,
                request.duration,
                now.to_rfc3339(),
                request.profile_id,
                request.content_type,
                request.content_id,
            ],
        )?;
        
        if rows_affected == 0 {
            return Err(XTauriError::internal("History item not found".to_string()));
        }
        
        Ok(())
    }
    
    /// Get history for a profile
    pub fn get_history(
        conn: &Connection,
        profile_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<XtreamHistory>> {
        let limit_value = limit.unwrap_or(50);
        
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, content_type, content_id, content_data, watched_at, position, duration 
             FROM xtream_history 
             WHERE profile_id = ?1 
             ORDER BY watched_at DESC 
             LIMIT ?2"
        )?;
        
        let history_iter = stmt.query_map(params![profile_id, limit_value], |row| {
            let content_data_bytes: Vec<u8> = row.get(4)?;
            let content_data: serde_json::Value = serde_json::from_slice(&content_data_bytes)
                .map_err(|e| rusqlite::Error::InvalidColumnType(4, "content_data".to_string(), rusqlite::types::Type::Blob))?;
            
            Ok(XtreamHistory {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                content_type: row.get(2)?,
                content_id: row.get(3)?,
                content_data,
                watched_at: row.get(5)?,
                position: row.get(6)?,
                duration: row.get(7)?,
            })
        })?;
        
        let mut history = Vec::new();
        for item in history_iter {
            history.push(item?);
        }
        
        Ok(history)
    }
    
    /// Get history by content type for a profile
    pub fn get_history_by_type(
        conn: &Connection,
        profile_id: &str,
        content_type: &str,
        limit: Option<i64>,
    ) -> Result<Vec<XtreamHistory>> {
        let limit_value = limit.unwrap_or(50);
        
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, content_type, content_id, content_data, watched_at, position, duration 
             FROM xtream_history 
             WHERE profile_id = ?1 AND content_type = ?2 
             ORDER BY watched_at DESC 
             LIMIT ?3"
        )?;
        
        let history_iter = stmt.query_map(params![profile_id, content_type, limit_value], |row| {
            let content_data_bytes: Vec<u8> = row.get(4)?;
            let content_data: serde_json::Value = serde_json::from_slice(&content_data_bytes)
                .map_err(|e| rusqlite::Error::InvalidColumnType(4, "content_data".to_string(), rusqlite::types::Type::Blob))?;
            
            Ok(XtreamHistory {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                content_type: row.get(2)?,
                content_id: row.get(3)?,
                content_data,
                watched_at: row.get(5)?,
                position: row.get(6)?,
                duration: row.get(7)?,
            })
        })?;
        
        let mut history = Vec::new();
        for item in history_iter {
            history.push(item?);
        }
        
        Ok(history)
    }
    
    /// Get a specific history item
    pub fn get_history_item(
        conn: &Connection,
        profile_id: &str,
        content_type: &str,
        content_id: &str,
    ) -> Result<Option<XtreamHistory>> {
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, content_type, content_id, content_data, watched_at, position, duration 
             FROM xtream_history 
             WHERE profile_id = ?1 AND content_type = ?2 AND content_id = ?3"
        )?;
        
        let mut history_iter = stmt.query_map(params![profile_id, content_type, content_id], |row| {
            let content_data_bytes: Vec<u8> = row.get(4)?;
            let content_data: serde_json::Value = serde_json::from_slice(&content_data_bytes)
                .map_err(|e| rusqlite::Error::InvalidColumnType(4, "content_data".to_string(), rusqlite::types::Type::Blob))?;
            
            Ok(XtreamHistory {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                content_type: row.get(2)?,
                content_id: row.get(3)?,
                content_data,
                watched_at: row.get(5)?,
                position: row.get(6)?,
                duration: row.get(7)?,
            })
        })?;
        
        match history_iter.next() {
            Some(item) => Ok(Some(item?)),
            None => Ok(None),
        }
    }
    
    /// Remove a history item
    pub fn remove_history(
        conn: &Connection,
        history_id: &str,
    ) -> Result<()> {
        let rows_affected = conn.execute(
            "DELETE FROM xtream_history WHERE id = ?1",
            params![history_id],
        )?;
        
        if rows_affected == 0 {
            return Err(XTauriError::internal("History item not found".to_string()));
        }
        
        Ok(())
    }
    
    /// Clear all history for a profile
    pub fn clear_history(
        conn: &Connection,
        profile_id: &str,
    ) -> Result<()> {
        conn.execute(
            "DELETE FROM xtream_history WHERE profile_id = ?1",
            params![profile_id],
        )?;
        
        Ok(())
    }
    
    /// Clear old history items (older than specified days)
    pub fn clear_old_history(
        conn: &Connection,
        profile_id: &str,
        days: i64,
    ) -> Result<usize> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days);
        
        let rows_affected = conn.execute(
            "DELETE FROM xtream_history WHERE profile_id = ?1 AND watched_at < ?2",
            params![profile_id, cutoff_date.to_rfc3339()],
        )?;
        
        Ok(rows_affected)
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
        
        // Create the xtream_history table
        conn.execute(
            "CREATE TABLE xtream_history (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content_id TEXT NOT NULL,
                content_data BLOB NOT NULL,
                watched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                position REAL,
                duration REAL,
                FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
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
    
    fn create_test_history_request() -> AddHistoryRequest {
        AddHistoryRequest {
            profile_id: "test-profile-1".to_string(),
            content_type: "movie".to_string(),
            content_id: "123".to_string(),
            content_data: serde_json::json!({
                "name": "Test Movie",
                "stream_id": 123,
            }),
            position: Some(120.5),
            duration: Some(7200.0),
        }
    }
    
    #[test]
    fn test_add_history() {
        let conn = create_test_db();
        let request = create_test_history_request();
        
        let history_id = XtreamHistoryDb::add_history(&conn, &request).unwrap();
        
        assert!(!history_id.is_empty());
        
        // Verify history was created
        let history = XtreamHistoryDb::get_history(&conn, "test-profile-1", None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content_type, "movie");
        assert_eq!(history[0].content_id, "123");
        assert_eq!(history[0].position, Some(120.5));
    }
    
    #[test]
    fn test_add_history_updates_existing() {
        let conn = create_test_db();
        let request = create_test_history_request();
        
        // Add first time
        let id1 = XtreamHistoryDb::add_history(&conn, &request).unwrap();
        
        // Add again with different position
        let mut request2 = request.clone();
        request2.position = Some(240.0);
        let id2 = XtreamHistoryDb::add_history(&conn, &request2).unwrap();
        
        // Should be the same ID (updated, not inserted)
        assert_eq!(id1, id2);
        
        // Should still only have one history item
        let history = XtreamHistoryDb::get_history(&conn, "test-profile-1", None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].position, Some(240.0));
    }
    
    #[test]
    fn test_update_position() {
        let conn = create_test_db();
        let request = create_test_history_request();
        
        XtreamHistoryDb::add_history(&conn, &request).unwrap();
        
        // Update position
        let update_request = UpdatePositionRequest {
            profile_id: "test-profile-1".to_string(),
            content_type: "movie".to_string(),
            content_id: "123".to_string(),
            position: 500.0,
            duration: Some(7200.0),
        };
        
        XtreamHistoryDb::update_position(&conn, &update_request).unwrap();
        
        // Verify position was updated
        let history = XtreamHistoryDb::get_history(&conn, "test-profile-1", None).unwrap();
        assert_eq!(history[0].position, Some(500.0));
    }
    
    #[test]
    fn test_get_history_by_type() {
        let conn = create_test_db();
        
        // Add different types of history
        let movie_request = AddHistoryRequest {
            profile_id: "test-profile-1".to_string(),
            content_type: "movie".to_string(),
            content_id: "123".to_string(),
            content_data: serde_json::json!({"name": "Movie"}),
            position: None,
            duration: None,
        };
        
        let channel_request = AddHistoryRequest {
            profile_id: "test-profile-1".to_string(),
            content_type: "channel".to_string(),
            content_id: "456".to_string(),
            content_data: serde_json::json!({"name": "Channel"}),
            position: None,
            duration: None,
        };
        
        XtreamHistoryDb::add_history(&conn, &movie_request).unwrap();
        XtreamHistoryDb::add_history(&conn, &channel_request).unwrap();
        
        // Get only movies
        let movies = XtreamHistoryDb::get_history_by_type(&conn, "test-profile-1", "movie", None).unwrap();
        assert_eq!(movies.len(), 1);
        assert_eq!(movies[0].content_type, "movie");
        
        // Get only channels
        let channels = XtreamHistoryDb::get_history_by_type(&conn, "test-profile-1", "channel", None).unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].content_type, "channel");
    }
    
    #[test]
    fn test_get_history_item() {
        let conn = create_test_db();
        let request = create_test_history_request();
        
        XtreamHistoryDb::add_history(&conn, &request).unwrap();
        
        // Get specific item
        let item = XtreamHistoryDb::get_history_item(&conn, "test-profile-1", "movie", "123").unwrap();
        assert!(item.is_some());
        assert_eq!(item.unwrap().content_id, "123");
        
        // Try to get non-existent item
        let item = XtreamHistoryDb::get_history_item(&conn, "test-profile-1", "movie", "999").unwrap();
        assert!(item.is_none());
    }
    
    #[test]
    fn test_remove_history() {
        let conn = create_test_db();
        let request = create_test_history_request();
        
        let history_id = XtreamHistoryDb::add_history(&conn, &request).unwrap();
        
        // Remove history
        XtreamHistoryDb::remove_history(&conn, &history_id).unwrap();
        
        // Verify it's gone
        let history = XtreamHistoryDb::get_history(&conn, "test-profile-1", None).unwrap();
        assert_eq!(history.len(), 0);
    }
    
    #[test]
    fn test_clear_history() {
        let conn = create_test_db();
        
        // Add multiple history items
        for i in 0..5 {
            let request = AddHistoryRequest {
                profile_id: "test-profile-1".to_string(),
                content_type: "movie".to_string(),
                content_id: i.to_string(),
                content_data: serde_json::json!({"name": format!("Movie {}", i)}),
                position: None,
                duration: None,
            };
            XtreamHistoryDb::add_history(&conn, &request).unwrap();
        }
        
        // Verify they exist
        let history = XtreamHistoryDb::get_history(&conn, "test-profile-1", None).unwrap();
        assert_eq!(history.len(), 5);
        
        // Clear all
        XtreamHistoryDb::clear_history(&conn, "test-profile-1").unwrap();
        
        // Verify they're gone
        let history = XtreamHistoryDb::get_history(&conn, "test-profile-1", None).unwrap();
        assert_eq!(history.len(), 0);
    }
    
    #[test]
    fn test_history_limit() {
        let conn = create_test_db();
        
        // Add 10 history items
        for i in 0..10 {
            let request = AddHistoryRequest {
                profile_id: "test-profile-1".to_string(),
                content_type: "movie".to_string(),
                content_id: i.to_string(),
                content_data: serde_json::json!({"name": format!("Movie {}", i)}),
                position: None,
                duration: None,
            };
            XtreamHistoryDb::add_history(&conn, &request).unwrap();
        }
        
        // Get with limit
        let history = XtreamHistoryDb::get_history(&conn, "test-profile-1", Some(5)).unwrap();
        assert_eq!(history.len(), 5);
    }
}
