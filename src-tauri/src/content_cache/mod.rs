// Content cache module for local Xtream content storage
pub mod background_scheduler;
pub mod commands;
pub mod db_performance;
pub mod db_utils;
pub mod fts;
pub mod query_optimizer;
pub mod schema;
pub mod sync_scheduler;



pub use background_scheduler::*;
pub use commands::*;
pub use db_performance::*;
pub use db_utils::*;
pub use fts::*;
pub use query_optimizer::*;
pub use schema::*;
pub use sync_scheduler::*;

/// Represents a channel from Xtream API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamChannel {
    pub stream_id: i64,
    pub num: Option<i64>,
    pub name: String,
    pub stream_type: Option<String>,
    pub stream_icon: Option<String>,
    pub thumbnail: Option<String>,
    pub epg_channel_id: Option<String>,
    pub added: Option<String>,
    pub category_id: Option<String>,
    pub custom_sid: Option<String>,
    pub tv_archive: Option<i64>,
    pub direct_source: Option<String>,
    pub tv_archive_duration: Option<i64>,
}

/// Filter options for querying channels
#[derive(Debug, Clone, Default)]
pub struct ChannelFilter {
    pub category_id: Option<String>,
    pub name_contains: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Represents a movie from Xtream API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamMovie {
    pub stream_id: i64,
    pub num: Option<i64>,
    pub name: String,
    pub title: Option<String>,
    pub year: Option<String>,
    pub stream_type: Option<String>,
    pub stream_icon: Option<String>,
    pub rating: Option<f64>,
    pub rating_5based: Option<f64>,
    pub genre: Option<String>,
    pub added: Option<String>,
    pub episode_run_time: Option<i64>,
    pub category_id: Option<String>,
    pub container_extension: Option<String>,
    pub custom_sid: Option<String>,
    pub direct_source: Option<String>,
    pub release_date: Option<String>,
    pub cast: Option<String>,
    pub director: Option<String>,
    pub plot: Option<String>,
    pub youtube_trailer: Option<String>,
}

/// Filter options for querying movies
#[derive(Debug, Clone, Default)]
pub struct MovieFilter {
    pub category_id: Option<String>,
    pub name_contains: Option<String>,
    pub genre: Option<String>,
    pub year: Option<String>,
    pub min_rating: Option<f64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Sort options for movies
#[derive(Debug, Clone)]
pub enum MovieSortBy {
    Name,
    Rating,
    Year,
    Added,
}

impl Default for MovieSortBy {
    fn default() -> Self {
        MovieSortBy::Name
    }
}

/// Sort direction
#[derive(Debug, Clone)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Asc
    }
}

/// Represents a series listing from Xtream API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamSeries {
    pub series_id: i64,
    pub num: Option<i64>,
    pub name: String,
    pub title: Option<String>,
    pub year: Option<String>,
    pub cover: Option<String>,
    pub plot: Option<String>,
    pub cast: Option<String>,
    pub director: Option<String>,
    pub genre: Option<String>,
    pub release_date: Option<String>,
    pub last_modified: Option<String>,
    pub rating: Option<String>,
    pub rating_5based: Option<f64>,
    pub episode_run_time: Option<String>,
    pub category_id: Option<String>,
}

/// Represents a season in a series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamSeason {
    pub season_number: i64,
    pub name: Option<String>,
    pub episode_count: Option<i64>,
    pub overview: Option<String>,
    pub air_date: Option<String>,
    pub cover: Option<String>,
    pub cover_big: Option<String>,
    pub vote_average: Option<f64>,
}

/// Represents an episode in a series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamEpisode {
    pub episode_id: String,
    pub season_number: i64,
    pub episode_num: String,
    pub title: Option<String>,
    pub container_extension: Option<String>,
    pub custom_sid: Option<String>,
    pub added: Option<String>,
    pub direct_source: Option<String>,
    pub info_json: Option<String>,
}

/// Complete series details with seasons and episodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamSeriesDetails {
    pub series: XtreamSeries,
    pub seasons: Vec<XtreamSeason>,
    pub episodes: Vec<XtreamEpisode>,
}

/// Filter options for querying series
#[derive(Debug, Clone, Default)]
pub struct SeriesFilter {
    pub category_id: Option<String>,
    pub name_contains: Option<String>,
    pub genre: Option<String>,
    pub year: Option<String>,
    pub min_rating: Option<f64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Represents a category for content organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamCategory {
    pub category_id: String,
    pub category_name: String,
    pub parent_id: Option<i64>,
}

/// Content type for category operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Channels,
    Movies,
    Series,
}

impl ContentType {
    /// Get the table name for this content type
    pub fn table_name(&self) -> &'static str {
        match self {
            ContentType::Channels => "xtream_channel_categories",
            ContentType::Movies => "xtream_movie_categories",
            ContentType::Series => "xtream_series_categories",
        }
    }

    /// Get the content table name for counting items
    pub fn content_table_name(&self) -> &'static str {
        match self {
            ContentType::Channels => "xtream_channels",
            ContentType::Movies => "xtream_movies",
            ContentType::Series => "xtream_series",
        }
    }
}

/// Category with item count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamCategoryWithCount {
    pub category_id: String,
    pub category_name: String,
    pub parent_id: Option<i64>,
    pub item_count: usize,
}

/// Filter options for querying categories
#[derive(Debug, Clone, Default)]
pub struct CategoryFilter {
    pub parent_id: Option<i64>,
    pub name_contains: Option<String>,
}

use crate::error::{Result, XTauriError};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Manages local content cache for Xtream data
///
/// This module provides persistent storage for Xtream content (channels, movies, series)
/// in SQLite tables, enabling fast local-first access without repeated API calls.
pub struct ContentCache {
    db: Arc<Mutex<Connection>>,
}

impl ContentCache {
    /// Create a new ContentCache instance
    ///
    /// # Arguments
    /// * `db` - Shared database connection
    ///
    /// # Returns
    /// A new ContentCache instance with initialized tables
    pub fn new(db: Arc<Mutex<Connection>>) -> Result<Self> {
        let cache = Self { db };
        cache.initialize_tables()?;
        Ok(cache)
    }

    /// Initialize all content cache tables
    ///
    /// Creates all necessary tables and indexes if they don't exist.
    /// This method is idempotent and safe to call multiple times.
    ///
    /// # Returns
    /// Ok(()) if initialization succeeds, error otherwise
    pub fn initialize_tables(&self) -> Result<()> {
        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        schema::initialize_content_cache_tables(&conn)?;

        Ok(())
    }

    /// Get a reference to the database connection
    ///
    /// This is useful for operations that need direct database access
    pub fn get_db(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.db)
    }

    /// Check if the cache is initialized for a specific profile
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to check
    ///
    /// # Returns
    /// true if the profile has sync metadata, false otherwise
    pub fn is_initialized(&self, profile_id: &str) -> Result<bool> {
        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM xtream_content_sync WHERE profile_id = ?1",
            [profile_id],
            |row| {
                let count: i32 = row.get(0)?;
                Ok(count > 0)
            },
        )?;

        Ok(exists)
    }

    /// Initialize sync metadata for a profile
    ///
    /// Creates an entry in the sync table for tracking synchronization status
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to initialize
    ///
    /// # Returns
    /// Ok(()) if initialization succeeds, error otherwise
    pub fn initialize_profile(&self, profile_id: &str) -> Result<()> {
        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        conn.execute(
            "INSERT OR IGNORE INTO xtream_content_sync (profile_id, sync_status) VALUES (?1, 'pending')",
            [profile_id],
        )?;

        // Also initialize sync settings with defaults
        conn.execute(
            "INSERT OR IGNORE INTO xtream_sync_settings (profile_id) VALUES (?1)",
            [profile_id],
        )?;

        Ok(())
    }

    /// Clear all cached content for a specific profile
    ///
    /// Removes all content data but preserves sync metadata and settings
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to clear
    ///
    /// # Returns
    /// Ok(()) if clearing succeeds, error otherwise
    pub fn clear_profile_content(&self, profile_id: &str) -> Result<()> {
        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        // Use a transaction for atomicity
        let tx = conn.unchecked_transaction()?;

        tx.execute(
            "DELETE FROM xtream_channels WHERE profile_id = ?1",
            [profile_id],
        )?;
        tx.execute(
            "DELETE FROM xtream_movies WHERE profile_id = ?1",
            [profile_id],
        )?;
        tx.execute(
            "DELETE FROM xtream_series WHERE profile_id = ?1",
            [profile_id],
        )?;
        tx.execute(
            "DELETE FROM xtream_seasons WHERE profile_id = ?1",
            [profile_id],
        )?;
        tx.execute(
            "DELETE FROM xtream_episodes WHERE profile_id = ?1",
            [profile_id],
        )?;
        tx.execute(
            "DELETE FROM xtream_channel_categories WHERE profile_id = ?1",
            [profile_id],
        )?;
        tx.execute(
            "DELETE FROM xtream_movie_categories WHERE profile_id = ?1",
            [profile_id],
        )?;
        tx.execute(
            "DELETE FROM xtream_series_categories WHERE profile_id = ?1",
            [profile_id],
        )?;

        // Reset sync status
        tx.execute(
            "UPDATE xtream_content_sync SET 
             sync_status = 'pending',
             sync_progress = 0,
             channels_count = 0,
             movies_count = 0,
             series_count = 0,
             last_sync_channels = NULL,
             last_sync_movies = NULL,
             last_sync_series = NULL,
             updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?1",
            [profile_id],
        )?;

        tx.commit()?;

        Ok(())
    }

    /// Get the total count of cached items for a profile
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to query
    ///
    /// # Returns
    /// A tuple of (channels_count, movies_count, series_count)
    pub fn get_content_counts(&self, profile_id: &str) -> Result<(usize, usize, usize)> {
        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let channels_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM xtream_channels WHERE profile_id = ?1",
            [profile_id],
            |row| row.get(0),
        )?;

        let movies_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM xtream_movies WHERE profile_id = ?1",
            [profile_id],
            |row| row.get(0),
        )?;

        let series_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM xtream_series WHERE profile_id = ?1",
            [profile_id],
            |row| row.get(0),
        )?;

        Ok((
            channels_count as usize,
            movies_count as usize,
            series_count as usize,
        ))
    }

    /// Perform database maintenance operations
    ///
    /// Runs ANALYZE and VACUUM to optimize database performance
    ///
    /// # Returns
    /// Ok(()) if maintenance succeeds, error otherwise
    pub fn perform_maintenance(&self) -> Result<()> {
        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        // Analyze tables for query optimization
        conn.execute("ANALYZE", [])?;

        // Note: VACUUM cannot be run inside a transaction
        // It should be called separately when needed

        Ok(())
    }

    /// Vacuum the database to reclaim space
    ///
    /// This should be called periodically but not too frequently
    /// as it can be an expensive operation
    ///
    /// # Returns
    /// Ok(()) if vacuum succeeds, error otherwise
    pub fn vacuum(&self) -> Result<()> {
        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        conn.execute("VACUUM", [])?;

        Ok(())
    }

    /// Create a DbPerformance instance for advanced performance operations
    ///
    /// # Arguments
    /// * `slow_query_threshold_ms` - Optional threshold for logging slow queries (default: 100ms)
    ///
    /// # Returns
    /// A new DbPerformance instance
    pub fn get_performance_manager(&self, slow_query_threshold_ms: Option<u64>) -> DbPerformance {
        DbPerformance::new(Arc::clone(&self.db), slow_query_threshold_ms)
    }

    /// Run ANALYZE on all tables to update query optimizer statistics
    ///
    /// This should be called after bulk inserts or significant data changes
    ///
    /// # Returns
    /// Ok(()) if successful, error otherwise
    pub fn analyze_tables(&self) -> Result<()> {
        let perf = self.get_performance_manager(None);
        perf.analyze_tables()
    }

    /// Check if VACUUM is recommended based on database fragmentation
    ///
    /// # Returns
    /// true if VACUUM is recommended, false otherwise
    pub fn should_vacuum(&self) -> Result<bool> {
        let perf = self.get_performance_manager(None);
        perf.should_vacuum()
    }

    /// Get database size and fragmentation statistics
    ///
    /// # Returns
    /// Tuple of (total_size_bytes, page_count, page_size, freelist_count)
    pub fn get_database_stats(&self) -> Result<(u64, i64, i64, i64)> {
        let perf = self.get_performance_manager(None);
        perf.get_database_stats()
    }

    /// Run database integrity check
    ///
    /// # Returns
    /// Ok(()) if database is healthy, error with details if corruption detected
    pub fn check_integrity(&self) -> Result<()> {
        let perf = self.get_performance_manager(None);
        perf.check_integrity()
    }

    /// Optimize database settings for performance
    ///
    /// Sets various PRAGMA settings for better performance
    ///
    /// # Returns
    /// Ok(()) if successful, error otherwise
    pub fn optimize_settings(&self) -> Result<()> {
        let perf = self.get_performance_manager(None);
        perf.optimize_settings()
    }

    // ==================== Channel Operations ====================

    /// Save channels to the cache with batch insert
    ///
    /// Uses UPSERT (INSERT OR REPLACE) to handle both new and updated channels.
    /// All operations are performed in a single transaction for atomicity.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID these channels belong to
    /// * `channels` - Vector of channels to save
    ///
    /// # Returns
    /// Number of channels successfully saved
    pub fn save_channels(&self, profile_id: &str, channels: Vec<XtreamChannel>) -> Result<usize> {
        validate_profile_id(profile_id)?;

        if channels.is_empty() {
            return Ok(0);
        }

        let mut conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let saved = batch_insert(&mut conn, "xtream_channels", &channels, |tx, channel| {
            validate_stream_id(channel.stream_id)?;

            tx.execute(
                "INSERT OR REPLACE INTO xtream_channels (
                    profile_id, stream_id, num, name, stream_type, stream_icon,
                    thumbnail, epg_channel_id, added, category_id, custom_sid,
                    tv_archive, direct_source, tv_archive_duration, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, CURRENT_TIMESTAMP)",
                params![
                    profile_id,
                    channel.stream_id,
                    channel.num,
                    channel.name,
                    channel.stream_type,
                    channel.stream_icon,
                    channel.thumbnail,
                    channel.epg_channel_id,
                    channel.added,
                    channel.category_id,
                    channel.custom_sid,
                    channel.tv_archive,
                    channel.direct_source,
                    channel.tv_archive_duration,
                ],
            )?;
            Ok(())
        })?;

        // Update sync metadata
        conn.execute(
            "UPDATE xtream_content_sync 
             SET channels_count = (SELECT COUNT(*) FROM xtream_channels WHERE profile_id = ?1),
                 last_sync_channels = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?1",
            [profile_id],
        )?;

        // Rebuild FTS index to ensure search works correctly
        // This is necessary because INSERT OR REPLACE may not trigger FTS updates properly
        fts::rebuild_fts_index(&conn, profile_id)?;

        Ok(saved)
    }

    /// Get channels from the cache with optional filtering
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to query
    /// * `filter` - Optional filter criteria
    ///
    /// # Returns
    /// Vector of channels matching the filter criteria
    pub fn get_channels(
        &self,
        profile_id: &str,
        filter: Option<ChannelFilter>,
    ) -> Result<Vec<XtreamChannel>> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();

        // Build query dynamically based on filter
        let mut query = String::from(
            "SELECT stream_id, num, name, stream_type, stream_icon, thumbnail,
                    epg_channel_id, added, category_id, custom_sid, tv_archive,
                    direct_source, tv_archive_duration
             FROM xtream_channels
             WHERE profile_id = ?1",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];

        if let Some(category_id) = &filter.category_id {
            query.push_str(" AND category_id = ?");
            params.push(Box::new(category_id.clone()));
        }

        if let Some(name_pattern) = &filter.name_contains {
            query.push_str(" AND name LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(name_pattern));
            params.push(Box::new(pattern));
        }

        query.push_str(" ORDER BY name COLLATE NOCASE");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = conn.prepare(&query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let channels = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamChannel {
                    stream_id: row.get(0)?,
                    num: row.get(1)?,
                    name: row.get(2)?,
                    stream_type: row.get(3)?,
                    stream_icon: row.get(4)?,
                    thumbnail: row.get(5)?,
                    epg_channel_id: row.get(6)?,
                    added: row.get(7)?,
                    category_id: row.get(8)?,
                    custom_sid: row.get(9)?,
                    tv_archive: row.get(10)?,
                    direct_source: row.get(11)?,
                    tv_archive_duration: row.get(12)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(channels)
    }

    /// Delete channels from the cache
    ///
    /// Can delete all channels for a profile or specific channels by stream_id
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID
    /// * `stream_ids` - Optional list of specific stream IDs to delete. If None, deletes all.
    ///
    /// # Returns
    /// Number of channels deleted
    pub fn delete_channels(&self, profile_id: &str, stream_ids: Option<Vec<i64>>) -> Result<usize> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let deleted = if let Some(ids) = stream_ids {
            if ids.is_empty() {
                return Ok(0);
            }

            // Validate all stream IDs
            for id in &ids {
                validate_stream_id(*id)?;
            }

            // Build IN clause for multiple IDs
            let placeholders = ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 2))
                .collect::<Vec<_>>()
                .join(", ");

            let query = format!(
                "DELETE FROM xtream_channels WHERE profile_id = ?1 AND stream_id IN ({})",
                placeholders
            );

            let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];
            for id in ids {
                params.push(Box::new(id));
            }

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            conn.execute(&query, param_refs.as_slice())?
        } else {
            // Delete all channels for the profile
            conn.execute(
                "DELETE FROM xtream_channels WHERE profile_id = ?1",
                [profile_id],
            )?
        };

        // Update sync metadata
        conn.execute(
            "UPDATE xtream_content_sync 
             SET channels_count = (SELECT COUNT(*) FROM xtream_channels WHERE profile_id = ?1),
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?1",
            [profile_id],
        )?;

        Ok(deleted)
    }

    /// Search channels with fuzzy matching
    ///
    /// Performs a case-insensitive fuzzy search across channel names.
    /// Results are ordered by relevance (exact matches first, then partial matches).
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to search within
    /// * `query` - The search query string
    /// * `filter` - Optional additional filter criteria (category, pagination)
    ///
    /// # Returns
    /// Vector of channels matching the search query
    pub fn search_channels(
        &self,
        profile_id: &str,
        query: &str,
        filter: Option<ChannelFilter>,
    ) -> Result<Vec<XtreamChannel>> {
        validate_profile_id(profile_id)?;

        if query.is_empty() {
            return self.get_channels(profile_id, filter);
        }

        let start_time = std::time::Instant::now();

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();

        // Build search query with fuzzy matching
        // Use LIKE for fuzzy search with wildcards
        let search_pattern = format!("%{}%", sanitize_like_pattern(query));

        let mut sql = String::from(
            "SELECT stream_id, num, name, stream_type, stream_icon, thumbnail,
                    epg_channel_id, added, category_id, custom_sid, tv_archive,
                    direct_source, tv_archive_duration,
                    CASE 
                        WHEN LOWER(name) = LOWER(?2) THEN 0
                        WHEN LOWER(name) LIKE LOWER(?2) || '%' THEN 1
                        WHEN LOWER(name) LIKE '%' || LOWER(?2) || '%' THEN 2
                        ELSE 3
                    END as relevance
             FROM xtream_channels
             WHERE profile_id = ?1 AND LOWER(name) LIKE LOWER(?3)",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(profile_id.to_string()),
            Box::new(query.to_string()),
            Box::new(search_pattern),
        ];

        if let Some(category_id) = &filter.category_id {
            sql.push_str(" AND category_id = ?");
            params.push(Box::new(category_id.clone()));
        }

        sql.push_str(" ORDER BY relevance, name COLLATE NOCASE");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = conn.prepare(&sql)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let channels = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamChannel {
                    stream_id: row.get(0)?,
                    num: row.get(1)?,
                    name: row.get(2)?,
                    stream_type: row.get(3)?,
                    stream_icon: row.get(4)?,
                    thumbnail: row.get(5)?,
                    epg_channel_id: row.get(6)?,
                    added: row.get(7)?,
                    category_id: row.get(8)?,
                    custom_sid: row.get(9)?,
                    tv_archive: row.get(10)?,
                    direct_source: row.get(11)?,
                    tv_archive_duration: row.get(12)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let duration = start_time.elapsed();

        #[cfg(debug_assertions)]
        println!(
            "[DEBUG] Channel search completed: query='{}', results={}, took {:?}",
            query,
            channels.len(),
            duration
        );

        // Warn if search is slow (target < 100ms)
        if duration.as_millis() > 100 {
            eprintln!(
                "[WARN] Slow channel search: query='{}' took {:?}",
                query, duration
            );
        }

        Ok(channels)
    }

    /// Get channel count for a specific filter
    ///
    /// Useful for pagination to know total results
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID
    /// * `filter` - Filter criteria (without pagination)
    ///
    /// # Returns
    /// Total count of channels matching the filter
    pub fn count_channels(&self, profile_id: &str, filter: Option<ChannelFilter>) -> Result<usize> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();

        let mut query = String::from("SELECT COUNT(*) FROM xtream_channels WHERE profile_id = ?1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];

        if let Some(category_id) = &filter.category_id {
            query.push_str(" AND category_id = ?");
            params.push(Box::new(category_id.clone()));
        }

        if let Some(name_pattern) = &filter.name_contains {
            query.push_str(" AND name LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(name_pattern));
            params.push(Box::new(pattern));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let count: i64 = conn.query_row(&query, param_refs.as_slice(), |row| row.get(0))?;

        Ok(count as usize)
    }

    // ==================== Movie Operations ====================

    /// Save movies to the cache with batch insert
    ///
    /// Uses UPSERT (INSERT OR REPLACE) to handle both new and updated movies.
    /// All operations are performed in a single transaction for atomicity.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID these movies belong to
    /// * `movies` - Vector of movies to save
    ///
    /// # Returns
    /// Number of movies successfully saved
    pub fn save_movies(&self, profile_id: &str, movies: Vec<XtreamMovie>) -> Result<usize> {
        validate_profile_id(profile_id)?;

        if movies.is_empty() {
            return Ok(0);
        }

        let mut conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let saved = batch_insert(&mut conn, "xtream_movies", &movies, |tx, movie| {
            validate_stream_id(movie.stream_id)?;

            tx.execute(
                "INSERT OR REPLACE INTO xtream_movies (
                    profile_id, stream_id, num, name, title, year, stream_type,
                    stream_icon, rating, rating_5based, genre, added, episode_run_time,
                    category_id, container_extension, custom_sid, direct_source,
                    release_date, cast, director, plot, youtube_trailer, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, CURRENT_TIMESTAMP)",
                params![
                    profile_id,
                    movie.stream_id,
                    movie.num,
                    movie.name,
                    movie.title,
                    movie.year,
                    movie.stream_type,
                    movie.stream_icon,
                    movie.rating,
                    movie.rating_5based,
                    movie.genre,
                    movie.added,
                    movie.episode_run_time,
                    movie.category_id,
                    movie.container_extension,
                    movie.custom_sid,
                    movie.direct_source,
                    movie.release_date,
                    movie.cast,
                    movie.director,
                    movie.plot,
                    movie.youtube_trailer,
                ],
            )?;
            Ok(())
        })?;

        // Update sync metadata
        conn.execute(
            "UPDATE xtream_content_sync 
             SET movies_count = (SELECT COUNT(*) FROM xtream_movies WHERE profile_id = ?1),
                 last_sync_movies = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?1",
            [profile_id],
        )?;

        // Rebuild FTS index to ensure search works correctly
        // This is necessary because INSERT OR REPLACE may not trigger FTS updates properly
        fts::rebuild_fts_index(&conn, profile_id)?;

        Ok(saved)
    }

    /// Get movies from the cache with optional filtering
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to query
    /// * `filter` - Optional filter criteria
    /// * `sort_by` - Optional sort field
    /// * `sort_direction` - Optional sort direction
    ///
    /// # Returns
    /// Vector of movies matching the filter criteria
    pub fn get_movies(
        &self,
        profile_id: &str,
        filter: Option<MovieFilter>,
        sort_by: Option<MovieSortBy>,
        sort_direction: Option<SortDirection>,
    ) -> Result<Vec<XtreamMovie>> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();
        let sort_by = sort_by.unwrap_or_default();
        let sort_direction = sort_direction.unwrap_or_default();

        // Build query dynamically based on filter
        let mut query = String::from(
            "SELECT stream_id, num, name, title, year, stream_type, stream_icon, \
             rating, rating_5based, genre, added, episode_run_time, category_id, \
             container_extension, custom_sid, direct_source, release_date, \
             \"cast\", director, plot, youtube_trailer \
             FROM xtream_movies \
             WHERE profile_id = ?1",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];

        if let Some(category_id) = &filter.category_id {
            query.push_str(" AND category_id = ?");
            params.push(Box::new(category_id.clone()));
        }

        if let Some(name_pattern) = &filter.name_contains {
            query.push_str(" AND name LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(name_pattern));
            params.push(Box::new(pattern));
        }

        if let Some(genre) = &filter.genre {
            query.push_str(" AND genre LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(genre));
            params.push(Box::new(pattern));
        }

        if let Some(year) = &filter.year {
            query.push_str(" AND year = ?");
            params.push(Box::new(year.clone()));
        }

        if let Some(min_rating) = filter.min_rating {
            query.push_str(" AND rating >= ?");
            params.push(Box::new(min_rating));
        }

        // Add sorting
        let sort_field = match sort_by {
            MovieSortBy::Name => "name COLLATE NOCASE",
            MovieSortBy::Rating => "rating",
            MovieSortBy::Year => "year",
            MovieSortBy::Added => "added",
        };

        let sort_dir = match sort_direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        query.push_str(&format!(" ORDER BY {} {}", sort_field, sort_dir));

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = conn.prepare(&query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let movies = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamMovie {
                    stream_id: row.get(0)?,
                    num: row.get(1)?,
                    name: row.get(2)?,
                    title: row.get(3)?,
                    year: row.get(4)?,
                    stream_type: row.get(5)?,
                    stream_icon: row.get(6)?,
                    rating: row.get(7)?,
                    rating_5based: row.get(8)?,
                    genre: row.get(9)?,
                    added: row.get(10)?,
                    episode_run_time: row.get(11)?,
                    category_id: row.get(12)?,
                    container_extension: row.get(13)?,
                    custom_sid: row.get(14)?,
                    direct_source: row.get(15)?,
                    release_date: row.get(16)?,
                    cast: row.get(17)?,
                    director: row.get(18)?,
                    plot: row.get(19)?,
                    youtube_trailer: row.get(20)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(movies)
    }

    /// Delete movies from the cache
    ///
    /// Can delete all movies for a profile or specific movies by stream_id
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID
    /// * `stream_ids` - Optional list of specific stream IDs to delete. If None, deletes all.
    ///
    /// # Returns
    /// Number of movies deleted
    pub fn delete_movies(&self, profile_id: &str, stream_ids: Option<Vec<i64>>) -> Result<usize> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let deleted = if let Some(ids) = stream_ids {
            if ids.is_empty() {
                return Ok(0);
            }

            // Validate all stream IDs
            for id in &ids {
                validate_stream_id(*id)?;
            }

            // Build IN clause for multiple IDs
            let placeholders = ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 2))
                .collect::<Vec<_>>()
                .join(", ");

            let query = format!(
                "DELETE FROM xtream_movies WHERE profile_id = ?1 AND stream_id IN ({})",
                placeholders
            );

            let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];
            for id in ids {
                params.push(Box::new(id));
            }

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            conn.execute(&query, param_refs.as_slice())?
        } else {
            // Delete all movies for the profile
            conn.execute(
                "DELETE FROM xtream_movies WHERE profile_id = ?1",
                [profile_id],
            )?
        };

        // Update sync metadata
        conn.execute(
            "UPDATE xtream_content_sync 
             SET movies_count = (SELECT COUNT(*) FROM xtream_movies WHERE profile_id = ?1),
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?1",
            [profile_id],
        )?;

        Ok(deleted)
    }

    /// Search movies with fuzzy matching
    ///
    /// Performs a case-insensitive fuzzy search across movie names, titles, and plots.
    /// Results are ordered by relevance (exact matches first, then partial matches).
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to search within
    /// * `query` - The search query string
    /// * `filter` - Optional additional filter criteria (category, genre, rating, pagination)
    /// * `sort_by` - Optional sort field
    /// * `sort_direction` - Optional sort direction
    ///
    /// # Returns
    /// Vector of movies matching the search query
    pub fn search_movies(
        &self,
        profile_id: &str,
        query: &str,
        filter: Option<MovieFilter>,
        sort_by: Option<MovieSortBy>,
        sort_direction: Option<SortDirection>,
    ) -> Result<Vec<XtreamMovie>> {
        validate_profile_id(profile_id)?;

        if query.is_empty() {
            return self.get_movies(profile_id, filter, sort_by, sort_direction);
        }

        let start_time = std::time::Instant::now();

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();
        let sort_by = sort_by.unwrap_or_default();
        let sort_direction = sort_direction.unwrap_or_default();

        // Build search query with fuzzy matching
        let search_pattern = format!("%{}%", sanitize_like_pattern(query));

        let mut sql = String::from(
            "SELECT stream_id, num, name, title, year, stream_type, stream_icon, \
             rating, rating_5based, genre, added, episode_run_time, category_id, \
             container_extension, custom_sid, direct_source, release_date, \
             \"cast\", director, plot, youtube_trailer \
             FROM xtream_movies \
             WHERE profile_id = ?1 AND (\
                 LOWER(name) LIKE LOWER(?2) OR \
                 LOWER(title) LIKE LOWER(?2) OR \
                 LOWER(plot) LIKE LOWER(?2)\
             )",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(profile_id.to_string()),
            Box::new(search_pattern.clone()),
        ];

        if let Some(category_id) = &filter.category_id {
            sql.push_str(" AND category_id = ?");
            params.push(Box::new(category_id.clone()));
        }

        if let Some(genre) = &filter.genre {
            sql.push_str(" AND genre LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(genre));
            params.push(Box::new(pattern));
        }

        if let Some(year) = &filter.year {
            sql.push_str(" AND year = ?");
            params.push(Box::new(year.clone()));
        }

        if let Some(min_rating) = filter.min_rating {
            sql.push_str(" AND rating >= ?");
            params.push(Box::new(min_rating));
        }

        // Add sorting
        let sort_field = match sort_by {
            MovieSortBy::Name => "name COLLATE NOCASE",
            MovieSortBy::Rating => "rating",
            MovieSortBy::Year => "year",
            MovieSortBy::Added => "added",
        };

        let sort_dir = match sort_direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        sql.push_str(&format!(" ORDER BY {} {}", sort_field, sort_dir));

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = conn.prepare(&sql)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let movies = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamMovie {
                    stream_id: row.get(0)?,
                    num: row.get(1)?,
                    name: row.get(2)?,
                    title: row.get(3)?,
                    year: row.get(4)?,
                    stream_type: row.get(5)?,
                    stream_icon: row.get(6)?,
                    rating: row.get(7)?,
                    rating_5based: row.get(8)?,
                    genre: row.get(9)?,
                    added: row.get(10)?,
                    episode_run_time: row.get(11)?,
                    category_id: row.get(12)?,
                    container_extension: row.get(13)?,
                    custom_sid: row.get(14)?,
                    direct_source: row.get(15)?,
                    release_date: row.get(16)?,
                    cast: row.get(17)?,
                    director: row.get(18)?,
                    plot: row.get(19)?,
                    youtube_trailer: row.get(20)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let duration = start_time.elapsed();

        #[cfg(debug_assertions)]
        println!(
            "[DEBUG] Movie search completed: query='{}', results={}, took {:?}",
            query,
            movies.len(),
            duration
        );

        // Warn if search is slow (target < 100ms)
        if duration.as_millis() > 100 {
            eprintln!(
                "[WARN] Slow movie search: query='{}' took {:?}",
                query, duration
            );
        }

        Ok(movies)
    }

    /// Get movie count for a specific filter
    ///
    /// Useful for pagination to know total results
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID
    /// * `filter` - Filter criteria (without pagination)
    ///
    /// # Returns
    /// Total count of movies matching the filter
    pub fn count_movies(&self, profile_id: &str, filter: Option<MovieFilter>) -> Result<usize> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();

        let mut query = String::from("SELECT COUNT(*) FROM xtream_movies WHERE profile_id = ?1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];

        if let Some(category_id) = &filter.category_id {
            query.push_str(" AND category_id = ?");
            params.push(Box::new(category_id.clone()));
        }

        if let Some(name_pattern) = &filter.name_contains {
            query.push_str(" AND name LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(name_pattern));
            params.push(Box::new(pattern));
        }

        if let Some(genre) = &filter.genre {
            query.push_str(" AND genre LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(genre));
            params.push(Box::new(pattern));
        }

        if let Some(year) = &filter.year {
            query.push_str(" AND year = ?");
            params.push(Box::new(year.clone()));
        }

        if let Some(min_rating) = filter.min_rating {
            query.push_str(" AND rating >= ?");
            params.push(Box::new(min_rating));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let count: i64 = conn.query_row(&query, param_refs.as_slice(), |row| row.get(0))?;

        Ok(count as usize)
    }

    // ==================== Series Operations ====================

    /// Save series listings to the cache with batch insert
    ///
    /// Uses UPSERT (INSERT OR REPLACE) to handle both new and updated series.
    /// All operations are performed in a single transaction for atomicity.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID these series belong to
    /// * `series` - Vector of series to save
    ///
    /// # Returns
    /// Number of series successfully saved
    pub fn save_series(&self, profile_id: &str, series: Vec<XtreamSeries>) -> Result<usize> {
        validate_profile_id(profile_id)?;

        if series.is_empty() {
            return Ok(0);
        }

        let mut conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let saved = batch_insert(&mut conn, "xtream_series", &series, |tx, s| {
            validate_stream_id(s.series_id)?;

            tx.execute(
                "INSERT OR REPLACE INTO xtream_series (
                    profile_id, series_id, num, name, title, year, cover, plot,
                    cast, director, genre, release_date, last_modified, rating,
                    rating_5based, episode_run_time, category_id, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, CURRENT_TIMESTAMP)",
                params![
                    profile_id,
                    s.series_id,
                    s.num,
                    s.name,
                    s.title,
                    s.year,
                    s.cover,
                    s.plot,
                    s.cast,
                    s.director,
                    s.genre,
                    s.release_date,
                    s.last_modified,
                    s.rating,
                    s.rating_5based,
                    s.episode_run_time,
                    s.category_id,
                ],
            )?;
            Ok(())
        })?;

        // Update sync metadata
        conn.execute(
            "UPDATE xtream_content_sync 
             SET series_count = (SELECT COUNT(*) FROM xtream_series WHERE profile_id = ?1),
                 last_sync_series = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?1",
            [profile_id],
        )?;

        // Rebuild FTS index to ensure search works correctly
        // This is necessary because INSERT OR REPLACE may not trigger FTS updates properly
        fts::rebuild_fts_index(&conn, profile_id)?;

        Ok(saved)
    }

    /// Save complete series details including seasons and episodes
    ///
    /// This saves the series info along with all its seasons and episodes.
    /// All operations are performed in a single transaction for atomicity.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID this series belongs to
    /// * `series_id` - The series ID
    /// * `details` - Complete series details with seasons and episodes
    ///
    /// # Returns
    /// Ok(()) if save succeeds, error otherwise
    pub fn save_series_details(
        &self,
        profile_id: &str,
        series_id: i64,
        details: XtreamSeriesDetails,
    ) -> Result<()> {
        validate_profile_id(profile_id)?;
        validate_stream_id(series_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let tx = conn.unchecked_transaction()?;

        // Save series info
        tx.execute(
            "INSERT OR REPLACE INTO xtream_series (
                profile_id, series_id, num, name, title, year, cover, plot,
                cast, director, genre, release_date, last_modified, rating,
                rating_5based, episode_run_time, category_id, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, CURRENT_TIMESTAMP)",
            params![
                profile_id,
                details.series.series_id,
                details.series.num,
                details.series.name,
                details.series.title,
                details.series.year,
                details.series.cover,
                details.series.plot,
                details.series.cast,
                details.series.director,
                details.series.genre,
                details.series.release_date,
                details.series.last_modified,
                details.series.rating,
                details.series.rating_5based,
                details.series.episode_run_time,
                details.series.category_id,
            ],
        )?;

        // Save seasons
        for season in &details.seasons {
            tx.execute(
                "INSERT OR REPLACE INTO xtream_seasons (
                    profile_id, series_id, season_number, name, episode_count,
                    overview, air_date, cover, cover_big, vote_average
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    profile_id,
                    series_id,
                    season.season_number,
                    season.name,
                    season.episode_count,
                    season.overview,
                    season.air_date,
                    season.cover,
                    season.cover_big,
                    season.vote_average,
                ],
            )?;
        }

        // Save episodes
        for episode in &details.episodes {
            tx.execute(
                "INSERT OR REPLACE INTO xtream_episodes (
                    profile_id, series_id, episode_id, season_number, episode_num,
                    title, container_extension, custom_sid, added, direct_source, info_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    profile_id,
                    series_id,
                    episode.episode_id,
                    episode.season_number,
                    episode.episode_num,
                    episode.title,
                    episode.container_extension,
                    episode.custom_sid,
                    episode.added,
                    episode.direct_source,
                    episode.info_json,
                ],
            )?;
        }

        tx.commit()?;

        Ok(())
    }

    /// Get series from the cache with optional filtering
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to query
    /// * `filter` - Optional filter criteria
    ///
    /// # Returns
    /// Vector of series matching the filter criteria
    pub fn get_series(
        &self,
        profile_id: &str,
        filter: Option<SeriesFilter>,
    ) -> Result<Vec<XtreamSeries>> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();

        // Build query dynamically based on filter
        let mut query = String::from(
            "SELECT series_id, num, name, title, year, cover, plot, \"cast\", director,
                    genre, release_date, last_modified, rating, rating_5based,
                    episode_run_time, category_id
             FROM xtream_series
             WHERE profile_id = ?1",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];

        if let Some(category_id) = &filter.category_id {
            query.push_str(" AND category_id = ?");
            params.push(Box::new(category_id.clone()));
        }

        if let Some(name_pattern) = &filter.name_contains {
            query.push_str(" AND name LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(name_pattern));
            params.push(Box::new(pattern));
        }

        if let Some(genre) = &filter.genre {
            query.push_str(" AND genre LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(genre));
            params.push(Box::new(pattern));
        }

        if let Some(year) = &filter.year {
            query.push_str(" AND year = ?");
            params.push(Box::new(year.clone()));
        }

        if let Some(min_rating) = filter.min_rating {
            query.push_str(" AND rating_5based >= ?");
            params.push(Box::new(min_rating));
        }

        query.push_str(" ORDER BY name COLLATE NOCASE");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = conn.prepare(&query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let series = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamSeries {
                    series_id: row.get(0)?,
                    num: row.get(1)?,
                    name: row.get(2)?,
                    title: row.get(3)?,
                    year: row.get(4)?,
                    cover: row.get(5)?,
                    plot: row.get(6)?,
                    cast: row.get(7)?,
                    director: row.get(8)?,
                    genre: row.get(9)?,
                    release_date: row.get(10)?,
                    last_modified: row.get(11)?,
                    rating: row.get(12)?,
                    rating_5based: row.get(13)?,
                    episode_run_time: row.get(14)?,
                    category_id: row.get(15)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(series)
    }

    /// Delete series from the cache
    ///
    /// Can delete all series for a profile or specific series by series_id.
    /// Also deletes associated seasons and episodes.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID
    /// * `series_ids` - Optional list of specific series IDs to delete. If None, deletes all.
    ///
    /// # Returns
    /// Number of series deleted
    pub fn delete_series(&self, profile_id: &str, series_ids: Option<Vec<i64>>) -> Result<usize> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let tx = conn.unchecked_transaction()?;

        let deleted = if let Some(ids) = series_ids {
            if ids.is_empty() {
                return Ok(0);
            }

            // Validate all series IDs
            for id in &ids {
                validate_stream_id(*id)?;
            }

            // Build IN clause for multiple IDs
            let placeholders = ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 2))
                .collect::<Vec<_>>()
                .join(", ");

            // Delete episodes first
            let query = format!(
                "DELETE FROM xtream_episodes WHERE profile_id = ?1 AND series_id IN ({})",
                placeholders
            );

            let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];
            for id in &ids {
                params.push(Box::new(*id));
            }

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            tx.execute(&query, param_refs.as_slice())?;

            // Delete seasons
            let query = format!(
                "DELETE FROM xtream_seasons WHERE profile_id = ?1 AND series_id IN ({})",
                placeholders
            );

            let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];
            for id in &ids {
                params.push(Box::new(*id));
            }

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            tx.execute(&query, param_refs.as_slice())?;

            // Delete series
            let query = format!(
                "DELETE FROM xtream_series WHERE profile_id = ?1 AND series_id IN ({})",
                placeholders
            );

            let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];
            for id in ids {
                params.push(Box::new(id));
            }

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            tx.execute(&query, param_refs.as_slice())?
        } else {
            // Delete all episodes for the profile
            tx.execute(
                "DELETE FROM xtream_episodes WHERE profile_id = ?1",
                [profile_id],
            )?;

            // Delete all seasons for the profile
            tx.execute(
                "DELETE FROM xtream_seasons WHERE profile_id = ?1",
                [profile_id],
            )?;

            // Delete all series for the profile
            tx.execute(
                "DELETE FROM xtream_series WHERE profile_id = ?1",
                [profile_id],
            )?
        };

        // Update sync metadata
        tx.execute(
            "UPDATE xtream_content_sync 
             SET series_count = (SELECT COUNT(*) FROM xtream_series WHERE profile_id = ?1),
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?1",
            [profile_id],
        )?;

        tx.commit()?;

        Ok(deleted)
    }

    /// Get complete series details including seasons and episodes
    ///
    /// This retrieves the series info along with all its seasons and episodes
    /// in a single efficient query with joins.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to query
    /// * `series_id` - The series ID to get details for
    ///
    /// # Returns
    /// Complete series details with seasons and episodes
    pub fn get_series_details(
        &self,
        profile_id: &str,
        series_id: i64,
    ) -> Result<XtreamSeriesDetails> {
        validate_profile_id(profile_id)?;
        validate_stream_id(series_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        // Get series info
        let series = conn
            .query_row(
                "SELECT series_id, num, name, title, year, cover, plot, \"cast\", director,
                    genre, release_date, last_modified, rating, rating_5based,
                    episode_run_time, category_id
             FROM xtream_series
             WHERE profile_id = ?1 AND series_id = ?2",
                params![profile_id, series_id],
                |row| {
                    Ok(XtreamSeries {
                        series_id: row.get(0)?,
                        num: row.get(1)?,
                        name: row.get(2)?,
                        title: row.get(3)?,
                        year: row.get(4)?,
                        cover: row.get(5)?,
                        plot: row.get(6)?,
                        cast: row.get(7)?,
                        director: row.get(8)?,
                        genre: row.get(9)?,
                        release_date: row.get(10)?,
                        last_modified: row.get(11)?,
                        rating: row.get(12)?,
                        rating_5based: row.get(13)?,
                        episode_run_time: row.get(14)?,
                        category_id: row.get(15)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => XTauriError::content_cache(format!(
                    "Series {} not found for profile {}",
                    series_id, profile_id
                )),
                _ => XTauriError::from(e),
            })?;

        // Get seasons
        let mut stmt = conn.prepare(
            "SELECT season_number, name, episode_count, overview, air_date,
                    cover, cover_big, vote_average
             FROM xtream_seasons
             WHERE profile_id = ?1 AND series_id = ?2
             ORDER BY season_number",
        )?;

        let seasons = stmt
            .query_map(params![profile_id, series_id], |row| {
                Ok(XtreamSeason {
                    season_number: row.get(0)?,
                    name: row.get(1)?,
                    episode_count: row.get(2)?,
                    overview: row.get(3)?,
                    air_date: row.get(4)?,
                    cover: row.get(5)?,
                    cover_big: row.get(6)?,
                    vote_average: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        // Get episodes
        let mut stmt = conn.prepare(
            "SELECT episode_id, season_number, episode_num, title,
                    container_extension, custom_sid, added, direct_source, info_json
             FROM xtream_episodes
             WHERE profile_id = ?1 AND series_id = ?2
             ORDER BY season_number, CAST(episode_num AS INTEGER)",
        )?;

        let episodes = stmt
            .query_map(params![profile_id, series_id], |row| {
                Ok(XtreamEpisode {
                    episode_id: row.get(0)?,
                    season_number: row.get(1)?,
                    episode_num: row.get(2)?,
                    title: row.get(3)?,
                    container_extension: row.get(4)?,
                    custom_sid: row.get(5)?,
                    added: row.get(6)?,
                    direct_source: row.get(7)?,
                    info_json: row.get(8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(XtreamSeriesDetails {
            series,
            seasons,
            episodes,
        })
    }

    /// Get seasons for a specific series
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to query
    /// * `series_id` - The series ID
    ///
    /// # Returns
    /// Vector of seasons for the series
    pub fn get_seasons(&self, profile_id: &str, series_id: i64) -> Result<Vec<XtreamSeason>> {
        validate_profile_id(profile_id)?;
        validate_stream_id(series_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let mut stmt = conn.prepare(
            "SELECT season_number, name, episode_count, overview, air_date,
                    cover, cover_big, vote_average
             FROM xtream_seasons
             WHERE profile_id = ?1 AND series_id = ?2
             ORDER BY season_number",
        )?;

        let seasons = stmt
            .query_map(params![profile_id, series_id], |row| {
                Ok(XtreamSeason {
                    season_number: row.get(0)?,
                    name: row.get(1)?,
                    episode_count: row.get(2)?,
                    overview: row.get(3)?,
                    air_date: row.get(4)?,
                    cover: row.get(5)?,
                    cover_big: row.get(6)?,
                    vote_average: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(seasons)
    }

    /// Get episodes for a specific series and season
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to query
    /// * `series_id` - The series ID
    /// * `season_number` - Optional season number to filter by
    ///
    /// # Returns
    /// Vector of episodes
    pub fn get_episodes(
        &self,
        profile_id: &str,
        series_id: i64,
        season_number: Option<i64>,
    ) -> Result<Vec<XtreamEpisode>> {
        validate_profile_id(profile_id)?;
        validate_stream_id(series_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let (query, params): (String, Vec<Box<dyn rusqlite::ToSql>>) =
            if let Some(season) = season_number {
                (
                    "SELECT episode_id, season_number, episode_num, title,
                        container_extension, custom_sid, added, direct_source, info_json
                 FROM xtream_episodes
                 WHERE profile_id = ?1 AND series_id = ?2 AND season_number = ?3
                 ORDER BY CAST(episode_num AS INTEGER)"
                        .to_string(),
                    vec![
                        Box::new(profile_id.to_string()),
                        Box::new(series_id),
                        Box::new(season),
                    ],
                )
            } else {
                (
                    "SELECT episode_id, season_number, episode_num, title,
                        container_extension, custom_sid, added, direct_source, info_json
                 FROM xtream_episodes
                 WHERE profile_id = ?1 AND series_id = ?2
                 ORDER BY season_number, CAST(episode_num AS INTEGER)"
                        .to_string(),
                    vec![Box::new(profile_id.to_string()), Box::new(series_id)],
                )
            };

        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let episodes = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamEpisode {
                    episode_id: row.get(0)?,
                    season_number: row.get(1)?,
                    episode_num: row.get(2)?,
                    title: row.get(3)?,
                    container_extension: row.get(4)?,
                    custom_sid: row.get(5)?,
                    added: row.get(6)?,
                    direct_source: row.get(7)?,
                    info_json: row.get(8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(episodes)
    }

    // ==================== Full-Text Search Operations ====================

    /// Search channels using FTS5 for fast fuzzy search
    ///
    /// Uses SQLite FTS5 virtual tables for high-performance full-text search
    /// with relevance scoring. Falls back to regular LIKE search if FTS is not available.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to search within
    /// * `query` - The search query string
    /// * `filter` - Optional additional filter criteria (category, pagination)
    ///
    /// # Returns
    /// Vector of channels matching the search query, ordered by relevance
    pub fn fts_search_channels(
        &self,
        profile_id: &str,
        query: &str,
        filter: Option<ChannelFilter>,
    ) -> Result<Vec<XtreamChannel>> {
        validate_profile_id(profile_id)?;

        if query.is_empty() {
            return self.get_channels(profile_id, filter);
        }

        let start_time = std::time::Instant::now();

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();

        // Prepare FTS query
        let fts_query = fts::prepare_fts_query(query);

        if fts_query.is_empty() {
            return self.get_channels(profile_id, Some(filter));
        }

        // Build FTS search query
        let mut sql = String::from(
            "SELECT c.stream_id, c.num, c.name, c.stream_type, c.stream_icon, c.thumbnail,
                    c.epg_channel_id, c.added, c.category_id, c.custom_sid, c.tv_archive,
                    c.direct_source, c.tv_archive_duration,
                    fts.rank
             FROM xtream_channels c
             INNER JOIN xtream_channels_fts fts ON c.id = fts.rowid
             WHERE fts.xtream_channels_fts MATCH ?1 AND c.profile_id = ?2",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(fts_query), Box::new(profile_id.to_string())];

        if let Some(category_id) = &filter.category_id {
            sql.push_str(" AND c.category_id = ?");
            params.push(Box::new(category_id.clone()));
        }

        // Order by FTS rank (lower rank = better match)
        sql.push_str(" ORDER BY fts.rank");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        } else {
            // Default limit for FTS to prevent huge result sets
            sql.push_str(" LIMIT 1000");
        }

        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = conn.prepare(&sql)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let channels = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamChannel {
                    stream_id: row.get(0)?,
                    num: row.get(1)?,
                    name: row.get(2)?,
                    stream_type: row.get(3)?,
                    stream_icon: row.get(4)?,
                    thumbnail: row.get(5)?,
                    epg_channel_id: row.get(6)?,
                    added: row.get(7)?,
                    category_id: row.get(8)?,
                    custom_sid: row.get(9)?,
                    tv_archive: row.get(10)?,
                    direct_source: row.get(11)?,
                    tv_archive_duration: row.get(12)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let duration = start_time.elapsed();

        #[cfg(debug_assertions)]
        println!(
            "[DEBUG] FTS channel search completed: query='{}', results={}, took {:?}",
            query,
            channels.len(),
            duration
        );

        // Warn if search is slow (target < 150ms)
        if duration.as_millis() > 150 {
            eprintln!(
                "[WARN] Slow FTS channel search: query='{}' took {:?}",
                query, duration
            );
        }

        Ok(channels)
    }

    /// Search movies using FTS5 for fast fuzzy search
    ///
    /// Uses SQLite FTS5 virtual tables for high-performance full-text search
    /// across name, title, genre, cast, director, and plot fields.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to search within
    /// * `query` - The search query string
    /// * `filter` - Optional additional filter criteria
    ///
    /// # Returns
    /// Vector of movies matching the search query, ordered by relevance
    pub fn fts_search_movies(
        &self,
        profile_id: &str,
        query: &str,
        filter: Option<MovieFilter>,
    ) -> Result<Vec<XtreamMovie>> {
        validate_profile_id(profile_id)?;

        if query.is_empty() {
            return self.get_movies(profile_id, filter, None, None);
        }

        let start_time = std::time::Instant::now();

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();

        // Prepare FTS query
        let fts_query = fts::prepare_fts_query(query);

        if fts_query.is_empty() {
            return self.get_movies(profile_id, Some(filter), None, None);
        }

        // Build FTS search query
        let mut sql = String::from(
            "SELECT m.stream_id, m.num, m.name, m.title, m.year, m.stream_type, m.stream_icon,
                    m.rating, m.rating_5based, m.genre, m.added, m.episode_run_time, m.category_id,
                    m.container_extension, m.custom_sid, m.direct_source, m.release_date,
                    m.cast, m.director, m.plot, m.youtube_trailer,
                    fts.rank
             FROM xtream_movies m
             INNER JOIN xtream_movies_fts fts ON m.id = fts.rowid
             WHERE fts.xtream_movies_fts MATCH ?1 AND m.profile_id = ?2",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(fts_query), Box::new(profile_id.to_string())];

        if let Some(category_id) = &filter.category_id {
            sql.push_str(" AND m.category_id = ?");
            params.push(Box::new(category_id.clone()));
        }

        if let Some(genre) = &filter.genre {
            sql.push_str(" AND m.genre LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(genre));
            params.push(Box::new(pattern));
        }

        if let Some(year) = &filter.year {
            sql.push_str(" AND m.year = ?");
            params.push(Box::new(year.clone()));
        }

        if let Some(min_rating) = filter.min_rating {
            sql.push_str(" AND m.rating >= ?");
            params.push(Box::new(min_rating));
        }

        // Order by FTS rank (lower rank = better match)
        sql.push_str(" ORDER BY fts.rank");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        } else {
            // Default limit for FTS to prevent huge result sets
            sql.push_str(" LIMIT 1000");
        }

        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = conn.prepare(&sql)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let movies = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamMovie {
                    stream_id: row.get(0)?,
                    num: row.get(1)?,
                    name: row.get(2)?,
                    title: row.get(3)?,
                    year: row.get(4)?,
                    stream_type: row.get(5)?,
                    stream_icon: row.get(6)?,
                    rating: row.get(7)?,
                    rating_5based: row.get(8)?,
                    genre: row.get(9)?,
                    added: row.get(10)?,
                    episode_run_time: row.get(11)?,
                    category_id: row.get(12)?,
                    container_extension: row.get(13)?,
                    custom_sid: row.get(14)?,
                    direct_source: row.get(15)?,
                    release_date: row.get(16)?,
                    cast: row.get(17)?,
                    director: row.get(18)?,
                    plot: row.get(19)?,
                    youtube_trailer: row.get(20)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let duration = start_time.elapsed();

        #[cfg(debug_assertions)]
        println!(
            "[DEBUG] FTS movie search completed: query='{}', results={}, took {:?}",
            query,
            movies.len(),
            duration
        );

        // Warn if search is slow (target < 150ms)
        if duration.as_millis() > 150 {
            eprintln!(
                "[WARN] Slow FTS movie search: query='{}' took {:?}",
                query, duration
            );
        }

        Ok(movies)
    }

    /// Search series using FTS5 for fast fuzzy search
    ///
    /// Uses SQLite FTS5 virtual tables for high-performance full-text search
    /// across name, title, genre, cast, director, and plot fields.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to search within
    /// * `query` - The search query string
    /// * `filter` - Optional additional filter criteria
    ///
    /// # Returns
    /// Vector of series matching the search query, ordered by relevance
    pub fn fts_search_series(
        &self,
        profile_id: &str,
        query: &str,
        filter: Option<SeriesFilter>,
    ) -> Result<Vec<XtreamSeries>> {
        validate_profile_id(profile_id)?;

        if query.is_empty() {
            return self.get_series(profile_id, filter);
        }

        let start_time = std::time::Instant::now();

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();

        // Prepare FTS query
        let fts_query = fts::prepare_fts_query(query);

        if fts_query.is_empty() {
            return self.get_series(profile_id, Some(filter));
        }

        // Build FTS search query
        let mut sql = String::from(
            "SELECT s.series_id, s.num, s.name, s.title, s.year, s.cover, s.plot, s.cast, s.director,
                    s.genre, s.release_date, s.last_modified, s.rating, s.rating_5based,
                    s.episode_run_time, s.category_id,
                    fts.rank
             FROM xtream_series s
             INNER JOIN xtream_series_fts fts ON s.id = fts.rowid
             WHERE fts.xtream_series_fts MATCH ?1 AND s.profile_id = ?2"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(fts_query), Box::new(profile_id.to_string())];

        if let Some(category_id) = &filter.category_id {
            sql.push_str(" AND s.category_id = ?");
            params.push(Box::new(category_id.clone()));
        }

        if let Some(genre) = &filter.genre {
            sql.push_str(" AND s.genre LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(genre));
            params.push(Box::new(pattern));
        }

        if let Some(year) = &filter.year {
            sql.push_str(" AND s.year = ?");
            params.push(Box::new(year.clone()));
        }

        if let Some(min_rating) = filter.min_rating {
            sql.push_str(" AND s.rating_5based >= ?");
            params.push(Box::new(min_rating));
        }

        // Order by FTS rank (lower rank = better match)
        sql.push_str(" ORDER BY fts.rank");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        } else {
            // Default limit for FTS to prevent huge result sets
            sql.push_str(" LIMIT 1000");
        }

        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = conn.prepare(&sql)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let series = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamSeries {
                    series_id: row.get(0)?,
                    num: row.get(1)?,
                    name: row.get(2)?,
                    title: row.get(3)?,
                    year: row.get(4)?,
                    cover: row.get(5)?,
                    plot: row.get(6)?,
                    cast: row.get(7)?,
                    director: row.get(8)?,
                    genre: row.get(9)?,
                    release_date: row.get(10)?,
                    last_modified: row.get(11)?,
                    rating: row.get(12)?,
                    rating_5based: row.get(13)?,
                    episode_run_time: row.get(14)?,
                    category_id: row.get(15)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let duration = start_time.elapsed();

        #[cfg(debug_assertions)]
        println!(
            "[DEBUG] FTS series search completed: query='{}', results={}, took {:?}",
            query,
            series.len(),
            duration
        );

        // Warn if search is slow (target < 150ms)
        if duration.as_millis() > 150 {
            eprintln!(
                "[WARN] Slow FTS series search: query='{}' took {:?}",
                query, duration
            );
        }

        Ok(series)
    }

    /// Rebuild FTS index for a specific profile
    ///
    /// This should be called after bulk inserts or when FTS tables get out of sync.
    /// Useful for maintenance or after data migration.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to rebuild index for
    ///
    /// # Returns
    /// Ok(()) if rebuild succeeds, error otherwise
    pub fn rebuild_fts_index(&self, profile_id: &str) -> Result<()> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        fts::rebuild_fts_index(&conn, profile_id)?;

        Ok(())
    }

    // ==================== Category Operations ====================

    /// Save categories to the cache with batch insert
    ///
    /// Uses UPSERT (INSERT OR REPLACE) to handle both new and updated categories.
    /// All operations are performed in a single transaction for atomicity.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID these categories belong to
    /// * `content_type` - The type of content (Channels, Movies, or Series)
    /// * `categories` - Vector of categories to save
    ///
    /// # Returns
    /// Number of categories successfully saved
    pub fn save_categories(
        &self,
        profile_id: &str,
        content_type: ContentType,
        categories: Vec<XtreamCategory>,
    ) -> Result<usize> {
        validate_profile_id(profile_id)?;

        if categories.is_empty() {
            return Ok(0);
        }

        let mut conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let table_name = content_type.table_name();

        let saved = batch_insert(&mut conn, table_name, &categories, |tx, category| {
            // Validate category_id is not empty
            if category.category_id.trim().is_empty() {
                return Err(XTauriError::profile_validation(
                    "category_id cannot be empty",
                ));
            }

            let query = format!(
                "INSERT OR REPLACE INTO {} (profile_id, category_id, category_name, parent_id) 
                 VALUES (?1, ?2, ?3, ?4)",
                table_name
            );

            tx.execute(
                &query,
                params![
                    profile_id,
                    category.category_id,
                    category.category_name,
                    category.parent_id,
                ],
            )?;
            Ok(())
        })?;

        Ok(saved)
    }

    /// Get categories from the cache with optional filtering
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to query
    /// * `content_type` - The type of content (Channels, Movies, or Series)
    /// * `filter` - Optional filter criteria
    ///
    /// # Returns
    /// Vector of categories matching the filter criteria
    pub fn get_categories(
        &self,
        profile_id: &str,
        content_type: ContentType,
        filter: Option<CategoryFilter>,
    ) -> Result<Vec<XtreamCategory>> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();
        let table_name = content_type.table_name();

        // Build query dynamically based on filter
        let mut query = format!(
            "SELECT category_id, category_name, parent_id 
             FROM {} 
             WHERE profile_id = ?1",
            table_name
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];

        if let Some(parent_id) = filter.parent_id {
            query.push_str(" AND parent_id = ?");
            params.push(Box::new(parent_id));
        }

        if let Some(name_pattern) = &filter.name_contains {
            query.push_str(" AND category_name LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(name_pattern));
            params.push(Box::new(pattern));
        }

        query.push_str(" ORDER BY category_name COLLATE NOCASE");

        let mut stmt = conn.prepare(&query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let categories = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamCategory {
                    category_id: row.get(0)?,
                    category_name: row.get(1)?,
                    parent_id: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(categories)
    }

    /// Get categories with item counts
    ///
    /// Returns categories along with the count of items in each category.
    /// This is useful for displaying category lists with item counts in the UI.
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID to query
    /// * `content_type` - The type of content (Channels, Movies, or Series)
    /// * `filter` - Optional filter criteria
    ///
    /// # Returns
    /// Vector of categories with item counts
    pub fn get_categories_with_counts(
        &self,
        profile_id: &str,
        content_type: ContentType,
        filter: Option<CategoryFilter>,
    ) -> Result<Vec<XtreamCategoryWithCount>> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();
        let table_name = content_type.table_name();
        let content_table = content_type.content_table_name();

        // Build query with LEFT JOIN to count items
        let mut query = format!(
            "SELECT c.category_id, c.category_name, c.parent_id, 
                    COUNT(ct.id) as item_count
             FROM {} c
             LEFT JOIN {} ct ON c.profile_id = ct.profile_id AND c.category_id = ct.category_id
             WHERE c.profile_id = ?1",
            table_name, content_table
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];

        if let Some(parent_id) = filter.parent_id {
            query.push_str(" AND c.parent_id = ?");
            params.push(Box::new(parent_id));
        }

        if let Some(name_pattern) = &filter.name_contains {
            query.push_str(" AND c.category_name LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(name_pattern));
            params.push(Box::new(pattern));
        }

        query.push_str(" GROUP BY c.category_id, c.category_name, c.parent_id");
        query.push_str(" ORDER BY c.category_name COLLATE NOCASE");

        let mut stmt = conn.prepare(&query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let categories = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(XtreamCategoryWithCount {
                    category_id: row.get(0)?,
                    category_name: row.get(1)?,
                    parent_id: row.get(2)?,
                    item_count: row.get::<_, i64>(3)? as usize,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(categories)
    }

    /// Delete categories from the cache
    ///
    /// Can delete all categories for a profile or specific categories by category_id
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID
    /// * `content_type` - The type of content (Channels, Movies, or Series)
    /// * `category_ids` - Optional list of specific category IDs to delete. If None, deletes all.
    ///
    /// # Returns
    /// Number of categories deleted
    pub fn delete_categories(
        &self,
        profile_id: &str,
        content_type: ContentType,
        category_ids: Option<Vec<String>>,
    ) -> Result<usize> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let table_name = content_type.table_name();

        let deleted = if let Some(ids) = category_ids {
            if ids.is_empty() {
                return Ok(0);
            }

            // Validate all category IDs
            for id in &ids {
                if id.trim().is_empty() {
                    return Err(XTauriError::profile_validation(
                        "category_id cannot be empty",
                    ));
                }
            }

            // Build IN clause for multiple IDs
            let placeholders = ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 2))
                .collect::<Vec<_>>()
                .join(", ");

            let query = format!(
                "DELETE FROM {} WHERE profile_id = ?1 AND category_id IN ({})",
                table_name, placeholders
            );

            let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];
            for id in ids {
                params.push(Box::new(id));
            }

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            conn.execute(&query, param_refs.as_slice())?
        } else {
            // Delete all categories for the profile
            let query = format!("DELETE FROM {} WHERE profile_id = ?1", table_name);
            conn.execute(&query, [profile_id])?
        };

        Ok(deleted)
    }

    /// Get category count for a specific filter
    ///
    /// Useful for pagination to know total results
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID
    /// * `content_type` - The type of content (Channels, Movies, or Series)
    /// * `filter` - Filter criteria (without pagination)
    ///
    /// # Returns
    /// Total count of categories matching the filter
    pub fn count_categories(
        &self,
        profile_id: &str,
        content_type: ContentType,
        filter: Option<CategoryFilter>,
    ) -> Result<usize> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let filter = filter.unwrap_or_default();
        let table_name = content_type.table_name();

        let mut query = format!("SELECT COUNT(*) FROM {} WHERE profile_id = ?1", table_name);
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];

        if let Some(parent_id) = filter.parent_id {
            query.push_str(" AND parent_id = ?");
            params.push(Box::new(parent_id));
        }

        if let Some(name_pattern) = &filter.name_contains {
            query.push_str(" AND category_name LIKE ?");
            let pattern = format!("%{}%", sanitize_like_pattern(name_pattern));
            params.push(Box::new(pattern));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let count: i64 = conn.query_row(&query, param_refs.as_slice(), |row| row.get(0))?;

        Ok(count as usize)
    }

    // ==================== Incremental Sync Support Methods ====================

    /// Get all content IDs for a specific content type
    ///
    /// Used for incremental sync to compare with server content
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID
    /// * `content_type` - Type of content ("channels", "movies", or "series")
    ///
    /// # Returns
    /// Vector of content IDs currently in the cache
    pub fn get_content_ids(&self, profile_id: &str, content_type: &str) -> Result<Vec<i64>> {
        validate_profile_id(profile_id)?;

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let (table, id_column) = match content_type {
            "channels" => ("xtream_channels", "stream_id"),
            "movies" => ("xtream_movies", "stream_id"),
            "series" => ("xtream_series", "series_id"),
            _ => {
                return Err(XTauriError::internal(format!(
                    "Invalid content type: {}",
                    content_type
                )))
            }
        };

        let query = format!("SELECT {} FROM {} WHERE profile_id = ?1", id_column, table);

        let mut stmt = conn.prepare(&query)?;
        let ids = stmt
            .query_map([profile_id], |row| row.get::<_, i64>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(ids)
    }

    /// Delete content by IDs for a specific content type
    ///
    /// Used for incremental sync to remove deleted items
    ///
    /// # Arguments
    /// * `profile_id` - The profile ID
    /// * `content_type` - Type of content ("channels", "movies", or "series")
    /// * `ids` - Vector of IDs to delete
    ///
    /// # Returns
    /// Number of items deleted
    pub fn delete_content_by_ids(
        &self,
        profile_id: &str,
        content_type: &str,
        ids: &[i64],
    ) -> Result<usize> {
        validate_profile_id(profile_id)?;

        if ids.is_empty() {
            return Ok(0);
        }

        let conn = self
            .db
            .lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;

        let (table, id_column) = match content_type {
            "channels" => ("xtream_channels", "stream_id"),
            "movies" => ("xtream_movies", "stream_id"),
            "series" => ("xtream_series", "series_id"),
            _ => {
                return Err(XTauriError::internal(format!(
                    "Invalid content type: {}",
                    content_type
                )))
            }
        };

        // Build IN clause for multiple IDs
        let placeholders = ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 2))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "DELETE FROM {} WHERE profile_id = ?1 AND {} IN ({})",
            table, id_column, placeholders
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(profile_id.to_string())];
        for id in ids {
            params.push(Box::new(*id));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let deleted = conn.execute(&query, param_refs.as_slice())?;

        // Update sync metadata count
        let count_column = match content_type {
            "channels" => "channels_count",
            "movies" => "movies_count",
            "series" => "series_count",
            _ => return Ok(deleted),
        };

        let update_query = format!(
            "UPDATE xtream_content_sync 
             SET {} = (SELECT COUNT(*) FROM {} WHERE profile_id = ?1),
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?1",
            count_column, table
        );

        conn.execute(&update_query, [profile_id])?;

        Ok(deleted)
    }
}
