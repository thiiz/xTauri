#[cfg(test)]
mod sync_settings_tests {
    use crate::content_cache::sync_scheduler::{SyncScheduler, SyncSettings};
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};

    fn create_test_db() -> Arc<Mutex<Connection>> {
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
        
        // Create sync settings table
        conn.execute(
            "CREATE TABLE xtream_sync_settings (
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
        )
        .unwrap();
        
        // Insert test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        )
        .unwrap();
        
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn test_get_default_sync_settings() {
        let db = create_test_db();
        let scheduler = SyncScheduler::new(db);
        
        // Get settings for profile without explicit settings (should return defaults)
        let settings = scheduler.get_sync_settings("test-profile").unwrap();
        
        assert_eq!(settings.auto_sync_enabled, true);
        assert_eq!(settings.sync_interval_hours, 24);
        assert_eq!(settings.wifi_only, true);
        assert_eq!(settings.notify_on_complete, false);
    }

    #[test]
    fn test_save_and_get_sync_settings() {
        let db = create_test_db();
        let scheduler = SyncScheduler::new(db);
        
        // Create custom settings
        let custom_settings = SyncSettings {
            auto_sync_enabled: false,
            sync_interval_hours: 12,
            wifi_only: false,
            notify_on_complete: true,
        };
        
        // Save settings
        let result = scheduler.update_sync_settings("test-profile", &custom_settings);
        assert!(result.is_ok());
        
        // Retrieve settings
        let retrieved_settings = scheduler.get_sync_settings("test-profile").unwrap();
        
        assert_eq!(retrieved_settings.auto_sync_enabled, false);
        assert_eq!(retrieved_settings.sync_interval_hours, 12);
        assert_eq!(retrieved_settings.wifi_only, false);
        assert_eq!(retrieved_settings.notify_on_complete, true);
    }

    #[test]
    fn test_update_existing_sync_settings() {
        let db = create_test_db();
        let scheduler = SyncScheduler::new(db);
        
        // Save initial settings
        let initial_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 24,
            wifi_only: true,
            notify_on_complete: false,
        };
        scheduler.update_sync_settings("test-profile", &initial_settings).unwrap();
        
        // Update settings
        let updated_settings = SyncSettings {
            auto_sync_enabled: false,
            sync_interval_hours: 48,
            wifi_only: false,
            notify_on_complete: true,
        };
        scheduler.update_sync_settings("test-profile", &updated_settings).unwrap();
        
        // Verify updated settings
        let retrieved_settings = scheduler.get_sync_settings("test-profile").unwrap();
        
        assert_eq!(retrieved_settings.auto_sync_enabled, false);
        assert_eq!(retrieved_settings.sync_interval_hours, 48);
        assert_eq!(retrieved_settings.wifi_only, false);
        assert_eq!(retrieved_settings.notify_on_complete, true);
    }

    #[test]
    fn test_sync_settings_validation() {
        let db = create_test_db();
        let scheduler = SyncScheduler::new(db);
        
        // Try to save settings with invalid interval (< 6 hours)
        let invalid_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 3,
            wifi_only: true,
            notify_on_complete: false,
        };
        
        let result = scheduler.update_sync_settings("test-profile", &invalid_settings);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 6 hours"));
    }

    #[test]
    fn test_sync_settings_persistence() {
        let db = create_test_db();
        let scheduler = SyncScheduler::new(Arc::clone(&db));
        
        // Save settings
        let settings = SyncSettings {
            auto_sync_enabled: false,
            sync_interval_hours: 12,
            wifi_only: false,
            notify_on_complete: true,
        };
        scheduler.update_sync_settings("test-profile", &settings).unwrap();
        
        // Create new scheduler instance with same database
        let scheduler2 = SyncScheduler::new(db);
        
        // Verify settings persisted
        let retrieved_settings = scheduler2.get_sync_settings("test-profile").unwrap();
        
        assert_eq!(retrieved_settings.auto_sync_enabled, false);
        assert_eq!(retrieved_settings.sync_interval_hours, 12);
        assert_eq!(retrieved_settings.wifi_only, false);
        assert_eq!(retrieved_settings.notify_on_complete, true);
    }

    #[test]
    fn test_sync_settings_for_nonexistent_profile() {
        let db = create_test_db();
        let scheduler = SyncScheduler::new(db);
        
        // Get settings for non-existent profile (should return defaults)
        let settings = scheduler.get_sync_settings("nonexistent-profile").unwrap();
        
        assert_eq!(settings.auto_sync_enabled, true);
        assert_eq!(settings.sync_interval_hours, 24);
        assert_eq!(settings.wifi_only, true);
        assert_eq!(settings.notify_on_complete, false);
    }

    #[test]
    fn test_default_settings_initialization() {
        let db = create_test_db();
        let conn = db.lock().unwrap();
        
        // Insert settings with only profile_id (should use defaults)
        conn.execute(
            "INSERT INTO xtream_sync_settings (profile_id) VALUES ('test-profile')",
            [],
        )
        .unwrap();
        
        drop(conn);
        
        let scheduler = SyncScheduler::new(db);
        let settings = scheduler.get_sync_settings("test-profile").unwrap();
        
        // Verify defaults were applied
        assert_eq!(settings.auto_sync_enabled, true);
        assert_eq!(settings.sync_interval_hours, 24);
        assert_eq!(settings.wifi_only, true);
        assert_eq!(settings.notify_on_complete, false);
    }

    #[test]
    fn test_sync_settings_boundary_values() {
        let db = create_test_db();
        let scheduler = SyncScheduler::new(db);
        
        // Test minimum valid interval (6 hours)
        let min_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 6,
            wifi_only: true,
            notify_on_complete: false,
        };
        let result = scheduler.update_sync_settings("test-profile", &min_settings);
        assert!(result.is_ok());
        
        let retrieved = scheduler.get_sync_settings("test-profile").unwrap();
        assert_eq!(retrieved.sync_interval_hours, 6);
        
        // Test large interval (48 hours)
        let max_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 48,
            wifi_only: true,
            notify_on_complete: false,
        };
        let result = scheduler.update_sync_settings("test-profile", &max_settings);
        assert!(result.is_ok());
        
        let retrieved = scheduler.get_sync_settings("test-profile").unwrap();
        assert_eq!(retrieved.sync_interval_hours, 48);
    }

    #[test]
    fn test_sync_settings_all_combinations() {
        let db = create_test_db();
        let scheduler = SyncScheduler::new(db);
        
        // Test all boolean combinations
        let test_cases = vec![
            (true, true, true),
            (true, true, false),
            (true, false, true),
            (true, false, false),
            (false, true, true),
            (false, true, false),
            (false, false, true),
            (false, false, false),
        ];
        
        for (auto_sync, wifi_only, notify) in test_cases {
            let settings = SyncSettings {
                auto_sync_enabled: auto_sync,
                sync_interval_hours: 24,
                wifi_only,
                notify_on_complete: notify,
            };
            
            scheduler.update_sync_settings("test-profile", &settings).unwrap();
            let retrieved = scheduler.get_sync_settings("test-profile").unwrap();
            
            assert_eq!(retrieved.auto_sync_enabled, auto_sync);
            assert_eq!(retrieved.wifi_only, wifi_only);
            assert_eq!(retrieved.notify_on_complete, notify);
        }
    }
}
