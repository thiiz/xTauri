use crate::error::{Result, XTauriError};
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to add jitter to delays
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with custom max retries
    pub fn with_max_retries(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }
    
    /// Create a retry config for quick operations (shorter delays)
    pub fn quick() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
    
    /// Create a retry config for long operations (more retries, longer delays)
    pub fn patient() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(2000),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
    
    /// Calculate delay for a given attempt number
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay.as_millis() as f64 
            * self.backoff_multiplier.powi(attempt as i32);
        
        let delay_ms = base_delay.min(self.max_delay.as_millis() as f64) as u64;
        
        let mut delay = Duration::from_millis(delay_ms);
        
        // Add jitter to prevent thundering herd
        if self.use_jitter {
            use rand::Rng;
            let jitter_factor = rand::thread_rng().gen_range(0.8..1.2);
            delay = Duration::from_millis((delay.as_millis() as f64 * jitter_factor) as u64);
        }
        
        delay
    }
}

/// Determines if an error is retryable
pub fn is_retryable_error(error: &XTauriError) -> bool {
    match error {
        // Network errors are retryable
        XTauriError::Network(e) => {
            // Retry on timeout, connection errors, but not on invalid URLs
            e.is_timeout() || e.is_connect() || e.is_request()
        }
        
        // Timeout errors are retryable
        XTauriError::Timeout { .. } => true,
        
        // Some Xtream API errors are retryable (5xx server errors)
        XTauriError::XtreamApiError { status, .. } => {
            *status >= 500 && *status < 600
        }
        
        // Authentication failures with network-related messages are retryable
        XTauriError::XtreamAuthenticationFailed { reason } => {
            reason.contains("Network error") 
                || reason.contains("Connection failed")
                || reason.contains("timeout")
        }
        
        // Cache errors are retryable
        XTauriError::Cache { .. } 
        | XTauriError::ContentCache { .. } => true,
        
        // Lock acquisition failures might be retryable
        XTauriError::LockAcquisition { .. } => true,
        
        // Most other errors are not retryable
        _ => false,
    }
}

/// Retry a future with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    operation: F,
    config: RetryConfig,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;
    
    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Check if error is retryable
                if !is_retryable_error(&e) {
                    return Err(e);
                }
                
                last_error = Some(e);
                
                // Don't sleep after the last attempt
                if attempt < config.max_retries {
                    let delay = config.calculate_delay(attempt);
                    sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| {
        XTauriError::internal("Retry failed without error")
    }))
}

/// Retry a future with a simple retry count
pub async fn retry_simple<F, Fut, T>(
    operation: F,
    max_retries: u32,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    retry_with_backoff(operation, RetryConfig::with_max_retries(max_retries)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(1000));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.use_jitter);
    }

    #[test]
    fn test_retry_config_quick() {
        let config = RetryConfig::quick();
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.initial_delay, Duration::from_millis(500));
    }

    #[test]
    fn test_retry_config_patient() {
        let config = RetryConfig::patient();
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(2000));
    }

    #[test]
    fn test_calculate_delay_exponential() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            use_jitter: false,
        };
        
        let delay0 = config.calculate_delay(0);
        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);
        
        assert_eq!(delay0, Duration::from_millis(1000));
        assert_eq!(delay1, Duration::from_millis(2000));
        assert_eq!(delay2, Duration::from_millis(4000));
    }

    #[test]
    fn test_calculate_delay_max_cap() {
        let config = RetryConfig {
            max_retries: 10,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            use_jitter: false,
        };
        
        let delay10 = config.calculate_delay(10);
        assert!(delay10 <= Duration::from_secs(5));
    }

    #[test]
    fn test_is_retryable_error() {
        // Network errors should be retryable (using a real network error)
        // We'll test with timeout error instead
        let timeout_err = XTauriError::Timeout {
            operation: "test".to_string(),
        };
        assert!(is_retryable_error(&timeout_err));

        
        // 5xx API errors should be retryable
        let api_err_500 = XTauriError::XtreamApiError {
            status: 500,
            message: "Server error".to_string(),
        };
        assert!(is_retryable_error(&api_err_500));
        
        // 4xx API errors should not be retryable
        let api_err_400 = XTauriError::XtreamApiError {
            status: 400,
            message: "Bad request".to_string(),
        };
        assert!(!is_retryable_error(&api_err_400));
        
        // Invalid credentials should not be retryable
        let invalid_creds = XTauriError::XtreamInvalidCredentials;
        assert!(!is_retryable_error(&invalid_creds));
        
        // Cache errors should be retryable
        let cache_err = XTauriError::Cache {
            operation: "test".to_string(),
        };
        assert!(is_retryable_error(&cache_err));
    }

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);
        
        let result = retry_simple(
            || {
                let counter = Arc::clone(&counter_clone);
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, XTauriError>(42)
                }
            },
            3,
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);
        
        let result = retry_simple(
            || {
                let counter = Arc::clone(&counter_clone);
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(XTauriError::Timeout {
                            operation: "test".to_string(),
                        })
                    } else {
                        Ok::<_, XTauriError>(42)
                    }
                }
            },
            3,
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_failure_after_max_retries() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);
        
        let result = retry_simple(
            || {
                let counter = Arc::clone(&counter_clone);
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>(XTauriError::Timeout {
                        operation: "test".to_string(),
                    })
                }
            },
            2,
        ).await;
        
        assert!(result.is_err());
        // Should try: initial + 2 retries = 3 attempts
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_non_retryable_error() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);
        
        let result = retry_simple(
            || {
                let counter = Arc::clone(&counter_clone);
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>(XTauriError::XtreamInvalidCredentials)
                }
            },
            3,
        ).await;
        
        assert!(result.is_err());
        // Should only try once since error is not retryable
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
