// Integration tests for sync control commands
#[cfg(test)]
mod tests {
    use crate::content_cache::{
        ContentCache, SyncScheduler, SyncProgress, SyncSettings, SyncStatus,
    };
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    use tokio_util::sync::CancellationToken;

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
    fn test_get_sync_status_uninitialized() {
        let (db, _cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");

        // Get status for uninitialized profile should return default pending status
        let status = scheduler.get_sync_status("test_profile").unwrap();
        assert_eq!(status.status, SyncStatus::Pending);
        assert_eq!(status.progress, 0);
        assert_eq!(status.channels_synced, 0);
        assert_eq!(status.movies_synced, 0);
        assert_eq!(status.series_synced, 0);
    }

    #[test]
    fn test_update_sync_status() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Update sync status
        let progress = SyncProgress {
            status: SyncStatus::Syncing,
            progress: 50,
            current_step: "Syncing channels...".to_string(),
            channels_synced: 100,
            movies_synced: 0,
            series_synced: 0,
            errors: Vec::new(),
        };

        scheduler
            .update_sync_status("test_profile", &progress)
            .unwrap();

        // Verify status was updated
        let status = scheduler.get_sync_status("test_profile").unwrap();
        assert_eq!(status.status, SyncStatus::Syncing);
        assert_eq!(status.progress, 50);
        assert_eq!(status.current_step, "Syncing channels...");
        assert_eq!(status.channels_synced, 100);
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
    fn test_update_sync_settings() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Update settings
        let new_settings = SyncSettings {
            auto_sync_enabled: false,
            sync_interval_hours: 12,
            wifi_only: false,
            notify_on_complete: true,
        };

        scheduler
            .update_sync_settings("test_profile", &new_settings)
            .unwrap();

        // Verify settings were updated
        let settings = scheduler.get_sync_settings("test_profile").unwrap();
        assert_eq!(settings.auto_sync_enabled, false);
        assert_eq!(settings.sync_interval_hours, 12);
        assert_eq!(settings.wifi_only, false);
        assert_eq!(settings.notify_on_complete, true);
    }

    #[test]
    fn test_update_sync_settings_validation() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Try to set invalid interval (less than 6 hours)
        let invalid_settings = SyncSettings {
            auto_sync_enabled: true,
            sync_interval_hours: 3,
            wifi_only: true,
            notify_on_complete: false,
        };

        let result = scheduler.update_sync_settings("test_profile", &invalid_settings);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 6 hours"));
    }

    #[test]
    fn test_is_sync_active() {
        let (db, _cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");

        // Initially no sync should be active
        assert!(!scheduler.is_sync_active("test_profile").unwrap());

        // Register a sync
        let cancel_token = CancellationToken::new();
        scheduler
            .register_sync("test_profile", cancel_token.clone())
            .unwrap();

        // Now sync should be active
        assert!(scheduler.is_sync_active("test_profile").unwrap());

        // Unregister sync
        scheduler.unregister_sync("test_profile").unwrap();

        // Sync should no longer be active
        assert!(!scheduler.is_sync_active("test_profile").unwrap());
    }

    #[test]
    fn test_register_sync_duplicate() {
        let (db, _cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");

        // Register first sync
        let cancel_token1 = CancellationToken::new();
        scheduler
            .register_sync("test_profile", cancel_token1)
            .unwrap();

        // Try to register second sync for same profile
        let cancel_token2 = CancellationToken::new();
        let result = scheduler.register_sync("test_profile", cancel_token2);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already in progress"));
    }

    #[test]
    fn test_cancel_sync() {
        let (db, _cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");

        // Register a sync
        let cancel_token = CancellationToken::new();
        scheduler
            .register_sync("test_profile", cancel_token.clone())
            .unwrap();

        // Cancel the sync
        scheduler.cancel_sync("test_profile").unwrap();

        // Verify cancellation token was triggered
        assert!(cancel_token.is_cancelled());
    }

    #[test]
    fn test_cancel_sync_not_active() {
        let (_db, _cache, scheduler) = setup_test_environment();

        // Try to cancel sync that doesn't exist
        let result = scheduler.cancel_sync("nonexistent_profile");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No active sync"));
    }

    #[test]
    fn test_active_sync_count() {
        let (db, _cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "profile1");
        insert_test_profile(&db, "profile2");

        // Initially no active syncs
        assert_eq!(scheduler.active_sync_count().unwrap(), 0);

        // Register first sync
        let token1 = CancellationToken::new();
        scheduler.register_sync("profile1", token1).unwrap();
        assert_eq!(scheduler.active_sync_count().unwrap(), 1);

        // Register second sync
        let token2 = CancellationToken::new();
        scheduler.register_sync("profile2", token2).unwrap();
        assert_eq!(scheduler.active_sync_count().unwrap(), 2);

        // Unregister first sync
        scheduler.unregister_sync("profile1").unwrap();
        assert_eq!(scheduler.active_sync_count().unwrap(), 1);

        // Unregister second sync
        scheduler.unregister_sync("profile2").unwrap();
        assert_eq!(scheduler.active_sync_count().unwrap(), 0);
    }

    #[test]
    fn test_update_last_sync_timestamp() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Update timestamp for channels
        scheduler
            .update_last_sync_timestamp("test_profile", "channels")
            .unwrap();

        // Verify timestamp was set (we can't check exact value, but we can verify no error)
        let status = scheduler.get_sync_status("test_profile").unwrap();
        // Status should still be pending since we only updated timestamp
        assert_eq!(status.status, SyncStatus::Pending);
    }

    #[test]
    fn test_update_last_sync_timestamp_invalid_type() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "test_profile");
        cache.initialize_profile("test_profile").unwrap();

        // Try to update with invalid content type
        let result = scheduler.update_last_sync_timestamp("test_profile", "invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid content type"));
    }

    #[test]
    fn test_calculate_progress() {
        // Test progress calculation
        assert_eq!(SyncScheduler::calculate_progress(0, 6, 0.0), 0);
        assert_eq!(SyncScheduler::calculate_progress(0, 6, 0.5), 8);
        assert_eq!(SyncScheduler::calculate_progress(1, 6, 0.0), 16);
        assert_eq!(SyncScheduler::calculate_progress(3, 6, 0.5), 58);
        assert_eq!(SyncScheduler::calculate_progress(6, 6, 0.0), 100);

        // Edge cases
        assert_eq!(SyncScheduler::calculate_progress(0, 0, 0.0), 100);
        assert_eq!(SyncScheduler::calculate_progress(10, 6, 1.0), 100); // Clamped to 100
    }

    #[tokio::test]
    async fn test_sync_handle_creation() {
        use crate::content_cache::SyncHandle;

        let (handle, progress_tx, cancel_token) = SyncHandle::new("test_profile".to_string());

        assert_eq!(handle.profile_id, "test_profile");
        assert!(!handle.is_cancelled());

        // Test cancellation
        handle.cancel();
        assert!(handle.is_cancelled());
        assert!(cancel_token.is_cancelled());

        // Test progress channel
        let test_progress = SyncProgress {
            status: SyncStatus::Syncing,
            progress: 50,
            current_step: "Test".to_string(),
            channels_synced: 0,
            movies_synced: 0,
            series_synced: 0,
            errors: Vec::new(),
        };

        progress_tx.send(test_progress.clone()).await.unwrap();
        // Note: We can't easily test receiving here without more complex async setup
    }

    #[test]
    fn test_sync_status_serialization() {
        // Test that SyncStatus can be serialized/deserialized
        let status = SyncStatus::Syncing;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: SyncStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_sync_progress_serialization() {
        // Test that SyncProgress can be serialized/deserialized
        let progress = SyncProgress {
            status: SyncStatus::Completed,
            progress: 100,
            current_step: "Done".to_string(),
            channels_synced: 100,
            movies_synced: 200,
            series_synced: 50,
            errors: vec!["Error 1".to_string()],
        };

        let json = serde_json::to_string(&progress).unwrap();
        let deserialized: SyncProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(progress.status, deserialized.status);
        assert_eq!(progress.progress, deserialized.progress);
        assert_eq!(progress.current_step, deserialized.current_step);
        assert_eq!(progress.channels_synced, deserialized.channels_synced);
        assert_eq!(progress.movies_synced, deserialized.movies_synced);
        assert_eq!(progress.series_synced, deserialized.series_synced);
        assert_eq!(progress.errors, deserialized.errors);
    }

    #[test]
    fn test_sync_settings_serialization() {
        // Test that SyncSettings can be serialized/deserialized
        let settings = SyncSettings {
            auto_sync_enabled: false,
            sync_interval_hours: 12,
            wifi_only: false,
            notify_on_complete: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: SyncSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.auto_sync_enabled, deserialized.auto_sync_enabled);
        assert_eq!(
            settings.sync_interval_hours,
            deserialized.sync_interval_hours
        );
        assert_eq!(settings.wifi_only, deserialized.wifi_only);
        assert_eq!(
            settings.notify_on_complete,
            deserialized.notify_on_complete
        );
    }

    #[test]
    fn test_sync_status_db_conversion() {
        // Test conversion to/from database strings
        assert_eq!(SyncStatus::Pending.to_db_string(), "pending");
        assert_eq!(SyncStatus::Syncing.to_db_string(), "syncing");
        assert_eq!(SyncStatus::Completed.to_db_string(), "completed");
        assert_eq!(SyncStatus::Failed.to_db_string(), "failed");
        assert_eq!(SyncStatus::Partial.to_db_string(), "partial");

        assert_eq!(SyncStatus::from_db_string("pending"), SyncStatus::Pending);
        assert_eq!(SyncStatus::from_db_string("syncing"), SyncStatus::Syncing);
        assert_eq!(
            SyncStatus::from_db_string("completed"),
            SyncStatus::Completed
        );
        assert_eq!(SyncStatus::from_db_string("failed"), SyncStatus::Failed);
        assert_eq!(SyncStatus::from_db_string("partial"), SyncStatus::Partial);
        assert_eq!(
            SyncStatus::from_db_string("unknown"),
            SyncStatus::Pending
        ); // Default
    }

    #[test]
    fn test_sync_progress_default() {
        let progress = SyncProgress::default();
        assert_eq!(progress.status, SyncStatus::Pending);
        assert_eq!(progress.progress, 0);
        assert_eq!(progress.current_step, "");
        assert_eq!(progress.channels_synced, 0);
        assert_eq!(progress.movies_synced, 0);
        assert_eq!(progress.series_synced, 0);
        assert_eq!(progress.errors.len(), 0);
    }

    #[test]
    fn test_sync_settings_default() {
        let settings = SyncSettings::default();
        assert_eq!(settings.auto_sync_enabled, true);
        assert_eq!(settings.sync_interval_hours, 24);
        assert_eq!(settings.wifi_only, true);
        assert_eq!(settings.notify_on_complete, false);
    }

    #[test]
    fn test_multiple_profiles_sync_status() {
        let (db, cache, scheduler) = setup_test_environment();
        insert_test_profile(&db, "profile1");
        insert_test_profile(&db, "profile2");
        cache.initialize_profile("profile1").unwrap();
        cache.initialize_profile("profile2").unwrap();

        // Update status for profile1
        let progress1 = SyncProgress {
            status: SyncStatus::Syncing,
            progress: 30,
            current_step: "Profile 1 syncing".to_string(),
            channels_synced: 50,
            movies_synced: 0,
            series_synced: 0,
            errors: Vec::new(),
        };
        scheduler
            .update_sync_status("profile1", &progress1)
            .unwrap();

        // Update status for profile2
        let progress2 = SyncProgress {
            status: SyncStatus::Completed,
            progress: 100,
            current_step: "Profile 2 complete".to_string(),
            channels_synced: 100,
            movies_synced: 200,
            series_synced: 50,
            errors: Vec::new(),
        };
        scheduler
            .update_sync_status("profile2", &progress2)
            .unwrap();

        // Verify each profile has independent status
        let status1 = scheduler.get_sync_status("profile1").unwrap();
        assert_eq!(status1.status, SyncStatus::Syncing);
        assert_eq!(status1.progress, 30);
        assert_eq!(status1.channels_synced, 50);

        let status2 = scheduler.get_sync_status("profile2").unwrap();
        assert_eq!(status2.status, SyncStatus::Completed);
        assert_eq!(status2.progress, 100);
        assert_eq!(status2.channels_synced, 100);
    }
}
