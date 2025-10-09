/// Performance benchmarks for QueryOptimizer
/// 
/// These benchmarks test query performance with different dataset sizes
/// to ensure we meet the performance targets:
/// - Query Response Time: < 100ms for 95% of queries
/// - Search Response Time: < 150ms for fuzzy search

#[cfg(test)]
mod benchmarks {
    use super::super::*;
    use rusqlite::{Connection, params};
    use std::time::Instant;
    
    /// Create a test database with a specified number of records
    fn create_benchmark_db(record_count: usize) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        
        // Create tables similar to actual schema
        conn.execute(
            "CREATE TABLE xtream_channels (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id TEXT NOT NULL,
                stream_id INTEGER NOT NULL,
                num INTEGER,
                name TEXT NOT NULL,
                stream_type TEXT,
                stream_icon TEXT,
                thumbnail TEXT,
                epg_channel_id TEXT,
                added TEXT,
                category_id TEXT,
                custom_sid TEXT,
                tv_archive INTEGER DEFAULT 0,
                direct_source TEXT,
                tv_archive_duration INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();
        
        conn.execute(
            "CREATE TABLE xtream_movies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id TEXT NOT NULL,
                stream_id INTEGER NOT NULL,
                num INTEGER,
                name TEXT NOT NULL,
                title TEXT,
                year TEXT,
                stream_type TEXT,
                stream_icon TEXT,
                rating REAL,
                rating_5based REAL,
                genre TEXT,
                added TEXT,
                episode_run_time INTEGER,
                category_id TEXT,
                container_extension TEXT,
                custom_sid TEXT,
                direct_source TEXT,
                release_date TEXT,
                cast TEXT,
                director TEXT,
                plot TEXT,
                youtube_trailer TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();
        
        // Create indexes
        conn.execute("CREATE INDEX idx_channels_profile ON xtream_channels(profile_id)", []).unwrap();
        conn.execute("CREATE INDEX idx_channels_category ON xtream_channels(category_id)", []).unwrap();
        conn.execute("CREATE INDEX idx_channels_name ON xtream_channels(name COLLATE NOCASE)", []).unwrap();
        
        conn.execute("CREATE INDEX idx_movies_profile ON xtream_movies(profile_id)", []).unwrap();
        conn.execute("CREATE INDEX idx_movies_category ON xtream_movies(category_id)", []).unwrap();
        conn.execute("CREATE INDEX idx_movies_name ON xtream_movies(name COLLATE NOCASE)", []).unwrap();
        conn.execute("CREATE INDEX idx_movies_rating ON xtream_movies(rating DESC)", []).unwrap();
        conn.execute("CREATE INDEX idx_movies_year ON xtream_movies(year DESC)", []).unwrap();
        
        // Insert test data
        println!("[BENCHMARK] Inserting {} records...", record_count);
        let insert_start = Instant::now();
        
        let tx = conn.unchecked_transaction().unwrap();
        
        for i in 0..record_count {
            let category_id = format!("cat_{}", i % 20);
            let year = 2000 + (i % 24);
            let rating = (i % 10) as f64 / 2.0;
            
            // Insert channels
            tx.execute(
                "INSERT INTO xtream_channels (profile_id, stream_id, name, category_id) VALUES (?, ?, ?, ?)",
                params!["test_profile", i as i64, format!("Channel {}", i), category_id],
            )
            .unwrap();
            
            // Insert movies
            tx.execute(
                "INSERT INTO xtream_movies (profile_id, stream_id, name, title, year, rating, rating_5based, genre, category_id) 
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    "test_profile",
                    i as i64,
                    format!("Movie {}", i),
                    format!("Movie Title {}", i),
                    year.to_string(),
                    rating,
                    rating,
                    if i % 3 == 0 { "Action" } else if i % 3 == 1 { "Comedy" } else { "Drama" },
                    category_id
                ],
            )
            .unwrap();
        }
        
        tx.commit().unwrap();
        
        println!("[BENCHMARK] Data insertion took {:?}", insert_start.elapsed());
        
        // Run ANALYZE for query optimization
        conn.execute("ANALYZE", []).unwrap();
        
        conn
    }
    
    /// Benchmark pagination performance
    #[test]
    fn benchmark_pagination() {
        let sizes = vec![1_000, 10_000, 50_000];
        
        for size in sizes {
            println!("\n=== Pagination Benchmark: {} records ===", size);
            let conn = create_benchmark_db(size);
            let optimizer = QueryOptimizer::new();
            
            let base_query = "SELECT id, name, category_id FROM xtream_channels WHERE profile_id = ? ORDER BY name";
            
            // Test different page sizes
            for page_size in vec![20, 50, 100] {
                let start = Instant::now();
                
                let results = optimizer
                    .paginated_query(
                        &conn,
                        base_query,
                        &[&"test_profile"],
                        0,
                        page_size,
                        |row| {
                            Ok((
                                row.get::<_, i64>(0)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, String>(2)?,
                            ))
                        },
                    )
                    .unwrap();
                
                let duration = start.elapsed();
                let duration_ms = duration.as_millis();
                
                println!(
                    "  Page size {}: {} results in {}ms {}",
                    page_size,
                    results.len(),
                    duration_ms,
                    if duration_ms < 100 { "✓" } else { "✗ SLOW" }
                );
                
                assert_eq!(results.len(), page_size);
            }
        }
    }
    
    /// Benchmark search performance
    #[test]
    fn benchmark_search() {
        let sizes = vec![1_000, 10_000, 50_000];
        
        for size in sizes {
            println!("\n=== Search Benchmark: {} records ===", size);
            let conn = create_benchmark_db(size);
            let optimizer = QueryOptimizer::new();
            
            // Test different search patterns
            let search_queries = vec![
                ("Movie 1", "exact prefix"),
                ("Movie", "common prefix"),
                ("123", "numeric"),
                ("xyz", "no results"),
            ];
            
            for (query, description) in search_queries {
                let start = Instant::now();
                
                let mut stmt = conn
                    .prepare(
                        "SELECT id, name FROM xtream_movies 
                         WHERE profile_id = ? AND LOWER(name) LIKE LOWER(?) 
                         ORDER BY name LIMIT 100"
                    )
                    .unwrap();
                
                let pattern = format!("%{}%", query);
                let results: Vec<(i64, String)> = stmt
                    .query_map(params!["test_profile", pattern], |row| {
                        Ok((row.get(0)?, row.get(1)?))
                    })
                    .unwrap()
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .unwrap();
                
                let duration = start.elapsed();
                let duration_ms = duration.as_millis();
                
                println!(
                    "  Search '{}' ({}): {} results in {}ms {}",
                    query,
                    description,
                    results.len(),
                    duration_ms,
                    if duration_ms < 150 { "✓" } else { "✗ SLOW" }
                );
            }
        }
    }
    
    /// Benchmark complex filtering
    #[test]
    fn benchmark_complex_filters() {
        let sizes = vec![1_000, 10_000, 50_000];
        
        for size in sizes {
            println!("\n=== Complex Filter Benchmark: {} records ===", size);
            let conn = create_benchmark_db(size);
            let optimizer = QueryOptimizer::new();
            
            // Test multi-field filtering
            let start = Instant::now();
            
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, year, rating, genre FROM xtream_movies 
                     WHERE profile_id = ? 
                     AND category_id = ? 
                     AND year >= ? 
                     AND rating >= ? 
                     AND genre = ?
                     ORDER BY rating DESC, name
                     LIMIT 50"
                )
                .unwrap();
            
            let results: Vec<(i64, String, String, f64, String)> = stmt
                .query_map(
                    params!["test_profile", "cat_5", "2010", 3.0, "Action"],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                        ))
                    },
                )
                .unwrap()
                .collect::<std::result::Result<Vec<_>, _>>()
                .unwrap();
            
            let duration = start.elapsed();
            let duration_ms = duration.as_millis();
            
            println!(
                "  Multi-field filter: {} results in {}ms {}",
                results.len(),
                duration_ms,
                if duration_ms < 100 { "✓" } else { "✗ SLOW" }
            );
        }
    }
    
    /// Benchmark sorting performance
    #[test]
    fn benchmark_sorting() {
        let sizes = vec![1_000, 10_000, 50_000];
        
        for size in sizes {
            println!("\n=== Sorting Benchmark: {} records ===", size);
            let conn = create_benchmark_db(size);
            
            let sort_tests = vec![
                ("name COLLATE NOCASE", "alphabetical"),
                ("rating DESC", "by rating"),
                ("year DESC, rating DESC", "multi-field"),
            ];
            
            for (order_by, description) in sort_tests {
                let start = Instant::now();
                
                let query = format!(
                    "SELECT id, name FROM xtream_movies WHERE profile_id = ? ORDER BY {} LIMIT 100",
                    order_by
                );
                
                let mut stmt = conn.prepare(&query).unwrap();
                let results: Vec<(i64, String)> = stmt
                    .query_map(params!["test_profile"], |row| {
                        Ok((row.get(0)?, row.get(1)?))
                    })
                    .unwrap()
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .unwrap();
                
                let duration = start.elapsed();
                let duration_ms = duration.as_millis();
                
                println!(
                    "  Sort by {} ({}): {} results in {}ms {}",
                    order_by,
                    description,
                    results.len(),
                    duration_ms,
                    if duration_ms < 100 { "✓" } else { "✗ SLOW" }
                );
            }
        }
    }
    
    /// Benchmark count queries
    #[test]
    fn benchmark_count_queries() {
        let sizes = vec![1_000, 10_000, 50_000];
        
        for size in sizes {
            println!("\n=== Count Query Benchmark: {} records ===", size);
            let conn = create_benchmark_db(size);
            
            // Test different count scenarios
            let count_tests = vec![
                ("SELECT COUNT(*) FROM xtream_movies WHERE profile_id = ?", "total count"),
                ("SELECT COUNT(*) FROM xtream_movies WHERE profile_id = ? AND category_id = ?", "filtered count"),
                ("SELECT COUNT(*) FROM xtream_movies WHERE profile_id = ? AND rating >= ?", "range count"),
            ];
            
            for (query, description) in count_tests {
                let start = Instant::now();
                
                let count: i64 = if query.contains("category_id") {
                    conn.query_row(query, params!["test_profile", "cat_5"], |row| row.get(0))
                        .unwrap()
                } else if query.contains("rating") {
                    conn.query_row(query, params!["test_profile", 3.0], |row| row.get(0))
                        .unwrap()
                } else {
                    conn.query_row(query, params!["test_profile"], |row| row.get(0))
                        .unwrap()
                };
                
                let duration = start.elapsed();
                let duration_ms = duration.as_millis();
                
                println!(
                    "  {} ({}): count={} in {}ms {}",
                    description,
                    query.split("WHERE").next().unwrap().trim(),
                    count,
                    duration_ms,
                    if duration_ms < 50 { "✓" } else { "✗ SLOW" }
                );
            }
        }
    }
    
    /// Benchmark ANALYZE performance
    #[test]
    fn benchmark_analyze() {
        let sizes = vec![1_000, 10_000, 50_000];
        
        for size in sizes {
            println!("\n=== ANALYZE Benchmark: {} records ===", size);
            let conn = create_benchmark_db(size);
            let optimizer = QueryOptimizer::new();
            
            let start = Instant::now();
            optimizer.analyze_tables(&conn).unwrap();
            let duration = start.elapsed();
            
            println!("  ANALYZE completed in {:?}", duration);
        }
    }
    
    /// Summary report of all benchmarks
    #[test]
    fn benchmark_summary() {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║         QueryOptimizer Performance Benchmark Summary       ║");
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║ Performance Targets:                                       ║");
        println!("║   • Query Response Time: < 100ms for 95% of queries       ║");
        println!("║   • Search Response Time: < 150ms for fuzzy search        ║");
        println!("║   • Count Queries: < 50ms                                 ║");
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║ Test Datasets:                                             ║");
        println!("║   • Small:  1,000 records                                 ║");
        println!("║   • Medium: 10,000 records                                ║");
        println!("║   • Large:  50,000 records                                ║");
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║ Run individual benchmarks with:                           ║");
        println!("║   cargo test --package xtauri --lib                       ║");
        println!("║     content_cache::query_optimizer_benchmarks::benchmarks ║");
        println!("║     -- --nocapture --test-threads=1                       ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
    }
}
