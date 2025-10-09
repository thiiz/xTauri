use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Performance metrics for monitoring system performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation_metrics: HashMap<String, OperationMetrics>,
    pub cache_metrics: CacheMetrics,
    pub api_metrics: ApiMetrics,
    pub database_metrics: DatabaseMetrics,
    pub last_updated: DateTime<Utc>,
}

/// Metrics for individual operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    pub operation_name: String,
    pub total_calls: u64,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub error_count: u64,
    pub last_execution: Option<DateTime<Utc>>,
}

/// Cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub eviction_count: u64,
    pub prefetch_hit_rate: f64,
    pub avg_access_time: Duration,
    pub memory_usage_bytes: usize,
    pub disk_usage_bytes: usize,
}

/// API call metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time: Duration,
    pub timeout_count: u64,
    pub retry_count: u64,
}

/// Database operation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub total_queries: u64,
    pub avg_query_time: Duration,
    pub slow_query_count: u64,
    pub connection_pool_size: usize,
    pub active_connections: usize,
}

/// Performance monitor for tracking system performance
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<PerformanceMetrics>>,
    slow_query_threshold: Duration,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PerformanceMetrics {
                operation_metrics: HashMap::new(),
                cache_metrics: CacheMetrics {
                    hit_rate: 0.0,
                    miss_rate: 0.0,
                    total_hits: 0,
                    total_misses: 0,
                    eviction_count: 0,
                    prefetch_hit_rate: 0.0,
                    avg_access_time: Duration::from_millis(0),
                    memory_usage_bytes: 0,
                    disk_usage_bytes: 0,
                },
                api_metrics: ApiMetrics {
                    total_requests: 0,
                    successful_requests: 0,
                    failed_requests: 0,
                    avg_response_time: Duration::from_millis(0),
                    timeout_count: 0,
                    retry_count: 0,
                },
                database_metrics: DatabaseMetrics {
                    total_queries: 0,
                    avg_query_time: Duration::from_millis(0),
                    slow_query_count: 0,
                    connection_pool_size: 1,
                    active_connections: 0,
                },
                last_updated: Utc::now(),
            })),
            slow_query_threshold: Duration::from_millis(100),
        }
    }

    /// Record an operation execution
    pub fn record_operation(&self, operation_name: &str, duration: Duration, success: bool) {
        let mut metrics = self.metrics.lock().unwrap();
        
        let op_metrics = metrics.operation_metrics
            .entry(operation_name.to_string())
            .or_insert_with(|| OperationMetrics {
                operation_name: operation_name.to_string(),
                total_calls: 0,
                total_duration: Duration::from_secs(0),
                avg_duration: Duration::from_secs(0),
                min_duration: Duration::from_secs(u64::MAX),
                max_duration: Duration::from_secs(0),
                error_count: 0,
                last_execution: None,
            });

        op_metrics.total_calls += 1;
        op_metrics.total_duration += duration;
        op_metrics.avg_duration = op_metrics.total_duration / op_metrics.total_calls as u32;
        op_metrics.min_duration = op_metrics.min_duration.min(duration);
        op_metrics.max_duration = op_metrics.max_duration.max(duration);
        op_metrics.last_execution = Some(Utc::now());
        
        if !success {
            op_metrics.error_count += 1;
        }

        metrics.last_updated = Utc::now();
    }

    /// Record cache hit
    pub fn record_cache_hit(&self, access_time: Duration) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.cache_metrics.total_hits += 1;
        
        // Update average access time
        let total_accesses = metrics.cache_metrics.total_hits + metrics.cache_metrics.total_misses;
        if total_accesses > 0 {
            let total_time = metrics.cache_metrics.avg_access_time.as_nanos() as f64 * (total_accesses - 1) as f64;
            let new_avg = (total_time + access_time.as_nanos() as f64) / total_accesses as f64;
            metrics.cache_metrics.avg_access_time = Duration::from_nanos(new_avg as u64);
        }
        
        self.update_cache_rates(&mut metrics);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.cache_metrics.total_misses += 1;
        self.update_cache_rates(&mut metrics);
    }

    /// Record cache eviction
    pub fn record_cache_eviction(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.cache_metrics.eviction_count += 1;
    }

    /// Update cache hit/miss rates
    fn update_cache_rates(&self, metrics: &mut PerformanceMetrics) {
        let total = metrics.cache_metrics.total_hits + metrics.cache_metrics.total_misses;
        if total > 0 {
            metrics.cache_metrics.hit_rate = metrics.cache_metrics.total_hits as f64 / total as f64;
            metrics.cache_metrics.miss_rate = metrics.cache_metrics.total_misses as f64 / total as f64;
        }
        metrics.last_updated = Utc::now();
    }

    /// Record API request
    pub fn record_api_request(&self, duration: Duration, success: bool, was_retry: bool, was_timeout: bool) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.api_metrics.total_requests += 1;
        
        if success {
            metrics.api_metrics.successful_requests += 1;
        } else {
            metrics.api_metrics.failed_requests += 1;
        }
        
        if was_retry {
            metrics.api_metrics.retry_count += 1;
        }
        
        if was_timeout {
            metrics.api_metrics.timeout_count += 1;
        }
        
        // Update average response time
        let total_successful = metrics.api_metrics.successful_requests;
        if total_successful > 0 && success {
            let total_time = metrics.api_metrics.avg_response_time.as_nanos() as f64 * (total_successful - 1) as f64;
            let new_avg = (total_time + duration.as_nanos() as f64) / total_successful as f64;
            metrics.api_metrics.avg_response_time = Duration::from_nanos(new_avg as u64);
        }
        
        metrics.last_updated = Utc::now();
    }

    /// Record database query
    pub fn record_database_query(&self, duration: Duration) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.database_metrics.total_queries += 1;
        
        // Check if this is a slow query
        if duration > self.slow_query_threshold {
            metrics.database_metrics.slow_query_count += 1;
        }
        
        // Update average query time
        let total_queries = metrics.database_metrics.total_queries;
        let total_time = metrics.database_metrics.avg_query_time.as_nanos() as f64 * (total_queries - 1) as f64;
        let new_avg = (total_time + duration.as_nanos() as f64) / total_queries as f64;
        metrics.database_metrics.avg_query_time = Duration::from_nanos(new_avg as u64);
        
        metrics.last_updated = Utc::now();
    }

    /// Update cache memory usage
    pub fn update_cache_memory_usage(&self, memory_bytes: usize, disk_bytes: usize) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.cache_metrics.memory_usage_bytes = memory_bytes;
        metrics.cache_metrics.disk_usage_bytes = disk_bytes;
        metrics.last_updated = Utc::now();
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Reset all metrics
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics = PerformanceMetrics {
            operation_metrics: HashMap::new(),
            cache_metrics: CacheMetrics {
                hit_rate: 0.0,
                miss_rate: 0.0,
                total_hits: 0,
                total_misses: 0,
                eviction_count: 0,
                prefetch_hit_rate: 0.0,
                avg_access_time: Duration::from_millis(0),
                memory_usage_bytes: 0,
                disk_usage_bytes: 0,
            },
            api_metrics: ApiMetrics {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                avg_response_time: Duration::from_millis(0),
                timeout_count: 0,
                retry_count: 0,
            },
            database_metrics: DatabaseMetrics {
                total_queries: 0,
                avg_query_time: Duration::from_millis(0),
                slow_query_count: 0,
                connection_pool_size: 1,
                active_connections: 0,
            },
            last_updated: Utc::now(),
        };
    }

    /// Get operation metrics for a specific operation
    pub fn get_operation_metrics(&self, operation_name: &str) -> Option<OperationMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.operation_metrics.get(operation_name).cloned()
    }

    /// Get slow operations (operations with avg duration > threshold)
    pub fn get_slow_operations(&self, threshold: Duration) -> Vec<OperationMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.operation_metrics
            .values()
            .filter(|op| op.avg_duration > threshold)
            .cloned()
            .collect()
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper struct for timing operations
pub struct OperationTimer {
    start: Instant,
    operation_name: String,
    monitor: Arc<PerformanceMonitor>,
}

impl OperationTimer {
    /// Start timing an operation
    pub fn start(operation_name: String, monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            start: Instant::now(),
            operation_name,
            monitor,
        }
    }

    /// Complete the operation and record metrics
    pub fn complete(self, success: bool) {
        let duration = self.start.elapsed();
        self.monitor.record_operation(&self.operation_name, duration, success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_record_operation() {
        let monitor = PerformanceMonitor::new();
        
        monitor.record_operation("test_op", Duration::from_millis(100), true);
        monitor.record_operation("test_op", Duration::from_millis(200), true);
        
        let metrics = monitor.get_operation_metrics("test_op").unwrap();
        assert_eq!(metrics.total_calls, 2);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.min_duration, Duration::from_millis(100));
        assert_eq!(metrics.max_duration, Duration::from_millis(200));
    }

    #[test]
    fn test_cache_metrics() {
        let monitor = PerformanceMonitor::new();
        
        monitor.record_cache_hit(Duration::from_millis(5));
        monitor.record_cache_hit(Duration::from_millis(10));
        monitor.record_cache_miss();
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.cache_metrics.total_hits, 2);
        assert_eq!(metrics.cache_metrics.total_misses, 1);
        assert!((metrics.cache_metrics.hit_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_operation_timer() {
        let monitor = Arc::new(PerformanceMonitor::new());
        
        {
            let timer = OperationTimer::start("timed_op".to_string(), monitor.clone());
            thread::sleep(Duration::from_millis(50));
            timer.complete(true);
        }
        
        let metrics = monitor.get_operation_metrics("timed_op").unwrap();
        assert_eq!(metrics.total_calls, 1);
        assert!(metrics.avg_duration >= Duration::from_millis(50));
    }
}
