#[cfg(test)]
mod tests {
    use crate::xtream::retry::{RetryConfig, is_retryable_error};
    use crate::error::XTauriError;
    use std::time::Duration;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(1000));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.use_jitter);
    }

    #[test]
    fn test_retry_config_custom() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 1.5,
            use_jitter: false,
        };
        
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(500));
        assert!(!config.use_jitter);
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
    fn test_should_retry_timeout_error() {
        let error = XTauriError::Timeout {
            operation: "fetch".to_string(),
        };
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn test_should_retry_server_error() {
        let error = XTauriError::XtreamApiError {
            status: 503,
            message: "Service Unavailable".to_string(),
        };
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn test_should_not_retry_client_error() {
        let error = XTauriError::XtreamApiError {
            status: 400,
            message: "Bad Request".to_string(),
        };
        assert!(!is_retryable_error(&error));
    }

    #[test]
    fn test_should_not_retry_auth_error() {
        let error = XTauriError::XtreamInvalidCredentials;
        assert!(!is_retryable_error(&error));
    }

    #[test]
    fn test_should_not_retry_validation_error() {
        let error = XTauriError::ProfileValidation {
            reason: "Invalid name".to_string(),
        };
        assert!(!is_retryable_error(&error));
    }

    #[test]
    fn test_exponential_backoff_calculation() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            use_jitter: false,
        };
        
        // Calculate delays for each retry
        let delay0 = config.calculate_delay(0);
        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);
        
        assert_eq!(delay0, Duration::from_millis(100));
        assert_eq!(delay1, Duration::from_millis(200));
        assert_eq!(delay2, Duration::from_millis(400));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig {
            max_retries: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            use_jitter: false,
        };
        
        // After several retries, delay should be capped at max_delay
        let delay10 = config.calculate_delay(10);
        assert!(delay10 <= config.max_delay);
    }

    #[test]
    fn test_should_retry_various_status_codes() {
        // 5xx errors should retry
        assert!(is_retryable_error(&XTauriError::XtreamApiError {
            status: 500,
            message: "Internal Server Error".to_string(),
        }));
        
        assert!(is_retryable_error(&XTauriError::XtreamApiError {
            status: 502,
            message: "Bad Gateway".to_string(),
        }));
        
        assert!(is_retryable_error(&XTauriError::XtreamApiError {
            status: 503,
            message: "Service Unavailable".to_string(),
        }));
        
        assert!(is_retryable_error(&XTauriError::XtreamApiError {
            status: 504,
            message: "Gateway Timeout".to_string(),
        }));
        
        // 4xx errors should not retry
        assert!(!is_retryable_error(&XTauriError::XtreamApiError {
            status: 400,
            message: "Bad Request".to_string(),
        }));
        
        assert!(!is_retryable_error(&XTauriError::XtreamApiError {
            status: 401,
            message: "Unauthorized".to_string(),
        }));
        
        assert!(!is_retryable_error(&XTauriError::XtreamApiError {
            status: 404,
            message: "Not Found".to_string(),
        }));
    }

    #[test]
    fn test_cache_errors_are_retryable() {
        let error = XTauriError::Cache {
            operation: "get".to_string(),
        };
        assert!(is_retryable_error(&error));
        
        let error2 = XTauriError::ContentCache {
            operation: "set".to_string(),
        };
        assert!(is_retryable_error(&error2));
    }

    #[test]
    fn test_lock_acquisition_errors_are_retryable() {
        let error = XTauriError::LockAcquisition {
            resource: "database".to_string(),
        };
        assert!(is_retryable_error(&error));
    }

    #[test]
    fn test_auth_failed_with_network_reason_is_retryable() {
        let error = XTauriError::XtreamAuthenticationFailed {
            reason: "Network error occurred".to_string(),
        };
        assert!(is_retryable_error(&error));
        
        let error2 = XTauriError::XtreamAuthenticationFailed {
            reason: "Connection failed".to_string(),
        };
        assert!(is_retryable_error(&error2));
    }

    #[test]
    fn test_auth_failed_without_network_reason_not_retryable() {
        let error = XTauriError::XtreamAuthenticationFailed {
            reason: "Invalid password".to_string(),
        };
        assert!(!is_retryable_error(&error));
    }
}
