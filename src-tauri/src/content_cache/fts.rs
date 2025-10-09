// Full-Text Search (FTS) module for content cache
// 
// This module implements SQLite FTS5 virtual tables for fast fuzzy search
// across channels, movies, and series with relevance scoring.

use crate::error::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// Search result with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub item: T,
    pub relevance_score: f64,
}

/// Initialize FTS virtual tables for all content types
/// 
/// Creates FTS5 virtual tables that mirror the main content tables
/// for fast full-text search capabilities.
pub fn initialize_fts_tables(conn: &Connection) -> Result<()> {
    // Create FTS table for channels
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS xtream_channels_fts USING fts5(
            profile_id UNINDEXED,
            stream_id UNINDEXED,
            name,
            epg_channel_id,
            content='xtream_channels',
            content_rowid='id'
        )",
        [],
    )?;
    
    // Create FTS table for movies
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS xtream_movies_fts USING fts5(
            profile_id UNINDEXED,
            stream_id UNINDEXED,
            name,
            title,
            genre,
            cast,
            director,
            plot,
            content='xtream_movies',
            content_rowid='id'
        )",
        [],
    )?;
    
    // Create FTS table for series
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS xtream_series_fts USING fts5(
            profile_id UNINDEXED,
            series_id UNINDEXED,
            name,
            title,
            genre,
            cast,
            director,
            plot,
            content='xtream_series',
            content_rowid='id'
        )",
        [],
    )?;
    
    // Create triggers to keep FTS tables in sync with main tables
    create_fts_triggers(conn)?;
    
    Ok(())
}

/// Create triggers to automatically update FTS tables
fn create_fts_triggers(conn: &Connection) -> Result<()> {
    // Channels triggers
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS xtream_channels_fts_insert AFTER INSERT ON xtream_channels BEGIN
            INSERT INTO xtream_channels_fts(rowid, profile_id, stream_id, name, epg_channel_id)
            VALUES (new.id, new.profile_id, new.stream_id, new.name, new.epg_channel_id);
        END",
        [],
    )?;
    
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS xtream_channels_fts_delete AFTER DELETE ON xtream_channels BEGIN
            INSERT INTO xtream_channels_fts(xtream_channels_fts, rowid, profile_id, stream_id, name, epg_channel_id)
            VALUES ('delete', old.id, old.profile_id, old.stream_id, old.name, old.epg_channel_id);
        END",
        [],
    )?;
    
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS xtream_channels_fts_update AFTER UPDATE ON xtream_channels BEGIN
            INSERT INTO xtream_channels_fts(xtream_channels_fts, rowid, profile_id, stream_id, name, epg_channel_id)
            VALUES ('delete', old.id, old.profile_id, old.stream_id, old.name, old.epg_channel_id);
            INSERT INTO xtream_channels_fts(rowid, profile_id, stream_id, name, epg_channel_id)
            VALUES (new.id, new.profile_id, new.stream_id, new.name, new.epg_channel_id);
        END",
        [],
    )?;
    
    // Movies triggers
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS xtream_movies_fts_insert AFTER INSERT ON xtream_movies BEGIN
            INSERT INTO xtream_movies_fts(rowid, profile_id, stream_id, name, title, genre, cast, director, plot)
            VALUES (new.id, new.profile_id, new.stream_id, new.name, new.title, new.genre, new.cast, new.director, new.plot);
        END",
        [],
    )?;
    
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS xtream_movies_fts_delete AFTER DELETE ON xtream_movies BEGIN
            INSERT INTO xtream_movies_fts(xtream_movies_fts, rowid, profile_id, stream_id, name, title, genre, cast, director, plot)
            VALUES ('delete', old.id, old.profile_id, old.stream_id, old.name, old.title, old.genre, old.cast, old.director, old.plot);
        END",
        [],
    )?;
    
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS xtream_movies_fts_update AFTER UPDATE ON xtream_movies BEGIN
            INSERT INTO xtream_movies_fts(xtream_movies_fts, rowid, profile_id, stream_id, name, title, genre, cast, director, plot)
            VALUES ('delete', old.id, old.profile_id, old.stream_id, old.name, old.title, old.genre, old.cast, old.director, old.plot);
            INSERT INTO xtream_movies_fts(rowid, profile_id, stream_id, name, title, genre, cast, director, plot)
            VALUES (new.id, new.profile_id, new.stream_id, new.name, new.title, new.genre, new.cast, new.director, new.plot);
        END",
        [],
    )?;
    
    // Series triggers
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS xtream_series_fts_insert AFTER INSERT ON xtream_series BEGIN
            INSERT INTO xtream_series_fts(rowid, profile_id, series_id, name, title, genre, cast, director, plot)
            VALUES (new.id, new.profile_id, new.series_id, new.name, new.title, new.genre, new.cast, new.director, new.plot);
        END",
        [],
    )?;
    
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS xtream_series_fts_delete AFTER DELETE ON xtream_series BEGIN
            INSERT INTO xtream_series_fts(xtream_series_fts, rowid, profile_id, series_id, name, title, genre, cast, director, plot)
            VALUES ('delete', old.id, old.profile_id, old.series_id, old.name, old.title, old.genre, old.cast, old.director, old.plot);
        END",
        [],
    )?;
    
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS xtream_series_fts_update AFTER UPDATE ON xtream_series BEGIN
            INSERT INTO xtream_series_fts(xtream_series_fts, rowid, profile_id, series_id, name, title, genre, cast, director, plot)
            VALUES ('delete', old.id, old.profile_id, old.series_id, old.name, old.title, old.genre, old.cast, old.director, old.plot);
            INSERT INTO xtream_series_fts(rowid, profile_id, series_id, name, title, genre, cast, director, plot)
            VALUES (new.id, new.profile_id, new.series_id, new.name, new.title, new.genre, new.cast, new.director, new.plot);
        END",
        [],
    )?;
    
    Ok(())
}

/// Rebuild FTS index for a specific profile
/// 
/// This should be called after bulk inserts or when FTS tables get out of sync
pub fn rebuild_fts_index(conn: &Connection, profile_id: &str) -> Result<()> {
    // Delete existing FTS entries for this profile
    conn.execute(
        "DELETE FROM xtream_channels_fts WHERE profile_id = ?1",
        [profile_id],
    )?;
    
    conn.execute(
        "DELETE FROM xtream_movies_fts WHERE profile_id = ?1",
        [profile_id],
    )?;
    
    conn.execute(
        "DELETE FROM xtream_series_fts WHERE profile_id = ?1",
        [profile_id],
    )?;
    
    // Rebuild from main tables
    conn.execute(
        "INSERT INTO xtream_channels_fts(rowid, profile_id, stream_id, name, epg_channel_id)
         SELECT id, profile_id, stream_id, name, epg_channel_id 
         FROM xtream_channels 
         WHERE profile_id = ?1",
        [profile_id],
    )?;
    
    conn.execute(
        "INSERT INTO xtream_movies_fts(rowid, profile_id, stream_id, name, title, genre, cast, director, plot)
         SELECT id, profile_id, stream_id, name, title, genre, \"cast\", director, plot 
         FROM xtream_movies 
         WHERE profile_id = ?1",
        [profile_id],
    )?;
    
    conn.execute(
        "INSERT INTO xtream_series_fts(rowid, profile_id, series_id, name, title, genre, cast, director, plot)
         SELECT id, profile_id, series_id, name, title, genre, \"cast\", director, plot 
         FROM xtream_series 
         WHERE profile_id = ?1",
        [profile_id],
    )?;
    
    Ok(())
}

/// Prepare FTS query string with proper escaping and operators
/// 
/// Converts user query into FTS5 query syntax:
/// - Escapes special characters
/// - Adds prefix matching for partial words
/// - Handles multi-word queries with OR operator
pub fn prepare_fts_query(query: &str) -> String {
    // Remove special FTS characters and split into words
    let cleaned = query
        .replace('"', "")
        .replace('*', "")
        .replace('(', "")
        .replace(')', "")
        .replace(':', "");
    
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    
    if words.is_empty() {
        return String::new();
    }
    
    // Build FTS query with prefix matching
    // For "action movie" -> "action* OR movie*"
    words
        .iter()
        .map(|word| format!("{}*", word))
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// Calculate relevance score based on match position and field
/// 
/// Higher scores for:
/// - Exact matches vs partial matches
/// - Matches in name/title vs plot
/// - Earlier position in text
fn calculate_relevance_score(
    query: &str,
    name: &Option<String>,
    title: &Option<String>,
    plot: &Option<String>,
) -> f64 {
    let query_lower = query.to_lowercase();
    let mut score = 0.0;
    
    // Check name field (highest weight)
    if let Some(name_val) = name {
        let name_lower = name_val.to_lowercase();
        if name_lower == query_lower {
            score += 100.0; // Exact match
        } else if name_lower.starts_with(&query_lower) {
            score += 50.0; // Prefix match
        } else if name_lower.contains(&query_lower) {
            score += 25.0; // Contains match
        }
    }
    
    // Check title field (high weight)
    if let Some(title_val) = title {
        let title_lower = title_val.to_lowercase();
        if title_lower == query_lower {
            score += 80.0;
        } else if title_lower.starts_with(&query_lower) {
            score += 40.0;
        } else if title_lower.contains(&query_lower) {
            score += 20.0;
        }
    }
    
    // Check plot field (lower weight)
    if let Some(plot_val) = plot {
        let plot_lower = plot_val.to_lowercase();
        if plot_lower.contains(&query_lower) {
            score += 10.0;
        }
    }
    
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prepare_fts_query_single_word() {
        let query = prepare_fts_query("action");
        assert_eq!(query, "action*");
    }
    
    #[test]
    fn test_prepare_fts_query_multiple_words() {
        let query = prepare_fts_query("action movie");
        assert_eq!(query, "action* OR movie*");
    }
    
    #[test]
    fn test_prepare_fts_query_special_chars() {
        let query = prepare_fts_query("action: \"movie\"");
        assert_eq!(query, "action* OR movie*");
    }
    
    #[test]
    fn test_prepare_fts_query_empty() {
        let query = prepare_fts_query("");
        assert_eq!(query, "");
    }
    
    #[test]
    fn test_calculate_relevance_exact_name_match() {
        let score = calculate_relevance_score(
            "HBO",
            &Some("HBO".to_string()),
            &None,
            &None,
        );
        assert_eq!(score, 100.0);
    }
    
    #[test]
    fn test_calculate_relevance_prefix_match() {
        let score = calculate_relevance_score(
            "HBO",
            &Some("HBO Sports".to_string()),
            &None,
            &None,
        );
        assert_eq!(score, 50.0);
    }
    
    #[test]
    fn test_calculate_relevance_contains_match() {
        let score = calculate_relevance_score(
            "HBO",
            &Some("Watch HBO Now".to_string()),
            &None,
            &None,
        );
        assert_eq!(score, 25.0);
    }
    
    #[test]
    fn test_calculate_relevance_title_match() {
        let score = calculate_relevance_score(
            "action",
            &None,
            &Some("Action Movie".to_string()),
            &None,
        );
        assert_eq!(score, 40.0);
    }
    
    #[test]
    fn test_calculate_relevance_plot_match() {
        let score = calculate_relevance_score(
            "action",
            &None,
            &None,
            &Some("This is an action-packed thriller".to_string()),
        );
        assert_eq!(score, 10.0);
    }
    
    #[test]
    fn test_calculate_relevance_multiple_matches() {
        let score = calculate_relevance_score(
            "action",
            &Some("Action Channel".to_string()),
            &Some("Action Movies".to_string()),
            &Some("Watch action content".to_string()),
        );
        // 50 (prefix in name) + 40 (prefix in title) + 10 (contains in plot) = 100
        assert_eq!(score, 100.0);
    }
    
    #[test]
    fn test_calculate_relevance_no_match() {
        let score = calculate_relevance_score(
            "xyz",
            &Some("HBO".to_string()),
            &Some("Sports".to_string()),
            &Some("Watch sports".to_string()),
        );
        assert_eq!(score, 0.0);
    }
}
