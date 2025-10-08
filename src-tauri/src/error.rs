use tauri::ipc::InvokeError;

/// Application-specific error types for the xTauri IPTV player
#[derive(Debug, thiserror::Error)]
pub enum XTauriError {
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Database initialization failed: {reason}")]
    DatabaseInitialization { reason: String },
    
    #[error("Database migration failed: {reason}")]
    DatabaseMigration { reason: String },
    
    // Network errors
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Failed to fetch playlist from URL: {url}")]
    PlaylistFetch { url: String },
    
    #[error("Failed to download file: {url}")]
    FileDownload { url: String },
    
    // File system errors
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("Failed to create directory: {path}")]
    DirectoryCreation { path: String },
    
    #[error("Failed to access data directory")]
    DataDirectoryAccess,
    
    #[error("Failed to read file: {path}")]
    FileRead { path: String },
    
    #[error("Failed to write file: {path}")]
    FileWrite { path: String },
    
    // Parsing errors
    #[error("Failed to parse M3U playlist: {reason}")]
    M3uParsing { reason: String },
    
    #[error("Failed to parse URL: {url}")]
    UrlParsing { url: String },
    
    #[error("Invalid regex pattern: {pattern}")]
    RegexError { pattern: String },
    
    // Cache errors
    #[error("Cache operation failed: {operation}")]
    Cache { operation: String },
    
    #[error("Image cache error: {reason}")]
    ImageCache { reason: String },
    
    #[error("Search cache error: {reason}")]
    SearchCache { reason: String },
    
    // Configuration errors
    #[error("Configuration error: {reason}")]
    Configuration { reason: String },
    
    #[error("Invalid setting value: {key} = {value}")]
    InvalidSetting { key: String, value: String },
    
    // Validation errors
    #[error("Invalid channel ID: {id}")]
    InvalidChannelId { id: String },
    
    #[error("Invalid playlist ID: {id}")]
    InvalidPlaylistId { id: String },
    
    #[error("Invalid URL format: {url}")]
    InvalidUrl { url: String },
    
    // Concurrency errors
    #[error("Failed to acquire lock: {resource}")]
    LockAcquisition { resource: String },
    
    #[error("Operation timeout: {operation}")]
    Timeout { operation: String },
    
    #[error("Operation was cancelled: {operation}")]
    Cancelled { operation: String },
    
    // Application state errors
    #[error("Application not initialized")]
    NotInitialized,
    
    #[error("Feature not available: {feature}")]
    FeatureNotAvailable { feature: String },
    
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    
    // Xtream-specific errors
    #[error("Xtream authentication failed: {reason}")]
    XtreamAuthenticationFailed { reason: String },
    
    #[error("Invalid Xtream credentials")]
    XtreamInvalidCredentials,
    
    #[error("Xtream profile not found: {id}")]
    XtreamProfileNotFound { id: String },
    
    #[error("Xtream API error: {status} - {message}")]
    XtreamApiError { status: u16, message: String },
    
    #[error("Credential encryption error: {reason}")]
    CredentialEncryption { reason: String },
    
    #[error("Credential decryption error: {reason}")]
    CredentialDecryption { reason: String },
    
    #[error("Content cache error: {operation}")]
    ContentCache { operation: String },
    
    #[error("Profile validation failed: {reason}")]
    ProfileValidation { reason: String },
    
    // Generic errors
    #[error("Internal error: {reason}")]
    Internal { reason: String },
    
    #[error("Unknown error occurred")]
    Unknown,
}

impl XTauriError {
    /// Create a new database initialization error
    pub fn database_init(reason: impl Into<String>) -> Self {
        Self::DatabaseInitialization {
            reason: reason.into(),
        }
    }
    
    /// Create a new directory creation error
    pub fn directory_creation(path: impl Into<String>) -> Self {
        Self::DirectoryCreation {
            path: path.into(),
        }
    }
    
    /// Create a new playlist fetch error
    pub fn playlist_fetch(url: impl Into<String>) -> Self {
        Self::PlaylistFetch {
            url: url.into(),
        }
    }
    
    /// Create a new file download error
    pub fn file_download(url: impl Into<String>) -> Self {
        Self::FileDownload {
            url: url.into(),
        }
    }
    
    /// Create a new M3U parsing error
    pub fn m3u_parsing(reason: impl Into<String>) -> Self {
        Self::M3uParsing {
            reason: reason.into(),
        }
    }
    
    /// Create a new cache error
    pub fn cache(operation: impl Into<String>) -> Self {
        Self::Cache {
            operation: operation.into(),
        }
    }
    
    /// Create a new lock acquisition error
    pub fn lock_acquisition(resource: impl Into<String>) -> Self {
        Self::LockAcquisition {
            resource: resource.into(),
        }
    }
    
    /// Create a new timeout error
    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }
    
    /// Create a new internal error
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::Internal {
            reason: reason.into(),
        }
    }
    
    /// Create a new Xtream authentication error
    pub fn xtream_auth_failed(reason: impl Into<String>) -> Self {
        Self::XtreamAuthenticationFailed {
            reason: reason.into(),
        }
    }
    
    /// Create a new Xtream profile not found error
    pub fn xtream_profile_not_found(id: impl Into<String>) -> Self {
        Self::XtreamProfileNotFound {
            id: id.into(),
        }
    }
    
    /// Create a new Xtream API error
    pub fn xtream_api_error(status: u16, message: impl Into<String>) -> Self {
        Self::XtreamApiError {
            status,
            message: message.into(),
        }
    }
    
    /// Create a new credential encryption error
    pub fn credential_encryption(reason: impl Into<String>) -> Self {
        Self::CredentialEncryption {
            reason: reason.into(),
        }
    }
    
    /// Create a new credential decryption error
    pub fn credential_decryption(reason: impl Into<String>) -> Self {
        Self::CredentialDecryption {
            reason: reason.into(),
        }
    }
    
    /// Create a new content cache error
    pub fn content_cache(operation: impl Into<String>) -> Self {
        Self::ContentCache {
            operation: operation.into(),
        }
    }
    
    /// Create a new profile validation error
    pub fn profile_validation(reason: impl Into<String>) -> Self {
        Self::ProfileValidation {
            reason: reason.into(),
        }
    }
    
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Network errors are often recoverable
            XTauriError::Network(_) | XTauriError::PlaylistFetch { .. } | XTauriError::FileDownload { .. } => true,
            
            // Timeout and cancellation errors are recoverable
            XTauriError::Timeout { .. } | XTauriError::Cancelled { .. } => true,
            
            // Cache errors are usually recoverable
            XTauriError::Cache { .. } | XTauriError::ImageCache { .. } | XTauriError::SearchCache { .. } => true,
            
            // Lock acquisition failures might be recoverable
            XTauriError::LockAcquisition { .. } => true,
            
            // Xtream authentication errors might be recoverable
            XTauriError::XtreamAuthenticationFailed { .. } | XTauriError::XtreamApiError { .. } => true,
            
            // Content cache errors are usually recoverable
            XTauriError::ContentCache { .. } => true,
            
            // Most other errors are not recoverable
            _ => false,
        }
    }
    
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            XTauriError::Database(_) => "Database operation failed. Please try again.".to_string(),
            XTauriError::DatabaseInitialization { .. } => "Failed to initialize database. Please check your permissions.".to_string(),
            XTauriError::Network(_) => "Network connection failed. Please check your internet connection.".to_string(),
            XTauriError::PlaylistFetch { .. } => "Failed to load playlist. Please check the URL and try again.".to_string(),
            XTauriError::FileDownload { .. } => "Failed to download file. Please check your connection and try again.".to_string(),
            XTauriError::DataDirectoryAccess => "Cannot access application data directory. Please check permissions.".to_string(),
            XTauriError::M3uParsing { .. } => "Invalid playlist format. Please check the playlist file.".to_string(),
            XTauriError::InvalidUrl { .. } => "Invalid URL format. Please check the URL and try again.".to_string(),
            XTauriError::Timeout { .. } => "Operation timed out. Please try again.".to_string(),
            XTauriError::NotFound { .. } => "Requested item not found.".to_string(),
            XTauriError::XtreamAuthenticationFailed { .. } => "Failed to authenticate with Xtream server. Please check your credentials.".to_string(),
            XTauriError::XtreamInvalidCredentials => "Invalid Xtream credentials. Please check your username, password, and server URL.".to_string(),
            XTauriError::XtreamProfileNotFound { .. } => "Xtream profile not found.".to_string(),
            XTauriError::XtreamApiError { .. } => "Xtream server error. Please try again later.".to_string(),
            XTauriError::CredentialEncryption { .. } | XTauriError::CredentialDecryption { .. } => "Failed to process credentials securely.".to_string(),
            XTauriError::ContentCache { .. } => "Content cache error. Data will be refreshed.".to_string(),
            XTauriError::ProfileValidation { .. } => "Profile validation failed. Please check your profile settings.".to_string(),
            _ => "An unexpected error occurred. Please try again.".to_string(),
        }
    }
    
    /// Get error category for logging/telemetry
    pub fn category(&self) -> &'static str {
        match self {
            XTauriError::Database(_) | XTauriError::DatabaseInitialization { .. } | XTauriError::DatabaseMigration { .. } => "database",
            XTauriError::Network(_) | XTauriError::PlaylistFetch { .. } | XTauriError::FileDownload { .. } => "network",
            XTauriError::FileSystem(_) | XTauriError::DirectoryCreation { .. } | XTauriError::DataDirectoryAccess | XTauriError::FileRead { .. } | XTauriError::FileWrite { .. } => "filesystem",
            XTauriError::M3uParsing { .. } | XTauriError::UrlParsing { .. } | XTauriError::RegexError { .. } => "parsing",
            XTauriError::Cache { .. } | XTauriError::ImageCache { .. } | XTauriError::SearchCache { .. } => "cache",
            XTauriError::Configuration { .. } | XTauriError::InvalidSetting { .. } => "configuration",
            XTauriError::InvalidChannelId { .. } | XTauriError::InvalidPlaylistId { .. } | XTauriError::InvalidUrl { .. } => "validation",
            XTauriError::LockAcquisition { .. } | XTauriError::Timeout { .. } | XTauriError::Cancelled { .. } => "concurrency",
            XTauriError::NotInitialized | XTauriError::FeatureNotAvailable { .. } | XTauriError::NotFound { .. } => "state",
            XTauriError::XtreamAuthenticationFailed { .. } | XTauriError::XtreamInvalidCredentials | XTauriError::XtreamProfileNotFound { .. } | XTauriError::XtreamApiError { .. } => "xtream",
            XTauriError::CredentialEncryption { .. } | XTauriError::CredentialDecryption { .. } => "security",
            XTauriError::ContentCache { .. } | XTauriError::ProfileValidation { .. } => "xtream",
            XTauriError::Internal { .. } | XTauriError::Unknown => "internal",
        }
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, XTauriError>;

/// Helper trait for converting results to XTauriError
pub trait ToXTauriError<T> {
    fn to_xtauri_error(self) -> Result<T>;
}

impl<T, E> ToXTauriError<T> for std::result::Result<T, E>
where
    E: Into<XTauriError>,
{
    fn to_xtauri_error(self) -> Result<T> {
        self.map_err(|e| e.into())
    }
}

/// Helper for converting XTauriError to String for Tauri commands
impl From<XTauriError> for String {
    fn from(error: XTauriError) -> String {
        error.user_message()
    }
}

/// Implementation for Tauri InvokeError compatibility
impl From<XTauriError> for InvokeError {
    fn from(error: XTauriError) -> InvokeError {
        InvokeError::from(error.user_message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = XTauriError::database_init("Connection failed");
        assert_eq!(
            error.to_string(),
            "Database initialization failed: Connection failed"
        );
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(XTauriError::Database(rusqlite::Error::InvalidPath("test".into())).category(), "database");
        assert_eq!(XTauriError::PlaylistFetch { url: "test".to_string() }.category(), "network");
        assert_eq!(XTauriError::DataDirectoryAccess.category(), "filesystem");
        assert_eq!(XTauriError::M3uParsing { reason: "test".to_string() }.category(), "parsing");
        assert_eq!(XTauriError::Cache { operation: "test".to_string() }.category(), "cache");
        assert_eq!(XTauriError::LockAcquisition { resource: "test".to_string() }.category(), "concurrency");
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(XTauriError::PlaylistFetch { url: "test".to_string() }.is_recoverable());
        assert!(XTauriError::Timeout { operation: "test".to_string() }.is_recoverable());
        assert!(XTauriError::Cache { operation: "test".to_string() }.is_recoverable());
        assert!(XTauriError::LockAcquisition { resource: "test".to_string() }.is_recoverable());
        
        assert!(!XTauriError::DatabaseInitialization { reason: "test".to_string() }.is_recoverable());
        assert!(!XTauriError::DataDirectoryAccess.is_recoverable());
    }

    #[test]
    fn test_user_messages() {
        let db_error = XTauriError::Database(rusqlite::Error::InvalidPath("test".into()));
        assert_eq!(db_error.user_message(), "Database operation failed. Please try again.");
        
        let network_error = XTauriError::PlaylistFetch { url: "http://example.com".to_string() };
        assert_eq!(network_error.user_message(), "Failed to load playlist. Please check the URL and try again.");
    }

    #[test]
    fn test_error_conversion_to_string() {
        let error = XTauriError::timeout("search operation");
        let error_string: String = error.into();
        assert_eq!(error_string, "Operation timed out. Please try again.");
    }

    #[test]
    fn test_builder_methods() {
        let error = XTauriError::database_init("Failed to connect");
        assert!(matches!(error, XTauriError::DatabaseInitialization { .. }));
        
        let error = XTauriError::directory_creation("/tmp/test");
        assert!(matches!(error, XTauriError::DirectoryCreation { .. }));
        
        let error = XTauriError::playlist_fetch("http://example.com");
        assert!(matches!(error, XTauriError::PlaylistFetch { .. }));
    }

    #[test]
    fn test_from_trait_conversions() {
        // Test rusqlite error conversion
        let rusqlite_error = rusqlite::Error::InvalidPath("test".into());
        let xtauri_error: XTauriError = rusqlite_error.into();
        assert!(matches!(xtauri_error, XTauriError::Database(_)));
        
        // Test IO error conversion
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let xtauri_error: XTauriError = io_error.into();
        assert!(matches!(xtauri_error, XTauriError::FileSystem(_)));
    }
}