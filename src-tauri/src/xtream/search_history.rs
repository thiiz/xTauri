use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryItem {
    pub id: String,
    pub profile_id: String,
    pub query: String,
    pub content_types: Vec<String>, // channels, movies, series
    pub results_count: usize,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSearchHistoryRequest {
    pub profile_id: String,
    pub query: String,
    pub content_types: Vec<String>,
    pub results_count: usize,
}

pub struct SearchHistoryDb;

impl SearchHistoryDb {
    /// Initialize search history table
    pub fn init(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS xtream_search_history (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                query TEXT NOT NULL,
                content_types TEXT NOT NULL,
                results_count INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create index for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_search_history_profile 
             ON xtream_search_history(profile_id, created_at DESC)",
            [],
        )?;

        Ok(())
    }

    /// Add a search to history
    pub fn add_search(conn: &Connection, request: &AddSearchHistoryRequest) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let content_types_json = serde_json::to_string(&request.content_types)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        conn.execute(
            "INSERT INTO xtream_search_history (id, profile_id, query, content_types, results_count)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                request.profile_id,
                request.query,
                content_types_json,
                request.results_count as i64,
            ],
        )?;

        Ok(id)
    }

    /// Get search history for a profile
    pub fn get_search_history(
        conn: &Connection,
        profile_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<SearchHistoryItem>> {
        let limit = limit.unwrap_or(50);
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, query, content_types, results_count, created_at
             FROM xtream_search_history
             WHERE profile_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;

        let items = stmt
            .query_map(params![profile_id, limit as i64], |row| {
                let content_types_json: String = row.get(3)?;
                let content_types: Vec<String> = serde_json::from_str(&content_types_json)
                    .unwrap_or_default();

                Ok(SearchHistoryItem {
                    id: row.get(0)?,
                    profile_id: row.get(1)?,
                    query: row.get(2)?,
                    content_types,
                    results_count: row.get::<_, i64>(4)? as usize,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(items)
    }

    /// Get unique search suggestions (most recent unique queries)
    pub fn get_search_suggestions(
        conn: &Connection,
        profile_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<String>> {
        let limit = limit.unwrap_or(10);
        let mut stmt = conn.prepare(
            "SELECT DISTINCT query
             FROM xtream_search_history
             WHERE profile_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;

        let suggestions = stmt
            .query_map(params![profile_id, limit as i64], |row| row.get(0))?
            .collect::<Result<Vec<_>>>()?;

        Ok(suggestions)
    }

    /// Clear search history for a profile
    pub fn clear_search_history(conn: &Connection, profile_id: &str) -> Result<()> {
        conn.execute(
            "DELETE FROM xtream_search_history WHERE profile_id = ?1",
            params![profile_id],
        )?;
        Ok(())
    }

    /// Remove a specific search history item
    pub fn remove_search_history_item(conn: &Connection, id: &str) -> Result<()> {
        conn.execute(
            "DELETE FROM xtream_search_history WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Clear old search history (older than specified days)
    pub fn clear_old_search_history(
        conn: &Connection,
        profile_id: &str,
        days: i64,
    ) -> Result<usize> {
        let deleted = conn.execute(
            "DELETE FROM xtream_search_history 
             WHERE profile_id = ?1 
             AND created_at < datetime('now', '-' || ?2 || ' days')",
            params![profile_id, days],
        )?;
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        SearchHistoryDb::init(&conn).unwrap();
        conn
    }

    #[test]
    fn test_add_and_get_search_history() {
        let conn = setup_test_db();
        let request = AddSearchHistoryRequest {
            profile_id: "test_profile".to_string(),
            query: "test query".to_string(),
            content_types: vec!["channels".to_string(), "movies".to_string()],
            results_count: 10,
        };

        let id = SearchHistoryDb::add_search(&conn, &request).unwrap();
        assert!(!id.is_empty());

        let history = SearchHistoryDb::get_search_history(&conn, "test_profile", None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].query, "test query");
        assert_eq!(history[0].results_count, 10);
    }

    #[test]
    fn test_search_suggestions() {
        let conn = setup_test_db();
        
        // Add multiple searches
        for i in 1..=5 {
            let request = AddSearchHistoryRequest {
                profile_id: "test_profile".to_string(),
                query: format!("query {}", i),
                content_types: vec!["channels".to_string()],
                results_count: i,
            };
            SearchHistoryDb::add_search(&conn, &request).unwrap();
        }

        let suggestions = SearchHistoryDb::get_search_suggestions(&conn, "test_profile", Some(3)).unwrap();
        assert_eq!(suggestions.len(), 3);
    }

    #[test]
    fn test_clear_search_history() {
        let conn = setup_test_db();
        let request = AddSearchHistoryRequest {
            profile_id: "test_profile".to_string(),
            query: "test query".to_string(),
            content_types: vec!["channels".to_string()],
            results_count: 5,
        };

        SearchHistoryDb::add_search(&conn, &request).unwrap();
        SearchHistoryDb::clear_search_history(&conn, "test_profile").unwrap();

        let history = SearchHistoryDb::get_search_history(&conn, "test_profile", None).unwrap();
        assert_eq!(history.len(), 0);
    }
}
