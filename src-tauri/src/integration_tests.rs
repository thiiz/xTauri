use crate::channels::*;
use crate::m3u_parser::Channel;
use crate::settings::*;
use crate::state::{ChannelCacheState, DbState};
use rusqlite::Connection;
use std::sync::Mutex;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();

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
    )
    .unwrap();

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
    )
    .unwrap();

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
    )
    .unwrap();

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
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS group_selections (
            channel_list_id INTEGER NOT NULL,
            group_name TEXT NOT NULL,
            is_enabled BOOLEAN NOT NULL DEFAULT 1,
            PRIMARY KEY (channel_list_id, group_name),
            FOREIGN KEY (channel_list_id) REFERENCES channel_lists(id) ON DELETE CASCADE
        )",
        [],
    )
    .unwrap();

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
    )
    .unwrap();

    // Insert default settings
    conn.execute(
        "INSERT INTO settings (id, player_command, cache_duration_hours, enable_preview, mute_on_start, show_controls, autoplay) VALUES (1, 'mpv', 24, 1, 0, 1, 0)",
        [],
    ).unwrap();

    conn
}

fn create_test_channel() -> Channel {
    Channel {
        name: "Test Channel".to_string(),
        logo: "http://example.com/logo.png".to_string(),
        url: "http://example.com/stream".to_string(),
        group_title: "Test Group".to_string(),
        tvg_id: "test123".to_string(),
        resolution: "1080p".to_string(),
        extra_info: "Test extra info".to_string(),
    }
}

// Mock State implementation for testing
struct MockState<T>(T);

impl<T> std::ops::Deref for MockState<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for MockState<T> {
    fn from(inner: T) -> Self {
        MockState(inner)
    }
}

// Settings Command Tests
// Temporarily disabled - functions not implemented
// #[test]
// fn test_get_player_command() {
//     let db_state = DbState {
//         db: Mutex::new(create_test_db()),
//     };
//     let state = MockState::from(db_state);
//
//     let result = get_player_command(unsafe { std::mem::transmute(&state) });
//     assert!(result.is_ok());
//     assert_eq!(result.unwrap(), "mpv");
// }

// #[test]
// fn test_set_player_command() {
//     let db_state = DbState {
//         db: Mutex::new(create_test_db()),
//     };
//     let state = MockState::from(db_state);
//
//     let result = set_player_command(unsafe { std::mem::transmute(&state) }, "vlc".to_string());
//     assert!(result.is_ok());
//
//     // Verify the command was set
//     let result = get_player_command(unsafe { std::mem::transmute(&state) });
//     assert!(result.is_ok());
//     assert_eq!(result.unwrap(), "vlc");
// }

#[test]
fn test_get_cache_duration() {
    let db_state = DbState {
        db: Mutex::new(create_test_db()),
    };
    let state = MockState::from(db_state);

    let result = get_cache_duration(unsafe { std::mem::transmute(&state) });
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 24);
}

#[test]
fn test_set_cache_duration() {
    let db_state = DbState {
        db: Mutex::new(create_test_db()),
    };
    let state = MockState::from(db_state);

    let result = set_cache_duration(unsafe { std::mem::transmute(&state) }, 48);
    assert!(result.is_ok());

    // Verify the duration was set
    let result = get_cache_duration(unsafe { std::mem::transmute(&state) });
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 48);
}

#[test]
fn test_get_enable_preview() {
    let db_state = DbState {
        db: Mutex::new(create_test_db()),
    };
    let state = MockState::from(db_state);

    let result = get_enable_preview(unsafe { std::mem::transmute(&state) });
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_set_enable_preview() {
    let db_state = DbState {
        db: Mutex::new(create_test_db()),
    };
    let state = MockState::from(db_state);

    let result = set_enable_preview(unsafe { std::mem::transmute(&state) }, false);
    assert!(result.is_ok());

    // Verify the setting was changed
    let result = get_enable_preview(unsafe { std::mem::transmute(&state) });
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

// Cache Command Tests
#[test]
fn test_invalidate_channel_cache() {
    let cache_state = ChannelCacheState {
        cache: Mutex::new(None),
    };
    let state = MockState::from(cache_state);

    // Cache should start empty
    {
        let cache = state.cache.lock().unwrap();
        assert!(cache.is_none());
    }

    // Invalidate should not fail even with empty cache
    let result = invalidate_channel_cache(unsafe { std::mem::transmute(&state) });
    assert!(result.is_ok());

    // Cache should still be empty
    {
        let cache = state.cache.lock().unwrap();
        assert!(cache.is_none());
    }
}

// Error Handling Tests
// Temporarily disabled - functions not implemented
// #[test]
// fn test_settings_commands_with_empty_database() {
//     let conn = Connection::open_in_memory().unwrap();
//     // Don't create the settings table to test error handling
//
//     let db_state = DbState {
//         db: Mutex::new(conn),
//     };
//     let state = MockState::from(db_state);
//
//     let result = get_player_command(unsafe { std::mem::transmute(&state) });
//     assert!(result.is_err());
// Integration test for search functionality
#[test]
fn test_search_integration() {
    use crate::fuzzy_search::FuzzyMatcher;

    let channels = vec![
        Channel {
            name: "BBC News".to_string(),
            logo: "http://example.com/bbc.png".to_string(),
            url: "http://example.com/bbc".to_string(),
            group_title: "News".to_string(),
            tvg_id: "bbc1".to_string(),
            resolution: "1080p".to_string(),
            extra_info: "HD".to_string(),
        },
        Channel {
            name: "CNN International".to_string(),
            logo: "http://example.com/cnn.png".to_string(),
            url: "http://example.com/cnn".to_string(),
            group_title: "News".to_string(),
            tvg_id: "cnn1".to_string(),
            resolution: "720p".to_string(),
            extra_info: "".to_string(),
        },
    ];

    let matcher = FuzzyMatcher::new();
    let results = matcher.search_channels(&channels, "News");

    assert_eq!(results.len(), 2);
    // Should find both channels that have "News" in their name or group
    let names: Vec<&str> = results.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"BBC News"));
    assert!(names.contains(&"CNN International"));
}

// Test command state interactions
// Temporarily disabled - functions not implemented
// #[test]
// fn test_settings_persistence() {
//     let db_state = DbState {
//         db: Mutex::new(create_test_db()),
//     };
//     let state = MockState::from(db_state);
//
//     // Test multiple setting changes
//     let result1 = set_player_command(unsafe { std::mem::transmute(&state) }, "vlc".to_string());
//     assert!(result1.is_ok());
//
//     let result2 = set_cache_duration(unsafe { std::mem::transmute(&state) }, 72);
//     assert!(result2.is_ok());
//
//     let result3 = set_enable_preview(unsafe { std::mem::transmute(&state) }, false);
//     assert!(result3.is_ok());
//
//     // Verify all settings were persisted
//     let player_cmd = get_player_command(unsafe { std::mem::transmute(&state) });
//     assert!(player_cmd.is_ok());
//     assert_eq!(player_cmd.unwrap(), "vlc");
//
//     let cache_duration = get_cache_duration(unsafe { std::mem::transmute(&state) });
//     assert!(cache_duration.is_ok());
//     assert_eq!(cache_duration.unwrap(), 72);
//
//     let enable_preview = get_enable_preview(unsafe { std::mem::transmute(&state) });
//     assert!(enable_preview.is_ok());
//     assert_eq!(enable_preview.unwrap(), false);
// }
