// Integration tests for sync settings commands
#[cfg(test)]
mod tests {
    use crate::content_cache::{ContentCache, SyncScheduler, SyncSettings};
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};

    /// Create a test database with required dependencies
    fn create_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();

        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Create xtream_profiles table (dependency for foreign keys)
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

        Arc::new(Mutex::new(conn))
    }

    /// Insert a test profile into the database
    fn insert_test_profile(db: &Arc<Mutex<Connection>>, profile_id: &str) {
        let conn = db.lock().unwrap();
        let profile_name = format!("Test Profile {}", profile_id);
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES (?1, ?2, 'http://test.com', 'testuser', X'00')",
            rusqlite::params![profile_id, profile_name],
        )
        .unwrap();
    }

    fn setup_test_environment() -> (Arc<Mutex<Connection>>, ContentCache, SyncScheduler) {
        let db = create_test_db();
        let cache = ContentCache::new(Arc::clone(&db)).unwrap();
        let scheduler = SyncScheduler::new(Arc::clone(&db));
        (db, cache, scheduler)
    }

    #[test]
    fn test_get_sync_settings_default() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Get default settings
        let settings = scheduler.get_sync_settings("test_profile").unwrap();
        assert_eq!(settings.auto_sync_enabled, true);
        assert_eq!(settings.sync_interval_hours, 24);
        assert_eq!(settings.wifi_only, true);
        assert_eq!(settings.notify_on_complete, false);
    }

    #[test]
    fn test_get_sync_settings_uninitialized_profile() {
        let (_db, _cache, scheduler) = setup_test_environment();

        // Get settings for a profile that doesn't exist (should return defaults)
        let settings = scheduler.get_sync_settings("nonexistent_profile").unwrap();
        assert_eq!(settings.auto_sync_enabled, true);
        assert_eq!(settings.sync_interval_hours, 24);
    }

    #[test]
    fn test_update_sync_settings() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Create custom settings
        let custom_settings = SyncSettings {
            auto_sync_enabled: false,
            sync_interval_hours: 12,
            wifi_only: false,
            notify_on_complete: true,
        };

        // Update settings
        let result = scheduler.update_sync_settings("test_profile", &custom_settings);
        assert!(result.is_ok());

        // Verify settings were updated
        let retrieved_settings = scheduler.get_sync_settings("test_profile").unwrap();
        assert_eq!(retrieved_settings.auto_sync_enabled, false);
        assert_eq!(retrieved_settings.sync_interval_hours, 12);
        assert_eq!(retrieved_settings.wifi_only, false);
        assert_eq!(retrieved_settings.notify_on_complete, true);
    }

    #[test]
    fn test_update_sync_settings_validation() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Create invalid settings (interval < 6 hours)
        let invalid_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 3,
            wifi_only: true,
            notify_on_complete: false,
        };

        // Update should fail validation
        let result = scheduler.update_sync_settings("test_profile", &invalid_settings);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("at least 6 hours"));
    }

    #[test]
    fn test_update_sync_settings_boundary_values() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Test minimum valid interval (6 hours)
        let min_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 6,
            wifi_only: true,
            notify_on_complete: false,
        };

        let result = scheduler.update_sync_settings("test_profile", &min_settings);
        assert!(result.is_ok());

        // Verify
        let retrieved = scheduler.get_sync_settings("test_profile").unwrap();
        assert_eq!(retrieved.sync_interval_hours, 6);

        // Test maximum recommended interval (48 hours)
        let max_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 48,
            wifi_only: true,
            notify_on_complete: false,
        };

        let result = scheduler.update_sync_settings("test_profile", &max_settings);
        assert!(result.is_ok());

        // Verify
        let retrieved = scheduler.get_sync_settings("test_profile").unwrap();
        assert_eq!(retrieved.sync_interval_hours, 48);
    }

    #[test]
    fn test_sync_settings_persistence() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Update settings
        let settings = SyncSettings {
            auto_sync_enabled: false,
            sync_interval_hours: 24,
            wifi_only: false,
            notify_on_complete: true,
        };

        scheduler.update_sync_settings("test_profile", &settings).unwrap();

        // Create a new scheduler with the same database to simulate app restart
        let scheduler2 = SyncScheduler::new(Arc::clone(&db));

        // Retrieve settings with new scheduler
        let retrieved = scheduler2.get_sync_settings("test_profile").unwrap();

        assert_eq!(retrieved.auto_sync_enabled, false);
        assert_eq!(retrieved.sync_interval_hours, 24);
        assert_eq!(retrieved.wifi_only, false);
        assert_eq!(retrieved.notify_on_complete, true);
    }

    #[test]
    fn test_multiple_profiles_independent_settings() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "profile1");
        insert_test_profile(&db, "profile2");
        cache.initialize_profile("profile1").unwrap();
        cache.initialize_profile("profile2").unwrap();

        // Set different settings for each profile
        let settings1 = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 12,
            wifi_only: true,
            notify_on_complete: false,
        };

        let settings2 = SyncSettings {
            auto_sync_enabled: false,
            sync_interval_hours: 48,
            wifi_only: false,
            notify_on_complete: true,
        };

        scheduler.update_sync_settings("profile1", &settings1).unwrap();
        scheduler.update_sync_settings("profile2", &settings2).unwrap();

        // Verify each profile has independent settings
        let retrieved1 = scheduler.get_sync_settings("profile1").unwrap();
        let retrieved2 = scheduler.get_sync_settings("profile2").unwrap();

        assert_eq!(retrieved1.auto_sync_enabled, true);
        assert_eq!(retrieved1.sync_interval_hours, 12);
        assert_eq!(retrieved1.wifi_only, true);
        assert_eq!(retrieved1.notify_on_complete, false);

        assert_eq!(retrieved2.auto_sync_enabled, false);
        assert_eq!(retrieved2.sync_interval_hours, 48);
        assert_eq!(retrieved2.wifi_only, false);
        assert_eq!(retrieved2.notify_on_complete, true);
    }

    #[test]
    fn test_update_settings_multiple_times() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Update settings multiple times
        for i in 0..5 {
            let settings = SyncSettings {
                auto_sync_enabled: i % 2 == 0,
                sync_interval_hours: 6 + (i * 6),
                wifi_only: i % 2 == 1,
                notify_on_complete: i % 2 == 0,
            };

            let result = scheduler.update_sync_settings("test_profile", &settings);
            assert!(result.is_ok());

            // Verify each update
            let retrieved = scheduler.get_sync_settings("test_profile").unwrap();

            assert_eq!(retrieved.auto_sync_enabled, i % 2 == 0);
            assert_eq!(retrieved.sync_interval_hours, 6 + (i * 6));
            assert_eq!(retrieved.wifi_only, i % 2 == 1);
            assert_eq!(retrieved.notify_on_complete, i % 2 == 0);
        }
    }

    #[test]
    fn test_settings_error_handling() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Test with interval = 0 (should fail)
        let invalid_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 0,
            wifi_only: true,
            notify_on_complete: false,
        };

        let result = scheduler.update_sync_settings("test_profile", &invalid_settings);
        assert!(result.is_err());

        // Test with interval = 5 (should fail)
        let invalid_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 5,
            wifi_only: true,
            notify_on_complete: false,
        };

        let result = scheduler.update_sync_settings("test_profile", &invalid_settings);
        assert!(result.is_err());
    }

    #[test]
    fn test_settings_all_boolean_combinations() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Test all combinations of boolean flags
        for auto_sync in [true, false] {
            for wifi_only in [true, false] {
                for notify in [true, false] {
                    let settings = SyncSettings {
                        auto_sync_enabled: auto_sync,
                        sync_interval_hours: 24,
                        wifi_only,
                        notify_on_complete: notify,
                    };

                    scheduler.update_sync_settings("test_profile", &settings).unwrap();

                    let retrieved = scheduler.get_sync_settings("test_profile").unwrap();

                    assert_eq!(retrieved.auto_sync_enabled, auto_sync);
                    assert_eq!(retrieved.wifi_only, wifi_only);
                    assert_eq!(retrieved.notify_on_complete, notify);
                }
            }
        }
    }

    #[test]
    fn test_settings_interval_values() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Test various valid interval values
        let valid_intervals = vec![6, 12, 24, 48, 72, 168]; // 6h, 12h, 24h, 48h, 72h, 1 week

        for interval in valid_intervals {
            let settings = SyncSettings {
                auto_sync_enabled: true,
                sync_interval_hours: interval,
                wifi_only: true,
                notify_on_complete: false,
            };

            let result = scheduler.update_sync_settings("test_profile", &settings);
            assert!(result.is_ok(), "Failed for interval: {}", interval);

            let retrieved = scheduler.get_sync_settings("test_profile").unwrap();
            assert_eq!(retrieved.sync_interval_hours, interval);
        }
    }
}
