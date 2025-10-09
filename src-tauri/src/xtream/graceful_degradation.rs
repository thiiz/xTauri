use crate::error::{Result, XTauriError};
use crate::xtream::content_cache::ContentCache;
use serde_json::Value;
use std::sync::Arc;

/// Strategy for handling failures with cached content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackStrategy {
    /// Use cached content if available, otherwise fail
    UseCacheOrFail,
    /// Use cached content if available, otherwise return empty result
    UseCacheOrEmpty,
    /// Always fail, never use cache
    NeverUseCache,
    /// Use stale cache (expired) if fresh cache is not available
    UseStaleCache,
}

/// Result of a graceful degradation operation
#[derive(Debug)]
pub struct DegradedResult<T> {
    /// The actual result data
    pub data: T,
    /// Whether the data came from cache
    pub from_cache: bool,
    /// Whether the cached data was stale (expired)
    pub is_stale: bool,
    /// The original error that caused fallback (if any)
    pub original_error: Option<String>,
}

impl<T> DegradedResult<T> {
    /// Create a fresh result (not from cache)
    pub fn fresh(data: T) -> Self {
        Self {
            data,
            from_cache: false,
            is_stale: false,
            original_error: None,
        }
    }
    
    /// Create a cached result
    pub fn cached(data: T, is_stale: bool, original_error: Option<String>) -> Self {
        Self {
            data,
            from_cache: true,
            is_stale,
            original_error,
        }
    }
    
    /// Check if this is a degraded result (from cache due to error)
    pub fn is_degraded(&self) -> bool {
        self.from_cache && self.original_error.is_some()
    }
}

/// Graceful degradation handler
pub struct GracefulDegradation {
    cache: Arc<ContentCache>,
    default_strategy: FallbackStrategy,
}

impl GracefulDegradation {
    /// Create a new graceful degradation handler
    pub fn new(cache: Arc<ContentCache>) -> Self {
        Self {
            cache,
            default_strategy: FallbackStrategy::UseCacheOrFail,
        }
    }
    
    /// Create with a specific default strategy
    pub fn with_strategy(cache: Arc<ContentCache>, strategy: FallbackStrategy) -> Self {
        Self {
            cache,
            default_strategy: strategy,
        }
    }
    
    /// Execute an operation with graceful degradation
    pub async fn execute<F, Fut>(
        &self,
        cache_key: &str,
        operation: F,
        strategy: Option<FallbackStrategy>,
    ) -> Result<DegradedResult<Value>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Value>>,
    {
        let strategy = strategy.unwrap_or(self.default_strategy);
        
        // Try the operation first
        match operation().await {
            Ok(data) => {
                // Success - cache the result and return
                let _ = self.cache.set(cache_key, &data, None);
                Ok(DegradedResult::fresh(data))
            }
            Err(e) => {
                // Operation failed - apply fallback strategy
                self.handle_failure(cache_key, e, strategy)
            }
        }
    }
    
    /// Handle operation failure according to strategy
    fn handle_failure(
        &self,
        cache_key: &str,
        error: XTauriError,
        strategy: FallbackStrategy,
    ) -> Result<DegradedResult<Value>> {
        match strategy {
            FallbackStrategy::NeverUseCache => {
                // Never use cache, just fail
                Err(error)
            }
            
            FallbackStrategy::UseCacheOrFail => {
                // Try to get from cache
                match self.cache.get::<Value>(cache_key) {
                    Ok(Some(cached_data)) => {
                        Ok(DegradedResult::cached(
                            cached_data,
                            false,
                            Some(error.to_string()),
                        ))
                    }
                    _ => Err(error),
                }
            }
            
            FallbackStrategy::UseCacheOrEmpty => {
                // Try to get from cache, or return empty
                match self.cache.get::<Value>(cache_key) {
                    Ok(Some(cached_data)) => {
                        Ok(DegradedResult::cached(
                            cached_data,
                            false,
                            Some(error.to_string()),
                        ))
                    }
                    _ => {
                        // Return empty array as fallback
                        Ok(DegradedResult::cached(
                            Value::Array(vec![]),
                            false,
                            Some(error.to_string()),
                        ))
                    }
                }
            }
            
            FallbackStrategy::UseStaleCache => {
                // Try to get from cache, including stale entries
                match self.cache.get_stale::<Value>(cache_key) {
                    Ok(Some(cached_data)) => {
                        Ok(DegradedResult::cached(
                            cached_data,
                            true,
                            Some(error.to_string()),
                        ))
                    }
                    _ => Err(error),
                }
            }
        }
    }
    
    /// Check if cached data is available for a key
    pub fn has_cache(&self, cache_key: &str) -> bool {
        self.cache.get::<Value>(cache_key).ok().flatten().is_some()
    }
    
    /// Check if stale cached data is available for a key
    pub fn has_stale_cache(&self, cache_key: &str) -> bool {
        self.cache.get_stale::<Value>(cache_key).ok().flatten().is_some()
    }
    
    /// Invalidate cache for a key
    pub fn invalidate_cache(&self, cache_key: &str) -> Result<()> {
        self.cache.invalidate(cache_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use rusqlite::Connection;
    use std::sync::Mutex;

    async fn create_test_cache() -> (Arc<ContentCache>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        
        // Create required tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS xtream_content_cache (
                cache_key TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                content_type TEXT NOT NULL,
                data BLOB NOT NULL,
                expires_at DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).unwrap();
        
        let db = Arc::new(Mutex::new(conn));
        let cache = Arc::new(ContentCache::new(db, std::time::Duration::from_secs(3600)));
        (cache, temp_dir)
    }

    #[tokio::test]
    async fn test_execute_success() {
        let (cache, _temp_dir) = create_test_cache().await;
        let degradation = GracefulDegradation::new(cache);
        
        let result = degradation.execute(
            "test_key",
            || async { Ok(Value::String("success".to_string())) },
            None,
        ).await;
        
        assert!(result.is_ok());
        let degraded = result.unwrap();
        assert!(!degraded.from_cache);
        assert!(!degraded.is_stale);
        assert!(degraded.original_error.is_none());
    }

    #[tokio::test]
    async fn test_execute_failure_with_cache() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        // Pre-populate cache
        cache.set("test_key", &Value::String("cached".to_string()), None).unwrap();
        
        let degradation = GracefulDegradation::new(cache);
        
        let result = degradation.execute(
            "test_key",
            || async {
                Err::<Value, _>(XTauriError::Timeout {
                    operation: "test".to_string(),
                })
            },
            Some(FallbackStrategy::UseCacheOrFail),
        ).await;
        
        assert!(result.is_ok());
        let degraded = result.unwrap();
        assert!(degraded.from_cache);
        assert!(!degraded.is_stale);
        assert!(degraded.original_error.is_some());
        assert!(degraded.is_degraded());
    }

    #[tokio::test]
    async fn test_execute_failure_without_cache() {
        let (cache, _temp_dir) = create_test_cache().await;
        let degradation = GracefulDegradation::new(cache);
        
        let result = degradation.execute(
            "test_key",
            || async {
                Err::<Value, _>(XTauriError::Timeout {
                    operation: "test".to_string(),
                })
            },
            Some(FallbackStrategy::UseCacheOrFail),
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_failure_with_empty_fallback() {
        let (cache, _temp_dir) = create_test_cache().await;
        let degradation = GracefulDegradation::new(cache);
        
        let result = degradation.execute(
            "test_key",
            || async {
                Err::<Value, _>(XTauriError::Timeout {
                    operation: "test".to_string(),
                })
            },
            Some(FallbackStrategy::UseCacheOrEmpty),
        ).await;
        
        assert!(result.is_ok());
        let degraded = result.unwrap();
        assert!(degraded.from_cache);
        assert!(degraded.data.is_array());
        assert_eq!(degraded.data.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_never_use_cache_strategy() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        // Pre-populate cache
        cache.set("test_key", &Value::String("cached".to_string()), None).unwrap();
        
        let degradation = GracefulDegradation::new(cache);
        
        let result = degradation.execute(
            "test_key",
            || async {
                Err::<Value, _>(XTauriError::Timeout {
                    operation: "test".to_string(),
                })
            },
            Some(FallbackStrategy::NeverUseCache),
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_has_cache() {
        let (cache, _temp_dir) = create_test_cache().await;
        let degradation = GracefulDegradation::new(Arc::clone(&cache));
        
        assert!(!degradation.has_cache("test_key"));
        
        cache.set("test_key", &Value::String("cached".to_string()), None).unwrap();
        
        assert!(degradation.has_cache("test_key"));
    }
}
