use crate::error::{Result, XTauriError};
use crate::m3u_parser::Channel;
use rusqlite::{Connection, Result as RusqliteResult};
use std::fs;

pub fn initialize_database() -> Result<Connection> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| XTauriError::DataDirectoryAccess)?
        .join("xtauri");

    fs::create_dir_all(&data_dir)
        .map_err(|_e| XTauriError::directory_creation(data_dir.display().to_string()))?;

    let db_path = data_dir.join("database.sqlite");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL,
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS channels (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL,
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS channels_fts USING fts5(name, content='channels', content_rowid='id')",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY,
            player_command TEXT NOT NULL,
            cache_duration_hours INTEGER NOT NULL DEFAULT 24,
            enable_preview BOOLEAN NOT NULL DEFAULT 1,
            mute_on_start BOOLEAN NOT NULL DEFAULT 0,
            show_controls BOOLEAN NOT NULL DEFAULT 1,
            autoplay BOOLEAN NOT NULL DEFAULT 0
        )",
        [],
    )?;

    // Add the enable_preview column to existing settings table if it doesn't exist
    conn.execute(
        "ALTER TABLE settings ADD COLUMN enable_preview BOOLEAN NOT NULL DEFAULT 1",
        [],
    )
    .ok(); // Use ok() to ignore error if column already exists

    // Add the mute_on_start column to existing settings table if it doesn't exist
    conn.execute(
        "ALTER TABLE settings ADD COLUMN mute_on_start BOOLEAN NOT NULL DEFAULT 0",
        [],
    )
    .ok();
    // Add the show_controls column to existing settings table if it doesn't exist
    conn.execute(
        "ALTER TABLE settings ADD COLUMN show_controls BOOLEAN NOT NULL DEFAULT 1",
        [],
    )
    .ok();
    // Add the autoplay column to existing settings table if it doesn't exist
    conn.execute(
        "ALTER TABLE settings ADD COLUMN autoplay BOOLEAN NOT NULL DEFAULT 0",
        [],
    )
    .ok();

    // Add the volume column to existing settings table if it doesn't exist
    conn.execute(
        "ALTER TABLE settings ADD COLUMN volume REAL NOT NULL DEFAULT 1.0",
        [],
    )
    .ok();

    // Add the is_muted column to existing settings table if it doesn't exist
    conn.execute(
        "ALTER TABLE settings ADD COLUMN is_muted BOOLEAN NOT NULL DEFAULT 0",
        [],
    )
    .ok();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS channel_lists (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            source TEXT NOT NULL,
            filepath TEXT,
            last_fetched INTEGER,
            is_default BOOLEAN NOT NULL DEFAULT 0,
            CONSTRAINT is_default_check CHECK (is_default IN (0, 1))
        )",
        [],
    )?;

    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_one_default_list ON channel_lists (is_default) WHERE is_default = 1",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS group_selections (
            channel_list_id INTEGER NOT NULL,
            group_name TEXT NOT NULL,
            is_enabled BOOLEAN NOT NULL DEFAULT 1,
            PRIMARY KEY (channel_list_id, group_name),
            FOREIGN KEY (channel_list_id) REFERENCES channel_lists(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS saved_filters (
            channel_list_id INTEGER NOT NULL,
            slot_number INTEGER NOT NULL CHECK (slot_number >= 0 AND slot_number <= 9),
            search_query TEXT NOT NULL DEFAULT '',
            selected_group TEXT,
            name TEXT NOT NULL DEFAULT '',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (channel_list_id, slot_number),
            FOREIGN KEY (channel_list_id) REFERENCES channel_lists(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Xtream Codes integration tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_profiles (
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
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_content_cache (
            cache_key TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL,
            content_type TEXT NOT NULL,
            data BLOB NOT NULL,
            expires_at DATETIME NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Create indexes for performance optimization
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_xtream_cache_profile_type 
         ON xtream_content_cache(profile_id, content_type)",
        [],
    )
    .ok();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_xtream_cache_expires 
         ON xtream_content_cache(expires_at)",
        [],
    )
    .ok();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_xtream_profiles_active 
         ON xtream_profiles(is_active) WHERE is_active = TRUE",
        [],
    )
    .ok();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_xtream_profiles_last_used 
         ON xtream_profiles(last_used DESC)",
        [],
    )
    .ok();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_favorites (
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
    )?;

    // Create indexes for favorites
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_xtream_favorites_profile_type 
         ON xtream_favorites(profile_id, content_type)",
        [],
    )
    .ok();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_xtream_favorites_content 
         ON xtream_favorites(profile_id, content_type, content_id)",
        [],
    )
    .ok();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS xtream_history (
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
    )?;

    // Create indexes for history
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_xtream_history_profile_watched 
         ON xtream_history(profile_id, watched_at DESC)",
        [],
    )
    .ok();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_xtream_history_content 
         ON xtream_history(profile_id, content_type, content_id)",
        [],
    )
    .ok();

    // Add position and duration columns to existing xtream_history table if they don't exist
    conn.execute("ALTER TABLE xtream_history ADD COLUMN position REAL", [])
        .ok(); // Use ok() to ignore error if column already exists

    conn.execute("ALTER TABLE xtream_history ADD COLUMN duration REAL", [])
        .ok(); // Use ok() to ignore error if column already exists

    // Search history table
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

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_search_history_profile 
         ON xtream_search_history(profile_id, created_at DESC)",
        [],
    )
    .ok();

    // Saved filters table
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

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_saved_filters_profile 
         ON xtream_saved_filters(profile_id, content_type)",
        [],
    )
    .ok();

    let list_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM channel_lists", [], |row| row.get(0))?;
    if list_count == 0 {
        conn.execute(
            "INSERT INTO channel_lists (name, source, is_default) VALUES (?1, ?2, ?3)",
            &["iptv-org", "https://iptv-org.github.io/iptv/index.m3u", "1"],
        )?;
    }

    // Ensure we have a default settings record
    let settings_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))?;
    if settings_count == 0 {
        conn.execute(
            "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay, volume, is_muted) VALUES (1, 'mpv', 24, 1, 0, 1, 0, 1.0, 0)",
            [],
        )?;
    }

    // Initialize content cache tables
    crate::content_cache::initialize_content_cache_tables(&conn)?;

    Ok(conn)
}

pub fn populate_channels(conn: &mut Connection, channels: &[Channel]) -> RusqliteResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT OR IGNORE INTO channels (name, logo, url, group_title, tvg_id, resolution, extra_info) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")?;
        for channel in channels {
            stmt.execute(&[
                &channel.name,
                &channel.logo,
                &channel.url,
                &channel.group_title,
                &channel.tvg_id,
                &channel.resolution,
                &channel.extra_info,
            ])?;
        }
    }
    tx.commit()?;

    conn.execute(
        "INSERT INTO channels_fts(rowid, name) SELECT id, name FROM channels",
        [],
    )?;

    Ok(())
}

pub fn get_enabled_groups(conn: &Connection, channel_list_id: i64) -> RusqliteResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT group_name FROM group_selections WHERE channel_list_id = ?1 AND is_enabled = 1",
    )?;
    let group_iter = stmt.query_map([channel_list_id], |row| Ok(row.get::<_, String>(0)?))?;

    let mut groups = Vec::new();
    for group in group_iter {
        groups.push(group?);
    }
    Ok(groups)
}

pub fn set_group_enabled(
    conn: &Connection,
    channel_list_id: i64,
    group_name: String,
    enabled: bool,
) -> RusqliteResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO group_selections (channel_list_id, group_name, is_enabled) VALUES (?1, ?2, ?3)",
        (channel_list_id, group_name, enabled),
    )?;
    Ok(())
}

pub fn sync_channel_list_groups(
    conn: &mut Connection,
    channel_list_id: i64,
    groups: Vec<String>,
) -> RusqliteResult<()> {
    let tx = conn.transaction()?;

    // Get existing groups for this channel list
    let existing_groups = {
        let mut stmt =
            tx.prepare("SELECT group_name FROM group_selections WHERE channel_list_id = ?1")?;
        let group_iter = stmt.query_map([channel_list_id], |row| Ok(row.get::<_, String>(0)?))?;

        let mut groups = Vec::new();
        for group in group_iter {
            groups.push(group?);
        }
        groups
    };

    // Remove groups that no longer exist
    for existing_group in &existing_groups {
        if !groups.contains(existing_group) {
            tx.execute(
                "DELETE FROM group_selections WHERE channel_list_id = ?1 AND group_name = ?2",
                (channel_list_id, existing_group),
            )?;
        }
    }

    // Add new groups (enabled by default)
    for group in &groups {
        if !existing_groups.contains(group) {
            tx.execute(
                "INSERT INTO group_selections (channel_list_id, group_name, is_enabled) VALUES (?1, ?2, ?3)",
                (channel_list_id, group, true),
            )?;
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn enable_all_groups(
    conn: &mut Connection,
    channel_list_id: i64,
    groups: Vec<String>,
) -> RusqliteResult<()> {
    // Enable all groups in a single transaction for much better performance
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare("INSERT OR REPLACE INTO group_selections (channel_list_id, group_name, is_enabled) VALUES (?1, ?2, ?3)")?;
        for group in groups {
            stmt.execute((channel_list_id, group, true))?;
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn disable_all_groups(
    conn: &mut Connection,
    channel_list_id: i64,
    groups: Vec<String>,
) -> RusqliteResult<()> {
    // Disable all groups in a single transaction for much better performance
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare("INSERT OR REPLACE INTO group_selections (channel_list_id, group_name, is_enabled) VALUES (?1, ?2, ?3)")?;
        for group in groups {
            stmt.execute((channel_list_id, group, false))?;
        }
    }

    tx.commit()?;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SavedFilter {
    pub slot_number: i32,
    pub search_query: String,
    pub selected_group: Option<String>,
    pub name: String,
}

pub fn save_filter(
    conn: &Connection,
    channel_list_id: i64,
    slot_number: i32,
    search_query: String,
    selected_group: Option<String>,
    name: String,
) -> RusqliteResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO saved_filters (channel_list_id, slot_number, search_query, selected_group, name) VALUES (?1, ?2, ?3, ?4, ?5)",
        (channel_list_id, slot_number, search_query, selected_group, name),
    )?;
    Ok(())
}

pub fn get_saved_filters(
    conn: &Connection,
    channel_list_id: i64,
) -> RusqliteResult<Vec<SavedFilter>> {
    let mut stmt = conn.prepare("SELECT slot_number, search_query, selected_group, name FROM saved_filters WHERE channel_list_id = ?1 ORDER BY slot_number")?;
    let filter_iter = stmt.query_map([channel_list_id], |row| {
        Ok(SavedFilter {
            slot_number: row.get(0)?,
            search_query: row.get(1)?,
            selected_group: row.get(2)?,
            name: row.get(3)?,
        })
    })?;

    let mut filters = Vec::new();
    for filter in filter_iter {
        filters.push(filter?);
    }
    Ok(filters)
}

pub fn delete_saved_filter(
    conn: &Connection,
    channel_list_id: i64,
    slot_number: i32,
) -> RusqliteResult<()> {
    conn.execute(
        "DELETE FROM saved_filters WHERE channel_list_id = ?1 AND slot_number = ?2",
        (channel_list_id, slot_number),
    )?;
    Ok(())
}

