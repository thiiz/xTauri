// Performance benchmarks for database operations
#[cfg(test)]
mod benchmarks {
    use crate::content_cache::*;
    use crate::error::Result;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;
    
    /// Helper to create test database with schema
    fn setup_benchmark_db() -> Result<Arc<Mutex<Connection>>> {
        let conn = Connection::open_in_memory()?;
        let db = Arc::new(Mutex::new(conn));
        
        // Initialize schema
        {
            let conn = db.lock().unwrap();
            schema::initialize_content_cache_tables(&conn)?;
        }
        
        Ok(db)
    }
    
    /// Generate test channels
    fn generate_test_channels(count: usize, _profile_id: &str) -> Vec<XtreamChannel> {
        (0..count)
            .map(|i| XtreamChannel {
                stream_id: i as i64,
                num: Some(i as i64),
                name: format!("Test Channel {}", i),
                stream_type: Some("live".to_string()),
                stream_icon: Some(format!("http://example.com/icon{}.png", i)),
                thumbnail: None,
                epg_channel_id: Some(format!("epg_{}", i)),
                added: Some("2024-01-01".to_string()),
                category_id: Some(format!("{}", i % 10)),
                custom_sid: None,
                tv_archive: Some(1),
                direct_source: None,
                tv_archive_duration: Some(7),
            })
            .collect()
    }
    
    /// Generate test movies
    fn generate_test_movies(count: usize, _profile_id: &str) -> Vec<XtreamMovie> {
        (0..count)
            .map(|i| XtreamMovie {
                stream_id: i as i64,
                num: Some(i as i64),
                name: format!("Test Movie {}", i),
                title: Some(format!("Movie Title {}", i)),
                year: Some(format!("{}", 2000 + (i % 24))),
                stream_type: Some("movie".to_string()),
                stream_icon: Some(format!("http://example.com/poster{}.jpg", i)),
                rating: Some(5.0 + (i % 5) as f64),
                rating_5based: Some(3.0 + (i % 3) as f64),
                genre: Some(format!("Genre{}", i % 5)),
                added: Some("2024-01-01".to_string()),
                episode_run_time: Some(120),
                category_id: Some(format!("{}", i % 10)),
                container_extension: Some("mp4".to_string()),
                custom_sid: None,
                direct_source: None,
                release_date: Some("2024-01-01".to_string()),
                cast: Some("Actor 1, Actor 2".to_string()),
                director: Some("Director Name".to_string()),
                plot: Some("This is a test movie plot".to_string()),
                youtube_trailer: None,
            })
            .collect()
    }
    
    /// Generate test series
    fn generate_test_series(count: usize, _profile_id: &str) -> Vec<XtreamSeries> {
        (0..count)
            .map(|i| XtreamSeries {
                series_id: i as i64,
                num: Some(i as i64),
                name: format!("Test Series {}", i),
                title: Some(format!("Series Title {}", i)),
                year: Some(format!("{}", 2000 + (i % 24))),
                cover: Some(format!("http://example.com/cover{}.jpg", i)),
                plot: Some("Test series plot".to_string()),
                cast: Some("Actor 1, Actor 2".to_string()),
                director: Some("Director".to_string()),
                genre: Some(format!("Genre{}", i % 5)),
                release_date: Some("2024-01-01".to_string()),
                last_modified: Some("2024-01-01".to_string()),
                rating: Some("8.5".to_string()),
                rating_5based: Some(4.2),
                episode_run_time: Some("45".to_string()),
                category_id: Some(format!("{}", i % 10)),
            })
            .collect()
    }
    
    #[test]
    fn benchmark_channel_insert_10k() {
        let db = setup_benchmark_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let profile_id = "bench_profile_1";
        
        cache.initialize_profile(profile_id).unwrap();
        
        let channels = generate_test_channels(10_000, profile_id);
        
        let start = Instant::now();
        let saved = cache.save_channels(profile_id, channels).unwrap();
        let duration = start.elapsed();
        
        println!("\n=== Benchmark: Insert 10k Channels ===");
        println!("Saved: {} channels", saved);
        println!("Duration: {:?}", duration);
        println!("Rate: {:.2} channels/sec", saved as f64 / duration.as_secs_f64());
        
        assert_eq!(saved, 10_000);
        assert!(duration.as_secs() < 10, "Insert should complete in under 10 seconds");
    }
    
    #[test]
    fn benchmark_channel_query_10k() {
        let db = setup_benchmark_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let profile_id = "bench_profile_2";
        
        cache.initialize_profile(profile_id).unwrap();
        
        // Insert test data
        let channels = generate_test_channels(10_000, profile_id);
        cache.save_channels(profile_id, channels).unwrap();
        
        // Benchmark query
        let start = Instant::now();
        let results = cache.get_channels(profile_id, None).unwrap();
        let duration = start.elapsed();
        
        println!("\n=== Benchmark: Query 10k Channels ===");
        println!("Retrieved: {} channels", results.len());
        println!("Duration: {:?}", duration);
        
        assert_eq!(results.len(), 10_000);
        assert!(duration.as_millis() < 100, "Query should complete in under 100ms");
    }
    
    #[test]
    fn benchmark_channel_search_10k() {
        let db = setup_benchmark_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let profile_id = "bench_profile_3";
        
        cache.initialize_profile(profile_id).unwrap();
        
        // Insert test data
        let channels = generate_test_channels(10_000, profile_id);
        cache.save_channels(profile_id, channels).unwrap();
        
        // Benchmark search
        let start = Instant::now();
        let results = cache.search_channels(profile_id, "Channel 5", None).unwrap();
        let duration = start.elapsed();
        
        println!("\n=== Benchmark: Search 10k Channels ===");
        println!("Found: {} channels", results.len());
        println!("Duration: {:?}", duration);
        
        assert!(results.len() > 0);
        assert!(duration.as_millis() < 100, "Search should complete in under 100ms");
    }
    
    #[test]
    fn benchmark_movie_insert_50k() {
        let db = setup_benchmark_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let profile_id = "bench_profile_4";
        
        cache.initialize_profile(profile_id).unwrap();
        
        let movies = generate_test_movies(50_000, profile_id);
        
        let start = Instant::now();
        let saved = cache.save_movies(profile_id, movies).unwrap();
        let duration = start.elapsed();
        
        println!("\n=== Benchmark: Insert 50k Movies ===");
        println!("Saved: {} movies", saved);
        println!("Duration: {:?}", duration);
        println!("Rate: {:.2} movies/sec", saved as f64 / duration.as_secs_f64());
        
        assert_eq!(saved, 50_000);
        assert!(duration.as_secs() < 30, "Insert should complete in under 30 seconds");
    }
    
    #[test]
    fn benchmark_movie_query_50k() {
        let db = setup_benchmark_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let profile_id = "bench_profile_5";
        
        cache.initialize_profile(profile_id).unwrap();
        
        // Insert test data
        let movies = generate_test_movies(50_000, profile_id);
        cache.save_movies(profile_id, movies).unwrap();
        
        // Benchmark query with pagination
        let filter = MovieFilter {
            limit: Some(100),
            offset: Some(0),
            ..Default::default()
        };
        
        let start = Instant::now();
        let results = cache.get_movies(profile_id, Some(filter), None, None).unwrap();
        let duration = start.elapsed();
        
        println!("\n=== Benchmark: Query 50k Movies (paginated) ===");
        println!("Retrieved: {} movies", results.len());
        println!("Duration: {:?}", duration);
        
        assert_eq!(results.len(), 100);
        assert!(duration.as_millis() < 100, "Paginated query should complete in under 100ms");
    }
    
    #[test]
    fn benchmark_movie_filter_50k() {
        let db = setup_benchmark_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let profile_id = "bench_profile_6";
        
        cache.initialize_profile(profile_id).unwrap();
        
        // Insert test data
        let movies = generate_test_movies(50_000, profile_id);
        cache.save_movies(profile_id, movies).unwrap();
        
        // Benchmark filtered query
        let filter = MovieFilter {
            genre: Some("Genre1".to_string()),
            min_rating: Some(7.0),
            limit: Some(100),
            ..Default::default()
        };
        
        let start = Instant::now();
        let results = cache.get_movies(profile_id, Some(filter), None, None).unwrap();
        let duration = start.elapsed();
        
        println!("\n=== Benchmark: Filter 50k Movies ===");
        println!("Found: {} movies", results.len());
        println!("Duration: {:?}", duration);
        
        assert!(duration.as_millis() < 150, "Filtered query should complete in under 150ms");
    }
    
    #[test]
    fn benchmark_series_insert_10k() {
        let db = setup_benchmark_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let profile_id = "bench_profile_7";
        
        cache.initialize_profile(profile_id).unwrap();
        
        let series = generate_test_series(10_000, profile_id);
        
        let start = Instant::now();
        let saved = cache.save_series(profile_id, series).unwrap();
        let duration = start.elapsed();
        
        println!("\n=== Benchmark: Insert 10k Series ===");
        println!("Saved: {} series", saved);
        println!("Duration: {:?}", duration);
        println!("Rate: {:.2} series/sec", saved as f64 / duration.as_secs_f64());
        
        assert_eq!(saved, 10_000);
        assert!(duration.as_secs() < 15, "Insert should complete in under 15 seconds");
    }
    
    #[test]
    fn benchmark_analyze_performance() {
        let db = setup_benchmark_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let perf = db_performance::DbPerformance::new(db.clone(), None);
        let profile_id = "bench_profile_8";
        
        cache.initialize_profile(profile_id).unwrap();
        
        // Insert significant data
        let channels = generate_test_channels(5_000, profile_id);
        cache.save_channels(profile_id, channels).unwrap();
        
        let movies = generate_test_movies(10_000, profile_id);
        cache.save_movies(profile_id, movies).unwrap();
        
        // Benchmark ANALYZE
        let start = Instant::now();
        perf.analyze_tables().unwrap();
        let duration = start.elapsed();
        
        println!("\n=== Benchmark: ANALYZE Tables ===");
        println!("Duration: {:?}", duration);
        
        assert!(duration.as_secs() < 5, "ANALYZE should complete in under 5 seconds");
    }
    
    #[test]
    fn benchmark_vacuum_performance() {
        let db = setup_benchmark_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let perf = db_performance::DbPerformance::new(db.clone(), None);
        let profile_id = "bench_profile_9";
        
        cache.initialize_profile(profile_id).unwrap();
        
        // Insert and delete data to create fragmentation
        let channels = generate_test_channels(5_000, profile_id);
        cache.save_channels(profile_id, channels).unwrap();
        cache.delete_channels(profile_id, None).unwrap();
        
        // Benchmark VACUUM
        let start = Instant::now();
        perf.vacuum().unwrap();
        let duration = start.elapsed();
        
        println!("\n=== Benchmark: VACUUM Database ===");
        println!("Duration: {:?}", duration);
        
        assert!(duration.as_secs() < 10, "VACUUM should complete in under 10 seconds");
    }
    
    #[test]
    fn benchmark_large_dataset_100k() {
        let db = setup_benchmark_db().unwrap();
        let cache = ContentCache::new(db.clone()).unwrap();
        let profile_id = "bench_profile_10";
        
        cache.initialize_profile(profile_id).unwrap();
        
        println!("\n=== Benchmark: Large Dataset (100k items) ===");
        
        // Insert 100k movies
        let movies = generate_test_movies(100_000, profile_id);
        let start = Instant::now();
        let saved = cache.save_movies(profile_id, movies).unwrap();
        let insert_duration = start.elapsed();
        
        println!("Insert 100k movies: {:?}", insert_duration);
        assert_eq!(saved, 100_000);
        
        // Query all
        let start = Instant::now();
        let all_results = cache.get_movies(profile_id, None, None, None).unwrap();
        let query_all_duration = start.elapsed();
        
        println!("Query all 100k movies: {:?}", query_all_duration);
        assert_eq!(all_results.len(), 100_000);
        
        // Paginated query
        let filter = MovieFilter {
            limit: Some(100),
            offset: Some(0),
            ..Default::default()
        };
        let start = Instant::now();
        let page_results = cache.get_movies(profile_id, Some(filter), None, None).unwrap();
        let query_page_duration = start.elapsed();
        
        println!("Query paginated (100 items): {:?}", query_page_duration);
        assert_eq!(page_results.len(), 100);
        assert!(query_page_duration.as_millis() < 100, "Paginated query should be fast");
        
        // Search
        let start = Instant::now();
        let search_results = cache.search_movies(profile_id, "Movie 5", None, None, None).unwrap();
        let search_duration = start.elapsed();
        
        println!("Search 100k movies: {:?}", search_duration);
        assert!(search_results.len() > 0);
        assert!(search_duration.as_millis() < 150, "Search should complete in under 150ms");
    }
}
