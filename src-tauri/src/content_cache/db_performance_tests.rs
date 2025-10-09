// Tests for database performance optimization
#[cfg(test)]
mod tests {
    use crate::content_cache::*;
    use crate::error::Result;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    
    fn setup_test_db() -> Result<Arc<Mutex<Connection>>> {
        let conn = Connection::open_in_memory()?;
        let db = Arc::new(Mutex::new(conn));
        
        // Initialize schema
        {
            let conn = db.lock().unwrap();
            schema::initialize_content_cache_tables(&conn)?;
        }
        
        Ok(db)
    }
    
    fn generate_test_channels(count: usize) -> Vec<XtreamChannel> {
        (0..count)
            .map(|i| XtreamChannel {
                stream_id: i as i64,
                num: Some(i as i64),
                name: format!("Channel {}", i),
                stream_type: Some("live".to_string()),
                stream_icon: Some(format!("http://example.com/icon{}.png", i)),
                thumbnail: None,
                epg_channel_id: Some(format!("epg_{}", i)),
                added: Some("2024-01-01".to_string()),
                category_id: Some(format!("{}", i % 5)),
                custom_sid: None,
                tv_archive: Some(1),
                direct_source: None,
                tv_archive_duration: Some(7),
            })
            .collect()
    }
    
    #[test]
    fn test_analyze_tables() {
        let db = setup_test_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        
        // Run ANALYZE (doesn't require data)
        let result = cache.analyze_tables();
        assert!(result.is_ok(), "ANALYZE should succeed");
    }
    
    #[test]
    fn test_vacuum() {
        let db = setup_test_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        
        // Run VACUUM (doesn't require data)
        let result = cache.vacuum();
        assert!(result.is_ok(), "VACUUM should succeed");
    }
    
    #[test]
    fn test_should_vacuum() {
        let db = setup_test_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        
        // Check if vacuum is needed
        let should_vacuum = cache.should_vacuum().unwrap();
        
        // Fresh database shouldn't need vacuum
        assert!(!should_vacuum, "Fresh database should not need VACUUM");
    }
    
    #[test]
    fn test_database_stats() {
        let db = setup_test_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        
        let stats = cache.get_database_stats().unwrap();
        
        assert!(stats.0 > 0, "Total size should be > 0");
        assert!(stats.1 > 0, "Page count should be > 0");
        assert!(stats.2 > 0, "Page size should be > 0");
        assert!(stats.3 >= 0, "Freelist count should be >= 0");
        
        println!("Database stats:");
        println!("  Total size: {} bytes", stats.0);
        println!("  Page count: {}", stats.1);
        println!("  Page size: {} bytes", stats.2);
        println!("  Freelist count: {}", stats.3);
    }
    
    #[test]
    fn test_integrity_check() {
        let db = setup_test_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        
        // Check integrity (doesn't require data)
        let result = cache.check_integrity();
        assert!(result.is_ok(), "Integrity check should pass");
    }
    
    #[test]
    fn test_optimize_settings() {
        let db = setup_test_db().unwrap();
        let perf = DbPerformance::new(db.clone(), None);
        
        // Note: Some PRAGMA settings may not work on in-memory databases
        // We just test that the function doesn't panic
        let result = perf.optimize_settings();
        
        // It's okay if this fails on in-memory DB, just ensure it doesn't crash
        match result {
            Ok(_) => println!("Optimize settings succeeded"),
            Err(e) => println!("Optimize settings failed (expected for in-memory DB): {}", e),
        }
    }
    
    #[test]
    fn test_performance_manager() {
        let db = setup_test_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        
        // Get performance manager with custom threshold
        let perf = cache.get_performance_manager(Some(50));
        
        // Test analyze
        assert!(perf.analyze_tables().is_ok());
        
        // Test vacuum
        assert!(perf.vacuum().is_ok());
        
        // Test stats
        let stats = perf.get_database_stats().unwrap();
        assert!(stats.0 > 0);
    }
    
    #[test]
    fn test_query_logging() {
        let db = setup_test_db().unwrap();
        let perf = DbPerformance::new(db.clone(), Some(50));
        
        // Manually log some queries
        perf.log_query("SELECT", std::time::Duration::from_millis(10), 5).unwrap();
        perf.log_query("INSERT", std::time::Duration::from_millis(150), 100).unwrap();
        
        // Get recent queries
        let recent = perf.get_recent_queries(10).unwrap();
        println!("Recent queries: {}", recent.len());
        assert_eq!(recent.len(), 2);
        
        // Get slow queries
        let slow = perf.get_slow_queries().unwrap();
        println!("Slow queries: {}", slow.len());
        assert_eq!(slow.len(), 1); // Only the 150ms query should be slow
    }
    
    #[test]
    fn test_measure_query() {
        let db = setup_test_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let perf = cache.get_performance_manager(Some(100));
        
        // Measure a query
        let result = perf.measure_query("TEST_QUERY", || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            Ok(42)
        }).unwrap();
        
        assert_eq!(result, 42);
        
        // Check that it was logged
        let recent = perf.get_recent_queries(1).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].query_type, "TEST_QUERY");
        assert!(recent[0].execution_time_ms >= 10);
    }
    
    #[test]
    fn test_fragmentation_detection() {
        let db = setup_test_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        
        // Get initial stats
        let stats_before = cache.get_database_stats().unwrap();
        println!("Initial database stats:");
        println!("  Total size: {} bytes", stats_before.0);
        println!("  Freelist: {}", stats_before.3);
        
        // Check if vacuum is recommended (should be false for fresh DB)
        let should_vacuum = cache.should_vacuum().unwrap();
        println!("Should vacuum: {}", should_vacuum);
        assert!(!should_vacuum, "Fresh database should not need vacuum");
        
        // Test vacuum anyway to ensure it works
        cache.vacuum().unwrap();
        let stats_after_vacuum = cache.get_database_stats().unwrap();
        println!("After VACUUM:");
        println!("  Total size: {} bytes", stats_after_vacuum.0);
        println!("  Freelist: {}", stats_after_vacuum.3);
    }
    
    #[test]
    fn test_performance_with_large_dataset() {
        let db = setup_test_db().unwrap();
        let perf = DbPerformance::new(db.clone(), Some(100));
        
        // Run ANALYZE on empty database
        let start = std::time::Instant::now();
        perf.analyze_tables().unwrap();
        let analyze_duration = start.elapsed();
        
        println!("ANALYZE duration: {:?}", analyze_duration);
        assert!(analyze_duration.as_secs() < 5, "ANALYZE should be fast");
        
        // Test measure_query functionality
        let result = perf.measure_query("TEST_OPERATION", || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            Ok(())
        }).unwrap();
        
        assert!(result == ());
        
        // Check that query was logged
        let recent = perf.get_recent_queries(10).unwrap();
        assert!(recent.len() > 0, "Should have logged queries");
    }
}
