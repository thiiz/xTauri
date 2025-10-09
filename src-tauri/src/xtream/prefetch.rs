use crate::error::Result;
use crate::xtream::content_cache::{ContentCache, PrefetchItem, CachePriority};
use crate::xtream::xtream_client::XtreamClient;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use chrono::Utc;

/// Intelligent prefetching system for Xtream content
pub struct PrefetchManager {
    cache: Arc<ContentCache>,
    prefetch_interval: Duration,
    max_concurrent_prefetch: usize,
}

impl PrefetchManager {
    /// Create a new prefetch manager
    pub fn new(cache: Arc<ContentCache>) -> Self {
        Self {
            cache,
            prefetch_interval: Duration::from_secs(5),
            max_concurrent_prefetch: 3,
        }
    }

    /// Create a new prefetch manager with custom settings
    pub fn with_settings(
        cache: Arc<ContentCache>,
        prefetch_interval: Duration,
        max_concurrent_prefetch: usize,
    ) -> Self {
        Self {
            cache,
            prefetch_interval,
            max_concurrent_prefetch,
        }
    }

    /// Start the prefetch worker
    pub async fn start_prefetch_worker(&self, client: Arc<XtreamClient>) -> Result<()> {
        loop {
            // Get next items to prefetch
            let mut items_to_prefetch = Vec::new();
            for _ in 0..self.max_concurrent_prefetch {
                if let Ok(Some(item)) = self.cache.get_next_prefetch_item() {
                    items_to_prefetch.push(item);
                } else {
                    break;
                }
            }

            if items_to_prefetch.is_empty() {
                // No items to prefetch, wait before checking again
                sleep(self.prefetch_interval).await;
                continue;
            }

            // Prefetch items concurrently
            let prefetch_tasks: Vec<_> = items_to_prefetch
                .into_iter()
                .map(|item| {
                    let client = client.clone();
                    tokio::spawn(async move {
                        Self::prefetch_item(&client, &item).await
                    })
                })
                .collect();

            // Wait for all prefetch tasks to complete
            for task in prefetch_tasks {
                let _ = task.await;
            }

            // Wait before next prefetch cycle
            sleep(self.prefetch_interval).await;
        }
    }

    /// Prefetch a single item
    async fn prefetch_item(client: &XtreamClient, item: &PrefetchItem) -> Result<()> {
        match item.content_type.as_str() {
            "channel_categories" => {
                let _ = client.get_channel_categories().await;
            }
            "movie_categories" => {
                let _ = client.get_movie_categories().await;
            }
            "series_categories" => {
                let _ = client.get_series_categories().await;
            }
            "channels" => {
                let category_id = item.identifier.as_deref();
                let _ = client.get_channels(category_id).await;
            }
            "movies" => {
                let category_id = item.identifier.as_deref();
                let _ = client.get_movies(category_id).await;
            }
            "series" => {
                let category_id = item.identifier.as_deref();
                let _ = client.get_series(category_id).await;
            }
            "epg" => {
                if let Some(channel_id) = &item.identifier {
                    let _ = client.get_short_epg(channel_id).await;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Schedule prefetch based on user behavior patterns
    pub fn schedule_intelligent_prefetch(
        &self,
        profile_id: &str,
        recent_content_types: &[String],
        recent_categories: &[String],
    ) -> Result<()> {
        // Prefetch categories for recently accessed content types
        for content_type in recent_content_types {
            let category_type = match content_type.as_str() {
                "channel" => "channel_categories",
                "movie" => "movie_categories",
                "series" => "series_categories",
                _ => continue,
            };

            self.cache.schedule_prefetch(PrefetchItem {
                profile_id: profile_id.to_string(),
                content_type: category_type.to_string(),
                identifier: None,
                priority: CachePriority::High,
                scheduled_at: Utc::now(),
            })?;
        }

        // Prefetch content for recently accessed categories
        for (content_type, category_id) in recent_content_types.iter().zip(recent_categories.iter()) {
            self.cache.schedule_prefetch(PrefetchItem {
                profile_id: profile_id.to_string(),
                content_type: content_type.clone(),
                identifier: Some(category_id.clone()),
                priority: CachePriority::Medium,
                scheduled_at: Utc::now(),
            })?;
        }

        Ok(())
    }

    /// Schedule prefetch for EPG data of frequently watched channels
    pub fn schedule_epg_prefetch(
        &self,
        profile_id: &str,
        channel_ids: &[String],
    ) -> Result<()> {
        for channel_id in channel_ids {
            self.cache.schedule_prefetch(PrefetchItem {
                profile_id: profile_id.to_string(),
                content_type: "epg".to_string(),
                identifier: Some(channel_id.clone()),
                priority: CachePriority::High,
                scheduled_at: Utc::now(),
            })?;
        }

        Ok(())
    }

    /// Schedule prefetch for content details based on browsing patterns
    pub fn schedule_detail_prefetch(
        &self,
        profile_id: &str,
        content_type: &str,
        content_ids: &[String],
    ) -> Result<()> {
        let detail_type = match content_type {
            "movie" => "movie_info",
            "series" => "series_info",
            _ => return Ok(()),
        };

        for content_id in content_ids {
            self.cache.schedule_prefetch(PrefetchItem {
                profile_id: profile_id.to_string(),
                content_type: detail_type.to_string(),
                identifier: Some(content_id.clone()),
                priority: CachePriority::Low,
                scheduled_at: Utc::now(),
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xtream::content_cache::ContentCache;
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn create_test_cache() -> Arc<ContentCache> {
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
        
        conn.execute(
            "INSERT INTO xtream_profiles (id, name, url, username, encrypted_credentials) 
             VALUES ('test-profile', 'Test', 'http://test.com', 'user', X'00')",
            [],
        ).unwrap();
        
        Arc::new(ContentCache::new(
            Arc::new(Mutex::new(conn)),
            Duration::from_secs(3600),
        ))
    }

    #[test]
    fn test_schedule_intelligent_prefetch() {
        let cache = create_test_cache();
        let manager = PrefetchManager::new(cache.clone());

        let recent_types = vec!["channel".to_string(), "movie".to_string()];
        let recent_categories = vec!["sports".to_string(), "action".to_string()];

        let result = manager.schedule_intelligent_prefetch(
            "test-profile",
            &recent_types,
            &recent_categories,
        );

        assert!(result.is_ok());

        // Verify items were scheduled
        let item = cache.get_next_prefetch_item().unwrap();
        assert!(item.is_some());
    }

    #[test]
    fn test_schedule_epg_prefetch() {
        let cache = create_test_cache();
        let manager = PrefetchManager::new(cache.clone());

        let channel_ids = vec!["123".to_string(), "456".to_string()];

        let result = manager.schedule_epg_prefetch("test-profile", &channel_ids);

        assert!(result.is_ok());

        // Verify items were scheduled
        let item = cache.get_next_prefetch_item().unwrap();
        assert!(item.is_some());
        assert_eq!(item.unwrap().content_type, "epg");
    }
}
