# Design Document

## Overview

Este documento descreve o design técnico para implementar um sistema de cache local de conteúdos Xtream no backend Tauri. O sistema armazenará canais, filmes, séries e suas categorias em tabelas SQLite otimizadas, permitindo acesso instantâneo aos dados sem necessidade de requisições HTTP ao servidor Xtream.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (React)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Movie Grid   │  │ Series Grid  │  │ Channel List │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
│                   ┌────────▼────────┐                        │
│                   │  Zustand Store  │                        │
│                   │ (xtreamContent) │                        │
│                   └────────┬────────┘                        │
└────────────────────────────┼──────────────────────────────────┘
                             │ Tauri IPC
┌────────────────────────────▼──────────────────────────────────┐
│                    Backend (Rust/Tauri)                        │
│                                                                │
│  ┌──────────────────────────────────────────────────────┐    │
│  │              Content Cache Manager                    │    │
│  │  ┌────────────────┐  ┌────────────────┐             │    │
│  │  │ Sync Scheduler │  │ Query Handler  │             │    │
│  │  └───────┬────────┘  └───────┬────────┘             │    │
│  │          │                    │                       │    │
│  │  ┌───────▼────────────────────▼────────┐            │    │
│  │  │      SQLite Database Layer          │            │    │
│  │  │  ┌──────────┐  ┌──────────┐        │            │    │
│  │  │  │ Channels │  │  Movies  │        │            │    │
│  │  │  ├──────────┤  ├──────────┤        │            │    │
│  │  │  │ Series   │  │ Episodes │        │            │    │
│  │  │  ├──────────┤  ├──────────┤        │            │    │
│  │  │  │Categories│  │   Sync   │        │            │    │
│  │  │  └──────────┘  └──────────┘        │            │    │
│  │  └─────────────────────────────────────┘            │    │
│  └──────────────────────────────────────────────────────┘    │
│                             │                                 │
│  ┌──────────────────────────▼──────────────────────────┐    │
│  │           Xtream API Client (HTTP)                   │    │
│  │  (usado apenas para sincronização)                   │    │
│  └──────────────────────────────────────────────────────┘    │
└────────────────────────────────────────────────────────────────┘
                             │
                             │ HTTP (apenas sync)
                             ▼
                    ┌─────────────────┐
                    │ Xtream Server   │
                    └─────────────────┘
```

## Components and Interfaces

### 1. Database Schema

#### Tables

**xtream_channels**
```sql
CREATE TABLE xtream_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id INTEGER NOT NULL,
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
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
    UNIQUE(profile_id, stream_id)
);

CREATE INDEX idx_channels_profile ON xtream_channels(profile_id);
CREATE INDEX idx_channels_category ON xtream_channels(category_id);
CREATE INDEX idx_channels_name ON xtream_channels(name COLLATE NOCASE);
CREATE INDEX idx_channels_stream_id ON xtream_channels(stream_id);
```

**xtream_movies**
```sql
CREATE TABLE xtream_movies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id INTEGER NOT NULL,
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
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
    UNIQUE(profile_id, stream_id)
);

CREATE INDEX idx_movies_profile ON xtream_movies(profile_id);
CREATE INDEX idx_movies_category ON xtream_movies(category_id);
CREATE INDEX idx_movies_name ON xtream_movies(name COLLATE NOCASE);
CREATE INDEX idx_movies_rating ON xtream_movies(rating DESC);
CREATE INDEX idx_movies_year ON xtream_movies(year DESC);
CREATE INDEX idx_movies_genre ON xtream_movies(genre);
```

**xtream_series**
```sql
CREATE TABLE xtream_series (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id INTEGER NOT NULL,
    series_id INTEGER NOT NULL,
    num INTEGER,
    name TEXT NOT NULL,
    title TEXT,
    year TEXT,
    cover TEXT,
    plot TEXT,
    cast TEXT,
    director TEXT,
    genre TEXT,
    release_date TEXT,
    last_modified TEXT,
    rating TEXT,
    rating_5based REAL,
    episode_run_time TEXT,
    category_id TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
    UNIQUE(profile_id, series_id)
);

CREATE INDEX idx_series_profile ON xtream_series(profile_id);
CREATE INDEX idx_series_category ON xtream_series(category_id);
CREATE INDEX idx_series_name ON xtream_series(name COLLATE NOCASE);
CREATE INDEX idx_series_rating ON xtream_series(rating_5based DESC);
```

**xtream_seasons**
```sql
CREATE TABLE xtream_seasons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id INTEGER NOT NULL,
    series_id INTEGER NOT NULL,
    season_number INTEGER NOT NULL,
    name TEXT,
    episode_count INTEGER,
    overview TEXT,
    air_date TEXT,
    cover TEXT,
    cover_big TEXT,
    vote_average REAL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
    UNIQUE(profile_id, series_id, season_number)
);

CREATE INDEX idx_seasons_series ON xtream_seasons(series_id);
```

**xtream_episodes**
```sql
CREATE TABLE xtream_episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id INTEGER NOT NULL,
    series_id INTEGER NOT NULL,
    episode_id TEXT NOT NULL,
    season_number INTEGER NOT NULL,
    episode_num TEXT NOT NULL,
    title TEXT,
    container_extension TEXT,
    custom_sid TEXT,
    added TEXT,
    direct_source TEXT,
    info_json TEXT, -- JSON com info completa do episódio
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
    UNIQUE(profile_id, episode_id)
);

CREATE INDEX idx_episodes_series ON xtream_episodes(series_id);
CREATE INDEX idx_episodes_season ON xtream_episodes(season_number);
```

**xtream_channel_categories**
```sql
CREATE TABLE xtream_channel_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id INTEGER NOT NULL,
    category_id TEXT NOT NULL,
    category_name TEXT NOT NULL,
    parent_id INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE,
    UNIQUE(profile_id, category_id)
);

CREATE INDEX idx_channel_cats_profile ON xtream_channel_categories(profile_id);
```

**xtream_movie_categories** (similar structure)
**xtream_series_categories** (similar structure)

**xtream_content_sync**
```sql
CREATE TABLE xtream_content_sync (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id INTEGER NOT NULL UNIQUE,
    last_sync_channels TIMESTAMP,
    last_sync_movies TIMESTAMP,
    last_sync_series TIMESTAMP,
    sync_status TEXT DEFAULT 'pending', -- pending, syncing, completed, failed, partial
    sync_progress INTEGER DEFAULT 0, -- 0-100
    sync_message TEXT,
    channels_count INTEGER DEFAULT 0,
    movies_count INTEGER DEFAULT 0,
    series_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
);
```

**xtream_sync_settings**
```sql
CREATE TABLE xtream_sync_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id INTEGER NOT NULL UNIQUE,
    auto_sync_enabled BOOLEAN DEFAULT 1,
    sync_interval_hours INTEGER DEFAULT 24, -- 6, 12, 24, 48
    wifi_only BOOLEAN DEFAULT 1,
    notify_on_complete BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
);
```

### 2. Rust Backend Modules

#### Module: `content_cache.rs`

```rust
pub struct ContentCache {
    db: Arc<Mutex<Connection>>,
    sync_scheduler: Arc<SyncScheduler>,
}

impl ContentCache {
    pub fn new(db: Arc<Mutex<Connection>>) -> Result<Self>;
    
    // Initialization
    pub fn initialize_tables(&self) -> Result<()>;
    
    // Channel operations
    pub async fn get_channels(&self, profile_id: i64, category_id: Option<String>) -> Result<Vec<XtreamChannel>>;
    pub async fn save_channels(&self, profile_id: i64, channels: Vec<XtreamChannel>) -> Result<()>;
    pub async fn search_channels(&self, profile_id: i64, query: &str) -> Result<Vec<XtreamChannel>>;
    
    // Movie operations
    pub async fn get_movies(&self, profile_id: i64, category_id: Option<String>) -> Result<Vec<XtreamMoviesListing>>;
    pub async fn save_movies(&self, profile_id: i64, movies: Vec<XtreamMoviesListing>) -> Result<()>;
    pub async fn search_movies(&self, profile_id: i64, query: &str) -> Result<Vec<XtreamMoviesListing>>;
    pub async fn filter_movies(&self, profile_id: i64, filters: MovieFilters) -> Result<Vec<XtreamMoviesListing>>;
    
    // Series operations
    pub async fn get_series(&self, profile_id: i64, category_id: Option<String>) -> Result<Vec<XtreamShowListing>>;
    pub async fn get_series_details(&self, profile_id: i64, series_id: i64) -> Result<XtreamShow>;
    pub async fn save_series(&self, profile_id: i64, series: Vec<XtreamShowListing>) -> Result<()>;
    pub async fn save_series_details(&self, profile_id: i64, series_id: i64, details: XtreamShow) -> Result<()>;
    
    // Category operations
    pub async fn get_categories(&self, profile_id: i64, content_type: ContentType) -> Result<Vec<XtreamCategory>>;
    pub async fn save_categories(&self, profile_id: i64, content_type: ContentType, categories: Vec<XtreamCategory>) -> Result<()>;
    
    // Sync status
    pub async fn get_sync_status(&self, profile_id: i64) -> Result<SyncStatus>;
    pub async fn update_sync_status(&self, profile_id: i64, status: SyncStatus) -> Result<()>;
    
    // Cache management
    pub async fn clear_cache(&self, profile_id: i64) -> Result<()>;
    pub async fn get_cache_stats(&self, profile_id: i64) -> Result<CacheStats>;
}
```

#### Module: `sync_scheduler.rs`

```rust
pub struct SyncScheduler {
    db: Arc<Mutex<Connection>>,
    xtream_client: Arc<XtreamClient>,
    active_syncs: Arc<Mutex<HashMap<i64, SyncHandle>>>,
}

impl SyncScheduler {
    pub fn new(db: Arc<Mutex<Connection>>, xtream_client: Arc<XtreamClient>) -> Self;
    
    // Sync operations
    pub async fn start_full_sync(&self, profile_id: i64) -> Result<SyncHandle>;
    pub async fn start_incremental_sync(&self, profile_id: i64) -> Result<SyncHandle>;
    pub async fn cancel_sync(&self, profile_id: i64) -> Result<()>;
    pub async fn get_sync_progress(&self, profile_id: i64) -> Result<SyncProgress>;
    
    // Background sync
    pub async fn check_and_sync_if_needed(&self, profile_id: i64) -> Result<()>;
    pub fn start_background_scheduler(&self);
    
    // Settings
    pub async fn get_sync_settings(&self, profile_id: i64) -> Result<SyncSettings>;
    pub async fn update_sync_settings(&self, profile_id: i64, settings: SyncSettings) -> Result<()>;
}

pub struct SyncHandle {
    profile_id: i64,
    cancel_token: CancellationToken,
    progress_rx: Receiver<SyncProgress>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncProgress {
    pub status: SyncStatus,
    pub progress: u8, // 0-100
    pub current_step: String,
    pub channels_synced: usize,
    pub movies_synced: usize,
    pub series_synced: usize,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncSettings {
    pub auto_sync_enabled: bool,
    pub sync_interval_hours: u32,
    pub wifi_only: bool,
    pub notify_on_complete: bool,
}
```

#### Module: `query_optimizer.rs`

```rust
pub struct QueryOptimizer {
    db: Arc<Mutex<Connection>>,
}

impl QueryOptimizer {
    // Optimized queries with pagination
    pub async fn paginated_query<T>(&self, query: &str, params: &[&dyn ToSql], page: usize, page_size: usize) -> Result<Vec<T>>;
    
    // Full-text search
    pub async fn fts_search(&self, table: &str, query: &str, limit: usize) -> Result<Vec<i64>>;
    
    // Batch operations
    pub async fn batch_insert<T>(&self, table: &str, items: Vec<T>) -> Result<()>;
    pub async fn batch_update<T>(&self, table: &str, items: Vec<T>) -> Result<()>;
    
    // Index management
    pub async fn analyze_tables(&self) -> Result<()>;
    pub async fn vacuum_database(&self) -> Result<()>;
}
```

### 3. Tauri Commands

```rust
// Content retrieval commands (local-first)
#[tauri::command]
async fn get_cached_channels(profile_id: String, category_id: Option<String>) -> Result<Vec<XtreamChannel>>;

#[tauri::command]
async fn get_cached_movies(profile_id: String, category_id: Option<String>) -> Result<Vec<XtreamMoviesListing>>;

#[tauri::command]
async fn get_cached_series(profile_id: String, category_id: Option<String>) -> Result<Vec<XtreamShowListing>>;

#[tauri::command]
async fn get_cached_series_details(profile_id: String, series_id: String) -> Result<XtreamShow>;

#[tauri::command]
async fn search_cached_content(profile_id: String, content_type: String, query: String) -> Result<Vec<serde_json::Value>>;

// Sync commands
#[tauri::command]
async fn start_content_sync(profile_id: String, full_sync: bool) -> Result<()>;

#[tauri::command]
async fn cancel_content_sync(profile_id: String) -> Result<()>;

#[tauri::command]
async fn get_sync_progress(profile_id: String) -> Result<SyncProgress>;

#[tauri::command]
async fn get_sync_status(profile_id: String) -> Result<SyncStatus>;

// Settings commands
#[tauri::command]
async fn get_sync_settings(profile_id: String) -> Result<SyncSettings>;

#[tauri::command]
async fn update_sync_settings(profile_id: String, settings: SyncSettings) -> Result<()>;

// Cache management commands
#[tauri::command]
async fn clear_content_cache(profile_id: String) -> Result<()>;

#[tauri::command]
async fn get_cache_stats(profile_id: String) -> Result<CacheStats>;
```

## Data Models

### Rust Structs

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct CacheStats {
    pub total_size_bytes: u64,
    pub channels_count: usize,
    pub movies_count: usize,
    pub series_count: usize,
    pub last_sync_channels: Option<DateTime<Utc>>,
    pub last_sync_movies: Option<DateTime<Utc>>,
    pub last_sync_series: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SyncStatus {
    Pending,
    Syncing,
    Completed,
    Failed,
    Partial,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ContentType {
    Channels,
    Movies,
    Series,
}

#[derive(Serialize, Deserialize)]
pub struct MovieFilters {
    pub category_id: Option<String>,
    pub genre: Option<String>,
    pub year: Option<String>,
    pub min_rating: Option<f32>,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Sync already in progress for profile {0}")]
    SyncInProgress(i64),
    
    #[error("Sync failed: {0}")]
    SyncFailed(String),
    
    #[error("Profile not found: {0}")]
    ProfileNotFound(i64),
    
    #[error("Cache not initialized for profile {0}")]
    CacheNotInitialized(i64),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

## Testing Strategy

### Unit Tests

1. **Database Operations**
   - Test CRUD operations for each table
   - Test index performance
   - Test transaction rollback
   - Test foreign key constraints

2. **Sync Logic**
   - Test full sync flow
   - Test incremental sync
   - Test sync cancellation
   - Test error recovery

3. **Query Optimization**
   - Test pagination
   - Test search performance
   - Test filter combinations
   - Test large dataset handling

### Integration Tests

1. **End-to-End Sync**
   - Test complete sync flow from API to cache
   - Test data consistency
   - Test concurrent syncs

2. **Frontend Integration**
   - Test Tauri command responses
   - Test error propagation
   - Test progress updates

### Performance Tests

1. **Query Performance**
   - Benchmark queries with 10k, 50k, 100k records
   - Test search response time (target < 100ms)
   - Test pagination performance

2. **Sync Performance**
   - Measure sync time for different dataset sizes
   - Test memory usage during sync
   - Test database size growth

## Migration Strategy

### Phase 1: Database Setup (Week 1)
- Create database schema
- Implement migration system
- Add indexes and constraints
- Test with sample data

### Phase 2: Core Cache Implementation (Week 2)
- Implement ContentCache module
- Add CRUD operations
- Implement query optimizer
- Add unit tests

### Phase 3: Sync System (Week 3)
- Implement SyncScheduler
- Add full sync logic
- Add incremental sync
- Implement progress tracking

### Phase 4: Tauri Commands (Week 4)
- Add all Tauri commands
- Implement error handling
- Add integration tests
- Test with frontend

### Phase 5: Frontend Integration (Week 5)
- Update Zustand stores
- Modify components to use cache
- Add sync UI
- Add settings UI

### Phase 6: Background Sync & Polish (Week 6)
- Implement background scheduler
- Add sync settings
- Optimize performance
- Final testing and bug fixes

## Performance Targets

- **Query Response Time**: < 100ms for 95% of queries
- **Search Response Time**: < 150ms for fuzzy search
- **Sync Time**: < 5 minutes for 10k items
- **Memory Usage**: < 200MB during sync
- **Database Size**: ~50MB per 10k items
- **Startup Time**: < 500ms additional overhead

## Security Considerations

1. **Data Isolation**: Profile data must be strictly isolated
2. **SQL Injection**: Use parameterized queries only
3. **Transaction Safety**: Use transactions for all multi-step operations
4. **Error Messages**: Don't expose sensitive data in errors
5. **Cache Invalidation**: Proper cleanup on profile deletion

## Monitoring and Logging

```rust
// Log levels for different operations
- INFO: Sync started/completed, cache hits
- WARN: Sync partial failures, slow queries
- ERROR: Sync failures, database errors
- DEBUG: Detailed sync progress, query execution

// Metrics to track
- Cache hit/miss ratio
- Query execution time
- Sync duration and success rate
- Database size growth
- Error frequency by type
```

## Future Enhancements

1. **Differential Sync**: Only sync changed items
2. **Compression**: Compress large text fields (plot, cast)
3. **Image Caching**: Cache poster/cover images locally
4. **Smart Prefetch**: Predict and prefetch likely-to-be-viewed content
5. **Export/Import**: Allow cache backup and restore
6. **Multi-Profile Sync**: Sync multiple profiles in parallel
