# Incremental Synchronization Implementation

## Overview

This document describes the incremental synchronization feature implemented for the Xtream content cache. Incremental sync allows the application to efficiently update cached content by only downloading and processing changes since the last synchronization, rather than re-downloading all content.

## Key Components

### 1. Timestamp Tracking

The system tracks the last sync time for each content type (channels, movies, series) in the `xtream_content_sync` table:
- `last_sync_channels`: Timestamp of last channel sync
- `last_sync_movies`: Timestamp of last movie sync
- `last_sync_series`: Timestamp of last series sync

### 2. Content Comparison

The incremental sync compares server content with cached content to identify:
- **New items**: Items on the server that don't exist in the cache
- **Updated items**: Items that exist in both but have been modified since last sync
- **Deleted items**: Items in the cache that no longer exist on the server

### 3. Differential Updates

Instead of replacing all content, the system:
1. Adds new items to the cache
2. Updates modified items in the cache
3. Removes deleted items from the cache

## Implementation Details

### Core Methods

#### `run_incremental_sync()`
Main entry point for incremental synchronization. Orchestrates the sync process for all content types.

```rust
pub async fn run_incremental_sync(
    &self,
    profile_id: &str,
    base_url: &str,
    username: &str,
    password: &str,
    content_cache: &ContentCache,
    progress_tx: &mpsc::Sender<SyncProgress>,
    cancel_token: &CancellationToken,
) -> Result<SyncProgress>
```

#### `sync_content_incremental()`
Performs incremental sync for a specific content type (channels, movies, or series).

Steps:
1. Fetch all content from the server
2. Get cached content IDs
3. Compare server content with cache
4. Apply changes (add, update, delete)

#### `compare_channels()`, `compare_movies()`, `compare_series()`
Compare server content with cached content to identify changes.

Returns:
- `new_items`: Items to add
- `updated_items`: Items to update
- `server_ids`: All IDs from the server (for detecting deletions)

#### `is_item_updated()`
Determines if an item has been updated since the last sync by comparing timestamps.

Supports two timestamp formats:
- Unix timestamps (e.g., "1234567890")
- ISO 8601 format (e.g., "2024-01-01T12:00:00Z")

### ContentCache Methods

#### `get_content_ids()`
Retrieves all content IDs for a specific content type from the cache.

```rust
pub fn get_content_ids(&self, profile_id: &str, content_type: &str) -> Result<Vec<i64>>
```

#### `delete_content_by_ids()`
Deletes specific content items by their IDs.

```rust
pub fn delete_content_by_ids(&self, profile_id: &str, content_type: &str, ids: &[i64]) -> Result<usize>
```

## Timestamp Comparison Logic

The system uses the following logic to determine if an item has been updated:

1. **Unix Timestamps**: Parse both timestamps as integers and compare
   - Item timestamp > Last sync timestamp = Updated
   
2. **ISO 8601 Timestamps**: Parse as RFC3339 and compare DateTime objects
   - Item DateTime > Last sync DateTime = Updated

3. **Fallback**: If timestamps can't be parsed, assume not updated

## Performance Considerations

### Advantages of Incremental Sync

1. **Reduced Network Traffic**: Only downloads changed content
2. **Faster Sync Times**: Processes fewer items
3. **Lower Server Load**: Fewer API requests
4. **Better User Experience**: Quicker updates

### When to Use Full vs Incremental Sync

**Use Full Sync:**
- First sync for a new profile
- After clearing cache
- When sync has never been performed
- When incremental sync fails

**Use Incremental Sync:**
- Regular background updates
- Manual refresh when content is already cached
- Scheduled automatic syncs

## Error Handling

The incremental sync is designed to be resilient:

1. **Partial Failures**: If one content type fails, others continue
2. **Timestamp Parsing**: Falls back to treating items as unchanged if timestamps can't be parsed
3. **Network Errors**: Uses retry logic with exponential backoff
4. **Database Errors**: Rolls back transactions to maintain consistency

## Testing

The implementation includes comprehensive tests:

- `test_get_content_ids_empty`: Verify empty cache returns no IDs
- `test_get_content_ids_with_data`: Verify IDs are retrieved correctly
- `test_delete_content_by_ids`: Verify deletion works correctly
- `test_is_item_updated_unix_timestamp`: Test Unix timestamp comparison
- `test_is_item_updated_iso8601`: Test ISO 8601 timestamp comparison
- `test_compare_channels_new_items`: Test detection of new items
- `test_compare_channels_updated_items`: Test detection of updated items
- `test_compare_channels_mixed`: Test mixed scenario with new, updated, and unchanged items

## Usage Example

```rust
// Start incremental sync
let scheduler = SyncScheduler::new(db);
let (handle, progress_tx, cancel_token) = SyncHandle::new(profile_id.to_string());

let result = scheduler.run_incremental_sync(
    &profile_id,
    &base_url,
    &username,
    &password,
    &content_cache,
    &progress_tx,
    &cancel_token,
).await?;

// Check result
match result.status {
    SyncStatus::Completed => println!("Sync completed successfully"),
    SyncStatus::Partial => println!("Sync completed with {} errors", result.errors.len()),
    SyncStatus::Failed => println!("Sync failed"),
    _ => {}
}
```

## Future Enhancements

Potential improvements for incremental sync:

1. **Smart Sync**: Only sync content types that have changed
2. **Batch Size Optimization**: Adjust batch sizes based on network conditions
3. **Conflict Resolution**: Handle cases where local and server data conflict
4. **Sync Scheduling**: Automatically determine optimal sync intervals
5. **Bandwidth Monitoring**: Pause sync on slow connections
6. **Delta Compression**: Compress change sets for faster transmission

## Requirements Satisfied

This implementation satisfies the following requirements from the spec:

- **Requirement 4.2**: Implements incremental synchronization with timestamp comparison
- **Requirement 4.4**: Detects and removes deleted items from cache
- **Requirement 10.1**: Uses transactions for atomicity
- **Requirement 10.2**: Implements proper error handling and logging
- **Requirement 10.3**: Maintains data isolation between profiles
