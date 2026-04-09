//! Response caching with LRU eviction.

use crate::{Result, ScrapRequest, ScrapResponse};
use crate::middleware::pipeline::Middleware;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Cache configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of cached responses.
    pub max_entries: usize,
    /// Time-to-live for cache entries in seconds.
    pub ttl_secs: u64,
    /// Whether to cache only successful responses.
    pub only_success: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            ttl_secs: 3600,
            only_success: true,
        }
    }
}

struct CacheEntry {
    response_body: bytes::Bytes,
    status: u16,
    headers: HashMap<String, String>,
    inserted_at: Instant,
}

/// In-memory LRU response cache.
pub struct Cache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    config: CacheConfig,
}

impl Cache {
    /// Create a new cache with the given configuration.
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get a cached response if available and not expired.
    pub async fn get(&self, url: &str) -> Option<(bytes::Bytes, u16, HashMap<String, String>)> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(url) {
            let ttl = Duration::from_secs(self.config.ttl_secs);
            if entry.inserted_at.elapsed() < ttl {
                tracing::debug!(url = %url, "Cache hit");
                return Some((
                    entry.response_body.clone(),
                    entry.status,
                    entry.headers.clone(),
                ));
            }
        }
        None
    }

    /// Store a response in the cache.
    pub async fn put(&self, url: &str, body: bytes::Bytes, status: u16, headers: HashMap<String, String>) {
        let mut entries = self.entries.write().await;

        // Evict if full (simple: remove oldest)
        if entries.len() >= self.config.max_entries {
            if let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, v)| v.inserted_at)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest_key);
            }
        }

        entries.insert(
            url.to_string(),
            CacheEntry {
                response_body: body,
                status,
                headers,
                inserted_at: Instant::now(),
            },
        );
    }

    /// Get total number of cached entries.
    pub async fn size(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Clear all cached entries.
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }
}

#[async_trait]
impl Middleware for Cache {
    async fn process_request(&self, request: ScrapRequest) -> Result<Option<ScrapRequest>> {
        // Check cache before sending request
        if let Some((_body, _status, _headers)) = self.get(&request.url).await {
            tracing::debug!(url = %request.url, "Serving from cache");
            // In a full impl, we'd short-circuit the request here.
            // For now, the cache is checked externally.
        }
        Ok(Some(request))
    }

    async fn process_response(&self, response: ScrapResponse) -> Result<Option<ScrapResponse>> {
        if !self.config.only_success || response.is_success() {
            self.put(
                &response.url,
                response.body.clone(),
                response.status,
                response.headers.clone(),
            )
            .await;
        }
        Ok(Some(response))
    }

    fn name(&self) -> &str {
        "cache"
    }

    fn priority(&self) -> i32 {
        110 // Run very early (before fingerprint, proxy, etc.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_put_and_get() {
        let cache = Cache::new(CacheConfig::default());
        let body = bytes::Bytes::from("hello");
        let headers = HashMap::new();

        cache.put("https://example.com", body.clone(), 200, headers).await;

        let result = cache.get("https://example.com").await;
        assert!(result.is_some());
        let (cached_body, status, _) = result.unwrap();
        assert_eq!(cached_body, body);
        assert_eq!(status, 200);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = Cache::new(CacheConfig::default());
        assert!(cache.get("https://notcached.com").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let cache = Cache::new(CacheConfig {
            max_entries: 2,
            ..Default::default()
        });

        cache.put("https://a.com", bytes::Bytes::new(), 200, HashMap::new()).await;
        cache.put("https://b.com", bytes::Bytes::new(), 200, HashMap::new()).await;
        cache.put("https://c.com", bytes::Bytes::new(), 200, HashMap::new()).await;

        assert_eq!(cache.size().await, 2);
    }
}
