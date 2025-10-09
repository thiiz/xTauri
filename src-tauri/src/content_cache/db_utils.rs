use crate::error::{Result, XTauriError};
use rusqlite::{Connection, Transaction};
use std::time::Instant;

/// Transaction helper that provides automatic rollback on error
/// 
/// This wrapper ensures transactions are properly committed or rolled back,
/// and provides logging for transaction operations.
pub struct TransactionHelper<'conn> {
    tx: Option<Transaction<'conn>>,
    operation_name: String,
    start_time: Instant,
}

impl<'conn> TransactionHelper<'conn> {
    /// Create a new transaction helper
    /// 
    /// # Arguments
    /// * `conn` - Database connection
    /// * `operation_name` - Name of the operation for logging
    /// 
    /// # Returns
    /// A new TransactionHelper with an active transaction
    pub fn new(conn: &'conn mut Connection, operation_name: &str) -> Result<Self> {
        let tx = conn.unchecked_transaction()?;
        
        #[cfg(debug_assertions)]
        println!("[DEBUG] Starting transaction: {}", operation_name);
        
        Ok(Self {
            tx: Some(tx),
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
        })
    }
    
    /// Get a reference to the underlying transaction
    pub fn transaction(&self) -> Result<&Transaction<'conn>> {
        self.tx.as_ref().ok_or_else(|| {
            XTauriError::content_cache("Transaction already consumed".to_string())
        })
    }
    
    /// Commit the transaction
    /// 
    /// # Returns
    /// Ok(()) if commit succeeds, error otherwise
    pub fn commit(mut self) -> Result<()> {
        let tx = self.tx.take().ok_or_else(|| {
            XTauriError::content_cache("Transaction already consumed".to_string())
        })?;
        
        tx.commit()?;
        
        let duration = self.start_time.elapsed();
        #[cfg(debug_assertions)]
        println!(
            "[INFO] Transaction committed: {} (took {:?})",
            self.operation_name,
            duration
        );
        
        Ok(())
    }
    
    /// Rollback the transaction explicitly
    /// 
    /// Note: Rollback also happens automatically on drop if not committed
    pub fn rollback(mut self) -> Result<()> {
        let tx = self.tx.take().ok_or_else(|| {
            XTauriError::content_cache("Transaction already consumed".to_string())
        })?;
        
        tx.rollback()?;
        
        eprintln!("[WARN] Transaction rolled back: {}", self.operation_name);
        
        Ok(())
    }
}

impl<'conn> Drop for TransactionHelper<'conn> {
    fn drop(&mut self) {
        if self.tx.is_some() {
            eprintln!(
                "[WARN] Transaction dropped without explicit commit/rollback: {}",
                self.operation_name
            );
        }
    }
}

/// Batch insert helper for efficient bulk inserts
/// 
/// This function performs batch inserts using a transaction for atomicity
/// and improved performance.
pub fn batch_insert<T, F>(
    conn: &mut Connection,
    table: &str,
    items: &[T],
    insert_fn: F,
) -> Result<usize>
where
    F: Fn(&Transaction, &T) -> Result<()>,
{
    if items.is_empty() {
        return Ok(0);
    }
    
    let start_time = Instant::now();
    let operation_name = format!("batch_insert_{}", table);
    
    #[cfg(debug_assertions)]
    println!("[DEBUG] Starting batch insert: {} items into {}", items.len(), table);
    
    let helper = TransactionHelper::new(conn, &operation_name)?;
    let tx = helper.transaction()?;
    
    let mut inserted = 0;
    let mut errors = Vec::new();
    
    for (idx, item) in items.iter().enumerate() {
        match insert_fn(tx, item) {
            Ok(_) => inserted += 1,
            Err(e) => {
                eprintln!("[WARN] Failed to insert item {} in {}: {}", idx, table, e);
                errors.push((idx, e.to_string()));
            }
        }
    }
    
    // Commit if we inserted at least some items
    if inserted > 0 {
        helper.commit()?;
        
        let duration = start_time.elapsed();
        println!(
            "[INFO] Batch insert completed: {}/{} items into {} (took {:?})",
            inserted,
            items.len(),
            table,
            duration
        );
        
        if !errors.is_empty() {
            eprintln!(
                "[WARN] Batch insert had {} errors out of {} items",
                errors.len(),
                items.len()
            );
        }
    } else {
        helper.rollback()?;
        return Err(XTauriError::content_cache(format!(
            "Failed to insert any items into {}",
            table
        )));
    }
    
    Ok(inserted)
}

/// Batch update helper for efficient bulk updates
/// 
/// This function performs batch updates using a transaction for atomicity
/// and improved performance.
pub fn batch_update<T, F>(
    conn: &mut Connection,
    table: &str,
    items: &[T],
    update_fn: F,
) -> Result<usize>
where
    F: Fn(&Transaction, &T) -> Result<()>,
{
    if items.is_empty() {
        return Ok(0);
    }
    
    let start_time = Instant::now();
    let operation_name = format!("batch_update_{}", table);
    
    #[cfg(debug_assertions)]
    println!("[DEBUG] Starting batch update: {} items in {}", items.len(), table);
    
    let helper = TransactionHelper::new(conn, &operation_name)?;
    let tx = helper.transaction()?;
    
    let mut updated = 0;
    let mut errors = Vec::new();
    
    for (idx, item) in items.iter().enumerate() {
        match update_fn(tx, item) {
            Ok(_) => updated += 1,
            Err(e) => {
                eprintln!("[WARN] Failed to update item {} in {}: {}", idx, table, e);
                errors.push((idx, e.to_string()));
            }
        }
    }
    
    // Commit if we updated at least some items
    if updated > 0 {
        helper.commit()?;
        
        let duration = start_time.elapsed();
        println!(
            "[INFO] Batch update completed: {}/{} items in {} (took {:?})",
            updated,
            items.len(),
            table,
            duration
        );
        
        if !errors.is_empty() {
            eprintln!(
                "[WARN] Batch update had {} errors out of {} items",
                errors.len(),
                items.len()
            );
        }
    } else {
        helper.rollback()?;
        return Err(XTauriError::content_cache(format!(
            "Failed to update any items in {}",
            table
        )));
    }
    
    Ok(updated)
}

/// Batch delete helper for efficient bulk deletes
/// 
/// This function performs batch deletes using a transaction for atomicity.
pub fn batch_delete<T, F>(
    conn: &mut Connection,
    table: &str,
    items: &[T],
    delete_fn: F,
) -> Result<usize>
where
    F: Fn(&Transaction, &T) -> Result<()>,
{
    if items.is_empty() {
        return Ok(0);
    }
    
    let start_time = Instant::now();
    let operation_name = format!("batch_delete_{}", table);
    
    #[cfg(debug_assertions)]
    println!("[DEBUG] Starting batch delete: {} items from {}", items.len(), table);
    
    let helper = TransactionHelper::new(conn, &operation_name)?;
    let tx = helper.transaction()?;
    
    let mut deleted = 0;
    
    for (idx, item) in items.iter().enumerate() {
        match delete_fn(tx, item) {
            Ok(_) => deleted += 1,
            Err(e) => {
                eprintln!("[WARN] Failed to delete item {} from {}: {}", idx, table, e);
            }
        }
    }
    
    helper.commit()?;
    
    let duration = start_time.elapsed();
    println!(
        "[INFO] Batch delete completed: {}/{} items from {} (took {:?})",
        deleted,
        items.len(),
        table,
        duration
    );
    
    Ok(deleted)
}

/// Execute a query with timing and logging
/// 
/// This helper logs query execution time and provides detailed error information.
pub fn execute_with_logging<F, T>(
    operation_name: &str,
    query_fn: F,
) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let start_time = Instant::now();
    
    #[cfg(debug_assertions)]
    println!("[DEBUG] Executing query: {}", operation_name);
    
    let result = query_fn();
    
    let duration = start_time.elapsed();
    
    match &result {
        Ok(_) => {
            #[cfg(debug_assertions)]
            println!("[DEBUG] Query completed: {} (took {:?})", operation_name, duration);
            
            // Warn if query is slow
            if duration.as_millis() > 100 {
                eprintln!(
                    "[WARN] Slow query detected: {} took {:?}",
                    operation_name,
                    duration
                );
            }
        }
        Err(e) => {
            eprintln!(
                "[ERROR] Query failed: {} (took {:?}): {}",
                operation_name,
                duration,
                e
            );
        }
    }
    
    result
}

/// Validate data before insertion
/// 
/// This function performs basic validation to ensure data integrity.
pub fn validate_profile_id(profile_id: &str) -> Result<()> {
    if profile_id.is_empty() {
        return Err(XTauriError::content_cache(
            "Profile ID cannot be empty".to_string(),
        ));
    }
    
    if profile_id.len() > 255 {
        return Err(XTauriError::content_cache(
            "Profile ID too long (max 255 characters)".to_string(),
        ));
    }
    
    Ok(())
}

/// Validate stream ID
pub fn validate_stream_id(stream_id: i64) -> Result<()> {
    if stream_id < 0 {
        return Err(XTauriError::content_cache(
            "Stream ID cannot be negative".to_string(),
        ));
    }
    
    Ok(())
}

/// Check if a record exists
/// 
/// Generic helper to check if a record exists in a table
pub fn record_exists(
    conn: &Connection,
    table: &str,
    where_clause: &str,
    params: &[&dyn rusqlite::ToSql],
) -> Result<bool> {
    let query = format!(
        "SELECT COUNT(*) FROM {} WHERE {}",
        table, where_clause
    );
    
    let count: i64 = conn.query_row(&query, params, |row| row.get(0))?;
    
    Ok(count > 0)
}

/// Get the last insert rowid
/// 
/// Returns the rowid of the most recent successful INSERT
pub fn last_insert_rowid(conn: &Connection) -> i64 {
    conn.last_insert_rowid()
}

/// Count records in a table with optional filter
pub fn count_records(
    conn: &Connection,
    table: &str,
    where_clause: Option<&str>,
    params: &[&dyn rusqlite::ToSql],
) -> Result<i64> {
    let query = if let Some(clause) = where_clause {
        format!("SELECT COUNT(*) FROM {} WHERE {}", table, clause)
    } else {
        format!("SELECT COUNT(*) FROM {}", table)
    };
    
    let count: i64 = conn.query_row(&query, params, |row| row.get(0))?;
    
    Ok(count)
}

/// Sanitize text for SQL LIKE queries
/// 
/// Escapes special characters in LIKE patterns
pub fn sanitize_like_pattern(pattern: &str) -> String {
    pattern
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Build a parameterized IN clause
/// 
/// Creates a string like "(?1, ?2, ?3)" for use in SQL IN clauses
pub fn build_in_clause(count: usize) -> String {
    if count == 0 {
        return "()".to_string();
    }
    
    let placeholders: Vec<String> = (1..=count).map(|i| format!("?{}", i)).collect();
    format!("({})", placeholders.join(", "))
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
                value INTEGER
            )",
            [],
        )
        .unwrap();
        
        conn
    }
    
    #[test]
    fn test_transaction_helper_commit() {
        let mut conn = create_test_db();
        
        {
            let helper = TransactionHelper::new(&mut conn, "test_commit").unwrap();
            let tx = helper.transaction().unwrap();
            
            tx.execute("INSERT INTO test_items (name, value) VALUES ('test', 42)", [])
                .unwrap();
            
            helper.commit().unwrap();
        }
        
        // Verify data was committed
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test_items", [], |row| row.get(0))
            .unwrap();
        
        assert_eq!(count, 1);
    }
    
    #[test]
    fn test_transaction_helper_rollback() {
        let mut conn = create_test_db();
        
        {
            let helper = TransactionHelper::new(&mut conn, "test_rollback").unwrap();
            let tx = helper.transaction().unwrap();
            
            tx.execute("INSERT INTO test_items (name, value) VALUES ('test', 42)", [])
                .unwrap();
            
            helper.rollback().unwrap();
        }
        
        // Verify data was rolled back
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test_items", [], |row| row.get(0))
            .unwrap();
        
        assert_eq!(count, 0);
    }
    
    #[test]
    fn test_batch_insert() {
        let mut conn = create_test_db();
        
        struct TestItem {
            name: String,
            value: i32,
        }
        
        let items = vec![
            TestItem {
                name: "item1".to_string(),
                value: 1,
            },
            TestItem {
                name: "item2".to_string(),
                value: 2,
            },
            TestItem {
                name: "item3".to_string(),
                value: 3,
            },
        ];
        
        let inserted = batch_insert(&mut conn, "test_items", &items, |tx, item| {
            tx.execute(
                "INSERT INTO test_items (name, value) VALUES (?1, ?2)",
                [&item.name, &item.value.to_string()],
            )?;
            Ok(())
        })
        .unwrap();
        
        assert_eq!(inserted, 3);
        
        // Verify data
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test_items", [], |row| row.get(0))
            .unwrap();
        
        assert_eq!(count, 3);
    }
    
    #[test]
    fn test_batch_update() {
        let mut conn = create_test_db();
        
        // Insert test data
        conn.execute("INSERT INTO test_items (name, value) VALUES ('item1', 1)", [])
            .unwrap();
        conn.execute("INSERT INTO test_items (name, value) VALUES ('item2', 2)", [])
            .unwrap();
        
        struct UpdateItem {
            name: String,
            new_value: i32,
        }
        
        let items = vec![
            UpdateItem {
                name: "item1".to_string(),
                new_value: 10,
            },
            UpdateItem {
                name: "item2".to_string(),
                new_value: 20,
            },
        ];
        
        let updated = batch_update(&mut conn, "test_items", &items, |tx, item| {
            tx.execute(
                "UPDATE test_items SET value = ?1 WHERE name = ?2",
                [&item.new_value.to_string(), &item.name],
            )?;
            Ok(())
        })
        .unwrap();
        
        assert_eq!(updated, 2);
        
        // Verify updates
        let value: i32 = conn
            .query_row(
                "SELECT value FROM test_items WHERE name = 'item1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        
        assert_eq!(value, 10);
    }
    
    #[test]
    fn test_batch_delete() {
        let mut conn = create_test_db();
        
        // Insert test data
        conn.execute("INSERT INTO test_items (name, value) VALUES ('item1', 1)", [])
            .unwrap();
        conn.execute("INSERT INTO test_items (name, value) VALUES ('item2', 2)", [])
            .unwrap();
        conn.execute("INSERT INTO test_items (name, value) VALUES ('item3', 3)", [])
            .unwrap();
        
        let names = vec!["item1".to_string(), "item2".to_string()];
        
        let deleted = batch_delete(&mut conn, "test_items", &names, |tx, name| {
            tx.execute("DELETE FROM test_items WHERE name = ?1", [name])?;
            Ok(())
        })
        .unwrap();
        
        assert_eq!(deleted, 2);
        
        // Verify deletion
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test_items", [], |row| row.get(0))
            .unwrap();
        
        assert_eq!(count, 1);
    }
    
    #[test]
    fn test_validate_profile_id() {
        assert!(validate_profile_id("valid-profile").is_ok());
        assert!(validate_profile_id("").is_err());
        
        let long_id = "a".repeat(256);
        assert!(validate_profile_id(&long_id).is_err());
    }
    
    #[test]
    fn test_validate_stream_id() {
        assert!(validate_stream_id(123).is_ok());
        assert!(validate_stream_id(0).is_ok());
        assert!(validate_stream_id(-1).is_err());
    }
    
    #[test]
    fn test_record_exists() {
        let conn = create_test_db();
        
        conn.execute("INSERT INTO test_items (name, value) VALUES ('test', 42)", [])
            .unwrap();
        
        let exists = record_exists(&conn, "test_items", "name = ?1", &[&"test"]).unwrap();
        assert!(exists);
        
        let not_exists = record_exists(&conn, "test_items", "name = ?1", &[&"nonexistent"]).unwrap();
        assert!(!not_exists);
    }
    
    #[test]
    fn test_count_records() {
        let conn = create_test_db();
        
        conn.execute("INSERT INTO test_items (name, value) VALUES ('item1', 1)", [])
            .unwrap();
        conn.execute("INSERT INTO test_items (name, value) VALUES ('item2', 2)", [])
            .unwrap();
        conn.execute("INSERT INTO test_items (name, value) VALUES ('item3', 3)", [])
            .unwrap();
        
        let total = count_records(&conn, "test_items", None, &[]).unwrap();
        assert_eq!(total, 3);
        
        let filtered = count_records(&conn, "test_items", Some("value > ?1"), &[&1]).unwrap();
        assert_eq!(filtered, 2);
    }
    
    #[test]
    fn test_sanitize_like_pattern() {
        assert_eq!(sanitize_like_pattern("test"), "test");
        assert_eq!(sanitize_like_pattern("test%"), "test\\%");
        assert_eq!(sanitize_like_pattern("test_"), "test\\_");
        assert_eq!(sanitize_like_pattern("test\\"), "test\\\\");
        assert_eq!(sanitize_like_pattern("test%_\\"), "test\\%\\_\\\\");
    }
    
    #[test]
    fn test_build_in_clause() {
        assert_eq!(build_in_clause(0), "()");
        assert_eq!(build_in_clause(1), "(?1)");
        assert_eq!(build_in_clause(3), "(?1, ?2, ?3)");
    }
    
    #[test]
    fn test_last_insert_rowid() {
        let conn = create_test_db();
        
        conn.execute("INSERT INTO test_items (name, value) VALUES ('test', 42)", [])
            .unwrap();
        
        let rowid = last_insert_rowid(&conn);
        assert_eq!(rowid, 1);
        
        conn.execute("INSERT INTO test_items (name, value) VALUES ('test2', 43)", [])
            .unwrap();
        
        let rowid = last_insert_rowid(&conn);
        assert_eq!(rowid, 2);
    }
    
    #[test]
    fn test_execute_with_logging() {
        let result = execute_with_logging("test_operation", || {
            Ok(42)
        });
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        let error_result: Result<i32> = execute_with_logging("test_error", || {
            Err(XTauriError::content_cache("test error".to_string()))
        });
        
        assert!(error_result.is_err());
    }
}
