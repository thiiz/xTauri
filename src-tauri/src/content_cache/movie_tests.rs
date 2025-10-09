use super::*;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Create a test database with required dependencies
fn create_test_db() -> Arc<Mutex<Connection>> {
    let conn = Connection::open_in_memory().unwrap();
    
    // Create xtream_profiles table (dependency for foreign keys)
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

/// Insert a test profile into the database
fn insert_test_profile(db: &Arc<Mutex<Connection>>, profile_id: &str) {
    let conn = db.lock().unwrap();
    let profile_name = format!("Test Profile {}", profile_id);
    conn.execute(
        "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
         VALUES (?1, ?2, 'http://test.com', 'testuser', X'00')",
        rusqlite::params![profile_id, profile_name],
    )
    .unwrap();
}

/// Create sample movie data for testing
fn create_sample_movie(stream_id: i64, name: &str, category_id: Option<&str>) -> XtreamMovie {
    XtreamMovie {
        stream_id,
        num: Some(stream_id),
        name: name.to_string(),
        title: Some(name.to_string()),
        year: Some("2023".to_string()),
        stream_type: Some("movie".to_string()),
        stream_icon: Some("http://example.com/icon.jpg".to_string()),
        rating: Some(7.5),
        rating_5based: Some(3.75),
        genre: Some("Action".to_string()),
        added: Some("2023-01-01".to_string()),
        episode_run_time: Some(120),
        category_id: category_id.map(|s| s.to_string()),
        container_extension: Some("mp4".to_string()),
        custom_sid: None,
        direct_source: None,
        release_date: Some("2023-01-01".to_string()),
        cast: Some("Actor 1, Actor 2".to_string()),
        director: Some("Director Name".to_string()),
        plot: Some("An exciting action movie".to_string()),
        youtube_trailer: None,
    }
}

// ==================== Movie CRUD Tests ====================

#[test]
fn test_save_movies_empty_list() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let result = cache.save_movies("test-profile", vec![]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_save_movies_single() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movie = create_sample_movie(1, "Test Movie", Some("1"));
    let result = cache.save_movies("test-profile", vec![movie.clone()]);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
    
    // Verify movie was saved
    let movies = cache.get_movies("test-profile", None, None, None).unwrap();
    assert_eq!(movies.len(), 1);
    assert_eq!(movies[0].stream_id, 1);
    assert_eq!(movies[0].name, "Test Movie");
}

#[test]
fn test_save_movies_batch() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movies = vec![
        create_sample_movie(1, "Movie 1", Some("1")),
        create_sample_movie(2, "Movie 2", Some("1")),
        create_sample_movie(3, "Movie 3", Some("2")),
    ];
    
    let result = cache.save_movies("test-profile", movies);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);
    
    let saved_movies = cache.get_movies("test-profile", None, None, None).unwrap();
    assert_eq!(saved_movies.len(), 3);
}

#[test]
fn test_save_movies_upsert() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Save initial movie
    let movie1 = create_sample_movie(1, "Original Title", Some("1"));
    cache.save_movies("test-profile", vec![movie1]).unwrap();
    
    // Update the same movie with new data
    let mut movie2 = create_sample_movie(1, "Updated Title", Some("1"));
    movie2.rating = Some(9.0);
    cache.save_movies("test-profile", vec![movie2]).unwrap();
    
    // Verify only one movie exists with updated data
    let movies = cache.get_movies("test-profile", None, None, None).unwrap();
    assert_eq!(movies.len(), 1);
    assert_eq!(movies[0].name, "Updated Title");
    assert_eq!(movies[0].rating, Some(9.0));
}

#[test]
fn test_save_movies_updates_sync_metadata() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db.clone()).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movies = vec![
        create_sample_movie(1, "Movie 1", Some("1")),
        create_sample_movie(2, "Movie 2", Some("1")),
    ];
    
    cache.save_movies("test-profile", movies).unwrap();
    
    // Verify sync metadata was updated
    let conn = db.lock().unwrap();
    let (count, last_sync): (i32, Option<String>) = conn
        .query_row(
            "SELECT movies_count, last_sync_movies FROM xtream_content_sync WHERE profile_id = ?1",
            ["test-profile"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    
    assert_eq!(count, 2);
    assert!(last_sync.is_some());
}

#[test]
fn test_get_movies_empty() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movies = cache.get_movies("test-profile", None, None, None).unwrap();
    assert_eq!(movies.len(), 0);
}

#[test]
fn test_get_movies_all() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movies = vec![
        create_sample_movie(1, "Movie A", Some("1")),
        create_sample_movie(2, "Movie B", Some("1")),
        create_sample_movie(3, "Movie C", Some("2")),
    ];
    
    cache.save_movies("test-profile", movies).unwrap();
    
    let retrieved = cache.get_movies("test-profile", None, None, None).unwrap();
    assert_eq!(retrieved.len(), 3);
}

#[test]
fn test_get_movies_with_category_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movies = vec![
        create_sample_movie(1, "Action Movie 1", Some("action")),
        create_sample_movie(2, "Action Movie 2", Some("action")),
        create_sample_movie(3, "Comedy Movie", Some("comedy")),
    ];
    
    cache.save_movies("test-profile", movies).unwrap();
    
    let filter = MovieFilter {
        category_id: Some("action".to_string()),
        ..Default::default()
    };
    
    let filtered = cache.get_movies("test-profile", Some(filter), None, None).unwrap();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|m| m.category_id == Some("action".to_string())));
}

#[test]
fn test_get_movies_with_genre_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let mut movie1 = create_sample_movie(1, "Action Movie", Some("1"));
    movie1.genre = Some("Action".to_string());
    
    let mut movie2 = create_sample_movie(2, "Drama Movie", Some("1"));
    movie2.genre = Some("Drama".to_string());
    
    cache.save_movies("test-profile", vec![movie1, movie2]).unwrap();
    
    let filter = MovieFilter {
        genre: Some("Action".to_string()),
        ..Default::default()
    };
    
    let filtered = cache.get_movies("test-profile", Some(filter), None, None).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "Action Movie");
}

#[test]
fn test_get_movies_with_rating_filter() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let mut movie1 = create_sample_movie(1, "Great Movie", Some("1"));
    movie1.rating = Some(9.0);
    
    let mut movie2 = create_sample_movie(2, "Good Movie", Some("1"));
    movie2.rating = Some(7.5);
    
    let mut movie3 = create_sample_movie(3, "Average Movie", Some("1"));
    movie3.rating = Some(5.0);
    
    cache.save_movies("test-profile", vec![movie1, movie2, movie3]).unwrap();
    
    let filter = MovieFilter {
        min_rating: Some(7.0),
        ..Default::default()
    };
    
    let filtered = cache.get_movies("test-profile", Some(filter), None, None).unwrap();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|m| m.rating.unwrap_or(0.0) >= 7.0));
}

#[test]
fn test_get_movies_sorted_by_rating() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let mut movie1 = create_sample_movie(1, "Movie 1", Some("1"));
    movie1.rating = Some(5.0);
    
    let mut movie2 = create_sample_movie(2, "Movie 2", Some("1"));
    movie2.rating = Some(9.0);
    
    let mut movie3 = create_sample_movie(3, "Movie 3", Some("1"));
    movie3.rating = Some(7.0);
    
    cache.save_movies("test-profile", vec![movie1, movie2, movie3]).unwrap();
    
    let sorted = cache.get_movies(
        "test-profile",
        None,
        Some(MovieSortBy::Rating),
        Some(SortDirection::Desc),
    ).unwrap();
    
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0].rating, Some(9.0));
    assert_eq!(sorted[1].rating, Some(7.0));
    assert_eq!(sorted[2].rating, Some(5.0));
}

#[test]
fn test_delete_movies_all() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movies = vec![
        create_sample_movie(1, "Movie 1", Some("1")),
        create_sample_movie(2, "Movie 2", Some("1")),
    ];
    
    cache.save_movies("test-profile", movies).unwrap();
    
    let deleted = cache.delete_movies("test-profile", None).unwrap();
    assert_eq!(deleted, 2);
    
    let remaining = cache.get_movies("test-profile", None, None, None).unwrap();
    assert_eq!(remaining.len(), 0);
}

#[test]
fn test_delete_movies_specific() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movies = vec![
        create_sample_movie(1, "Movie 1", Some("1")),
        create_sample_movie(2, "Movie 2", Some("1")),
        create_sample_movie(3, "Movie 3", Some("1")),
    ];
    
    cache.save_movies("test-profile", movies).unwrap();
    
    let deleted = cache.delete_movies("test-profile", Some(vec![1, 3])).unwrap();
    assert_eq!(deleted, 2);
    
    let remaining = cache.get_movies("test-profile", None, None, None).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].stream_id, 2);
}

// ==================== Movie Search Tests ====================

#[test]
fn test_search_movies_by_name() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movies = vec![
        create_sample_movie(1, "The Matrix", Some("1")),
        create_sample_movie(2, "Matrix Reloaded", Some("1")),
        create_sample_movie(3, "Inception", Some("1")),
    ];
    
    cache.save_movies("test-profile", movies).unwrap();
    
    let results = cache.search_movies("test-profile", "Matrix", None, None, None).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|m| m.name == "The Matrix"));
    assert!(results.iter().any(|m| m.name == "Matrix Reloaded"));
}

#[test]
fn test_search_movies_by_plot() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let mut movie1 = create_sample_movie(1, "Movie 1", Some("1"));
    movie1.plot = Some("A thrilling adventure in space".to_string());
    
    let mut movie2 = create_sample_movie(2, "Movie 2", Some("1"));
    movie2.plot = Some("A romantic comedy".to_string());
    
    cache.save_movies("test-profile", vec![movie1, movie2]).unwrap();
    
    let results = cache.search_movies("test-profile", "space", None, None, None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Movie 1");
}

#[test]
fn test_search_movies_case_insensitive() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movie = create_sample_movie(1, "The Dark Knight", Some("1"));
    cache.save_movies("test-profile", vec![movie]).unwrap();
    
    let results1 = cache.search_movies("test-profile", "dark", None, None, None).unwrap();
    let results2 = cache.search_movies("test-profile", "DARK", None, None, None).unwrap();
    let results3 = cache.search_movies("test-profile", "DaRk", None, None, None).unwrap();
    
    assert_eq!(results1.len(), 1);
    assert_eq!(results2.len(), 1);
    assert_eq!(results3.len(), 1);
}

#[test]
fn test_search_movies_with_filters() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let mut movie1 = create_sample_movie(1, "Action Movie 1", Some("action"));
    movie1.genre = Some("Action".to_string());
    movie1.rating = Some(8.0);
    
    let mut movie2 = create_sample_movie(2, "Action Movie 2", Some("action"));
    movie2.genre = Some("Action".to_string());
    movie2.rating = Some(6.0);
    
    let mut movie3 = create_sample_movie(3, "Drama Movie", Some("drama"));
    movie3.genre = Some("Drama".to_string());
    movie3.rating = Some(9.0);
    
    cache.save_movies("test-profile", vec![movie1, movie2, movie3]).unwrap();
    
    let filter = MovieFilter {
        category_id: Some("action".to_string()),
        min_rating: Some(7.0),
        ..Default::default()
    };
    
    let results = cache.search_movies("test-profile", "Movie", Some(filter), None, None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Action Movie 1");
}

#[test]
fn test_count_movies() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    let movies = vec![
        create_sample_movie(1, "Movie 1", Some("1")),
        create_sample_movie(2, "Movie 2", Some("1")),
        create_sample_movie(3, "Movie 3", Some("2")),
    ];
    
    cache.save_movies("test-profile", movies).unwrap();
    
    let total_count = cache.count_movies("test-profile", None).unwrap();
    assert_eq!(total_count, 3);
    
    let filter = MovieFilter {
        category_id: Some("1".to_string()),
        ..Default::default()
    };
    
    let filtered_count = cache.count_movies("test-profile", Some(filter)).unwrap();
    assert_eq!(filtered_count, 2);
}

// ==================== Movie Performance Tests ====================

#[test]
fn test_search_movies_performance() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert a large number of movies
    let mut movies = Vec::new();
    for i in 1..=1000 {
        let mut movie = create_sample_movie(i, &format!("Movie {}", i), Some("1"));
        movie.plot = Some(format!("This is an exciting movie number {}", i));
        movies.push(movie);
    }
    
    cache.save_movies("test-profile", movies).unwrap();
    
    // Measure search performance
    let start = std::time::Instant::now();
    let results = cache.search_movies("test-profile", "exciting", None, None, None).unwrap();
    let duration = start.elapsed();
    
    println!("Search took {:?} for {} results", duration, results.len());
    
    // Target: < 100ms for search
    assert!(duration.as_millis() < 100, "Search took too long: {:?}", duration);
    assert_eq!(results.len(), 1000); // All movies should match
}

#[test]
fn test_get_movies_with_filters_performance() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert movies with various attributes
    let mut movies = Vec::new();
    for i in 1..=1000 {
        let mut movie = create_sample_movie(i, &format!("Movie {}", i), Some("action"));
        movie.genre = Some("Action".to_string());
        movie.year = Some(format!("{}", 2020 + (i % 5)));
        movie.rating = Some(5.0 + (i % 5) as f64);
        movies.push(movie);
    }
    
    cache.save_movies("test-profile", movies).unwrap();
    
    // Measure filter performance
    let filter = MovieFilter {
        category_id: Some("action".to_string()),
        genre: Some("Action".to_string()),
        min_rating: Some(7.0),
        ..Default::default()
    };
    
    let start = std::time::Instant::now();
    let results = cache.get_movies("test-profile", Some(filter), None, None).unwrap();
    let duration = start.elapsed();
    
    println!("Filtered query took {:?} for {} results", duration, results.len());
    
    // Target: < 100ms for filtered query
    assert!(duration.as_millis() < 100, "Filtered query took too long: {:?}", duration);
    assert!(results.len() > 0);
}

#[test]
fn test_sort_movies_performance() {
    let db = create_test_db();
    insert_test_profile(&db, "test-profile");
    let cache = ContentCache::new(db).unwrap();
    cache.initialize_profile("test-profile").unwrap();
    
    // Insert movies
    let mut movies = Vec::new();
    for i in 1..=1000 {
        let mut movie = create_sample_movie(i, &format!("Movie {}", i), Some("1"));
        movie.rating = Some((i % 10) as f64);
        movies.push(movie);
    }
    
    cache.save_movies("test-profile", movies).unwrap();
    
    // Measure sort performance
    let start = std::time::Instant::now();
    let results = cache.get_movies(
        "test-profile",
        None,
        Some(MovieSortBy::Rating),
        Some(SortDirection::Desc),
    ).unwrap();
    let duration = start.elapsed();
    
    println!("Sorted query took {:?} for {} results", duration, results.len());
    
    // Target: < 100ms for sorted query
    assert!(duration.as_millis() < 100, "Sorted query took too long: {:?}", duration);
    assert_eq!(results.len(), 1000);
    
    // Verify sorting is correct
    for i in 0..results.len()-1 {
        assert!(results[i].rating >= results[i+1].rating, "Results not properly sorted");
    }
}
