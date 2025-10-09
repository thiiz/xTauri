use crate::error::{Result, XTauriError};
use crate::xtream::types::ProfileCredentials;
use crate::xtream::XtreamClient;
use crate::xtream::content_cache::ContentCache;
use crate::xtream::retry::{RetryConfig, retry_with_backoff};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde_json::Value;

/// Session state for a profile
#[derive(Debug, Clone)]
pub struct SessionState {
    pub profile_id: String,
    pub is_authenticated: bool,
    pub last_auth_time: Option<Instant>,
    pub auth_failures: u32,
    pub server_info: Option<Value>,
}

impl SessionState {
    pub fn new(profile_id: String) -> Self {
        Self {
            profile_id,
            is_authenticated: false,
            last_auth_time: None,
            auth_failures: 0,
            server_info: None,
        }
    }
    
    pub fn mark_authenticated(&mut self, server_info: Value) {
        self.is_authenticated = true;
        self.last_auth_time = Some(Instant::now());
        self.auth_failures = 0;
        self.server_info = Some(server_info);
    }
    
    pub fn mark_auth_failed(&mut self) {
        self.is_authenticated = false;
        self.auth_failures += 1;
    }
    
    pub fn reset(&mut self) {
        self.is_authenticated = false;
        self.last_auth_time = None;
        self.auth_failures = 0;
        self.server_info = None;
    }
    
    pub fn should_reauth(&self, max_session_age: Duration) -> bool {
        if !self.is_authenticated {
            return true;
        }
        
        if let Some(last_auth) = self.last_auth_time {
            last_auth.elapsed() > max_session_age
        } else {
            true
        }
    }
}

/// Manages authentication sessions and automatic re-authentication
pub struct SessionManager {
    sessions: Arc<Mutex<std::collections::HashMap<String, SessionState>>>,
    max_session_age: Duration,
    max_auth_failures: u32,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
            max_session_age: Duration::from_secs(3600), // 1 hour
            max_auth_failures: 3,
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(max_session_age: Duration, max_auth_failures: u32) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
            max_session_age,
            max_auth_failures,
        }
    }
    
    /// Get session state for a profile
    pub fn get_session(&self, profile_id: &str) -> Result<SessionState> {
        let sessions = self.sessions.lock()
            .map_err(|_| XTauriError::lock_acquisition("session manager"))?;
        
        Ok(sessions.get(profile_id)
            .cloned()
            .unwrap_or_else(|| SessionState::new(profile_id.to_string())))
    }
    
    /// Update session state
    fn update_session(&self, profile_id: &str, state: SessionState) -> Result<()> {
        let mut sessions = self.sessions.lock()
            .map_err(|_| XTauriError::lock_acquisition("session manager"))?;
        
        sessions.insert(profile_id.to_string(), state);
        Ok(())
    }
    
    /// Check if session needs re-authentication
    pub fn needs_reauth(&self, profile_id: &str) -> Result<bool> {
        let session = self.get_session(profile_id)?;
        Ok(session.should_reauth(self.max_session_age))
    }
    
    /// Authenticate or re-authenticate a session
    pub async fn authenticate(
        &self,
        profile_id: &str,
        credentials: &ProfileCredentials,
        cache: Arc<ContentCache>,
    ) -> Result<Value> {
        let mut session = self.get_session(profile_id)?;
        
        // Check if we've exceeded max failures
        if session.auth_failures >= self.max_auth_failures {
            return Err(XTauriError::xtream_auth_failed(
                format!("Maximum authentication failures ({}) exceeded", self.max_auth_failures)
            ));
        }
        
        // Create client and attempt authentication
        let client = XtreamClient::new(credentials.clone(), cache)?;
        
        match self.try_authenticate(&client).await {
            Ok(server_info) => {
                session.mark_authenticated(server_info.clone());
                self.update_session(profile_id, session)?;
                Ok(server_info)
            }
            Err(e) => {
                session.mark_auth_failed();
                self.update_session(profile_id, session)?;
                Err(e)
            }
        }
    }
    
    /// Try to authenticate with retry logic
    async fn try_authenticate(&self, client: &XtreamClient) -> Result<Value> {
        retry_with_backoff(
            || client.authenticate(),
            RetryConfig::quick(),
        ).await
    }
    
    /// Execute an operation with automatic re-authentication
    pub async fn with_auth<F, Fut, T>(
        &self,
        profile_id: &str,
        credentials: &ProfileCredentials,
        cache: Arc<ContentCache>,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check if we need to re-authenticate
        if self.needs_reauth(profile_id)? {
            self.authenticate(profile_id, credentials, Arc::clone(&cache)).await?;
        }
        
        // Try the operation
        match operation().await {
            Ok(result) => Ok(result),
            Err(e) => {
                // Check if it's an authentication error
                if self.is_auth_error(&e) {
                    // Try to re-authenticate
                    self.authenticate(profile_id, credentials, Arc::clone(&cache)).await?;
                    
                    // Retry the operation once
                    operation().await
                } else {
                    Err(e)
                }
            }
        }
    }
    
    /// Check if an error is authentication-related
    fn is_auth_error(&self, error: &XTauriError) -> bool {
        matches!(
            error,
            XTauriError::XtreamAuthenticationFailed { .. }
                | XTauriError::XtreamInvalidCredentials
                | XTauriError::XtreamApiError { status: 401 | 403, .. }
        )
    }
    
    /// Clear session for a profile
    pub fn clear_session(&self, profile_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock()
            .map_err(|_| XTauriError::lock_acquisition("session manager"))?;
        
        sessions.remove(profile_id);
        Ok(())
    }
    
    /// Clear all sessions
    pub fn clear_all_sessions(&self) -> Result<()> {
        let mut sessions = self.sessions.lock()
            .map_err(|_| XTauriError::lock_acquisition("session manager"))?;
        
        sessions.clear();
        Ok(())
    }
    
    /// Get authentication failure count for a profile
    pub fn get_failure_count(&self, profile_id: &str) -> Result<u32> {
        let session = self.get_session(profile_id)?;
        Ok(session.auth_failures)
    }
    
    /// Reset authentication failure count
    pub fn reset_failure_count(&self, profile_id: &str) -> Result<()> {
        let mut session = self.get_session(profile_id)?;
        session.auth_failures = 0;
        self.update_session(profile_id, session)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_new() {
        let state = SessionState::new("test-profile".to_string());
        assert_eq!(state.profile_id, "test-profile");
        assert!(!state.is_authenticated);
        assert_eq!(state.auth_failures, 0);
        assert!(state.last_auth_time.is_none());
    }

    #[test]
    fn test_session_state_mark_authenticated() {
        let mut state = SessionState::new("test-profile".to_string());
        let server_info = serde_json::json!({"status": "active"});
        
        state.mark_authenticated(server_info.clone());
        
        assert!(state.is_authenticated);
        assert_eq!(state.auth_failures, 0);
        assert!(state.last_auth_time.is_some());
        assert_eq!(state.server_info, Some(server_info));
    }

    #[test]
    fn test_session_state_mark_auth_failed() {
        let mut state = SessionState::new("test-profile".to_string());
        
        state.mark_auth_failed();
        
        assert!(!state.is_authenticated);
        assert_eq!(state.auth_failures, 1);
        
        state.mark_auth_failed();
        assert_eq!(state.auth_failures, 2);
    }

    #[test]
    fn test_session_state_should_reauth() {
        let mut state = SessionState::new("test-profile".to_string());
        
        // Should reauth when not authenticated
        assert!(state.should_reauth(Duration::from_secs(3600)));
        
        // Mark as authenticated
        state.mark_authenticated(serde_json::json!({}));
        
        // Should not reauth immediately
        assert!(!state.should_reauth(Duration::from_secs(3600)));
        
        // Should reauth after session expires (simulated with 0 duration)
        assert!(state.should_reauth(Duration::from_secs(0)));
    }

    #[test]
    fn test_session_manager_new() {
        let manager = SessionManager::new();
        assert_eq!(manager.max_session_age, Duration::from_secs(3600));
        assert_eq!(manager.max_auth_failures, 3);
    }

    #[test]
    fn test_session_manager_get_session() {
        let manager = SessionManager::new();
        
        let session = manager.get_session("test-profile").unwrap();
        assert_eq!(session.profile_id, "test-profile");
        assert!(!session.is_authenticated);
    }

    #[test]
    fn test_session_manager_update_session() {
        let manager = SessionManager::new();
        
        let mut session = SessionState::new("test-profile".to_string());
        session.mark_authenticated(serde_json::json!({}));
        
        manager.update_session("test-profile", session).unwrap();
        
        let retrieved = manager.get_session("test-profile").unwrap();
        assert!(retrieved.is_authenticated);
    }

    #[test]
    fn test_session_manager_clear_session() {
        let manager = SessionManager::new();
        
        let mut session = SessionState::new("test-profile".to_string());
        session.mark_authenticated(serde_json::json!({}));
        manager.update_session("test-profile", session).unwrap();
        
        manager.clear_session("test-profile").unwrap();
        
        let retrieved = manager.get_session("test-profile").unwrap();
        assert!(!retrieved.is_authenticated);
    }

    #[test]
    fn test_session_manager_failure_count() {
        let manager = SessionManager::new();
        
        let mut session = SessionState::new("test-profile".to_string());
        session.mark_auth_failed();
        session.mark_auth_failed();
        manager.update_session("test-profile", session).unwrap();
        
        let count = manager.get_failure_count("test-profile").unwrap();
        assert_eq!(count, 2);
        
        manager.reset_failure_count("test-profile").unwrap();
        let count = manager.get_failure_count("test-profile").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_is_auth_error() {
        let manager = SessionManager::new();
        
        assert!(manager.is_auth_error(&XTauriError::XtreamInvalidCredentials));
        assert!(manager.is_auth_error(&XTauriError::xtream_auth_failed("test".to_string())));
        assert!(manager.is_auth_error(&XTauriError::xtream_api_error(401, "Unauthorized".to_string())));
        assert!(manager.is_auth_error(&XTauriError::xtream_api_error(403, "Forbidden".to_string())));
        
        assert!(!manager.is_auth_error(&XTauriError::xtream_api_error(500, "Server Error".to_string())));
        assert!(!manager.is_auth_error(&XTauriError::timeout("test".to_string())));
    }
}
