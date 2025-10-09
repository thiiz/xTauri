// Tests for incremental synchronization functionality
#[cfg(test)]
mod tests {
    use crate::content_cache::*;
    use crate::content_cache::sync_scheduler::*;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    
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
        ).unwrap();
        
        // Insert test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        ).unwrap();
        
        conn
    }
    
    #[test]
    fn test_get_content_ids_empty() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let cache = ContentCache::new(db).unwrap();
        
        // Get IDs for empty cache
        let channel_ids = cache.get_content_ids("test-profile", "channels").unwrap();
        assert_eq!(channel_ids.len(), 0);
        
        let movie_ids = cache.get_content_ids("test-profile", "movies").unwrap();
        assert_eq!(movie_ids.len(), 0);
        
        let series_ids = cache.get_content_ids("test-profile", "series").unwrap();
        assert_eq!(series_ids.len(), 0);
    }
    
    #[test]
    fn test_get_content_ids_with_data() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let cache = ContentCache::new(db).unwrap();
        
        // Add some channels
        let channels = vec![
            XtreamChannel {
                stream_id: 1,
                num: Some(1),
                name: "Channel 1".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567890".to_string()),
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 2,
                num: Some(2),
                name: "Channel 2".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567891".to_string()),
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
        ];
        
        cache.save_channels("test-profile", channels).unwrap();
        
        // Get IDs
        let channel_ids = cache.get_content_ids("test-profile", "channels").unwrap();
        assert_eq!(channel_ids.len(), 2);
        assert!(channel_ids.contains(&1));
        assert!(channel_ids.contains(&2));
    }
    
    #[test]
    fn test_delete_content_by_ids() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let cache = ContentCache::new(db).unwrap();
        
        // Add some channels
        let channels = vec![
            XtreamChannel {
                stream_id: 1,
                num: Some(1),
                name: "Channel 1".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567890".to_string()),
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 2,
                num: Some(2),
                name: "Channel 2".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567891".to_string()),
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 3,
                num: Some(3),
                name: "Channel 3".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567892".to_string()),
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
        ];
        
        cache.save_channels("test-profile", channels).unwrap();
        
        // Delete some channels
        let deleted = cache.delete_content_by_ids("test-profile", "channels", &[1, 3]).unwrap();
        assert_eq!(deleted, 2);
        
        // Verify remaining channels
        let remaining_ids = cache.get_content_ids("test-profile", "channels").unwrap();
        assert_eq!(remaining_ids.len(), 1);
        assert!(remaining_ids.contains(&2));
        assert!(!remaining_ids.contains(&1));
        assert!(!remaining_ids.contains(&3));
    }
    
    #[test]
    fn test_delete_content_by_ids_empty() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let cache = ContentCache::new(db).unwrap();
        
        // Delete with empty list
        let deleted = cache.delete_content_by_ids("test-profile", "channels", &[]).unwrap();
        assert_eq!(deleted, 0);
    }
    
    #[test]
    fn test_delete_content_by_ids_nonexistent() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let cache = ContentCache::new(db).unwrap();
        
        // Delete non-existent IDs
        let deleted = cache.delete_content_by_ids("test-profile", "channels", &[999, 1000]).unwrap();
        assert_eq!(deleted, 0);
    }
    
    #[test]
    fn test_is_item_updated_unix_timestamp() {
        // Test with Unix timestamps
        let last_sync = "1234567890";
        
        // Item updated after last sync
        let item_time_newer = Some("1234567900".to_string());
        assert!(SyncScheduler::is_item_updated(&item_time_newer, last_sync));
        
        // Item updated before last sync
        let item_time_older = Some("1234567880".to_string());
        assert!(!SyncScheduler::is_item_updated(&item_time_older, last_sync));
        
        // Item updated at same time
        let item_time_same = Some("1234567890".to_string());
        assert!(!SyncScheduler::is_item_updated(&item_time_same, last_sync));
        
        // No item timestamp
        assert!(!SyncScheduler::is_item_updated(&None, last_sync));
    }
    
    #[test]
    fn test_is_item_updated_iso8601() {
        // Test with ISO 8601 timestamps
        let last_sync = "2024-01-01T12:00:00Z";
        
        // Item updated after last sync
        let item_time_newer = Some("2024-01-01T13:00:00Z".to_string());
        assert!(SyncScheduler::is_item_updated(&item_time_newer, last_sync));
        
        // Item updated before last sync
        let item_time_older = Some("2024-01-01T11:00:00Z".to_string());
        assert!(!SyncScheduler::is_item_updated(&item_time_older, last_sync));
    }
    
    #[test]
    fn test_compare_channels_new_items() {
        let server_channels = vec![
            XtreamChannel {
                stream_id: 1,
                num: Some(1),
                name: "Channel 1".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567890".to_string()),
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 2,
                num: Some(2),
                name: "Channel 2".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567891".to_string()),
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
        ];
        
        let cached_ids = vec![];
        let last_sync = None;
        
        let (new_items, updated_items, server_ids) = 
            SyncScheduler::compare_channels(&server_channels, &cached_ids, last_sync);
        
        assert_eq!(new_items.len(), 2);
        assert_eq!(updated_items.len(), 0);
        assert_eq!(server_ids.len(), 2);
        assert!(server_ids.contains(&1));
        assert!(server_ids.contains(&2));
    }
    
    #[test]
    fn test_compare_channels_updated_items() {
        let server_channels = vec![
            XtreamChannel {
                stream_id: 1,
                num: Some(1),
                name: "Channel 1 Updated".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567900".to_string()), // Newer timestamp
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
        ];
        
        let cached_ids = vec![1];
        let last_sync = Some("1234567890");
        
        let (new_items, updated_items, server_ids) = 
            SyncScheduler::compare_channels(&server_channels, &cached_ids, last_sync);
        
        assert_eq!(new_items.len(), 0);
        assert_eq!(updated_items.len(), 1);
        assert_eq!(server_ids.len(), 1);
    }
    
    #[test]
    fn test_compare_channels_mixed() {
        let server_channels = vec![
            XtreamChannel {
                stream_id: 1,
                num: Some(1),
                name: "Channel 1".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567880".to_string()), // Older, not updated
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 2,
                num: Some(2),
                name: "Channel 2 Updated".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567900".to_string()), // Newer, updated
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 3,
                num: Some(3),
                name: "Channel 3".to_string(),
                stream_type: None,
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: Some("1234567891".to_string()), // New
                category_id: Some("1".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
        ];
        
        let cached_ids = vec![1, 2];
        let last_sync = Some("1234567890");
        
        let (new_items, updated_items, server_ids) = 
            SyncScheduler::compare_channels(&server_channels, &cached_ids, last_sync);
        
        assert_eq!(new_items.len(), 1); // Channel 3
        assert_eq!(updated_items.len(), 1); // Channel 2
        assert_eq!(server_ids.len(), 3);
    }
    
    #[test]
    fn test_get_last_sync_timestamps() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db.clone());
        
        // Initialize sync record
        let progress = SyncProgress::default();
        scheduler.update_sync_status("test-profile", &progress).unwrap();
        
        // Update timestamps
        scheduler.update_last_sync_timestamp("test-profile", "channels").unwrap();
        scheduler.update_last_sync_timestamp("test-profile", "movies").unwrap();
        
        // Get timestamps
        let timestamps = scheduler.get_last_sync_timestamps("test-profile").unwrap();
        
        assert!(timestamps.channels.is_some());
        assert!(timestamps.movies.is_some());
        assert!(timestamps.series.is_none());
    }
    
    #[test]
    fn test_get_last_sync_timestamps_no_record() {
        let conn = create_test_db();
        let db = Arc::new(Mutex::new(conn));
        let scheduler = SyncScheduler::new(db);
        
        // Get timestamps without sync record
        let timestamps = scheduler.get_last_sync_timestamps("test-profile").unwrap();
        
        assert!(timestamps.channels.is_none());
        assert!(timestamps.movies.is_none());
        assert!(timestamps.series.is_none());
    }
}
