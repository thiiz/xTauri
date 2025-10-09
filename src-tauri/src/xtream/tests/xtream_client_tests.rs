#[cfg(test)]
mod tests {
    // Note: Full Xtream client tests require mock HTTP server (wiremock)
    // These are basic unit tests for URL generation and data structures
    
    use crate::xtream::types::{ProfileCredentials, StreamURLRequest, ContentType};

    #[test]
    fn test_profile_credentials_creation() {
        let credentials = ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        assert_eq!(credentials.url, "http://example.com:8080");
        assert_eq!(credentials.username, "testuser");
        assert_eq!(credentials.password, "testpass");
    }

    #[test]
    fn test_stream_url_request_channel() {
        let request = StreamURLRequest {
            content_type: ContentType::Channel,
            content_id: "1001".to_string(),
            extension: Some("m3u8".to_string()),
        };
        
        assert_eq!(request.content_id, "1001");
        match request.content_type {
            ContentType::Channel => {},
            _ => panic!("Expected Channel content type"),
        }
    }

    #[test]
    fn test_stream_url_request_movie() {
        let request = StreamURLRequest {
            content_type: ContentType::Movie,
            content_id: "2001".to_string(),
            extension: Some("mp4".to_string()),
        };
        
        assert_eq!(request.content_id, "2001");
        match request.content_type {
            ContentType::Movie => {},
            _ => panic!("Expected Movie content type"),
        }
    }

    #[test]
    fn test_stream_url_request_series() {
        let request = StreamURLRequest {
            content_type: ContentType::Series,
            content_id: "3001".to_string(),
            extension: Some("mkv".to_string()),
        };
        
        assert_eq!(request.content_id, "3001");
        match request.content_type {
            ContentType::Series => {},
            _ => panic!("Expected Series content type"),
        }
    }

    #[test]
    fn test_credentials_clone() {
        let credentials = ProfileCredentials {
            url: "http://example.com:8080".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let cloned = credentials.clone();
        assert_eq!(credentials.url, cloned.url);
        assert_eq!(credentials.username, cloned.username);
        assert_eq!(credentials.password, cloned.password);
    }

    // Integration tests with wiremock would go here
    // Example structure:
    // #[tokio::test]
    // async fn test_authenticate_success() {
    //     let mock_server = MockServer::start().await;
    //     // Setup mock responses
    //     // Test authentication
    // }
}
