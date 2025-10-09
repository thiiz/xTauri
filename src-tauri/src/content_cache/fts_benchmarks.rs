// Benchmarks for Full-Text Search performance
// 
// These tests verify that FTS search meets the < 150ms performance target
// across different dataset sizes and query types.

#[cfg(test)]
mod benchmarks {
    use crate::content_cache::*;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;
    
    fn create_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        
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
    fn benchmark_fts_channels_10k() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        println!("\n=== FTS Channels Benchmark: 10,000 records ===");
        
        // Insert 10,000 channels
        let mut channels = Vec::new();
        for i in 1..=10000 {
            channels.push(XtreamChannel {
                stream_id: i,
                num: Some(i),
                name: format!("Channel {} - {}", i, if i % 3 == 0 { "Sports" } else if i % 3 == 1 { "News" } else { "Entertainment" }),
                stream_type: Some("live".to_string()),
                stream_icon: None,
                thumbnail: None,
                epg_channel_id: Some(format!("epg_{}", i)),
                added: None,
                category_id: Some(format!("cat_{}", i % 10)),
                custom_sid: None,
                tv_archive: None,
                direct_source: None,
                tv_archive_duration: None,
            });
        }
        
        let insert_start = Instant::now();
        cache.save_channels("test-profile", channels).unwrap();
        let insert_duration = insert_start.elapsed();
        println!("Insert time: {:?}", insert_duration);
        
        // Test 1: Exact match search
        let start = Instant::now();
        let results = cache.fts_search_channels("test-profile", "Channel 5000", None).unwrap();
        let duration = start.elapsed();
        println!("Exact match search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Exact match search took {:?}, expected < 150ms", duration);
        
        // Test 2: Partial match search
        let start = Instant::now();
        let results = cache.fts_search_channels("test-profile", "Sports", None).unwrap();
        let duration = start.elapsed();
        println!("Partial match search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Partial match search took {:?}, expected < 150ms", duration);
        
        // Test 3: Multi-word search
        let start = Instant::now();
        let results = cache.fts_search_channels("test-profile", "Channel Sports", None).unwrap();
        let duration = start.elapsed();
        println!("Multi-word search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Multi-word search took {:?}, expected < 150ms", duration);
        
        // Test 4: Prefix search
        let start = Instant::now();
        let results = cache.fts_search_channels("test-profile", "Chan", None).unwrap();
        let duration = start.elapsed();
        println!("Prefix search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Prefix search took {:?}, expected < 150ms", duration);
    }
    
    #[test]
    fn benchmark_fts_movies_10k() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        println!("\n=== FTS Movies Benchmark: 10,000 records ===");
        
        // Insert 10,000 movies
        let mut movies = Vec::new();
        let genres = vec!["Action", "Comedy", "Drama", "Thriller", "Sci-Fi", "Horror"];
        let actors = vec!["Tom Hanks", "Brad Pitt", "Leonardo DiCaprio", "Meryl Streep"];
        
        for i in 1..=10000 {
            movies.push(XtreamMovie {
                stream_id: i,
                num: Some(i),
                name: format!("Movie {}", i),
                title: Some(format!("The Great Movie {}", i)),
                year: Some(format!("{}", 2000 + (i % 24))),
                stream_type: Some("movie".to_string()),
                stream_icon: None,
                rating: Some(5.0 + (i % 5) as f64),
                rating_5based: Some(2.5 + (i % 5) as f64 / 2.0),
                genre: Some(genres[i as usize % genres.len()].to_string()),
                added: None,
                episode_run_time: Some(90 + (i % 60) as i64),
                category_id: Some(format!("cat_{}", i % 10)),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: None,
                cast: Some(actors[i as usize % actors.len()].to_string()),
                director: Some(format!("Director {}", i % 100)),
                plot: Some(format!("This is an exciting {} movie about adventure and drama", genres[i as usize % genres.len()])),
                youtube_trailer: None,
            });
        }
        
        let insert_start = Instant::now();
        cache.save_movies("test-profile", movies).unwrap();
        let insert_duration = insert_start.elapsed();
        println!("Insert time: {:?}", insert_duration);
        
        // Test 1: Search by title
        let start = Instant::now();
        let results = cache.fts_search_movies("test-profile", "Great Movie", None).unwrap();
        let duration = start.elapsed();
        println!("Title search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Title search took {:?}, expected < 150ms", duration);
        
        // Test 2: Search by genre
        let start = Instant::now();
        let results = cache.fts_search_movies("test-profile", "Action", None).unwrap();
        let duration = start.elapsed();
        println!("Genre search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Genre search took {:?}, expected < 150ms", duration);
        
        // Test 3: Search by actor
        let start = Instant::now();
        let results = cache.fts_search_movies("test-profile", "Tom Hanks", None).unwrap();
        let duration = start.elapsed();
        println!("Actor search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Actor search took {:?}, expected < 150ms", duration);
        
        // Test 4: Search by plot keyword
        let start = Instant::now();
        let results = cache.fts_search_movies("test-profile", "adventure", None).unwrap();
        let duration = start.elapsed();
        println!("Plot search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Plot search took {:?}, expected < 150ms", duration);
        
        // Test 5: Complex multi-field search
        let start = Instant::now();
        let results = cache.fts_search_movies("test-profile", "action adventure", None).unwrap();
        let duration = start.elapsed();
        println!("Multi-field search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Multi-field search took {:?}, expected < 150ms", duration);
    }
    
    #[test]
    fn benchmark_fts_series_10k() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        println!("\n=== FTS Series Benchmark: 10,000 records ===");
        
        // Insert 10,000 series
        let mut series = Vec::new();
        let genres = vec!["Drama", "Comedy", "Thriller", "Fantasy", "Crime"];
        
        for i in 1..=10000 {
            series.push(XtreamSeries {
                series_id: i,
                num: Some(i),
                name: format!("Series {}", i),
                title: Some(format!("The Amazing Series {}", i)),
                year: Some(format!("{}", 2010 + (i % 14))),
                cover: None,
                plot: Some(format!("An epic {} series about heroes and villains", genres[i as usize % genres.len()])),
                cast: Some(format!("Actor {} and Actor {}", i % 100, (i + 1) % 100)),
                director: Some(format!("Director {}", i % 50)),
                genre: Some(genres[i as usize % genres.len()].to_string()),
                release_date: None,
                last_modified: None,
                rating: Some(format!("{:.1}", 7.0 + (i % 3) as f64)),
                rating_5based: Some(3.5 + (i % 3) as f64 / 2.0),
                episode_run_time: Some(format!("{}", 40 + (i % 20))),
                category_id: Some(format!("cat_{}", i % 10)),
            });
        }
        
        let insert_start = Instant::now();
        cache.save_series("test-profile", series).unwrap();
        let insert_duration = insert_start.elapsed();
        println!("Insert time: {:?}", insert_duration);
        
        // Test 1: Search by name
        let start = Instant::now();
        let results = cache.fts_search_series("test-profile", "Series 5000", None).unwrap();
        let duration = start.elapsed();
        println!("Name search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Name search took {:?}, expected < 150ms", duration);
        
        // Test 2: Search by genre
        let start = Instant::now();
        let results = cache.fts_search_series("test-profile", "Drama", None).unwrap();
        let duration = start.elapsed();
        println!("Genre search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Genre search took {:?}, expected < 150ms", duration);
        
        // Test 3: Search by plot keyword
        let start = Instant::now();
        let results = cache.fts_search_series("test-profile", "heroes", None).unwrap();
        let duration = start.elapsed();
        println!("Plot search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Plot search took {:?}, expected < 150ms", duration);
        
        // Test 4: Multi-word search
        let start = Instant::now();
        let results = cache.fts_search_series("test-profile", "Amazing Series", None).unwrap();
        let duration = start.elapsed();
        println!("Multi-word search: {:?} ({} results)", duration, results.len());
        assert!(duration.as_millis() < 150, "Multi-word search took {:?}, expected < 150ms", duration);
    }
    
    #[test]
    fn benchmark_fts_vs_like_comparison() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        println!("\n=== FTS vs LIKE Performance Comparison ===");
        
        // Insert 5,000 movies for comparison
        let mut movies = Vec::new();
        for i in 1..=5000 {
            movies.push(XtreamMovie {
                stream_id: i,
                num: Some(i),
                name: format!("Movie {}", i),
                title: Some(format!("Title {}", i)),
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
                plot: Some(format!("Plot for movie {}", i)),
                youtube_trailer: None,
            });
        }
        
        cache.save_movies("test-profile", movies).unwrap();
        
        // Test FTS search
        let start = Instant::now();
        let fts_results = cache.fts_search_movies("test-profile", "Movie", None).unwrap();
        let fts_duration = start.elapsed();
        
        // Test LIKE search (existing search_movies method)
        let start = Instant::now();
        let like_results = cache.search_movies("test-profile", "Movie", None, None, None).unwrap();
        let like_duration = start.elapsed();
        
        println!("FTS search: {:?} ({} results)", fts_duration, fts_results.len());
        println!("LIKE search: {:?} ({} results)", like_duration, like_results.len());
        println!("FTS speedup: {:.2}x", like_duration.as_secs_f64() / fts_duration.as_secs_f64());
        
        // FTS should be faster or at least comparable
        assert!(fts_duration.as_millis() < 150, "FTS search should be < 150ms");
    }
    
    #[test]
    fn benchmark_fts_with_pagination() {
        let db = create_test_db();
        insert_test_profile(&db, "test-profile");
        
        let cache = ContentCache::new(db.clone()).unwrap();
        cache.initialize_profile("test-profile").unwrap();
        
        println!("\n=== FTS Pagination Benchmark ===");
        
        // Insert 10,000 channels
        let mut channels = Vec::new();
        for i in 1..=10000 {
            channels.push(XtreamChannel {
                stream_id: i,
                num: Some(i),
                name: format!("Channel {}", i),
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
            });
        }
        
        cache.save_channels("test-profile", channels).unwrap();
        
        // Test paginated search
        let page_size = 50;
        let mut total_duration = std::time::Duration::ZERO;
        
        for page in 0..5 {
            let filter = ChannelFilter {
                limit: Some(page_size),
                offset: Some(page * page_size),
                ..Default::default()
            };
            
            let start = Instant::now();
            let results = cache.fts_search_channels("test-profile", "Channel", Some(filter)).unwrap();
            let duration = start.elapsed();
            
            total_duration += duration;
            
            println!("Page {} ({} results): {:?}", page + 1, results.len(), duration);
            assert!(duration.as_millis() < 150, "Page {} search took {:?}, expected < 150ms", page + 1, duration);
        }
        
        println!("Average per page: {:?}", total_duration / 5);
    }
}
