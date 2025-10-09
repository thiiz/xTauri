// Database performance optimization module
use crate::error::{Result, XTauriError};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Performance metrics for database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub query_type: String,
    pub execution_time_ms: u64,
    pub rows_affected: usize,
    pub timestamp: String,
}

/// Database performance optimizer
pub struct DbPerformance {
    db: Arc<Mutex<Connection>>,
    query_log: Arc<Mutex<Vec<QueryMetrics>>>,
    slow_query_threshold_ms: u64,
}

impl DbPerformance {
    /// Create a new DbPerformance instance
    /// 
    /// # Arguments
    /// * `db` - Shared database connection
    /// * `slow_query_threshold_ms` - Threshold in milliseconds for logging slow queries (default: 100ms)
    pub fn new(db: Arc<Mutex<Connection>>, slow_query_threshold_ms: Option<u64>) -> Self {
        Self {
            db,
            query_log: Arc::new(Mutex::new(Vec::new())),
            slow_query_threshold_ms: slow_query_threshold_ms.unwrap_or(100),
        }
    }
    
    /// Run ANALYZE on all content cache tables
    /// 
    /// ANALYZE gathers statistics about the distribution of data in tables and indexes.
    /// This helps the query optimizer make better decisions about query plans.
    /// Should be run after bulk inserts or significant data changes.
    /// 
    /// # Returns
    /// Ok(()) if successful, error otherwise
    pub fn analyze_tables(&self) -> Result<()> {
        let start = Instant::now();
        
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        // Run ANALYZE on all content cache tables
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
            conn.execute(&format!("ANALYZE {}", table), [])?;
        }
        
        // Also run global ANALYZE
        conn.execute("ANALYZE", [])?;
        
        let duration = start.elapsed();
        
        println!(
            "[INFO] Database ANALYZE completed in {:?}",
            duration
        );
        
        self.log_query("ANALYZE", duration, 0)?;
        
        Ok(())
    }
    
    /// Run VACUUM to reclaim unused space and defragment the database
    /// 
    /// VACUUM rebuilds the database file, repacking it into a minimal amount of disk space.
    /// This operation can be expensive and should be run periodically, not frequently.
    /// 
    /// Note: VACUUM cannot be run inside a transaction.
    /// 
    /// # Returns
    /// Ok(()) if successful, error otherwise
    pub fn vacuum(&self) -> Result<()> {
        let start = Instant::now();
        
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        conn.execute("VACUUM", [])?;
        
        let duration = start.elapsed();
        
        println!(
            "[INFO] Database VACUUM completed in {:?}",
            duration
        );
        
        self.log_query("VACUUM", duration, 0)?;
        
        Ok(())
    }
    
    /// Check if VACUUM is needed based on database fragmentation
    /// 
    /// Returns true if the database has significant fragmentation (>20% free pages)
    /// 
    /// # Returns
    /// true if VACUUM is recommended, false otherwise
    pub fn should_vacuum(&self) -> Result<bool> {
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        // Get page count and freelist count
        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let freelist_count: i64 = conn.query_row("PRAGMA freelist_count", [], |row| row.get(0))?;
        
        if page_count == 0 {
            return Ok(false);
        }
        
        let fragmentation_ratio = (freelist_count as f64) / (page_count as f64);
        
        // Recommend VACUUM if more than 20% of pages are free
        Ok(fragmentation_ratio > 0.20)
    }
    
    /// Get database size statistics
    /// 
    /// # Returns
    /// Tuple of (total_size_bytes, page_count, page_size, freelist_count)
    pub fn get_database_stats(&self) -> Result<(u64, i64, i64, i64)> {
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
        let freelist_count: i64 = conn.query_row("PRAGMA freelist_count", [], |row| row.get(0))?;
        
        let total_size = (page_count * page_size) as u64;
        
        Ok((total_size, page_count, page_size, freelist_count))
    }
    
    /// Log a query execution for performance monitoring
    /// 
    /// # Arguments
    /// * `query_type` - Type of query (e.g., "SELECT", "INSERT", "ANALYZE")
    /// * `duration` - Execution time
    /// * `rows_affected` - Number of rows affected
    pub fn log_query(&self, query_type: &str, duration: Duration, rows_affected: usize) -> Result<()> {
        let execution_time_ms = duration.as_millis() as u64;
        
        // Log slow queries
        if execution_time_ms > self.slow_query_threshold_ms {
            eprintln!(
                "[WARN] Slow query detected: type='{}', duration={:?}, rows={}",
                query_type, duration, rows_affected
            );
        }
        
        let metric = QueryMetrics {
            query_type: query_type.to_string(),
            execution_time_ms,
            rows_affected,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let mut log = self.query_log.lock()
            .map_err(|_| XTauriError::lock_acquisition("query log"))?;
        
        log.push(metric);
        
        // Keep only last 1000 entries to prevent unbounded growth
        if log.len() > 1000 {
            let drain_count = log.len() - 1000;
            log.drain(0..drain_count);
        }
        
        Ok(())
    }
    
    /// Get recent query metrics
    /// 
    /// # Arguments
    /// * `limit` - Maximum number of recent queries to return
    /// 
    /// # Returns
    /// Vector of recent query metrics
    pub fn get_recent_queries(&self, limit: usize) -> Result<Vec<QueryMetrics>> {
        let log = self.query_log.lock()
            .map_err(|_| XTauriError::lock_acquisition("query log"))?;
        
        let start = if log.len() > limit {
            log.len() - limit
        } else {
            0
        };
        
        Ok(log[start..].to_vec())
    }
    
    /// Get slow query statistics
    /// 
    /// # Returns
    /// Vector of queries that exceeded the slow query threshold
    pub fn get_slow_queries(&self) -> Result<Vec<QueryMetrics>> {
        let log = self.query_log.lock()
            .map_err(|_| XTauriError::lock_acquisition("query log"))?;
        
        let slow_queries: Vec<QueryMetrics> = log.iter()
            .filter(|m| m.execution_time_ms > self.slow_query_threshold_ms)
            .cloned()
            .collect();
        
        Ok(slow_queries)
    }
    
    /// Clear query log
    pub fn clear_query_log(&self) -> Result<()> {
        let mut log = self.query_log.lock()
            .map_err(|_| XTauriError::lock_acquisition("query log"))?;
        
        log.clear();
        
        Ok(())
    }
    
    /// Measure execution time of a query
    /// 
    /// This is a helper function to wrap query execution with timing
    /// 
    /// # Arguments
    /// * `query_type` - Type of query for logging
    /// * `f` - Function to execute
    /// 
    /// # Returns
    /// Result of the function execution
    pub fn measure_query<F, T>(&self, query_type: &str, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let start = Instant::now();
        let result = f()?;
        let duration = start.elapsed();
        
        self.log_query(query_type, duration, 0)?;
        
        Ok(result)
    }
    
    /// Run database integrity check
    /// 
    /// # Returns
    /// Ok(()) if database is healthy, error with details if corruption detected
    pub fn check_integrity(&self) -> Result<()> {
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        let result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        
        if result != "ok" {
            return Err(XTauriError::Internal {
                reason: format!("Database integrity check failed: {}", result),
            });
        }
        
        println!("[INFO] Database integrity check passed");
        
        Ok(())
    }
    
    /// Optimize database settings for performance
    /// 
    /// Sets various PRAGMA settings for better performance
    pub fn optimize_settings(&self) -> Result<()> {
        let conn = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        // Set journal mode to WAL for better concurrency
        conn.execute("PRAGMA journal_mode=WAL", [])?;
        
        // Increase cache size (negative value = KB, -64000 = 64MB)
        conn.execute("PRAGMA cache_size=-64000", [])?;
        
        // Use memory for temp storage
        conn.execute("PRAGMA temp_store=MEMORY", [])?;
        
        // Synchronous mode to NORMAL for better performance (still safe with WAL)
        conn.execute("PRAGMA synchronous=NORMAL", [])?;
        
        // Enable memory-mapped I/O (256MB)
        conn.execute("PRAGMA mmap_size=268435456", [])?;
        
        println!("[INFO] Database performance settings optimized");
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    
    fn setup_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        Arc::new(Mutex::new(conn))
    }
    
    #[test]
    fn test_analyze_tables() {
        let db = setup_test_db();
        let perf = DbPerformance::new(db.clone(), None);
        
        // Should not fail even with empty database
        assert!(perf.analyze_tables().is_ok());
    }
    
    #[test]
    fn test_vacuum() {
        let db = setup_test_db();
        let perf = DbPerformance::new(db.clone(), None);
        
        assert!(perf.vacuum().is_ok());
    }
    
    #[test]
    fn test_database_stats() {
        let db = setup_test_db();
        let perf = DbPerformance::new(db.clone(), None);
        
        let stats = perf.get_database_stats().unwrap();
        assert!(stats.0 > 0); // total_size
        assert!(stats.1 > 0); // page_count
        assert!(stats.2 > 0); // page_size
    }
    
    #[test]
    fn test_query_logging() {
        let db = setup_test_db();
        let perf = DbPerformance::new(db.clone(), Some(50));
        
        // Log a fast query
        perf.log_query("SELECT", Duration::from_millis(10), 5).unwrap();
        
        // Log a slow query
        perf.log_query("SELECT", Duration::from_millis(150), 100).unwrap();
        
        let recent = perf.get_recent_queries(10).unwrap();
        assert_eq!(recent.len(), 2);
        
        let slow = perf.get_slow_queries().unwrap();
        assert_eq!(slow.len(), 1);
        assert_eq!(slow[0].execution_time_ms, 150);
    }
    
    #[test]
    fn test_measure_query() {
        let db = setup_test_db();
        let perf = DbPerformance::new(db.clone(), None);
        
        let result = perf.measure_query("TEST", || {
            std::thread::sleep(Duration::from_millis(10));
            Ok(42)
        }).unwrap();
        
        assert_eq!(result, 42);
        
        let recent = perf.get_recent_queries(1).unwrap();
        assert_eq!(recent.len(), 1);
        assert!(recent[0].execution_time_ms >= 10);
    }
    
    #[test]
    fn test_integrity_check() {
        let db = setup_test_db();
        let perf = DbPerformance::new(db.clone(), None);
        
        assert!(perf.check_integrity().is_ok());
    }
    
    #[test]
    fn test_optimize_settings() {
        let db = setup_test_db();
        let perf = DbPerformance::new(db.clone(), None);
        
        assert!(perf.optimize_settings().is_ok());
    }
}
