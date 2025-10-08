use crate::error::XTauriError;
use crate::xtream::{
    ProfileManager, XtreamClient, ContentCache, ProfileCredentials, 
    CreateProfileRequest, UpdateProfileRequest, StreamURLRequest,
    XtreamProfile, AuthenticationResult, AuthenticationErrorType
};
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

/// State for managing Xtream profiles and clients
pub struct XtreamState {
    pub profile_manager: Arc<ProfileManager>,
    pub content_cache: Arc<ContentCache>,
}

impl XtreamState {
    pub fn new(profile_manager: Arc<ProfileManager>, content_cache: Arc<ContentCache>) -> Self {
        Self {
            profile_manager,
            content_cache,
        }
    }
}

/// Create a new Xtream profile
#[tauri::command]
pub async fn create_xtream_profile(
    state: State<'_, XtreamState>,
    request: CreateProfileRequest,
) -> Result<String, String> {
    state
        .profile_manager
        .create_profile_async_wrapper(request)
        .await
        .map_err(|e| e.to_string())
}

/// Update an existing Xtream profile
#[tauri::command]
pub async fn update_xtream_profile(
    state: State<'_, XtreamState>,
    id: String,
    request: UpdateProfileRequest,
) -> Result<(), String> {
    state
        .profile_manager
        .update_profile_async_wrapper(&id, request)
        .await
        .map_err(|e| e.to_string())
}

/// Delete an Xtream profile
#[tauri::command]
pub async fn delete_xtream_profile(
    state: State<'_, XtreamState>,
    id: String,
) -> Result<(), String> {
    state
        .profile_manager
        .delete_profile_async_wrapper(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Get all Xtream profiles
#[tauri::command]
pub async fn get_xtream_profiles(
    state: State<'_, XtreamState>,
) -> Result<Vec<XtreamProfile>, String> {
    state
        .profile_manager
        .get_profiles_async_wrapper()
        .await
        .map_err(|e| e.to_string())
}

/// Get a specific Xtream profile by ID
#[tauri::command]
pub async fn get_xtream_profile(
    state: State<'_, XtreamState>,
    id: String,
) -> Result<Option<XtreamProfile>, String> {
    state
        .profile_manager
        .get_profile_async_wrapper(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Validate Xtream profile credentials
#[tauri::command]
pub async fn validate_xtream_credentials(
    state: State<'_, XtreamState>,
    credentials: ProfileCredentials,
) -> Result<AuthenticationResult, String> {
    // Create a temporary client to test authentication
    let client = match XtreamClient::new(credentials.clone(), state.content_cache.clone()) {
        Ok(client) => client,
        Err(e) => {
            return Ok(AuthenticationResult {
                success: false,
                error_message: Some(e.user_message()),
                error_type: AuthenticationErrorType::ValidationError,
                server_info: None,
            });
        }
    };

    match client.authenticate().await {
        Ok(profile_data) => Ok(AuthenticationResult {
            success: true,
            error_message: None,
            error_type: AuthenticationErrorType::None,
            server_info: Some(profile_data),
        }),
        Err(e) => {
            let error_type = match &e {
                XTauriError::XtreamInvalidCredentials => AuthenticationErrorType::InvalidCredentials,
                XTauriError::XtreamAuthenticationFailed { .. } => AuthenticationErrorType::AuthenticationFailed,
                XTauriError::Network(_) => AuthenticationErrorType::NetworkError,
                XTauriError::Timeout { .. } => AuthenticationErrorType::TimeoutError,
                XTauriError::XtreamApiError { status, .. } => {
                    if *status >= 500 {
                        AuthenticationErrorType::ServerError
                    } else {
                        AuthenticationErrorType::ClientError
                    }
                }
                _ => AuthenticationErrorType::UnknownError,
            };

            Ok(AuthenticationResult {
                success: false,
                error_message: Some(e.user_message()),
                error_type,
                server_info: None,
            })
        }
    }
}

/// Authenticate with Xtream server and get profile information
#[tauri::command]
pub async fn authenticate_xtream_profile(
    state: State<'_, XtreamState>,
    profile_id: String,
) -> Result<Value, String> {
    // Get profile credentials
    let _profile = state
        .profile_manager
        .get_profile_async_wrapper(&profile_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Profile not found: {}", profile_id))?;

    // Get credentials for the profile
    let credentials = state
        .profile_manager
        .get_profile_credentials_async_wrapper(&profile_id)
        .await
        .map_err(|e| e.to_string())?;

    // Create client and authenticate
    let client = XtreamClient::new(credentials, state.content_cache.clone())
        .map_err(|e| e.to_string())?;

    let profile_data = client.authenticate().await.map_err(|e| e.to_string())?;

    // Update last used timestamp
    state
        .profile_manager
        .update_last_used(&profile_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(profile_data)
}

/// Get live channel categories
#[tauri::command]
pub async fn get_xtream_channel_categories(
    state: State<'_, XtreamState>,
    profile_id: String,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client.get_channel_categories().await.map_err(|e| e.to_string())
}

/// Get live channels
#[tauri::command]
pub async fn get_xtream_channels(
    state: State<'_, XtreamState>,
    profile_id: String,
    category_id: Option<String>,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client
        .get_channels(category_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Get live channels with pagination
#[tauri::command]
pub async fn get_xtream_channels_paginated(
    state: State<'_, XtreamState>,
    profile_id: String,
    category_id: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client
        .get_channels_with_pagination(category_id.as_deref(), limit, offset)
        .await
        .map_err(|e| e.to_string())
}

/// Get VOD (movie) categories
#[tauri::command]
pub async fn get_xtream_movie_categories(
    state: State<'_, XtreamState>,
    profile_id: String,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client.get_movie_categories().await.map_err(|e| e.to_string())
}

/// Get VOD (movies)
#[tauri::command]
pub async fn get_xtream_movies(
    state: State<'_, XtreamState>,
    profile_id: String,
    category_id: Option<String>,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client
        .get_movies(category_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Get VOD (movies) with pagination
#[tauri::command]
pub async fn get_xtream_movies_paginated(
    state: State<'_, XtreamState>,
    profile_id: String,
    category_id: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client
        .get_movies_with_pagination(category_id.as_deref(), limit, offset)
        .await
        .map_err(|e| e.to_string())
}

/// Get movie information with enhanced metadata
#[tauri::command]
pub async fn get_xtream_movie_info(
    state: State<'_, XtreamState>,
    profile_id: String,
    movie_id: String,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client.get_movie_info(&movie_id).await.map_err(|e| e.to_string())
}

/// Get TV series categories
#[tauri::command]
pub async fn get_xtream_series_categories(
    state: State<'_, XtreamState>,
    profile_id: String,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client.get_series_categories().await.map_err(|e| e.to_string())
}

/// Get TV series
#[tauri::command]
pub async fn get_xtream_series(
    state: State<'_, XtreamState>,
    profile_id: String,
    category_id: Option<String>,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client
        .get_series(category_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Get TV series with pagination
#[tauri::command]
pub async fn get_xtream_series_paginated(
    state: State<'_, XtreamState>,
    profile_id: String,
    category_id: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client
        .get_series_with_pagination(category_id.as_deref(), limit, offset)
        .await
        .map_err(|e| e.to_string())
}

/// Get series information with enhanced metadata
#[tauri::command]
pub async fn get_xtream_series_info(
    state: State<'_, XtreamState>,
    profile_id: String,
    series_id: String,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client.get_series_info(&series_id).await.map_err(|e| e.to_string())
}

/// Generate episode streaming URL
#[tauri::command]
pub async fn generate_xtream_episode_stream_url(
    state: State<'_, XtreamState>,
    profile_id: String,
    series_id: String,
    episode_id: String,
    extension: Option<String>,
) -> Result<String, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client
        .generate_episode_stream_url(&series_id, &episode_id, extension.as_deref())
        .map_err(|e| e.to_string())
}

/// Get short EPG for a channel
#[tauri::command]
pub async fn get_xtream_short_epg(
    state: State<'_, XtreamState>,
    profile_id: String,
    channel_id: String,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client.get_short_epg(&channel_id).await.map_err(|e| e.to_string())
}

/// Get full EPG for a channel with optional date range
#[tauri::command]
pub async fn get_xtream_full_epg(
    state: State<'_, XtreamState>,
    profile_id: String,
    channel_id: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client
        .get_full_epg(&channel_id, start_date.as_deref(), end_date.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Get EPG for multiple channels
#[tauri::command]
pub async fn get_xtream_epg_for_channels(
    state: State<'_, XtreamState>,
    profile_id: String,
    channel_ids: Vec<String>,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    let channel_refs: Vec<&str> = channel_ids.iter().map(|s| s.as_str()).collect();
    client
        .get_epg_for_channels(&channel_refs)
        .await
        .map_err(|e| e.to_string())
}

/// Get EPG for a specific date range using timestamps
#[tauri::command]
pub async fn get_xtream_epg_by_date_range(
    state: State<'_, XtreamState>,
    profile_id: String,
    channel_id: String,
    start_timestamp: u64,
    end_timestamp: u64,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client
        .get_epg_by_date_range(&channel_id, start_timestamp, end_timestamp)
        .await
        .map_err(|e| e.to_string())
}

/// Format EPG timestamp for display
#[tauri::command]
pub fn format_epg_time(timestamp: i64, timezone: Option<String>) -> String {
    XtreamClient::format_epg_time(timestamp, timezone.as_deref())
}

/// Get current timestamp for EPG queries
#[tauri::command]
pub fn get_current_timestamp() -> u64 {
    XtreamClient::get_current_timestamp()
}

/// Get timestamp for a specific number of hours from now
#[tauri::command]
pub fn get_timestamp_hours_from_now(hours: i64) -> u64 {
    XtreamClient::get_timestamp_hours_from_now(hours)
}

/// Parse EPG data and extract program information
#[tauri::command]
pub fn parse_epg_programs(epg_data: Value) -> Result<Vec<Value>, String> {
    XtreamClient::parse_epg_programs(&epg_data).map_err(|e| e.to_string())
}

/// Parse and enhance EPG data with formatted times and additional metadata
#[tauri::command]
pub fn parse_and_enhance_epg_data(epg_data: Value, timezone: Option<String>) -> Result<Value, String> {
    XtreamClient::parse_and_enhance_epg_data(&epg_data, timezone.as_deref()).map_err(|e| e.to_string())
}

/// Get EPG data for current and next programs on a channel
#[tauri::command]
pub async fn get_xtream_current_and_next_epg(
    state: State<'_, XtreamState>,
    profile_id: String,
    channel_id: String,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client.get_current_and_next_epg(&channel_id).await.map_err(|e| e.to_string())
}

/// Filter EPG programs by time range
#[tauri::command]
pub fn filter_epg_by_time_range(
    epg_data: Value,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
) -> Result<Value, String> {
    XtreamClient::filter_epg_by_time_range(&epg_data, start_timestamp, end_timestamp).map_err(|e| e.to_string())
}

/// Search EPG programs by title or description
#[tauri::command]
pub fn search_epg_programs(epg_data: Value, search_query: String) -> Result<Value, String> {
    XtreamClient::search_epg_programs(&epg_data, &search_query).map_err(|e| e.to_string())
}

/// Generate streaming URL for content
#[tauri::command]
pub async fn generate_xtream_stream_url(
    state: State<'_, XtreamState>,
    profile_id: String,
    content_type: String,
    content_id: String,
    extension: Option<String>,
) -> Result<String, String> {
    use crate::xtream::ContentType;
    
    let content_type_enum = match content_type.as_str() {
        "Channel" => ContentType::Channel,
        "Movie" => ContentType::Movie,
        "Series" => ContentType::Series,
        _ => return Err(format!("Invalid content type: {}", content_type)),
    };
    
    let request = StreamURLRequest {
        content_type: content_type_enum,
        content_id,
        extension,
    };
    
    let client = create_authenticated_client(&state, &profile_id).await?;
    client.generate_stream_url(&request).map_err(|e| e.to_string())
}

/// Filter channels by various criteria
#[tauri::command]
pub fn filter_xtream_channels(
    channels: Value,
    name_filter: Option<String>,
    category_filter: Option<String>,
    has_epg: Option<bool>,
    has_archive: Option<bool>,
) -> Result<Value, String> {
    XtreamClient::filter_channels(
        &channels,
        name_filter.as_deref(),
        category_filter.as_deref(),
        has_epg,
        has_archive,
    )
    .map_err(|e| e.to_string())
}

/// Sort channels by various criteria
#[tauri::command]
pub fn sort_xtream_channels(
    channels: Value,
    sort_by: String,
    ascending: bool,
) -> Result<Value, String> {
    XtreamClient::sort_channels(&channels, &sort_by, ascending).map_err(|e| e.to_string())
}

/// Search channels by name with fuzzy matching
#[tauri::command]
pub fn search_xtream_channels(
    channels: Value,
    search_query: String,
) -> Result<Value, String> {
    if search_query.trim().is_empty() {
        return Ok(channels);
    }
    
    XtreamClient::filter_channels(&channels, Some(&search_query), None, None, None)
        .map_err(|e| e.to_string())
}

/// Filter movies by various criteria
#[tauri::command]
pub fn filter_xtream_movies(
    movies: Value,
    name_filter: Option<String>,
    category_filter: Option<String>,
    genre_filter: Option<String>,
    rating_min: Option<f64>,
    year_filter: Option<String>,
) -> Result<Value, String> {
    XtreamClient::filter_movies(
        &movies,
        name_filter.as_deref(),
        category_filter.as_deref(),
        genre_filter.as_deref(),
        rating_min,
        year_filter.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Sort movies by various criteria
#[tauri::command]
pub fn sort_xtream_movies(
    movies: Value,
    sort_by: String,
    ascending: bool,
) -> Result<Value, String> {
    XtreamClient::sort_movies(&movies, &sort_by, ascending).map_err(|e| e.to_string())
}

/// Search movies by name with fuzzy matching
#[tauri::command]
pub fn search_xtream_movies(
    movies: Value,
    search_query: String,
) -> Result<Value, String> {
    if search_query.trim().is_empty() {
        return Ok(movies);
    }
    
    XtreamClient::filter_movies(&movies, Some(&search_query), None, None, None, None)
        .map_err(|e| e.to_string())
}

/// Get channel counts by category
#[tauri::command]
pub async fn get_xtream_channel_counts_by_category(
    state: State<'_, XtreamState>,
    profile_id: String,
) -> Result<Value, String> {
    let client = create_authenticated_client(&state, &profile_id).await?;
    client
        .get_channel_counts_by_category()
        .await
        .map_err(|e| e.to_string())
}

/// Validate channel data structure
#[tauri::command]
pub fn validate_xtream_channel_data(channel: Value) -> bool {
    XtreamClient::validate_channel_data(&channel)
}

/// Validate movie data structure
#[tauri::command]
pub fn validate_xtream_movie_data(movie: Value) -> bool {
    XtreamClient::validate_movie_data(&movie)
}

/// Filter series by various criteria
#[tauri::command]
pub fn filter_xtream_series(
    series: Value,
    name_filter: Option<String>,
    category_filter: Option<String>,
    genre_filter: Option<String>,
    rating_min: Option<f64>,
    year_filter: Option<String>,
) -> Result<Value, String> {
    XtreamClient::filter_series(
        &series,
        name_filter.as_deref(),
        category_filter.as_deref(),
        genre_filter.as_deref(),
        rating_min,
        year_filter.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Sort series by various criteria
#[tauri::command]
pub fn sort_xtream_series(
    series: Value,
    sort_by: String,
    ascending: bool,
) -> Result<Value, String> {
    XtreamClient::sort_series(&series, &sort_by, ascending).map_err(|e| e.to_string())
}

/// Search series by name with fuzzy matching
#[tauri::command]
pub fn search_xtream_series(
    series: Value,
    search_query: String,
) -> Result<Value, String> {
    if search_query.trim().is_empty() {
        return Ok(series);
    }
    
    XtreamClient::filter_series(&series, Some(&search_query), None, None, None, None)
        .map_err(|e| e.to_string())
}

/// Validate series data structure
#[tauri::command]
pub fn validate_xtream_series_data(series: Value) -> bool {
    XtreamClient::validate_series_data(&series)
}

/// Get playback history for a profile
#[tauri::command]
pub async fn get_xtream_playback_history(
    state: State<'_, XtreamState>,
    profile_id: String,
) -> Result<Value, String> {
    state
        .profile_manager
        .get_playback_history(&profile_id)
        .await
        .map_err(|e| e.to_string())
}

/// Add content to playback history
#[tauri::command]
pub async fn add_to_xtream_playback_history(
    state: State<'_, XtreamState>,
    profile_id: String,
    content_type: String,
    content_id: String,
    content_data: Value,
    position: Option<f64>,
    duration: Option<f64>,
) -> Result<(), String> {
    state
        .profile_manager
        .add_to_playback_history(&profile_id, &content_type, &content_id, &content_data, position, duration)
        .await
        .map_err(|e| e.to_string())
}

/// Update playback position for resume functionality
#[tauri::command]
pub async fn update_xtream_playback_position(
    state: State<'_, XtreamState>,
    profile_id: String,
    content_type: String,
    content_id: String,
    position: f64,
    duration: Option<f64>,
) -> Result<(), String> {
    state
        .profile_manager
        .update_playback_position(&profile_id, &content_type, &content_id, position, duration)
        .await
        .map_err(|e| e.to_string())
}

/// Helper function to create an authenticated client for a profile
async fn create_authenticated_client(
    state: &State<'_, XtreamState>,
    profile_id: &str,
) -> Result<XtreamClient, String> {
    // Get profile credentials
    let credentials = state
        .profile_manager
        .get_profile_credentials_async_wrapper(profile_id)
        .await
        .map_err(|e| e.to_string())?;

    // Create and return client
    XtreamClient::new(credentials, state.content_cache.clone()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_channels_by_name() {
        let channels = serde_json::json!([
            {
                "stream_id": 1,
                "name": "CNN International",
                "category_id": "1"
            },
            {
                "stream_id": 2,
                "name": "BBC World News",
                "category_id": "1"
            },
            {
                "stream_id": 3,
                "name": "ESPN Sports",
                "category_id": "2"
            }
        ]);

        let result = filter_xtream_channels(
            channels,
            Some("CNN".to_string()),
            None,
            None,
            None,
        ).unwrap();

        let filtered_array = result.as_array().unwrap();
        assert_eq!(filtered_array.len(), 1);
        assert_eq!(filtered_array[0]["name"], "CNN International");
    }

    #[test]
    fn test_sort_channels_by_name() {
        let channels = serde_json::json!([
            {
                "stream_id": 1,
                "name": "CNN International",
                "num": 3
            },
            {
                "stream_id": 2,
                "name": "BBC World News",
                "num": 1
            },
            {
                "stream_id": 3,
                "name": "ESPN Sports",
                "num": 2
            }
        ]);

        let result = sort_xtream_channels(
            channels,
            "name".to_string(),
            true,
        ).unwrap();

        let sorted_array = result.as_array().unwrap();
        assert_eq!(sorted_array[0]["name"], "BBC World News");
        assert_eq!(sorted_array[1]["name"], "CNN International");
        assert_eq!(sorted_array[2]["name"], "ESPN Sports");
    }

    #[test]
    fn test_validate_channel_data() {
        let valid_channel = serde_json::json!({
            "stream_id": 123,
            "name": "Test Channel"
        });

        let invalid_channel = serde_json::json!({
            "name": "Test Channel"
            // Missing stream_id
        });

        assert!(validate_xtream_channel_data(valid_channel));
        assert!(!validate_xtream_channel_data(invalid_channel));
    }

    #[test]
    fn test_filter_movies_by_name() {
        let movies = serde_json::json!([
            {
                "stream_id": 1,
                "name": "The Matrix",
                "category_id": "1",
                "genre": "Action, Sci-Fi",
                "rating_5based": 4.5
            },
            {
                "stream_id": 2,
                "name": "Inception",
                "category_id": "1",
                "genre": "Action, Thriller",
                "rating_5based": 4.8
            },
            {
                "stream_id": 3,
                "name": "The Godfather",
                "category_id": "2",
                "genre": "Crime, Drama",
                "rating_5based": 4.9
            }
        ]);

        let result = filter_xtream_movies(
            movies,
            Some("Matrix".to_string()),
            None,
            None,
            None,
            None,
        ).unwrap();

        let filtered_array = result.as_array().unwrap();
        assert_eq!(filtered_array.len(), 1);
        assert_eq!(filtered_array[0]["name"], "The Matrix");
    }

    #[test]
    fn test_sort_movies_by_rating() {
        let movies = serde_json::json!([
            {
                "stream_id": 1,
                "name": "The Matrix",
                "rating_5based": 4.5
            },
            {
                "stream_id": 2,
                "name": "Inception",
                "rating_5based": 4.8
            },
            {
                "stream_id": 3,
                "name": "The Godfather",
                "rating_5based": 4.9
            }
        ]);

        let result = sort_xtream_movies(
            movies,
            "rating".to_string(),
            false, // descending
        ).unwrap();

        let sorted_array = result.as_array().unwrap();
        assert_eq!(sorted_array[0]["name"], "The Godfather");
        assert_eq!(sorted_array[1]["name"], "Inception");
        assert_eq!(sorted_array[2]["name"], "The Matrix");
    }

    #[test]
    fn test_validate_movie_data() {
        let valid_movie = serde_json::json!({
            "stream_id": 123,
            "name": "Test Movie"
        });

        let invalid_movie = serde_json::json!({
            "name": "Test Movie"
            // Missing stream_id
        });

        assert!(validate_xtream_movie_data(valid_movie));
        assert!(!validate_xtream_movie_data(invalid_movie));
    }

    #[test]
    fn test_filter_series_by_name() {
        let series = serde_json::json!([
            {
                "series_id": 1,
                "name": "Breaking Bad",
                "category_id": "1",
                "genre": "Crime, Drama",
                "rating_5based": 4.9
            },
            {
                "series_id": 2,
                "name": "Game of Thrones",
                "category_id": "1",
                "genre": "Fantasy, Drama",
                "rating_5based": 4.7
            },
            {
                "series_id": 3,
                "name": "The Office",
                "category_id": "2",
                "genre": "Comedy",
                "rating_5based": 4.5
            }
        ]);

        let result = filter_xtream_series(
            series,
            Some("Breaking".to_string()),
            None,
            None,
            None,
            None,
        ).unwrap();

        let filtered_array = result.as_array().unwrap();
        assert_eq!(filtered_array.len(), 1);
        assert_eq!(filtered_array[0]["name"], "Breaking Bad");
    }

    #[test]
    fn test_sort_series_by_rating() {
        let series = serde_json::json!([
            {
                "series_id": 1,
                "name": "Breaking Bad",
                "rating_5based": 4.9
            },
            {
                "series_id": 2,
                "name": "Game of Thrones",
                "rating_5based": 4.7
            },
            {
                "series_id": 3,
                "name": "The Office",
                "rating_5based": 4.5
            }
        ]);

        let result = sort_xtream_series(
            series,
            "rating".to_string(),
            false, // descending
        ).unwrap();

        let sorted_array = result.as_array().unwrap();
        assert_eq!(sorted_array[0]["name"], "Breaking Bad");
        assert_eq!(sorted_array[1]["name"], "Game of Thrones");
        assert_eq!(sorted_array[2]["name"], "The Office");
    }

    #[test]
    fn test_validate_series_data() {
        let valid_series = serde_json::json!({
            "series_id": 123,
            "name": "Test Series"
        });

        let invalid_series = serde_json::json!({
            "name": "Test Series"
            // Missing series_id
        });

        assert!(validate_xtream_series_data(valid_series));
        assert!(!validate_xtream_series_data(invalid_series));
    }

    #[test]
    fn test_search_series() {
        let series = serde_json::json!([
            {
                "series_id": 1,
                "name": "Breaking Bad",
                "category_id": "1"
            },
            {
                "series_id": 2,
                "name": "Game of Thrones",
                "category_id": "1"
            }
        ]);

        let result = search_xtream_series(
            series,
            "Game".to_string(),
        ).unwrap();

        let filtered_array = result.as_array().unwrap();
        assert_eq!(filtered_array.len(), 1);
        assert_eq!(filtered_array[0]["name"], "Game of Thrones");
    }

    #[test]
    fn test_filter_series_by_genre() {
        let series = serde_json::json!([
            {
                "series_id": 1,
                "name": "Breaking Bad",
                "genre": "Crime, Drama",
                "category_id": "1"
            },
            {
                "series_id": 2,
                "name": "The Office",
                "genre": "Comedy",
                "category_id": "2"
            }
        ]);

        let result = filter_xtream_series(
            series,
            None,
            None,
            Some("Comedy".to_string()),
            None,
            None,
        ).unwrap();

        let filtered_array = result.as_array().unwrap();
        assert_eq!(filtered_array.len(), 1);
        assert_eq!(filtered_array[0]["name"], "The Office");
    }
}