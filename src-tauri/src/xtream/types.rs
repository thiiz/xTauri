use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Profile credentials for Xtream authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCredentials {
    pub url: String,
    pub username: String,
    pub password: String,
}

/// Xtream profile stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtreamProfile {
    pub id: String,
    pub name: String,
    pub url: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// Request to create a new profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProfileRequest {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
}

/// Request to update an existing profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Request to generate a stream URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamURLRequest {
    pub content_type: ContentType,
    pub content_id: String,
    pub extension: Option<String>,
}

/// Type of content for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Channel,
    Movie,
    Series,
}

/// Cached content item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedContent {
    pub data: Vec<u8>,
    pub expires_at: DateTime<Utc>,
    pub content_type: String,
}

/// Cache key components
#[derive(Debug, Clone)]
pub struct CacheKey {
    pub profile_id: String,
    pub content_type: String,
    pub identifier: Option<String>,
}

impl CacheKey {
    pub fn new(profile_id: String, content_type: String, identifier: Option<String>) -> Self {
        Self {
            profile_id,
            content_type,
            identifier,
        }
    }
    
    pub fn to_string(&self) -> String {
        match &self.identifier {
            Some(id) => format!("{}:{}:{}", self.profile_id, self.content_type, id),
            None => format!("{}:{}", self.profile_id, self.content_type),
        }
    }
}

/// Result of authentication testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub error_type: AuthenticationErrorType,
    pub server_info: Option<serde_json::Value>,
}

/// Types of authentication errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationErrorType {
    None,
    ValidationError,
    InvalidCredentials,
    AuthenticationFailed,
    NetworkError,
    TimeoutError,
    ServerError,
    ClientError,
    UnknownError,
}