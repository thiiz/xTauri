mod channels;
pub mod database;
mod error;
mod favorites;
mod filters;
pub mod fuzzy_search;
mod groups;
mod history;
pub mod image_cache;
mod image_cache_api;
pub mod m3u_parser;
mod m3u_parser_helpers;
mod playlists;
pub mod search;
mod settings;
mod state;
mod utils;
pub mod xtream;

#[cfg(test)]
mod integration_tests;

use error::{Result, XTauriError};
use image_cache::ImageCache;
use playlists::FetchState;
use state::{ChannelCacheState, DbState, ImageCacheState};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use xtream::{XtreamState, ProfileManager, CredentialManager, ContentCache};

// Import all the command functions from their respective modules
use channels::*;
use favorites::*;
use filters::*;
use groups::*;
use history::*;
use image_cache_api::*;
use playlists::*;
use search::*;
use settings::*;
use xtream::commands::*;

fn initialize_application() -> Result<(rusqlite::Connection, Vec<m3u_parser::Channel>)> {
    let mut db_connection = database::initialize_database()
        .map_err(|e| XTauriError::database_init(format!("Database initialization failed: {}", e)))?;

    // Run cleanup on startup to remove orphaned channel list files
    if let Err(e) = utils::cleanup_orphaned_channel_files(&db_connection) {
        println!("Warning: Channel list cleanup failed: {}", e);
    }

    let channels = m3u_parser::get_channels(&mut db_connection, None);
    database::populate_channels(&mut db_connection, &channels)
        .map_err(|e| XTauriError::database_init(format!("Failed to populate channels: {}", e)))?;

    Ok((db_connection, channels))
}

fn setup_image_cache(app: &tauri::App) -> Result<ImageCache> {
    ImageCache::new(app.handle())
        .map_err(|e| XTauriError::internal(format!("Failed to initialize image cache: {}", e)))
}

fn setup_xtream_state(db_connection: Arc<Mutex<rusqlite::Connection>>) -> Result<XtreamState> {
    // Create credential manager
    let credential_manager = Arc::new(
        CredentialManager::new()
            .map_err(|e| XTauriError::internal(format!("Failed to initialize credential manager: {}", e)))?
    );
    
    // Create content cache using the same database connection
    let content_cache = Arc::new(
        ContentCache::new(Arc::clone(&db_connection), std::time::Duration::from_secs(3600))
    );
    
    // Create profile manager using the same database connection
    let profile_manager = Arc::new(ProfileManager::new(db_connection, credential_manager));
    
    Ok(XtreamState::new(profile_manager, content_cache))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (db_connection, _channels) = match initialize_application() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Fatal error during application initialization: {}", e);
            eprintln!("Error details: {}", e);
            std::process::exit(1);
        }
    };

    let db_arc = Arc::new(Mutex::new(db_connection));
    
    tauri::Builder::default()
        .manage(DbState {
            db: Mutex::new(
                // Create a new connection for the DbState since we need to share the Arc
                database::initialize_database()
                    .map_err(|e| XTauriError::database_init(format!("Failed to create second DB connection: {}", e)))
                    .unwrap()
            ),
        })
        .manage(ChannelCacheState {
            cache: Mutex::new(None),
        })
        .manage(FetchState::new())
        .setup(|app| {
            let image_cache = match setup_image_cache(app) {
                Ok(cache) => cache,
                Err(e) => {
                    eprintln!("Failed to initialize image cache: {}", e);
                    return Err(Box::new(e));
                }
            };
            app.manage(ImageCacheState {
                cache: Arc::new(image_cache),
            });
            
            // Initialize Xtream state
            let xtream_state = match setup_xtream_state(db_arc) {
                Ok(state) => state,
                Err(e) => {
                    eprintln!("Failed to initialize Xtream state: {}", e);
                    return Err(Box::new(e));
                }
            };
            app.manage(xtream_state);
            
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Channel commands
            get_channels,
            get_groups,
            add_favorite,
            remove_favorite,
            get_favorites,
            get_history,
            search_channels,
            invalidate_channel_cache,
            invalidate_search_cache,
            get_cache_stats,
            warm_cache_with_common_searches,
            // Async channel commands
            get_channels_async,
            get_groups_async,
            search_channels_async,
            add_favorite_async,
            remove_favorite_async,
            get_favorites_async,
            get_history_async,
            // Settings commands
            get_cache_duration,
            set_cache_duration,
            get_enable_preview,
            set_enable_preview,
            get_mute_on_start,
            set_mute_on_start,
            get_show_controls,
            set_show_controls,
            get_autoplay,
            set_autoplay,
            // Playlist commands
            get_channel_lists,
            add_channel_list,
            set_default_channel_list,
            refresh_channel_list,
            validate_and_add_channel_list,
            delete_channel_list,
            update_channel_list,
            start_channel_list_selection,
            start_channel_list_selection_async,
            // Async playlist commands
            refresh_channel_list_async,
            validate_and_add_channel_list_async,
            get_playlist_fetch_status,
            get_all_playlist_fetch_status,
            // Image cache commands (sync)
            get_cached_image,
            clear_image_cache,
            get_image_cache_size,
            // Async image cache commands
            get_cached_image_async,
            clear_image_cache_async,
            get_image_cache_size_async,
            get_image_download_status,
            preload_images,
            // Group commands
            get_enabled_groups,
            update_group_selection,
            sync_channel_list_groups,
            enable_all_groups,
            disable_all_groups,
            // Filter commands
            save_filter,
            get_saved_filters,
            delete_saved_filter,
            // Xtream commands
            create_xtream_profile,
            update_xtream_profile,
            delete_xtream_profile,
            get_xtream_profiles,
            get_xtream_profile,
            validate_xtream_credentials,
            authenticate_xtream_profile,
            get_xtream_channel_categories,
            get_xtream_channels,
            get_xtream_channels_paginated,
            get_xtream_movie_categories,
            get_xtream_movies,
            get_xtream_movies_paginated,
            get_xtream_movie_info,
            get_xtream_series_categories,
            get_xtream_series,
            get_xtream_series_paginated,
            get_xtream_series_info,
            get_xtream_short_epg,
            get_xtream_full_epg,
            get_xtream_epg_for_channels,
            get_xtream_epg_by_date_range,
            format_epg_time,
            get_current_timestamp,
            get_timestamp_hours_from_now,
            parse_epg_programs,
            parse_and_enhance_epg_data,
            get_xtream_current_and_next_epg,
            filter_epg_by_time_range,
            search_epg_programs,
            generate_xtream_stream_url,
            filter_xtream_channels,
            sort_xtream_channels,
            search_xtream_channels,
            get_xtream_channel_counts_by_category,
            validate_xtream_channel_data,
            filter_xtream_movies,
            sort_xtream_movies,
            search_xtream_movies,
            validate_xtream_movie_data,
            generate_xtream_episode_stream_url,
            filter_xtream_series,
            sort_xtream_series,
            search_xtream_series,
            validate_xtream_series_data,
            // Playback history commands
            get_xtream_playback_history,
            add_to_xtream_playback_history,
            update_xtream_playback_position,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| {
            eprintln!("Failed to run Tauri application: {}", e);
            std::process::exit(1);
        })
        .unwrap();
}
