use crate::error::{Result, XTauriError};
use rusqlite::Connection;

/// Database schema version
pub const SCHEMA_VERSION: i32 = 1;

/// Initialize all content cache tables
pub fn initialize_content_cache_tables(conn: &Connection) -> Result<()> {
    // Check current schema version
    let current_version = get_schema_version(conn)?;
    
    if current_version == 0 {
        // Fresh install - create all tables
        create_all_tables(conn)?;
        
        // Initialize FTS tables
        crate::content_cache::fts::initialize_fts_tables(conn)?;
        
        set_schema_version(conn, SCHEMA_VERSION)?;
    } else if current_version < SCHEMA_VERSION {
        // Run migrations
        run_migrations(conn, current_version, SCHEMA_VERSION)?;
    }
    
    Ok(())
}

/// Create all content cache tables
fn create_all_tables(conn: &Connection) -> Result<()> {
    // Create channels table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_channels (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            stream_id INTEGER NOT NULL,
            num INTEGER,
            name TEXT NOT NULL,
            stream_type TEXT,
            stream_icon TEXT,
            thumbnail TEXT,
            epg_channel_id TEXT,
            added TEXT,
            category_id TEXT,
            custom_sid TEXT,
            tv_archive INTEGER DEFAULT 0,
            direct_source TEXT,
            tv_archive_duration INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
            UNIQUE(profile_id, stream_id)
        )",
        [],
    )?;
    
    // Create indexes for channels
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channels_profile ON xtream_channels(profile_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channels_category ON xtream_channels(category_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channels_name ON xtream_channels(name COLLATE NOCASE)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channels_stream_id ON xtream_channels(stream_id)",
        [],
    )?;
    
    // Create movies table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_movies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            stream_id INTEGER NOT NULL,
            num INTEGER,
            name TEXT NOT NULL,
            title TEXT,
            year TEXT,
            stream_type TEXT,
            stream_icon TEXT,
            rating REAL,
            rating_5based REAL,
            genre TEXT,
            added TEXT,
            episode_run_time INTEGER,
            category_id TEXT,
            container_extension TEXT,
            custom_sid TEXT,
            direct_source TEXT,
            release_date TEXT,
            cast TEXT,
            director TEXT,
            plot TEXT,
            youtube_trailer TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
            UNIQUE(profile_id, stream_id)
        )",
        [],
    )?;
    
    // Create indexes for movies
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_movies_profile ON xtream_movies(profile_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_movies_category ON xtream_movies(category_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_movies_name ON xtream_movies(name COLLATE NOCASE)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_movies_rating ON xtream_movies(rating DESC)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_movies_year ON xtream_movies(year DESC)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_movies_genre ON xtream_movies(genre)",
        [],
    )?;
    
    // Create series table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_series (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            series_id INTEGER NOT NULL,
            num INTEGER,
            name TEXT NOT NULL,
            title TEXT,
            year TEXT,
            cover TEXT,
            plot TEXT,
            cast TEXT,
            director TEXT,
            genre TEXT,
            release_date TEXT,
            last_modified TEXT,
            rating TEXT,
            rating_5based REAL,
            episode_run_time TEXT,
            category_id TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
            UNIQUE(profile_id, series_id)
        )",
        [],
    )?;
    
    // Create indexes for series
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_series_profile ON xtream_series(profile_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_series_category ON xtream_series(category_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_series_name ON xtream_series(name COLLATE NOCASE)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_series_rating ON xtream_series(rating_5based DESC)",
        [],
    )?;
    
    // Create seasons table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_seasons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            series_id INTEGER NOT NULL,
            season_number INTEGER NOT NULL,
            name TEXT,
            episode_count INTEGER,
            overview TEXT,
            air_date TEXT,
            cover TEXT,
            cover_big TEXT,
            vote_average REAL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
            UNIQUE(profile_id, series_id, season_number)
        )",
        [],
    )?;
    
    // Create index for seasons
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_seasons_series ON xtream_seasons(series_id)",
        [],
    )?;
    
    // Create episodes table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_episodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            series_id INTEGER NOT NULL,
            episode_id TEXT NOT NULL,
            season_number INTEGER NOT NULL,
            episode_num TEXT NOT NULL,
            title TEXT,
            container_extension TEXT,
            custom_sid TEXT,
            added TEXT,
            direct_source TEXT,
            info_json TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
            UNIQUE(profile_id, episode_id)
        )",
        [],
    )?;
    
    // Create indexes for episodes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_episodes_series ON xtream_episodes(series_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_episodes_season ON xtream_episodes(season_number)",
        [],
    )?;
    
    // Create channel categories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_channel_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            category_id TEXT NOT NULL,
            category_name TEXT NOT NULL,
            parent_id INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
            UNIQUE(profile_id, category_id)
        )",
        [],
    )?;
    
    // Create index for channel categories
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channel_cats_profile ON xtream_channel_categories(profile_id)",
        [],
    )?;
    
    // Create movie categories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_movie_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            category_id TEXT NOT NULL,
            category_name TEXT NOT NULL,
            parent_id INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
            UNIQUE(profile_id, category_id)
        )",
        [],
    )?;
    
    // Create index for movie categories
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_movie_cats_profile ON xtream_movie_categories(profile_id)",
        [],
    )?;
    
    // Create series categories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_series_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            category_id TEXT NOT NULL,
            category_name TEXT NOT NULL,
            parent_id INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
            UNIQUE(profile_id, category_id)
        )",
        [],
    )?;
    
    // Create index for series categories
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_series_cats_profile ON xtream_series_categories(profile_id)",
        [],
    )?;
    
    // Create content sync table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_content_sync (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL UNIQUE,
            last_sync_channels TIMESTAMP,
            last_sync_movies TIMESTAMP,
            last_sync_series TIMESTAMP,
            sync_status TEXT DEFAULT 'pending',
            sync_progress INTEGER DEFAULT 0,
            sync_message TEXT,
            channels_count INTEGER DEFAULT 0,
            movies_count INTEGER DEFAULT 0,
            series_count INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    // Create sync settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_sync_settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL UNIQUE,
            auto_sync_enabled BOOLEAN DEFAULT 1,
            sync_interval_hours INTEGER DEFAULT 24,
            wifi_only BOOLEAN DEFAULT 1,
            notify_on_complete BOOLEAN DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    Ok(())
}

/// Get current schema version
fn get_schema_version(conn: &Connection) -> Result<i32> {
    // Check if schema_version table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get::<_, i32>(0),
        )
        .map(|count| count > 0)?;
    
    if !table_exists {
        // Create schema_version table
        conn.execute(
            "CREATE TABLE schema_version (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                version INTEGER NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        return Ok(0);
    }
    
    // Get current version
    let version = conn
        .query_row("SELECT version FROM schema_version WHERE id = 1", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);
    
    Ok(version)
}

/// Set schema version
fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO schema_version (id, version, updated_at) VALUES (1, ?1, CURRENT_TIMESTAMP)",
        [version],
    )?;
    Ok(())
}

/// Run migrations from old version to new version
fn run_migrations(conn: &Connection, from_version: i32, to_version: i32) -> Result<()> {
    for version in (from_version + 1)..=to_version {
        match version {
            1 => migrate_to_v1(conn)?,
            _ => {
                return Err(XTauriError::content_cache(format!(
                    "Unknown migration version: {}",
                    version
                )))
            }
        }
        set_schema_version(conn, version)?;
    }
    Ok(())
}

/// Migration to version 1 (initial schema)
fn migrate_to_v1(conn: &Connection) -> Result<()> {
    // This is the initial schema, so just create all tables
    create_all_tables(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    
    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        
        // Create xtream_profiles table (dependency)
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
        )
        .unwrap();
        
        conn
    }
    
    #[test]
    fn test_schema_initialization() {
        let conn = create_test_db();
        
        // Initialize schema
        let result = initialize_content_cache_tables(&conn);
        assert!(result.is_ok());
        
        // Verify schema version
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }
    
    #[test]
    fn test_all_tables_created() {
        let conn = create_test_db();
        initialize_content_cache_tables(&conn).unwrap();
        
        // Check that all tables exist
        let tables = vec![
            "xtream_channels",
            "xtream_movies",
            "xtream_series",
            "xtream_seasons",
            "xtream_episodes",
            "xtream_channel_categories",
            "xtream_movie_categories",
            "xtream_series_categories",
            "xtream_content_sync",
            "xtream_sync_settings",
        ];
        
        for table in tables {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get::<_, i32>(0),
                )
                .map(|count| count > 0)
                .unwrap();
            
            assert!(exists, "Table {} should exist", table);
        }
    }
    
    #[test]
    fn test_indexes_created() {
        let conn = create_test_db();
        initialize_content_cache_tables(&conn).unwrap();
        
        // Check that indexes exist
        let indexes = vec![
            "idx_channels_profile",
            "idx_channels_category",
            "idx_channels_name",
            "idx_movies_profile",
            "idx_movies_category",
            "idx_series_profile",
            "idx_series_category",
        ];
        
        for index in indexes {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?1",
                    [index],
                    |row| row.get::<_, i32>(0),
                )
                .map(|count| count > 0)
                .unwrap();
            
            assert!(exists, "Index {} should exist", index);
        }
    }
    
    #[test]
    fn test_foreign_key_constraints() {
        let conn = create_test_db();
        
        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        
        initialize_content_cache_tables(&conn).unwrap();
        
        // Insert a test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        )
        .unwrap();
        
        // Try to insert a channel with valid profile_id
        let result = conn.execute(
            "INSERT INTO xtream_channels (profile_id, stream_id, name) VALUES ('test-profile', 1, 'Test Channel')",
            [],
        );
        assert!(result.is_ok());
        
        // Try to insert a channel with invalid profile_id
        let result = conn.execute(
            "INSERT INTO xtream_channels (profile_id, stream_id, name) VALUES ('invalid-profile', 2, 'Test Channel 2')",
            [],
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_unique_constraints() {
        let conn = create_test_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        initialize_content_cache_tables(&conn).unwrap();
        
        // Insert a test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        )
        .unwrap();
        
        // Insert a channel
        conn.execute(
            "INSERT INTO xtream_channels (profile_id, stream_id, name) VALUES ('test-profile', 1, 'Test Channel')",
            [],
        )
        .unwrap();
        
        // Try to insert duplicate channel (same profile_id and stream_id)
        let result = conn.execute(
            "INSERT INTO xtream_channels (profile_id, stream_id, name) VALUES ('test-profile', 1, 'Duplicate Channel')",
            [],
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_cascade_delete() {
        let conn = create_test_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        initialize_content_cache_tables(&conn).unwrap();
        
        // Insert a test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        )
        .unwrap();
        
        // Insert some content
        conn.execute(
            "INSERT INTO xtream_channels (profile_id, stream_id, name) VALUES ('test-profile', 1, 'Test Channel')",
            [],
        )
        .unwrap();
        
        conn.execute(
            "INSERT INTO xtream_movies (profile_id, stream_id, name) VALUES ('test-profile', 1, 'Test Movie')",
            [],
        )
        .unwrap();
        
        // Delete the profile
        conn.execute("DELETE FROM xtream_profiles WHERE id = 'test-profile'", [])
            .unwrap();
        
        // Verify content was deleted
        let channel_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM xtream_channels WHERE profile_id = 'test-profile'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(channel_count, 0);
        
        let movie_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM xtream_movies WHERE profile_id = 'test-profile'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(movie_count, 0);
    }
    
    #[test]
    fn test_migration_system() {
        let conn = create_test_db();
        
        // Initialize with version 0
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, 0);
        
        // Run initialization (should create tables and set version to 1)
        initialize_content_cache_tables(&conn).unwrap();
        
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        
        // Running again should not error
        let result = initialize_content_cache_tables(&conn);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_default_values() {
        let conn = create_test_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        initialize_content_cache_tables(&conn).unwrap();
        
        // Insert a test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        )
        .unwrap();
        
        // Insert sync settings with minimal data
        conn.execute(
            "INSERT INTO xtream_sync_settings (profile_id) VALUES ('test-profile')",
            [],
        )
        .unwrap();
        
        // Verify default values
        let (auto_sync, interval, wifi_only, notify): (bool, i32, bool, bool) = conn
            .query_row(
                "SELECT auto_sync_enabled, sync_interval_hours, wifi_only, notify_on_complete 
                 FROM xtream_sync_settings WHERE profile_id = 'test-profile'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        
        assert_eq!(auto_sync, true);
        assert_eq!(interval, 24);
        assert_eq!(wifi_only, true);
        assert_eq!(notify, false);
    }
}
