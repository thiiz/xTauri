// Tests for Full-Text Search functionality

#[cfg(test)]
mod tests {
    use crate::content_cache::*;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    
    fn create_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        
        // Create xtream_profiles table (dependency)
        conn.execute(
            "CREATE TABLE xtream_profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                url TEXT NOT NULL,
                username TEXT NOT NULL,
                encrypted_credentials BLOB NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_used DATETIME,
                is_active BOOLEAN DEFAULT FALSE
            )",
            [],
        )
        .unwrap();
        
        Arc::new(Mutex::new(conn))
    }
    
    fn insert_test_profile(db: &Arc<Mutex<Connection>>, profile_id: &str) {
        let conn = db.lock().unwrap();
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES (?1, ?2, 'http://test.com', 'user', X'00')",
            [profile_id, &format!("Profile {}", profile_id)],
        )
        .unwrap();
    }
    
    #[test]
    fn test_fts_tables_initialization() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        // Verify FTS tables exist
        let conn = db.lock().unwrap();
        
        let channels_fts_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='xtream_channels_fts'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap();
        
        assert!(channels_fts_exists, "Channels FTS table should exist");
        
        let movies_fts_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='xtream_movies_fts'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap();
        
        assert!(movies_fts_exists, "Movies FTS table should exist");
        
        let series_fts_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='xtream_series_fts'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap();
        
        assert!(series_fts_exists, "Series FTS table should exist");
    }
    
    #[test]
    fn test_fts_search_channels_basic() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        // Insert test channels
        let channels = vec![
            XtreamChannel {
                stream_id: 1,
                num: Some(1),
                name: "HBO Sports".to_string(),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: Some("sports".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 2,
                num: Some(2),
                name: "ESPN Sports".to_string(),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: Some("sports".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 3,
                num: Some(3),
                name: "CNN News".to_string(),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: Some("news".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
        ];
        
        cache.save_channels("test-profile", channels).unwrap();
        
        // Search for "sports"
        let results = cache.fts_search_channels("test-profile", "sports", None).unwrap();
        
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|c| c.name == "HBO Sports"));
        assert!(results.iter().any(|c| c.name == "ESPN Sports"));
    }
    
    #[test]
    fn test_fts_search_channels_partial_match() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        let channels = vec![
            XtreamChannel {
                stream_id: 1,
                num: Some(1),
                name: "HBO".to_string(),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: None,
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
            XtreamChannel {
                stream_id: 2,
                num: Some(2),
                name: "HBO Max".to_string(),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: None,
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
        ];
        
        cache.save_channels("test-profile", channels).unwrap();
        
        // Search with partial word
        let results = cache.fts_search_channels("test-profile", "HB", None).unwrap();
        
        assert_eq!(results.len(), 2);
    }
    
    #[test]
    fn test_fts_search_movies_basic() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        // Insert test movies
        let movies = vec![
            XtreamMovie {
                stream_id: 1,
                num: Some(1),
                name: "The Matrix".to_string(),
                title: Some("The Matrix".to_string()),
                year: Some("1999".to_string()),
                stream_type: Some("movie".to_string()),
                stream_icon: None,
                rating: Some(8.7),
                rating_5based: Some(4.35),
                genre: Some("Action, Sci-Fi".to_string()),
                added: None,
                episode_run_time: Some(136),
                category_id: Some("action".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: Some("1999-03-31".to_string()),
                cast: Some("Keanu Reeves, Laurence Fishburne".to_string()),
                director: Some("Wachowski Brothers".to_string()),
                plot: Some("A computer hacker learns about the true nature of reality".to_string()),
                youtube_trailer: None,
            },
            XtreamMovie {
                stream_id: 2,
                num: Some(2),
                name: "Inception".to_string(),
                title: Some("Inception".to_string()),
                year: Some("2010".to_string()),
                stream_type: Some("movie".to_string()),
                stream_icon: None,
                rating: Some(8.8),
                rating_5based: Some(4.4),
                genre: Some("Action, Thriller".to_string()),
                added: None,
                episode_run_time: Some(148),
                category_id: Some("action".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: Some("2010-07-16".to_string()),
                cast: Some("Leonardo DiCaprio".to_string()),
                director: Some("Christopher Nolan".to_string()),
                plot: Some("A thief who steals corporate secrets through dream-sharing technology".to_string()),
                youtube_trailer: None,
            },
        ];
        
        cache.save_movies("test-profile", movies).unwrap();
        
        // Search for "matrix"
        let results = cache.fts_search_movies("test-profile", "matrix", None).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "The Matrix");
    }
    
    #[test]
    fn test_fts_search_movies_by_cast() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        let movies = vec![
            XtreamMovie {
                stream_id: 1,
                num: Some(1),
                name: "The Matrix".to_string(),
                title: Some("The Matrix".to_string()),
                year: Some("1999".to_string()),
                stream_type: Some("movie".to_string()),
                stream_icon: None,
                rating: Some(8.7),
                rating_5based: Some(4.35),
                genre: Some("Action".to_string()),
                added: None,
                episode_run_time: Some(136),
                category_id: Some("action".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: Some("1999-03-31".to_string()),
                cast: Some("Keanu Reeves, Laurence Fishburne".to_string()),
                director: Some("Wachowski Brothers".to_string()),
                plot: Some("A computer hacker learns about reality".to_string()),
                youtube_trailer: None,
            },
        ];
        
        cache.save_movies("test-profile", movies).unwrap();
        
        // Search by actor name
        let results = cache.fts_search_movies("test-profile", "Keanu", None).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "The Matrix");
    }
    
    #[test]
    fn test_fts_search_movies_by_plot() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        let movies = vec![
            XtreamMovie {
                stream_id: 1,
                num: Some(1),
                name: "Inception".to_string(),
                title: Some("Inception".to_string()),
                year: Some("2010".to_string()),
                stream_type: Some("movie".to_string()),
                stream_icon: None,
                rating: Some(8.8),
                rating_5based: Some(4.4),
                genre: Some("Action".to_string()),
                added: None,
                episode_run_time: Some(148),
                category_id: Some("action".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: Some("2010-07-16".to_string()),
                cast: Some("Leonardo DiCaprio".to_string()),
                director: Some("Christopher Nolan".to_string()),
                plot: Some("A thief who steals corporate secrets through dream-sharing technology".to_string()),
                youtube_trailer: None,
            },
        ];
        
        cache.save_movies("test-profile", movies).unwrap();
        
        // Search by plot keyword
        let results = cache.fts_search_movies("test-profile", "dream", None).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Inception");
    }
    
    #[test]
    fn test_fts_search_series_basic() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        // Insert test series
        let series = vec![
            XtreamSeries {
                series_id: 1,
                num: Some(1),
                name: "Breaking Bad".to_string(),
                title: Some("Breaking Bad".to_string()),
                year: Some("2008".to_string()),
                cover: None,
                plot: Some("A high school chemistry teacher turned methamphetamine producer".to_string()),
                cast: Some("Bryan Cranston, Aaron Paul".to_string()),
                director: Some("Vince Gilligan".to_string()),
                genre: Some("Crime, Drama".to_string()),
                release_date: Some("2008-01-20".to_string()),
                last_modified: None,
                rating: Some("9.5".to_string()),
                rating_5based: Some(4.75),
                episode_run_time: Some("47".to_string()),
                category_id: Some("drama".to_string()),
            },
            XtreamSeries {
                series_id: 2,
                num: Some(2),
                name: "Game of Thrones".to_string(),
                title: Some("Game of Thrones".to_string()),
                year: Some("2011".to_string()),
                cover: None,
                plot: Some("Nine noble families fight for control of the lands of Westeros".to_string()),
                cast: Some("Emilia Clarke, Kit Harington".to_string()),
                director: Some("David Benioff".to_string()),
                genre: Some("Fantasy, Drama".to_string()),
                release_date: Some("2011-04-17".to_string()),
                last_modified: None,
                rating: Some("9.3".to_string()),
                rating_5based: Some(4.65),
                episode_run_time: Some("57".to_string()),
                category_id: Some("fantasy".to_string()),
            },
        ];
        
        cache.save_series("test-profile", series).unwrap();
        
        // Search for "breaking"
        let results = cache.fts_search_series("test-profile", "breaking", None).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Breaking Bad");
    }
    
    #[test]
    fn test_fts_search_series_by_genre() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        let series = vec![
            XtreamSeries {
                series_id: 1,
                num: Some(1),
                name: "Breaking Bad".to_string(),
                title: Some("Breaking Bad".to_string()),
                year: Some("2008".to_string()),
                cover: None,
                plot: Some("A chemistry teacher turned meth producer".to_string()),
                cast: Some("Bryan Cranston".to_string()),
                director: Some("Vince Gilligan".to_string()),
                genre: Some("Crime, Drama".to_string()),
                release_date: None,
                last_modified: None,
                rating: Some("9.5".to_string()),
                rating_5based: Some(4.75),
                episode_run_time: None,
                category_id: Some("drama".to_string()),
            },
        ];
        
        cache.save_series("test-profile", series).unwrap();
        
        // Search by genre
        let results = cache.fts_search_series("test-profile", "crime", None).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Breaking Bad");
    }
    
    #[test]
    fn test_fts_search_performance_channels() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        // Insert 1000 channels
        let mut channels = Vec::new();
        for i in 1..=1000 {
            channels.push(XtreamChannel {
                stream_id: i,
                num: Some(i),
                name: format!("Channel {}", i),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: Some("general".to_string()),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            });
        }
        
        cache.save_channels("test-profile", channels).unwrap();
        
        // Measure search performance
        let start = std::time::Instant::now();
        let results = cache.fts_search_channels("test-profile", "Channel 5", None).unwrap();
        let duration = start.elapsed();
        
        assert!(!results.is_empty());
        
        // Should be faster than 150ms (target)
        assert!(
            duration.as_millis() < 150,
            "FTS search took {:?}, expected < 150ms",
            duration
        );
        
        println!("FTS search of 1000 channels took: {:?}", duration);
    }
    
    #[test]
    fn test_fts_search_performance_movies() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        // Insert 1000 movies
        let mut movies = Vec::new();
        for i in 1..=1000 {
            movies.push(XtreamMovie {
                stream_id: i,
                num: Some(i),
                name: format!("Movie {}", i),
                title: Some(format!("Movie Title {}", i)),
                year: Some("2020".to_string()),
                stream_type: Some("movie".to_string()),
                stream_icon: None,
                rating: Some(7.5),
                rating_5based: Some(3.75),
                genre: Some("Action".to_string()),
                added: None,
                episode_run_time: Some(120),
                category_id: Some("action".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: None,
                cast: Some("Actor Name".to_string()),
                director: Some("Director Name".to_string()),
                plot: Some(format!("This is the plot for movie {}", i)),
                youtube_trailer: None,
            });
        }
        
        cache.save_movies("test-profile", movies).unwrap();
        
        // Measure search performance
        let start = std::time::Instant::now();
        let results = cache.fts_search_movies("test-profile", "Movie 5", None).unwrap();
        let duration = start.elapsed();
        
        assert!(!results.is_empty());
        
        // Should be faster than 150ms (target)
        assert!(
            duration.as_millis() < 150,
            "FTS search took {:?}, expected < 150ms",
            duration
        );
        
        println!("FTS search of 1000 movies took: {:?}", duration);
    }
    
    #[test]
    fn test_fts_rebuild_index() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        // Insert channels
        let channels = vec![
            XtreamChannel {
                stream_id: 1,
                num: Some(1),
                name: "Test Channel".to_string(),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: None,
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
        ];
        
        cache.save_channels("test-profile", channels).unwrap();
        
        // Rebuild FTS index
        let result = cache.rebuild_fts_index("test-profile");
        if let Err(e) = &result {
            eprintln!("Rebuild FTS index error: {:?}", e);
        }
        assert!(result.is_ok());
        
        // Verify search still works
        let results = cache.fts_search_channels("test-profile", "Test", None).unwrap();
        assert_eq!(results.len(), 1);
    }
    
    #[test]
    fn test_fts_search_with_filters() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        let movies = vec![
            XtreamMovie {
                stream_id: 1,
                num: Some(1),
                name: "Action Movie 1".to_string(),
                title: Some("Action Movie 1".to_string()),
                year: Some("2020".to_string()),
                stream_type: Some("movie".to_string()),
                stream_icon: None,
                rating: Some(8.0),
                rating_5based: Some(4.0),
                genre: Some("Action".to_string()),
                added: None,
                episode_run_time: Some(120),
                category_id: Some("action".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: None,
                cast: None,
                director: None,
                plot: Some("An action-packed thriller".to_string()),
                youtube_trailer: None,
            },
            XtreamMovie {
                stream_id: 2,
                num: Some(2),
                name: "Action Movie 2".to_string(),
                title: Some("Action Movie 2".to_string()),
                year: Some("2021".to_string()),
                stream_type: Some("movie".to_string()),
                stream_icon: None,
                rating: Some(7.0),
                rating_5based: Some(3.5),
                genre: Some("Action".to_string()),
                added: None,
                episode_run_time: Some(110),
                category_id: Some("comedy".to_string()),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: None,
                cast: None,
                director: None,
                plot: Some("Another action movie".to_string()),
                youtube_trailer: None,
            },
        ];
        
        cache.save_movies("test-profile", movies).unwrap();
        
        // Search with category filter
        let filter = MovieFilter {
            category_id: Some("action".to_string()),
            ..Default::default()
        };
        
        let results = cache.fts_search_movies("test-profile", "action", Some(filter)).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Action Movie 1");
    }
    
    #[test]
    fn test_fts_search_empty_query() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        let channels = vec![
            XtreamChannel {
                stream_id: 1,
                num: Some(1),
                name: "Test Channel".to_string(),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: None,
                added: None,
                category_id: None,
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            },
        ];
        
        cache.save_channels("test-profile", channels).unwrap();
        
        // Empty query should fall back to regular get_channels
        let results = cache.fts_search_channels("test-profile", "", None).unwrap();
        
        assert_eq!(results.len(), 1);
    }
}
