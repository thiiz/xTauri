/// Mock Xtream API responses for testing
use serde_json::json;

pub fn mock_player_api_response() -> serde_json::Value {
    json!({
        "user_info": {
            "username": "testuser",
            "password": "testpass",
            "message": "Active",
            "auth": 1,
            "status": "Active",
            "exp_date": "1735689600",
            "is_trial": "0",
            "active_cons": "1",
            "created_at": "1704067200",
            "max_connections": "1",
            "allowed_output_formats": ["m3u8", "ts"]
        },
        "server_info": {
            "url": "http://example.com:8080",
            "port": "8080",
            "https_port": "8443",
            "server_protocol": "http",
            "rtmp_port": "1935",
            "timezone": "America/New_York",
            "timestamp_now": 1704153600,
            "time_now": "2024-01-01 12:00:00"
        }
    })
}

pub fn mock_live_categories_response() -> serde_json::Value {
    json!([
        {
            "category_id": "1",
            "category_name": "Sports",
            "parent_id": 0
        },
        {
            "category_id": "2",
            "category_name": "News",
            "parent_id": 0
        },
        {
            "category_id": "3",
            "category_name": "Entertainment",
            "parent_id": 0
        }
    ])
}

pub fn mock_live_streams_response() -> serde_json::Value {
    json!([
        {
            "num": 1,
            "name": "ESPN HD",
            "stream_type": "live",
            "stream_id": 1001,
            "stream_icon": "http://example.com/icons/espn.png",
            "epg_channel_id": "espn.us",
            "added": "1704067200",
            "category_id": "1",
            "custom_sid": "",
            "tv_archive": 1,
            "direct_source": "",
            "tv_archive_duration": 7
        },
        {
            "num": 2,
            "name": "CNN HD",
            "stream_type": "live",
            "stream_id": 1002,
            "stream_icon": "http://example.com/icons/cnn.png",
            "epg_channel_id": "cnn.us",
            "added": "1704067200",
            "category_id": "2",
            "custom_sid": "",
            "tv_archive": 0,
            "direct_source": "",
            "tv_archive_duration": 0
        }
    ])
}

pub fn mock_vod_categories_response() -> serde_json::Value {
    json!([
        {
            "category_id": "10",
            "category_name": "Action Movies",
            "parent_id": 0
        },
        {
            "category_id": "11",
            "category_name": "Comedy Movies",
            "parent_id": 0
        }
    ])
}

pub fn mock_vod_streams_response() -> serde_json::Value {
    json!([
        {
            "num": 1,
            "name": "Action Movie 1",
            "stream_type": "movie",
            "stream_id": 2001,
            "stream_icon": "http://example.com/posters/action1.jpg",
            "rating": "8.5",
            "rating_5based": 4.25,
            "added": "1704067200",
            "category_id": "10",
            "container_extension": "mp4",
            "custom_sid": "",
            "direct_source": ""
        },
        {
            "num": 2,
            "name": "Comedy Movie 1",
            "stream_type": "movie",
            "stream_id": 2002,
            "stream_icon": "http://example.com/posters/comedy1.jpg",
            "rating": "7.8",
            "rating_5based": 3.9,
            "added": "1704067200",
            "category_id": "11",
            "container_extension": "mkv",
            "custom_sid": "",
            "direct_source": ""
        }
    ])
}

pub fn mock_vod_info_response() -> serde_json::Value {
    json!({
        "info": {
            "tmdb_id": "12345",
            "name": "Action Movie 1",
            "o_name": "Original Action Movie 1",
            "cover_big": "http://example.com/covers/action1_big.jpg",
            "movie_image": "http://example.com/posters/action1.jpg",
            "releasedate": "2023-01-15",
            "episode_run_time": "120",
            "youtube_trailer": "https://youtube.com/watch?v=abc123",
            "director": "John Director",
            "actors": "Actor One, Actor Two, Actor Three",
            "cast": "Actor One, Actor Two",
            "description": "An exciting action movie with lots of explosions.",
            "plot": "A hero must save the world from evil.",
            "age": "PG-13",
            "mpaa_rating": "PG-13",
            "rating": "8.5",
            "rating_5based": 4.25,
            "country": "USA",
            "genre": "Action, Adventure",
            "backdrop_path": ["http://example.com/backdrops/action1_1.jpg"],
            "duration_secs": 7200,
            "duration": "2:00:00",
            "video": {
                "index": 0,
                "codec_name": "h264",
                "codec_long_name": "H.264 / AVC / MPEG-4 AVC / MPEG-4 part 10",
                "profile": "High",
                "codec_type": "video",
                "width": 1920,
                "height": 1080
            },
            "audio": {
                "index": 1,
                "codec_name": "aac",
                "codec_long_name": "AAC (Advanced Audio Coding)",
                "codec_type": "audio",
                "channels": 2,
                "sample_rate": "48000"
            },
            "bitrate": 5000
        },
        "movie_data": {
            "stream_id": 2001,
            "name": "Action Movie 1",
            "added": "1704067200",
            "category_id": "10",
            "container_extension": "mp4",
            "custom_sid": "",
            "direct_source": ""
        }
    })
}

pub fn mock_series_categories_response() -> serde_json::Value {
    json!([
        {
            "category_id": "20",
            "category_name": "Drama Series",
            "parent_id": 0
        },
        {
            "category_id": "21",
            "category_name": "Sci-Fi Series",
            "parent_id": 0
        }
    ])
}

pub fn mock_series_response() -> serde_json::Value {
    json!([
        {
            "num": 1,
            "name": "Drama Show 1",
            "series_id": 3001,
            "cover": "http://example.com/covers/drama1.jpg",
            "plot": "A compelling drama series",
            "cast": "Lead Actor, Supporting Actor",
            "director": "Series Director",
            "genre": "Drama",
            "releaseDate": "2022-01-01",
            "last_modified": "1704067200",
            "rating": "9.0",
            "rating_5based": 4.5,
            "backdrop_path": ["http://example.com/backdrops/drama1.jpg"],
            "youtube_trailer": "https://youtube.com/watch?v=xyz789",
            "episode_run_time": "45",
            "category_id": "20"
        }
    ])
}

pub fn mock_series_info_response() -> serde_json::Value {
    json!({
        "seasons": [
            {
                "air_date": "2022-01-01",
                "episode_count": 10,
                "id": 1,
                "name": "Season 1",
                "overview": "The first season",
                "season_number": 1,
                "cover": "http://example.com/covers/drama1_s1.jpg",
                "cover_big": "http://example.com/covers/drama1_s1_big.jpg"
            },
            {
                "air_date": "2023-01-01",
                "episode_count": 12,
                "id": 2,
                "name": "Season 2",
                "overview": "The second season",
                "season_number": 2,
                "cover": "http://example.com/covers/drama1_s2.jpg",
                "cover_big": "http://example.com/covers/drama1_s2_big.jpg"
            }
        ],
        "info": {
            "name": "Drama Show 1",
            "cover": "http://example.com/covers/drama1.jpg",
            "plot": "A compelling drama series",
            "cast": "Lead Actor, Supporting Actor",
            "director": "Series Director",
            "genre": "Drama",
            "releaseDate": "2022-01-01",
            "last_modified": "1704067200",
            "rating": "9.0",
            "rating_5based": 4.5,
            "backdrop_path": ["http://example.com/backdrops/drama1.jpg"],
            "youtube_trailer": "https://youtube.com/watch?v=xyz789",
            "episode_run_time": "45",
            "category_id": "20"
        },
        "episodes": {
            "1": [
                {
                    "id": "1",
                    "episode_num": 1,
                    "title": "Pilot",
                    "container_extension": "mp4",
                    "info": {
                        "air_date": "2022-01-01",
                        "crew": "Director: Episode Director",
                        "rating": "8.5",
                        "plot": "The beginning of the story",
                        "duration_secs": 2700,
                        "duration": "45:00",
                        "movie_image": "http://example.com/episodes/drama1_s1e1.jpg"
                    },
                    "custom_sid": "",
                    "added": "1704067200",
                    "season": 1,
                    "direct_source": ""
                },
                {
                    "id": "2",
                    "episode_num": 2,
                    "title": "Episode 2",
                    "container_extension": "mp4",
                    "info": {
                        "air_date": "2022-01-08",
                        "crew": "Director: Episode Director",
                        "rating": "8.7",
                        "plot": "The story continues",
                        "duration_secs": 2700,
                        "duration": "45:00",
                        "movie_image": "http://example.com/episodes/drama1_s1e2.jpg"
                    },
                    "custom_sid": "",
                    "added": "1704067200",
                    "season": 1,
                    "direct_source": ""
                }
            ],
            "2": [
                {
                    "id": "11",
                    "episode_num": 1,
                    "title": "Season 2 Premiere",
                    "container_extension": "mp4",
                    "info": {
                        "air_date": "2023-01-01",
                        "crew": "Director: Episode Director",
                        "rating": "9.0",
                        "plot": "A new chapter begins",
                        "duration_secs": 2700,
                        "duration": "45:00",
                        "movie_image": "http://example.com/episodes/drama1_s2e1.jpg"
                    },
                    "custom_sid": "",
                    "added": "1704067200",
                    "season": 2,
                    "direct_source": ""
                }
            ]
        }
    })
}

pub fn mock_short_epg_response() -> serde_json::Value {
    json!({
        "epg_listings": [
            {
                "id": "1",
                "epg_id": "espn.us",
                "title": "SportsCenter",
                "lang": "en",
                "start": "2024-01-01 10:00:00",
                "end": "2024-01-01 11:00:00",
                "description": "Latest sports news and highlights",
                "channel_id": "espn.us",
                "start_timestamp": "1704110400",
                "stop_timestamp": "1704114000",
                "now_playing": 1,
                "has_archive": 1
            },
            {
                "id": "2",
                "epg_id": "espn.us",
                "title": "NBA Game",
                "lang": "en",
                "start": "2024-01-01 11:00:00",
                "end": "2024-01-01 14:00:00",
                "description": "Live NBA basketball game",
                "channel_id": "espn.us",
                "start_timestamp": "1704114000",
                "stop_timestamp": "1704124800",
                "now_playing": 0,
                "has_archive": 1
            }
        ]
    })
}

pub fn mock_error_response(status: u16, message: &str) -> serde_json::Value {
    json!({
        "error": message,
        "status": status
    })
}

pub fn mock_invalid_credentials_response() -> serde_json::Value {
    json!({
        "user_info": {
            "auth": 0,
            "message": "Invalid credentials"
        }
    })
}

pub fn mock_server_error_response() -> serde_json::Value {
    json!({
        "error": "Internal server error",
        "status": 500
    })
}
