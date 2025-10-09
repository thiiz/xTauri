use crate::error::{Result, XTauriError};
use crate::xtream::types::{CachedContent, CacheKey};
use dashmap::DashMap;
use rusqlite::Connection;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::time::Instant;

/// Content-type specific caching policies
#[derive(Debug, Clone)]
pub struct CachePolicies {
    pub channels: CachePolicy,
    pub movies: CachePolicy,
    pub series: CachePolicy,
    pub epg: CachePolicy,
    pub categories: CachePolicy,
}

impl Default for CachePolicies {
    fn default() -> Self {
        Self {
            channels: CachePolicy {
                ttl: Duration::from_secs(3600), // 1 hour
                max_entries: 1000,
                prefetch_enabled: true,
                priority: CachePriority::High,
            },
            movies: CachePolicy {
                ttl: Duration::from_secs(7200), // 2 hours
                max_entries: 500,
                prefetch_enabled: false,
                priority: CachePriority::Medium,
            },
            series: CachePolicy {
                ttl: Duration::from_secs(7200), // 2 hours
                max_entries: 500,
                prefetch_enabled: false,
                priority: CachePriority::Medium,
            },
            epg: CachePolicy {
                ttl: Duration::from_secs(1800), // 30 minutes
                max_entries: 200,
                prefetch_enabled: true,
                priority: CachePriority::High,
            },
            categories: CachePolicy {
                ttl: Duration::from_secs(14400), // 4 hours
                max_entries: 100,
                prefetch_enabled: true,
                priority: CachePriority::High,
            },
        }
    }
}

/// Individual cache policy for a content type
#[derive(Debug, Clone)]
pub struct CachePolicy {
    pub ttl: Duration,
    pub max_entries: usize,
    pub prefetch_enabled: bool,
    pub priority: CachePriority,
}

/// Cache priority levels for eviction policies
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePriority {
    Low = 1,
    Medium = 2,
    High = 3,
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub prefetch_hits: u64,
    pub prefetch_misses: u64,
    pub last_cleanup: Option<DateTime<Utc>>,
    pub content_type_stats: HashMap<String, ContentTypeStats>,
}

/// Statistics per content type
#[derive(Debug, Clone, Default)]
pub struct ContentTypeStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub total_size: usize,
    pub avg_access_time: Duration,
}

/// Item for prefetching queue
#[derive(Debug, Clone)]
pub struct PrefetchItem {
    pub profile_id: String,
    pub content_type: String,
    pub identifier: Option<String>,
    pub priority: CachePriority,
    pub scheduled_at: DateTime<Utc>,
}

/// Cache warming configuration
#[derive(Debug, Clone)]
pub struct CacheWarmingConfig {
    pub enabled: bool,
    pub warm_on_profile_switch: bool,
    pub warm_categories: bool,
    pub warm_recent_content: bool,
    pub max_warm_items: usize,
}

impl Default for CacheWarmingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            warm_on_profile_switch: true,
            warm_categories: true,
            warm_recent_content: true,
            max_warm_items: 50,
        }
    }
}

/// Manages content caching for Xtream data with both memory and disk storage
pub struct ContentCache {
    db: Arc<Mutex<Connection>>,
    memory_cache: Arc<DashMap<String, CachedContent>>,
    default_ttl: Duration,
    cache_policies: CachePolicies,
    cache_stats: Arc<Mutex<CacheStatistics>>,
    prefetch_queue: Arc<Mutex<Vec<PrefetchItem>>>,
}

impl ContentCache {
    /// Create a new content cache
    pub fn new(db: Arc<Mutex<Connection>>, default_ttl: Duration) -> Self {
        Self {
            db,
            memory_cache: Arc::new(DashMap::new()),
            default_ttl,
            cache_policies: CachePolicies::default(),
            cache_stats: Arc::new(Mutex::new(CacheStatistics::default())),
            prefetch_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Create a new content cache with custom policies
    pub fn with_policies(db: Arc<Mutex<Connection>>, default_ttl: Duration, policies: CachePolicies) -> Self {
        Self {
            db,
            memory_cache: Arc::new(DashMap::new()),
            default_ttl,
            cache_policies: policies,
            cache_stats: Arc::new(Mutex::new(CacheStatistics::default())),
            prefetch_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get cached content by key
    pub fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let start_time = Instant::now();
        let content_type = self.extract_content_type_from_key(key);
        
        // First check memory cache
        if let Some(cached) = self.memory_cache.get(key) {
            if cached.expires_at > Utc::now() {
                self.record_cache_hit(&content_type, start_time.elapsed());
                return self.deserialize_content(&cached.data);
            } else {
                // Remove expired entry
                self.memory_cache.remove(key);
            }
        }
        
        // Check database cache
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let mut stmt = db.prepare(
            "SELECT data, expires_at FROM xtream_content_cache WHERE cache_key = ? AND expires_at > datetime('now')"
        )?;
        
        let result = stmt.query_row([key], |row| {
            let data: Vec<u8> = row.get(0)?;
            let expires_at_str: String = row.get(1)?;
            let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(1, "expires_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            Ok(CachedContent {
                data,
                expires_at,
                content_type: "unknown".to_string(), // We don't need this for deserialization
            })
        });
        
        match result {
            Ok(cached) => {
                // Store in memory cache for faster access
                self.memory_cache.insert(key.to_string(), cached.clone());
                self.record_cache_hit(&content_type, start_time.elapsed());
                self.deserialize_content(&cached.data)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                self.record_cache_miss(&content_type);
                Ok(None)
            }
            Err(e) => Err(XTauriError::Database(e)),
        }
    }
    
    /// Get cached content by key, including stale (expired) entries
    pub fn get_stale<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let start_time = Instant::now();
        let content_type = self.extract_content_type_from_key(key);
        
        // First check memory cache (including expired)
        if let Some(cached) = self.memory_cache.get(key) {
            self.record_cache_hit(&content_type, start_time.elapsed());
            return self.deserialize_content(&cached.data);
        }
        
        // Check database cache (including expired)
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let mut stmt = db.prepare(
            "SELECT data, expires_at FROM xtream_content_cache WHERE cache_key = ?"
        )?;
        
        let result = stmt.query_row([key], |row| {
            let data: Vec<u8> = row.get(0)?;
            let expires_at_str: String = row.get(1)?;
            let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(1, "expires_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc);
            
            Ok(CachedContent {
                data,
                expires_at,
                content_type: "unknown".to_string(),
            })
        });
        
        match result {
            Ok(cached) => {
                // Store in memory cache for faster access
                self.memory_cache.insert(key.to_string(), cached.clone());
                self.record_cache_hit(&content_type, start_time.elapsed());
                self.deserialize_content(&cached.data)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                self.record_cache_miss(&content_type);
                Ok(None)
            }
            Err(e) => Err(XTauriError::Database(e)),
        }
    }
    
    /// Set cached content with optional TTL
    pub fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize,
    {
        let data = self.serialize_content(value)?;
        let content_type_str = self.extract_content_type_from_key(key);
        
        // Use content-type specific TTL if not provided
        let ttl = ttl.unwrap_or_else(|| self.get_ttl_for_content_type(&content_type_str));
        
        // Check if we need to evict entries based on policy
        self.enforce_cache_policy(&content_type_str)?;
        
        let expires_at = Utc::now() + chrono::Duration::from_std(ttl)
            .map_err(|e| XTauriError::content_cache(format!("Invalid TTL: {}", e)))?;
        
        let cached_content = CachedContent {
            data: data.clone(),
            expires_at,
            content_type: content_type_str.clone(),
        };
        
        // Store in memory cache
        self.memory_cache.insert(key.to_string(), cached_content);
        
        // Store in database
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        let expires_at_str = expires_at.to_rfc3339();
        
        // Extract profile_id from cache key for foreign key constraint
        let profile_id = self.extract_profile_id_from_key(key)?;
        
        db.execute(
            "INSERT OR REPLACE INTO xtream_content_cache (cache_key, profile_id, content_type, data, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            (key, &profile_id, &content_type_str, &data, &expires_at_str),
        )?;
        
        Ok(())
    }
    
    /// Invalidate cache entries matching a pattern
    pub fn invalidate(&self, pattern: &str) -> Result<()> {
        // Remove from memory cache
        self.memory_cache.retain(|key, _| !key.contains(pattern));
        
        // Remove from database
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        let pattern_owned = format!("%{}%", pattern);
        
        db.execute(
            "DELETE FROM xtream_content_cache WHERE cache_key LIKE ?1",
            [&pattern_owned],
        )?;
        
        Ok(())
    }
    
    /// Clear all cache for a specific profile
    pub fn clear_profile_cache(&self, profile_id: &str) -> Result<()> {
        // Remove from memory cache
        let profile_prefix = format!("{}:", profile_id);
        self.memory_cache.retain(|key, _| !key.starts_with(&profile_prefix));
        
        // Remove from database
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        db.execute(
            "DELETE FROM xtream_content_cache WHERE profile_id = ?1",
            [profile_id],
        )?;
        
        Ok(())
    }
    
    /// Clear expired entries from both memory and database
    pub fn cleanup_expired(&self) -> Result<()> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        
        // Remove expired entries from memory cache
        self.memory_cache.retain(|_, cached| cached.expires_at > now);
        
        // Remove expired entries from database
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        db.execute(
            "DELETE FROM xtream_content_cache WHERE expires_at <= ?",
            [&now_str],
        )?;
        
        Ok(())
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> Result<CacheStats> {
        let memory_entries = self.memory_cache.len();
        
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
            
        let entries: i64 = db.query_row(
            "SELECT COUNT(*) FROM xtream_content_cache",
            [],
            |row| row.get(0),
        )?;
        
        let size: i64 = db.query_row(
            "SELECT COALESCE(SUM(LENGTH(data)), 0) FROM xtream_content_cache",
            [],
            |row| row.get(0),
        )?;
        
        Ok(CacheStats {
            memory_entries,
            database_entries: entries as usize,
            total_size_bytes: size as usize,
        })
    }
    
    /// Generate cache key from components
    pub fn generate_key(&self, cache_key: &CacheKey) -> String {
        cache_key.to_string()
    }
    
    /// Warm cache for a profile with commonly accessed content
    pub async fn warm_cache_for_profile(&self, profile_id: &str, config: &CacheWarmingConfig) -> Result<()> {
        if !config.enabled {
            return Ok(());
        }
        
        let mut warm_items = Vec::new();
        
        if config.warm_categories {
            // Add category warming items
            warm_items.push(PrefetchItem {
                profile_id: profile_id.to_string(),
                content_type: "channel_categories".to_string(),
                identifier: None,
                priority: CachePriority::High,
                scheduled_at: Utc::now(),
            });
            
            warm_items.push(PrefetchItem {
                profile_id: profile_id.to_string(),
                content_type: "movie_categories".to_string(),
                identifier: None,
                priority: CachePriority::Medium,
                scheduled_at: Utc::now(),
            });
            
            warm_items.push(PrefetchItem {
                profile_id: profile_id.to_string(),
                content_type: "series_categories".to_string(),
                identifier: None,
                priority: CachePriority::Medium,
                scheduled_at: Utc::now(),
            });
        }
        
        if config.warm_recent_content {
            // Add recent content based on history (this would need integration with history tracking)
            // For now, we'll add some common content types
            warm_items.push(PrefetchItem {
                profile_id: profile_id.to_string(),
                content_type: "channels".to_string(),
                identifier: None,
                priority: CachePriority::High,
                scheduled_at: Utc::now(),
            });
        }
        
        // Limit the number of items to warm
        warm_items.truncate(config.max_warm_items);
        
        // Add to prefetch queue
        {
            let mut queue = self.prefetch_queue.lock()
                .map_err(|_| XTauriError::lock_acquisition("prefetch queue"))?;
            queue.extend(warm_items);
            
            // Sort by priority and scheduled time
            queue.sort_by(|a, b| {
                b.priority.cmp(&a.priority)
                    .then_with(|| a.scheduled_at.cmp(&b.scheduled_at))
            });
        }
        
        Ok(())
    }
    
    /// Add item to prefetch queue
    pub fn schedule_prefetch(&self, item: PrefetchItem) -> Result<()> {
        let mut queue = self.prefetch_queue.lock()
            .map_err(|_| XTauriError::lock_acquisition("prefetch queue"))?;
        
        // Check if item already exists in queue
        if !queue.iter().any(|existing| {
            existing.profile_id == item.profile_id 
                && existing.content_type == item.content_type 
                && existing.identifier == item.identifier
        }) {
            queue.push(item);
            
            // Sort by priority and scheduled time
            queue.sort_by(|a, b| {
                b.priority.cmp(&a.priority)
                    .then_with(|| a.scheduled_at.cmp(&b.scheduled_at))
            });
        }
        
        Ok(())
    }
    
    /// Get next item from prefetch queue
    pub fn get_next_prefetch_item(&self) -> Result<Option<PrefetchItem>> {
        let mut queue = self.prefetch_queue.lock()
            .map_err(|_| XTauriError::lock_acquisition("prefetch queue"))?;
        
        // Remove items that are scheduled for the future
        let now = Utc::now();
        if let Some(pos) = queue.iter().position(|item| item.scheduled_at <= now) {
            Ok(Some(queue.remove(pos)))
        } else {
            Ok(None)
        }
    }
    
    /// Get cache policies
    pub fn get_cache_policies(&self) -> &CachePolicies {
        &self.cache_policies
    }
    
    /// Update cache policies
    pub fn update_cache_policies(&mut self, policies: CachePolicies) {
        self.cache_policies = policies;
    }
    
    /// Get detailed cache statistics
    pub fn get_detailed_stats(&self) -> Result<CacheStatistics> {
        let stats = self.cache_stats.lock()
            .map_err(|_| XTauriError::lock_acquisition("cache statistics"))?;
        Ok(stats.clone())
    }
    
    /// Reset cache statistics
    pub fn reset_stats(&self) -> Result<()> {
        let mut stats = self.cache_stats.lock()
            .map_err(|_| XTauriError::lock_acquisition("cache statistics"))?;
        *stats = CacheStatistics::default();
        Ok(())
    }
    
    /// Get cache hit ratio
    pub fn get_hit_ratio(&self) -> Result<f64> {
        let stats = self.cache_stats.lock()
            .map_err(|_| XTauriError::lock_acquisition("cache statistics"))?;
        
        let total = stats.hits + stats.misses;
        if total == 0 {
            Ok(0.0)
        } else {
            Ok(stats.hits as f64 / total as f64)
        }
    }
    
    /// Get prefetch hit ratio
    pub fn get_prefetch_hit_ratio(&self) -> Result<f64> {
        let stats = self.cache_stats.lock()
            .map_err(|_| XTauriError::lock_acquisition("cache statistics"))?;
        
        let total = stats.prefetch_hits + stats.prefetch_misses;
        if total == 0 {
            Ok(0.0)
        } else {
            Ok(stats.prefetch_hits as f64 / total as f64)
        }
    }
    
    /// Perform intelligent cache cleanup based on policies
    pub fn intelligent_cleanup(&self) -> Result<()> {
        // Clean up expired entries first
        self.cleanup_expired()?;
        
        // Enforce cache policies for each content type
        self.enforce_cache_policy("channels")?;
        self.enforce_cache_policy("movies")?;
        self.enforce_cache_policy("series")?;
        self.enforce_cache_policy("epg")?;
        self.enforce_cache_policy("categories")?;
        
        // Update cleanup timestamp
        {
            let mut stats = self.cache_stats.lock()
                .map_err(|_| XTauriError::lock_acquisition("cache statistics"))?;
            stats.last_cleanup = Some(Utc::now());
        }
        
        Ok(())
    }
    
    /// Serialize content for storage
    fn serialize_content<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        serde_json::to_vec(value)
            .map_err(|e| XTauriError::content_cache(format!("Serialization failed: {}", e)))
    }
    
    /// Deserialize content from storage
    fn deserialize_content<T>(&self, data: &[u8]) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        match serde_json::from_slice(data) {
            Ok(value) => Ok(Some(value)),
            Err(e) => Err(XTauriError::content_cache(format!("Deserialization failed: {}", e))),
        }
    }
    
    /// Extract profile ID from cache key (assumes format "profile_id:content_type:...")
    fn extract_profile_id_from_key(&self, key: &str) -> Result<String> {
        key.split(':')
            .next()
            .map(|s| s.to_string())
            .ok_or_else(|| XTauriError::content_cache("Invalid cache key format".to_string()))
    }
    
    /// Extract content type from cache key (assumes format "profile_id:content_type:...")
    fn extract_content_type_from_key(&self, key: &str) -> String {
        key.split(':')
            .nth(1)
            .unwrap_or("unknown")
            .to_string()
    }
    
    /// Get TTL for specific content type
    fn get_ttl_for_content_type(&self, content_type: &str) -> Duration {
        match content_type {
            "channels" => self.cache_policies.channels.ttl,
            "movies" => self.cache_policies.movies.ttl,
            "series" => self.cache_policies.series.ttl,
            "epg" => self.cache_policies.epg.ttl,
            "categories" | "channel_categories" | "movie_categories" | "series_categories" => {
                self.cache_policies.categories.ttl
            }
            _ => self.default_ttl,
        }
    }
    
    /// Get cache policy for specific content type
    fn get_policy_for_content_type(&self, content_type: &str) -> &CachePolicy {
        match content_type {
            "channels" => &self.cache_policies.channels,
            "movies" => &self.cache_policies.movies,
            "series" => &self.cache_policies.series,
            "epg" => &self.cache_policies.epg,
            "categories" | "channel_categories" | "movie_categories" | "series_categories" => {
                &self.cache_policies.categories
            }
            _ => &self.cache_policies.channels, // Default fallback
        }
    }
    
    /// Enforce cache policy for a content type (evict if over limit)
    fn enforce_cache_policy(&self, content_type: &str) -> Result<()> {
        let policy = self.get_policy_for_content_type(content_type);
        
        // Count entries of this content type in memory cache
        let entries: Vec<_> = self.memory_cache
            .iter()
            .filter(|entry| self.extract_content_type_from_key(entry.key()) == content_type)
            .map(|entry| (entry.key().clone(), entry.value().expires_at))
            .collect();
        
        if entries.len() > policy.max_entries {
            // Sort by expiration time (oldest first) and priority
            let mut sorted_entries = entries;
            sorted_entries.sort_by(|a, b| a.1.cmp(&b.1));
            
            // Remove oldest entries
            let to_remove = sorted_entries.len() - policy.max_entries;
            for (key, _) in sorted_entries.iter().take(to_remove) {
                self.memory_cache.remove(key);
                
                // Record eviction
                if let Ok(mut stats) = self.cache_stats.lock() {
                    stats.evictions += 1;
                }
            }
        }
        
        // Also enforce database limits
        let db = self.db.lock()
            .map_err(|_| XTauriError::lock_acquisition("database connection"))?;
        
        let count: i64 = db.query_row(
            "SELECT COUNT(*) FROM xtream_content_cache WHERE content_type = ?",
            [content_type],
            |row| row.get(0),
        )?;
        
        if count as usize > policy.max_entries {
            let to_remove = count as usize - policy.max_entries;
            db.execute(
                "DELETE FROM xtream_content_cache WHERE cache_key IN (
                    SELECT cache_key FROM xtream_content_cache 
                    WHERE content_type = ? 
                    ORDER BY created_at ASC 
                    LIMIT ?
                )",
                (content_type, to_remove),
            )?;
        }
        
        Ok(())
    }
    
    /// Record cache hit in statistics
    fn record_cache_hit(&self, content_type: &str, access_time: Duration) {
        if let Ok(mut stats) = self.cache_stats.lock() {
            stats.hits += 1;
            
            let content_stats = stats.content_type_stats
                .entry(content_type.to_string())
                .or_default();
            content_stats.hits += 1;
            
            // Update average access time (simple moving average)
            if content_stats.hits == 1 {
                content_stats.avg_access_time = access_time;
            } else {
                let total_time = content_stats.avg_access_time.as_nanos() as f64 * (content_stats.hits - 1) as f64;
                let new_avg = (total_time + access_time.as_nanos() as f64) / content_stats.hits as f64;
                content_stats.avg_access_time = Duration::from_nanos(new_avg as u64);
            }
        }
    }
    
    /// Record cache miss in statistics
    fn record_cache_miss(&self, content_type: &str) {
        if let Ok(mut stats) = self.cache_stats.lock() {
            stats.misses += 1;
            
            let content_stats = stats.content_type_stats
                .entry(content_type.to_string())
                .or_default();
            content_stats.misses += 1;
        }
    }
    
    /// Record prefetch hit in statistics
    fn record_prefetch_hit(&self, content_type: &str) {
        if let Ok(mut stats) = self.cache_stats.lock() {
            stats.prefetch_hits += 1;
            
            let content_stats = stats.content_type_stats
                .entry(content_type.to_string())
                .or_default();
            // Prefetch hits are also regular hits
            content_stats.hits += 1;
        }
    }
    
    /// Record prefetch miss in statistics
    fn record_prefetch_miss(&self, _content_type: &str) {
        if let Ok(mut stats) = self.cache_stats.lock() {
            stats.prefetch_misses += 1;
        }
    }
}

/// Cache statistics (legacy, kept for compatibility)
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub memory_entries: usize,
    pub database_entries: usize,
    pub total_size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
    }
    
    fn create_test_db() -> Connection {
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
        ).unwrap();
        
        conn.execute(
            "CREATE TABLE xtream_content_cache (
                cache_key TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                content_type TEXT NOT NULL,
                data BLOB NOT NULL,
                expires_at DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (profile_id) REFERENCES xtream_profiles(id) ON DELETE CASCADE
            )",
            [],
        ).unwrap();
        
        // Insert a test profile
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        ).unwrap();
        
        conn
    }
    
    #[tokio::test]
    async fn test_cache_warming() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        let config = CacheWarmingConfig::default();
        
        // Warm cache for profile
        cache.warm_cache_for_profile("test-profile", &config).await.unwrap();
        
        // Check that items were added to prefetch queue
        let queue = cache.prefetch_queue.lock().unwrap();
        assert!(!queue.is_empty());
        
        // Check that categories are prioritized
        let has_high_priority = queue.iter().any(|item| item.priority == CachePriority::High);
        assert!(has_high_priority);
    }
    
    #[test]
    fn test_content_type_specific_ttl() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        // Test different content types get different TTLs
        assert_eq!(cache.get_ttl_for_content_type("channels"), Duration::from_secs(3600));
        assert_eq!(cache.get_ttl_for_content_type("movies"), Duration::from_secs(7200));
        assert_eq!(cache.get_ttl_for_content_type("epg"), Duration::from_secs(1800));
        assert_eq!(cache.get_ttl_for_content_type("categories"), Duration::from_secs(14400));
    }
    
    #[test]
    fn test_cache_policy_enforcement() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let mut policies = CachePolicies::default();
        policies.channels.max_entries = 2; // Set low limit for testing
        
        let cache = ContentCache::with_policies(db, Duration::from_secs(3600), policies);
        
        let test_data = TestData {
            id: 1,
            name: "Test Item".to_string(),
        };
        
        // Add more entries than the limit
        cache.set("test-profile:channels:1", &test_data, None).unwrap();
        cache.set("test-profile:channels:2", &test_data, None).unwrap();
        cache.set("test-profile:channels:3", &test_data, None).unwrap();
        
        // Enforce policy
        cache.enforce_cache_policy("channels").unwrap();
        
        // Should have evicted oldest entries
        let channel_entries: Vec<_> = cache.memory_cache
            .iter()
            .filter(|entry| cache.extract_content_type_from_key(entry.key()) == "channels")
            .collect();
        
        assert!(channel_entries.len() <= 2);
    }
    
    #[test]
    fn test_prefetch_queue() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        let item1 = PrefetchItem {
            profile_id: "test-profile".to_string(),
            content_type: "channels".to_string(),
            identifier: None,
            priority: CachePriority::Low,
            scheduled_at: Utc::now(),
        };
        
        let item2 = PrefetchItem {
            profile_id: "test-profile".to_string(),
            content_type: "categories".to_string(),
            identifier: None,
            priority: CachePriority::High,
            scheduled_at: Utc::now(),
        };
        
        // Add items to queue
        cache.schedule_prefetch(item1).unwrap();
        cache.schedule_prefetch(item2).unwrap();
        
        // High priority item should come first
        let next_item = cache.get_next_prefetch_item().unwrap().unwrap();
        assert_eq!(next_item.priority, CachePriority::High);
        assert_eq!(next_item.content_type, "categories");
    }
    
    #[test]
    fn test_cache_statistics() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        let test_data = TestData {
            id: 1,
            name: "Test Item".to_string(),
        };
        
        // Set and get data to generate stats
        cache.set("test-profile:channels:1", &test_data, None).unwrap();
        let _: Option<TestData> = cache.get("test-profile:channels:1").unwrap();
        let _: Option<TestData> = cache.get("test-profile:channels:nonexistent").unwrap();
        
        // Check statistics
        let hit_ratio = cache.get_hit_ratio().unwrap();
        assert!(hit_ratio > 0.0 && hit_ratio <= 1.0);
        
        let detailed_stats = cache.get_detailed_stats().unwrap();
        assert!(detailed_stats.hits > 0);
        assert!(detailed_stats.misses > 0);
        assert!(detailed_stats.content_type_stats.contains_key("channels"));
    }
    
    #[test]
    fn test_intelligent_cleanup() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        let test_data = TestData {
            id: 1,
            name: "Test Item".to_string(),
        };
        
        // Add some data
        cache.set("test-profile:channels:1", &test_data, Some(Duration::from_millis(1))).unwrap();
        cache.set("test-profile:movies:1", &test_data, None).unwrap();
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(10));
        
        // Run intelligent cleanup
        cache.intelligent_cleanup().unwrap();
        
        // Expired entry should be gone
        let retrieved: Option<TestData> = cache.get("test-profile:channels:1").unwrap();
        assert_eq!(retrieved, None);
        
        // Non-expired entry should still exist
        let retrieved: Option<TestData> = cache.get("test-profile:movies:1").unwrap();
        assert_eq!(retrieved, Some(test_data));
        
        // Check that cleanup timestamp was updated
        let stats = cache.get_detailed_stats().unwrap();
        assert!(stats.last_cleanup.is_some());
    }
    
    #[test]
    fn test_cache_set_and_get() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        let test_data = TestData {
            id: 1,
            name: "Test Item".to_string(),
        };
        
        let key = "test-profile:channels:all";
        
        // Set data in cache
        cache.set(key, &test_data, None).unwrap();
        
        // Get data from cache
        let retrieved: Option<TestData> = cache.get(key).unwrap();
        assert_eq!(retrieved, Some(test_data));
    }
    
    #[test]
    fn test_cache_expiration() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_millis(100));
        
        let test_data = TestData {
            id: 1,
            name: "Test Item".to_string(),
        };
        
        let key = "test-profile:channels:all";
        
        // Set data with short TTL
        cache.set(key, &test_data, Some(Duration::from_millis(1))).unwrap();
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(50));
        
        // Run cleanup to remove expired entries
        cache.cleanup_expired().unwrap();
        
        // Should be expired and cleaned up
        let retrieved: Option<TestData> = cache.get(key).unwrap();
        assert_eq!(retrieved, None);
    }
    
    #[test]
    fn test_cache_invalidation() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        // Insert another test profile
        {
            let db_ref = cache.db.lock().unwrap();
            db_ref.execute(
                "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) VALUES ('other-profile', 'Other', 'http://other.com', 'user', X'00')",
                [],
            ).unwrap();
        }
        
        let test_data = TestData {
            id: 1,
            name: "Test Item".to_string(),
        };
        
        // Set multiple cache entries
        cache.set("test-profile:channels:all", &test_data, None).unwrap();
        cache.set("test-profile:movies:all", &test_data, None).unwrap();
        cache.set("other-profile:channels:all", &test_data, None).unwrap();
        
        // Invalidate entries containing "channels"
        cache.invalidate("channels").unwrap();
        
        // Channels entries should be gone
        let retrieved: Option<TestData> = cache.get("test-profile:channels:all").unwrap();
        assert_eq!(retrieved, None);
        
        let retrieved: Option<TestData> = cache.get("other-profile:channels:all").unwrap();
        assert_eq!(retrieved, None);
        
        // Movies entry should still exist
        let retrieved: Option<TestData> = cache.get("test-profile:movies:all").unwrap();
        assert_eq!(retrieved, Some(test_data));
    }
    
    #[test]
    fn test_clear_profile_cache() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        // Insert another test profile
        {
            let db_ref = cache.db.lock().unwrap();
            db_ref.execute(
                "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) VALUES ('other-profile', 'Other', 'http://other.com', 'user', X'00')",
                [],
            ).unwrap();
        }
        
        let test_data = TestData {
            id: 1,
            name: "Test Item".to_string(),
        };
        
        // Set cache entries for different profiles
        cache.set("test-profile:channels:all", &test_data, None).unwrap();
        cache.set("test-profile:movies:all", &test_data, None).unwrap();
        cache.set("other-profile:channels:all", &test_data, None).unwrap();
        
        // Clear cache for test-profile
        cache.clear_profile_cache("test-profile").unwrap();
        
        // test-profile entries should be gone
        let retrieved: Option<TestData> = cache.get("test-profile:channels:all").unwrap();
        assert_eq!(retrieved, None);
        
        let retrieved: Option<TestData> = cache.get("test-profile:movies:all").unwrap();
        assert_eq!(retrieved, None);
        
        // other-profile entry should still exist
        let retrieved: Option<TestData> = cache.get("other-profile:channels:all").unwrap();
        assert_eq!(retrieved, Some(test_data));
    }
    
    #[test]
    fn test_cache_stats() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        let test_data = TestData {
            id: 1,
            name: "Test Item".to_string(),
        };
        
        // Initially empty
        let stats = cache.get_stats().unwrap();
        assert_eq!(stats.memory_entries, 0);
        assert_eq!(stats.database_entries, 0);
        
        // Add some entries
        cache.set("test-profile:channels:all", &test_data, None).unwrap();
        cache.set("test-profile:movies:all", &test_data, None).unwrap();
        
        let stats = cache.get_stats().unwrap();
        assert_eq!(stats.memory_entries, 2);
        assert_eq!(stats.database_entries, 2);
        assert!(stats.total_size_bytes > 0);
    }
    
    #[test]
    fn test_cleanup_expired() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        let test_data = TestData {
            id: 1,
            name: "Test Item".to_string(),
        };
        
        // Set one entry with short TTL and one with long TTL
        cache.set("test-profile:channels:short", &test_data, Some(Duration::from_millis(1))).unwrap();
        cache.set("test-profile:channels:long", &test_data, Some(Duration::from_secs(3600))).unwrap();
        
        // Wait for short TTL to expire
        std::thread::sleep(Duration::from_millis(50));
        
        // Cleanup expired entries
        cache.cleanup_expired().unwrap();
        
        // Check memory cache directly - should be empty for expired entry
        assert!(!cache.memory_cache.contains_key("test-profile:channels:short"));
        
        // Check database directly - should also be empty for expired entry
        {
            let db_ref = cache.db.lock().unwrap();
            let count: i64 = db_ref.query_row(
                "SELECT COUNT(*) FROM xtream_content_cache WHERE cache_key = ? AND expires_at > datetime('now')",
                ["test-profile:channels:short"],
                |row| row.get(0),
            ).unwrap();
            assert_eq!(count, 0);
        }
        
        // Long TTL entry should still exist
        let retrieved: Option<TestData> = cache.get("test-profile:channels:long").unwrap();
        assert_eq!(retrieved, Some(test_data));
    }
    
    #[test]
    fn test_cache_key_generation() {
        let db = Arc::new(Mutex::new(create_test_db()));
        let cache = ContentCache::new(db, Duration::from_secs(3600));
        
        let cache_key = CacheKey::new(
            "profile123".to_string(),
            "channels".to_string(),
            Some("category1".to_string()),
        );
        
        let key = cache.generate_key(&cache_key);
        assert_eq!(key, "profile123:channels:category1");
        
        let cache_key_no_id = CacheKey::new(
            "profile123".to_string(),
            "channels".to_string(),
            None,
        );
        
        let key = cache.generate_key(&cache_key_no_id);
        assert_eq!(key, "profile123:channels");
    }
}