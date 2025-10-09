use rusqlite::{params, Connection, Result, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFilter {
    pub id: String,
    pub profile_id: String,
    pub name: String,
    pub content_type: String, // "channels", "movies", "series"
    pub filter_data: String, // JSON serialized filter
    pub created_at: String,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavedFilterRequest {
    pub profile_id: String,
    pub name: String,
    pub content_type: String,
    pub filter_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSavedFilterRequest {
    pub name: Option<String>,
    pub filter_data: Option<String>,
}

pub struct SavedFiltersDb;

impl SavedFiltersDb {
    /// Initialize saved filters table
    pub fn init(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS xtream_saved_filters (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                name TEXT NOT NULL,
                content_type TEXT NOT NULL,
                filter_data TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_used DATETIME,
                FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
                UNIQUE(profile_id, name, content_type)
            )",
            [],
        )?;

        // Create index for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_saved_filters_profile 
             ON xtream_saved_filters(profile_id, content_type)",
            [],
        )?;

        Ok(())
    }

    /// Create a new saved filter
    pub fn create_filter(conn: &Connection, request: &CreateSavedFilterRequest) -> Result<String> {
        let id = Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO xtream_saved_filters (id, profile_id, name, content_type, filter_data)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                request.profile_id,
                request.name,
                request.content_type,
                request.filter_data,
            ],
        )?;

        Ok(id)
    }

    /// Get all saved filters for a profile
    pub fn get_filters(
        conn: &Connection,
        profile_id: &str,
        content_type: Option<&str>,
    ) -> Result<Vec<SavedFilter>> {
        let mut query = String::from(
            "SELECT id, profile_id, name, content_type, filter_data, created_at, last_used
             FROM xtream_saved_filters
             WHERE profile_id = ?1"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];

        if let Some(ct) = content_type {
            query.push_str(" AND content_type = ?2");
            params.push(Box::new(ct.to_string()));
        }

        query.push_str(" ORDER BY last_used DESC, created_at DESC");

        let mut stmt = conn.prepare(&query)?;
        let filters = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                Ok(SavedFilter {
                    id: row.get(0)?,
                    profile_id: row.get(1)?,
                    name: row.get(2)?,
                    content_type: row.get(3)?,
                    filter_data: row.get(4)?,
                    created_at: row.get(5)?,
                    last_used: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(filters)
    }

    /// Get a specific saved filter by ID
    pub fn get_filter(conn: &Connection, id: &str) -> Result<Option<SavedFilter>> {
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, name, content_type, filter_data, created_at, last_used
             FROM xtream_saved_filters
             WHERE id = ?1",
        )?;

        let filter = stmt
            .query_row(params![id], |row| {
                Ok(SavedFilter {
                    id: row.get(0)?,
                    profile_id: row.get(1)?,
                    name: row.get(2)?,
                    content_type: row.get(3)?,
                    filter_data: row.get(4)?,
                    created_at: row.get(5)?,
                    last_used: row.get(6)?,
                })
            })
            .optional()?;

        Ok(filter)
    }

    /// Update a saved filter
    pub fn update_filter(
        conn: &Connection,
        id: &str,
        request: &UpdateSavedFilterRequest,
    ) -> Result<()> {
        let mut query_parts = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(name) = &request.name {
            query_parts.push("name = ?");
            params.push(Box::new(name.clone()));
        }

        if let Some(filter_data) = &request.filter_data {
            query_parts.push("filter_data = ?");
            params.push(Box::new(filter_data.clone()));
        }

        if query_parts.is_empty() {
            return Ok(());
        }

        params.push(Box::new(id.to_string()));

        let query = format!(
            "UPDATE xtream_saved_filters SET {} WHERE id = ?",
            query_parts.join(", ")
        );

        conn.execute(&query, rusqlite::params_from_iter(params.iter()))?;
        Ok(())
    }

    /// Update last used timestamp
    pub fn update_last_used(conn: &Connection, id: &str) -> Result<()> {
        conn.execute(
            "UPDATE xtream_saved_filters SET last_used = CURRENT_TIMESTAMP WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Delete a saved filter
    pub fn delete_filter(conn: &Connection, id: &str) -> Result<()> {
        conn.execute(
            "DELETE FROM xtream_saved_filters WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Clear all saved filters for a profile
    pub fn clear_filters(conn: &Connection, profile_id: &str) -> Result<()> {
        conn.execute(
            "DELETE FROM xtream_saved_filters WHERE profile_id = ?1",
            params![profile_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        SavedFiltersDb::init(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_filter() {
        let conn = setup_test_db();
        let request = CreateSavedFilterRequest {
            profile_id: "test_profile".to_string(),
            name: "My Filter".to_string(),
            content_type: "movies".to_string(),
            filter_data: r#"{"genre":"Action","min_rating":4.0}"#.to_string(),
        };

        let id = SavedFiltersDb::create_filter(&conn, &request).unwrap();
        assert!(!id.is_empty());

        let filters = SavedFiltersDb::get_filters(&conn, "test_profile", None).unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].name, "My Filter");
    }

    #[test]
    fn test_get_filters_by_content_type() {
        let conn = setup_test_db();
        
        // Create filters for different content types
        for content_type in &["channels", "movies", "series"] {
            let request = CreateSavedFilterRequest {
                profile_id: "test_profile".to_string(),
                name: format!("{} filter", content_type),
                content_type: content_type.to_string(),
                filter_data: "{}".to_string(),
            };
            SavedFiltersDb::create_filter(&conn, &request).unwrap();
        }

        let movie_filters = SavedFiltersDb::get_filters(&conn, "test_profile", Some("movies")).unwrap();
        assert_eq!(movie_filters.len(), 1);
        assert_eq!(movie_filters[0].content_type, "movies");
    }

    #[test]
    fn test_update_filter() {
        let conn = setup_test_db();
        let request = CreateSavedFilterRequest {
            profile_id: "test_profile".to_string(),
            name: "Original Name".to_string(),
            content_type: "movies".to_string(),
            filter_data: "{}".to_string(),
        };

        let id = SavedFiltersDb::create_filter(&conn, &request).unwrap();

        let update_request = UpdateSavedFilterRequest {
            name: Some("Updated Name".to_string()),
            filter_data: None,
        };

        SavedFiltersDb::update_filter(&conn, &id, &update_request).unwrap();

        let filter = SavedFiltersDb::get_filter(&conn, &id).unwrap().unwrap();
        assert_eq!(filter.name, "Updated Name");
    }

    #[test]
    fn test_delete_filter() {
        let conn = setup_test_db();
        let request = CreateSavedFilterRequest {
            profile_id: "test_profile".to_string(),
            name: "Test Filter".to_string(),
            content_type: "movies".to_string(),
            filter_data: "{}".to_string(),
        };

        let id = SavedFiltersDb::create_filter(&conn, &request).unwrap();
        SavedFiltersDb::delete_filter(&conn, &id).unwrap();

        let filter = SavedFiltersDb::get_filter(&conn, &id).unwrap();
        assert!(filter.is_none());
    }

    #[test]
    fn test_update_last_used() {
        let conn = setup_test_db();
        let request = CreateSavedFilterRequest {
            profile_id: "test_profile".to_string(),
            name: "Test Filter".to_string(),
            content_type: "movies".to_string(),
            filter_data: "{}".to_string(),
        };

        let id = SavedFiltersDb::create_filter(&conn, &request).unwrap();
        
        // Initially last_used should be None
        let filter = SavedFiltersDb::get_filter(&conn, &id).unwrap().unwrap();
        assert!(filter.last_used.is_none());

        // Update last_used
        SavedFiltersDb::update_last_used(&conn, &id).unwrap();

        // Now last_used should be set
        let filter = SavedFiltersDb::get_filter(&conn, &id).unwrap().unwrap();
        assert!(filter.last_used.is_some());
    }
}
