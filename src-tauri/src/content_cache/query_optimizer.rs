use crate::error::Result;
use rusqlite::{Connection, params, ToSql};
use std::time::Instant;

/// Query optimizer for efficient database operations
/// 
/// Provides pagination, complex filtering, and performance optimization
/// for content cache queries.
pub struct QueryOptimizer {
    /// Performance threshold in milliseconds for slow query warnings
    slow_query_threshold_ms: u128,
}

impl QueryOptimizer {
    /// Create a new QueryOptimizer with default settings
    pub fn new() -> Self {
        Self {
            slow_query_threshold_ms: 100, // Target < 100ms per requirements
        }
    }
    
    /// Create a QueryOptimizer with custom slow query threshold
    pub fn with_threshold(threshold_ms: u128) -> Self {
        Self {
            slow_query_threshold_ms: threshold_ms,
        }
    }
    
    /// Execute a paginated query
    /// 
    /// # Arguments
    /// * `conn` - Database connection
    /// * `base_query` - Base SQL query without LIMIT/OFFSET
    /// * `params` - Query parameters
    /// * `page` - Page number (0-indexed)
    /// * `page_size` - Number of items per page
    /// * `mapper` - Function to map rows to result type
    /// 
    /// # Returns
    /// Vector of results for the requested page
    pub fn paginated_query<T, F>(
        &self,
        conn: &Connection,
        base_query: &str,
        params: &[&dyn ToSql],
        page: usize,
        page_size: usize,
        mapper: F,
    ) -> Result<Vec<T>>
    where
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let start_time = Instant::now();
        
        // Calculate offset
        let offset = page * page_size;
        
        // Build paginated query
        let query = format!(
            "{} LIMIT {} OFFSET {}",
            base_query, page_size, offset
        );
        
        #[cfg(debug_assertions)]
        println!(
            "[DEBUG] Executing paginated query: page={}, page_size={}, offset={}",
            page, page_size, offset
        );
        
        let mut stmt = conn.prepare(&query)?;
        let results: Vec<T> = stmt
            .query_map(params, mapper)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        
        let duration = start_time.elapsed();
        self.log_query_performance("paginated_query", duration, results.len());
        
        Ok(results)
    }
    
    /// Execute a counted paginated query
    /// 
    /// Returns both the results and the total count for pagination UI
    /// 
    /// # Arguments
    /// * `conn` - Database connection
    /// * `base_query` - Base SQL query without LIMIT/OFFSET
    /// * `count_query` - Query to count total results
    /// * `params` - Query parameters
    /// * `page` - Page number (0-indexed)
    /// * `page_size` - Number of items per page
    /// * `mapper` - Function to map rows to result type
    /// 
    /// # Returns
    /// Tuple of (results, total_count)
    pub fn paginated_query_with_count<T, F>(
        &self,
        conn: &Connection,
        base_query: &str,
        count_query: &str,
        params: &[&dyn ToSql],
        page: usize,
        page_size: usize,
        mapper: F,
    ) -> Result<(Vec<T>, usize)>
    where
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let start_time = Instant::now();
        
        // Get total count
        let total_count: i64 = conn.query_row(count_query, params, |row| row.get(0))?;
        
        // Get paginated results
        let results = self.paginated_query(conn, base_query, params, page, page_size, mapper)?;
        
        let duration = start_time.elapsed();
        self.log_query_performance(
            "paginated_query_with_count",
            duration,
            results.len(),
        );
        
        Ok((results, total_count as usize))
    }
    
    /// Build a dynamic WHERE clause from filters
    /// 
    /// # Arguments
    /// * `filters` - Vector of (column, operator, value) tuples
    /// 
    /// # Returns
    /// Tuple of (where_clause, params)
    pub fn build_where_clause(
        &self,
        filters: Vec<Filter>,
    ) -> (String, Vec<Box<dyn ToSql>>) {
        if filters.is_empty() {
            return (String::new(), Vec::new());
        }
        
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn ToSql>> = Vec::new();
        
        for filter in filters {
            match filter {
                Filter::Equals(column, value) => {
                    conditions.push(format!("{} = ?", column));
                    params.push(value);
                }
                Filter::NotEquals(column, value) => {
                    conditions.push(format!("{} != ?", column));
                    params.push(value);
                }
                Filter::GreaterThan(column, value) => {
                    conditions.push(format!("{} > ?", column));
                    params.push(value);
                }
                Filter::GreaterThanOrEqual(column, value) => {
                    conditions.push(format!("{} >= ?", column));
                    params.push(value);
                }
                Filter::LessThan(column, value) => {
                    conditions.push(format!("{} < ?", column));
                    params.push(value);
                }
                Filter::LessThanOrEqual(column, value) => {
                    conditions.push(format!("{} <= ?", column));
                    params.push(value);
                }
                Filter::Like(column, pattern) => {
                    conditions.push(format!("{} LIKE ?", column));
                    params.push(Box::new(pattern));
                }
                Filter::In(column, values) => {
                    let placeholders = values.iter()
                        .map(|_| "?")
                        .collect::<Vec<_>>()
                        .join(", ");
                    conditions.push(format!("{} IN ({})", column, placeholders));
                    for value in values {
                        params.push(value);
                    }
                }
                Filter::IsNull(column) => {
                    conditions.push(format!("{} IS NULL", column));
                }
                Filter::IsNotNull(column) => {
                    conditions.push(format!("{} IS NOT NULL", column));
                }
                Filter::Between(column, min, max) => {
                    conditions.push(format!("{} BETWEEN ? AND ?", column));
                    params.push(min);
                    params.push(max);
                }
            }
        }
        
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };
        
        (where_clause, params)
    }
    
    /// Build an ORDER BY clause
    /// 
    /// # Arguments
    /// * `sort_by` - Vector of (column, direction) tuples
    /// 
    /// # Returns
    /// ORDER BY clause string
    pub fn build_order_by(&self, sort_by: Vec<SortColumn>) -> String {
        if sort_by.is_empty() {
            return String::new();
        }
        
        let clauses: Vec<String> = sort_by
            .iter()
            .map(|sort| {
                let direction = match sort.direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                
                let collation = if sort.case_insensitive {
                    " COLLATE NOCASE"
                } else {
                    ""
                };
                
                format!("{}{} {}", sort.column, collation, direction)
            })
            .collect();
        
        format!("ORDER BY {}", clauses.join(", "))
    }
    
    /// Perform full-text search using SQLite FTS
    /// 
    /// # Arguments
    /// * `conn` - Database connection
    /// * `fts_table` - Name of the FTS virtual table
    /// * `query` - Search query
    /// * `limit` - Maximum number of results
    /// 
    /// # Returns
    /// Vector of matching row IDs
    pub fn fts_search(
        &self,
        conn: &Connection,
        fts_table: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<i64>> {
        let start_time = Instant::now();
        
        let sql = format!(
            "SELECT rowid FROM {} WHERE {} MATCH ? ORDER BY rank LIMIT ?",
            fts_table, fts_table
        );
        
        let mut stmt = conn.prepare(&sql)?;
        let ids: Vec<i64> = stmt
            .query_map(params![query, limit as i64], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        
        let duration = start_time.elapsed();
        self.log_query_performance("fts_search", duration, ids.len());
        
        Ok(ids)
    }
    
    /// Perform fuzzy search without FTS
    /// 
    /// Uses LIKE with wildcards for fuzzy matching
    /// 
    /// # Arguments
    /// * `conn` - Database connection
    /// * `table` - Table name
    /// * `search_columns` - Columns to search in
    /// * `query` - Search query
    /// * `additional_where` - Additional WHERE conditions
    /// * `limit` - Maximum number of results
    /// 
    /// # Returns
    /// Vector of matching row IDs with relevance scores
    pub fn fuzzy_search(
        &self,
        conn: &Connection,
        table: &str,
        search_columns: &[&str],
        query: &str,
        additional_where: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(i64, i32)>> {
        let start_time = Instant::now();
        
        // Sanitize query for LIKE
        let sanitized = self.sanitize_like_pattern(query);
        let pattern = format!("%{}%", sanitized);
        
        // Build relevance scoring
        let relevance_cases: Vec<String> = search_columns
            .iter()
            .enumerate()
            .flat_map(|(idx, col)| {
                vec![
                    format!("WHEN LOWER({}) = LOWER(?) THEN {}", col, idx * 3),
                    format!("WHEN LOWER({}) LIKE LOWER(?) || '%' THEN {}", col, idx * 3 + 1),
                    format!("WHEN LOWER({}) LIKE '%' || LOWER(?) || '%' THEN {}", col, idx * 3 + 2),
                ]
            })
            .collect();
        
        let relevance_case = format!(
            "CASE {} ELSE 999 END",
            relevance_cases.join(" ")
        );
        
        // Build search conditions
        let search_conditions: Vec<String> = search_columns
            .iter()
            .map(|col| format!("LOWER({}) LIKE LOWER(?)", col))
            .collect();
        
        let search_clause = search_conditions.join(" OR ");
        
        let where_clause = if let Some(additional) = additional_where {
            format!("WHERE ({}) AND ({})", search_clause, additional)
        } else {
            format!("WHERE {}", search_clause)
        };
        
        let sql = format!(
            "SELECT id, {} as relevance FROM {} {} ORDER BY relevance, id LIMIT ?",
            relevance_case, table, where_clause
        );
        
        // Build params: query for each relevance case + pattern for each search column + limit
        let mut params: Vec<Box<dyn ToSql>> = Vec::new();
        
        // Add query for relevance scoring (3 times per column)
        for _ in 0..search_columns.len() * 3 {
            params.push(Box::new(query.to_string()));
        }
        
        // Add pattern for search conditions
        for _ in 0..search_columns.len() {
            params.push(Box::new(pattern.clone()));
        }
        
        // Add limit
        params.push(Box::new(limit as i64));
        
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let mut stmt = conn.prepare(&sql)?;
        let results: Vec<(i64, i32)> = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        
        let duration = start_time.elapsed();
        self.log_query_performance("fuzzy_search", duration, results.len());
        
        Ok(results)
    }
    
    /// Analyze database tables for query optimization
    /// 
    /// Runs ANALYZE to update query planner statistics
    pub fn analyze_tables(&self, conn: &Connection) -> Result<()> {
        let start_time = Instant::now();
        
        conn.execute("ANALYZE", [])?;
        
        let duration = start_time.elapsed();
        println!("[INFO] Database ANALYZE completed in {:?}", duration);
        
        Ok(())
    }
    
    /// Vacuum database to reclaim space and optimize
    /// 
    /// Note: This can be an expensive operation
    pub fn vacuum_database(&self, conn: &Connection) -> Result<()> {
        let start_time = Instant::now();
        
        println!("[INFO] Starting database VACUUM...");
        conn.execute("VACUUM", [])?;
        
        let duration = start_time.elapsed();
        println!("[INFO] Database VACUUM completed in {:?}", duration);
        
        Ok(())
    }
    
    /// Get query execution plan
    /// 
    /// Useful for debugging slow queries
    pub fn explain_query(
        &self,
        conn: &Connection,
        query: &str,
        params: &[&dyn ToSql],
    ) -> Result<Vec<String>> {
        let explain_query = format!("EXPLAIN QUERY PLAN {}", query);
        
        let mut stmt = conn.prepare(&explain_query)?;
        let plans: Vec<String> = stmt
            .query_map(params, |row| {
                let detail: String = row.get(3)?;
                Ok(detail)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(plans)
    }
    
    /// Sanitize text for SQL LIKE queries
    fn sanitize_like_pattern(&self, pattern: &str) -> String {
        pattern
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    }
    
    /// Log query performance
    fn log_query_performance(&self, operation: &str, duration: std::time::Duration, result_count: usize) {
        let duration_ms = duration.as_millis();
        
        #[cfg(debug_assertions)]
        println!(
            "[DEBUG] Query completed: {} - {} results in {}ms",
            operation, result_count, duration_ms
        );
        
        if duration_ms > self.slow_query_threshold_ms {
            eprintln!(
                "[WARN] Slow query detected: {} took {}ms (threshold: {}ms)",
                operation, duration_ms, self.slow_query_threshold_ms
            );
        }
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter types for building WHERE clauses
/// 
/// Note: Cannot derive Debug/Clone due to Box<dyn ToSql>
pub enum Filter {
    Equals(String, Box<dyn ToSql>),
    NotEquals(String, Box<dyn ToSql>),
    GreaterThan(String, Box<dyn ToSql>),
    GreaterThanOrEqual(String, Box<dyn ToSql>),
    LessThan(String, Box<dyn ToSql>),
    LessThanOrEqual(String, Box<dyn ToSql>),
    Like(String, String),
    In(String, Vec<Box<dyn ToSql>>),
    IsNull(String),
    IsNotNull(String),
    Between(String, Box<dyn ToSql>, Box<dyn ToSql>),
}

/// Sort column specification
#[derive(Debug, Clone)]
pub struct SortColumn {
    pub column: String,
    pub direction: SortDirection,
    pub case_insensitive: bool,
}

impl SortColumn {
    pub fn new(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            direction: SortDirection::Asc,
            case_insensitive: false,
        }
    }
    
    pub fn desc(mut self) -> Self {
        self.direction = SortDirection::Desc;
        self
    }
    
    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;
        self
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Pagination helper
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: usize,
    pub page_size: usize,
}

impl Pagination {
    pub fn new(page: usize, page_size: usize) -> Self {
        Self { page, page_size }
    }
    
    pub fn offset(&self) -> usize {
        self.page * self.page_size
    }
    
    pub fn limit(&self) -> usize {
        self.page_size
    }
    
    pub fn total_pages(&self, total_items: usize) -> usize {
        (total_items + self.page_size - 1) / self.page_size
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    
    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        
        conn.execute(
            "CREATE TABLE test_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                category TEXT,
                value INTEGER,
                rating REAL
            )",
            [],
        )
        .unwrap();
        
        // Insert test data
        for i in 1..=100 {
            conn.execute(
                "INSERT INTO test_items (name, category, value, rating) VALUES (?, ?, ?, ?)",
                params![
                    format!("Item {}", i),
                    if i % 3 == 0 { "A" } else if i % 3 == 1 { "B" } else { "C" },
                    i,
                    (i as f64) / 10.0
                ],
            )
            .unwrap();
        }
        
        conn
    }
    
    #[test]
    fn test_paginated_query() {
        let conn = create_test_db();
        let optimizer = QueryOptimizer::new();
        
        let base_query = "SELECT id, name, value FROM test_items ORDER BY id";
        
        let results = optimizer
            .paginated_query(
                &conn,
                base_query,
                &[],
                0,
                10,
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i32>(2)?,
                    ))
                },
            )
            .unwrap();
        
        assert_eq!(results.len(), 10);
        assert_eq!(results[0].0, 1);
        assert_eq!(results[9].0, 10);
        
        // Test second page
        let results_page2 = optimizer
            .paginated_query(
                &conn,
                base_query,
                &[],
                1,
                10,
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i32>(2)?,
                    ))
                },
            )
            .unwrap();
        
        assert_eq!(results_page2.len(), 10);
        assert_eq!(results_page2[0].0, 11);
    }
    
    #[test]
    fn test_paginated_query_with_count() {
        let conn = create_test_db();
        let optimizer = QueryOptimizer::new();
        
        let base_query = "SELECT id, name FROM test_items WHERE category = ?";
        let count_query = "SELECT COUNT(*) FROM test_items WHERE category = ?";
        
        let (results, total) = optimizer
            .paginated_query_with_count(
                &conn,
                base_query,
                count_query,
                &[&"A"],
                0,
                10,
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .unwrap();
        
        assert_eq!(total, 33); // Items where i % 3 == 0
        assert!(results.len() <= 10);
    }
    
    #[test]
    fn test_build_where_clause() {
        let optimizer = QueryOptimizer::new();
        
        let filters = vec![
            Filter::Equals("category".to_string(), Box::new("A".to_string())),
            Filter::GreaterThan("value".to_string(), Box::new(50)),
        ];
        
        let (where_clause, params) = optimizer.build_where_clause(filters);
        
        assert!(where_clause.contains("WHERE"));
        assert!(where_clause.contains("category = ?"));
        assert!(where_clause.contains("value > ?"));
        assert!(where_clause.contains("AND"));
        assert_eq!(params.len(), 2);
    }
    
    #[test]
    fn test_build_order_by() {
        let optimizer = QueryOptimizer::new();
        
        let sort = vec![
            SortColumn::new("name").case_insensitive(),
            SortColumn::new("value").desc(),
        ];
        
        let order_by = optimizer.build_order_by(sort);
        
        assert!(order_by.contains("ORDER BY"));
        assert!(order_by.contains("name COLLATE NOCASE ASC"));
        assert!(order_by.contains("value DESC"));
    }
    
    #[test]
    fn test_analyze_tables() {
        let conn = create_test_db();
        let optimizer = QueryOptimizer::new();
        
        let result = optimizer.analyze_tables(&conn);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_explain_query() {
        let conn = create_test_db();
        let optimizer = QueryOptimizer::new();
        
        let query = "SELECT * FROM test_items WHERE category = ?";
        let plans = optimizer.explain_query(&conn, query, &[&"A"]).unwrap();
        
        assert!(!plans.is_empty());
    }
    
    #[test]
    fn test_pagination_helper() {
        let pagination = Pagination::new(2, 20);
        
        assert_eq!(pagination.offset(), 40);
        assert_eq!(pagination.limit(), 20);
        assert_eq!(pagination.total_pages(100), 5);
        assert_eq!(pagination.total_pages(95), 5);
        assert_eq!(pagination.total_pages(101), 6);
    }
    
    #[test]
    fn test_sort_column_builder() {
        let sort = SortColumn::new("name").desc().case_insensitive();
        
        assert_eq!(sort.column, "name");
        assert_eq!(sort.direction, SortDirection::Desc);
        assert!(sort.case_insensitive);
    }
}
