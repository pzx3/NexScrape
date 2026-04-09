//! Proxy pool management with rotation strategies.

use crate::{Result, ScrapRequest};
use crate::middleware::pipeline::Middleware;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Proxy rotation strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationStrategy {
    /// Round-robin rotation.
    RoundRobin,
    /// Random proxy selection.
    Random,
    /// Sticky session — same proxy per domain.
    StickySession,
}

/// Proxy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub protocol: String,
}

impl ProxyConfig {
    /// Create a new proxy config from a URL string.
    ///
    /// Supports formats:
    /// - `http://host:port`
    /// - `http://user:pass@host:port`
    /// - `socks5://host:port`
    pub fn from_url(url: impl Into<String>) -> Self {
        let url = url.into();
        Self {
            protocol: if url.starts_with("socks5") {
                "socks5".into()
            } else {
                "http".into()
            },
            url,
            username: None,
            password: None,
        }
    }
}

/// Manages a pool of proxies with rotation and health checking.
pub struct ProxyPool {
    proxies: Arc<RwLock<Vec<ProxyConfig>>>,
    healthy: Arc<RwLock<Vec<bool>>>,
    current_index: AtomicUsize,
    strategy: RotationStrategy,
    domain_map: Arc<RwLock<std::collections::HashMap<String, usize>>>,
}

impl ProxyPool {
    /// Create a new proxy pool.
    pub fn new(proxies: Vec<ProxyConfig>, strategy: RotationStrategy) -> Self {
        let len = proxies.len();
        Self {
            proxies: Arc::new(RwLock::new(proxies)),
            healthy: Arc::new(RwLock::new(vec![true; len])),
            current_index: AtomicUsize::new(0),
            strategy,
            domain_map: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create a proxy pool from URL strings.
    pub fn from_urls(urls: Vec<&str>, strategy: RotationStrategy) -> Self {
        let proxies = urls.into_iter().map(ProxyConfig::from_url).collect();
        Self::new(proxies, strategy)
    }

    /// Get the next proxy based on the rotation strategy.
    pub async fn next_proxy(&self, domain: Option<&str>) -> Option<ProxyConfig> {
        let proxies = self.proxies.read().await;
        let healthy = self.healthy.read().await;

        if proxies.is_empty() {
            return None;
        }

        let index = match self.strategy {
            RotationStrategy::RoundRobin => {
                let idx = self.current_index.fetch_add(1, Ordering::Relaxed) % proxies.len();
                // Find next healthy proxy
                for i in 0..proxies.len() {
                    let check = (idx + i) % proxies.len();
                    if healthy[check] {
                        return Some(proxies[check].clone());
                    }
                }
                return None;
            }
            RotationStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let healthy_indices: Vec<usize> = healthy
                    .iter()
                    .enumerate()
                    .filter(|(_, h)| **h)
                    .map(|(i, _)| i)
                    .collect();
                if healthy_indices.is_empty() {
                    return None;
                }
                healthy_indices[rng.gen_range(0..healthy_indices.len())]
            }
            RotationStrategy::StickySession => {
                if let Some(domain) = domain {
                    let mut map = self.domain_map.write().await;
                    if let Some(&idx) = map.get(domain) {
                        if healthy[idx] {
                            return Some(proxies[idx].clone());
                        }
                    }
                    // Assign new proxy for this domain
                    let idx = self.current_index.fetch_add(1, Ordering::Relaxed) % proxies.len();
                    map.insert(domain.to_string(), idx);
                    idx
                } else {
                    self.current_index.fetch_add(1, Ordering::Relaxed) % proxies.len()
                }
            }
        };

        Some(proxies[index].clone())
    }

    /// Mark a proxy as unhealthy.
    pub async fn mark_unhealthy(&self, proxy_url: &str) {
        let proxies = self.proxies.read().await;
        let mut healthy = self.healthy.write().await;

        for (i, p) in proxies.iter().enumerate() {
            if p.url == proxy_url {
                healthy[i] = false;
                tracing::warn!(proxy = %proxy_url, "Proxy marked as unhealthy");
                break;
            }
        }
    }

    /// Get count of healthy proxies.
    pub async fn healthy_count(&self) -> usize {
        self.healthy.read().await.iter().filter(|h| **h).count()
    }

    /// Get total proxy count.
    pub async fn total_count(&self) -> usize {
        self.proxies.read().await.len()
    }
}

#[async_trait]
impl Middleware for ProxyPool {
    async fn process_request(&self, mut request: ScrapRequest) -> Result<Option<ScrapRequest>> {
        let domain = url::Url::parse(&request.url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()));

        if let Some(proxy) = self.next_proxy(domain.as_deref()).await {
            request.meta.insert(
                "_proxy".into(),
                serde_json::Value::String(proxy.url.clone()),
            );
            tracing::debug!(proxy = %proxy.url, url = %request.url, "Assigned proxy");
        } else {
            tracing::warn!("No healthy proxies available");
        }

        Ok(Some(request))
    }

    async fn process_error(&self, request: &ScrapRequest, _error: &crate::NexError) {
        if let Some(proxy_url) = request.meta.get("_proxy").and_then(|v| v.as_str()) {
            self.mark_unhealthy(proxy_url).await;
        }
    }

    fn name(&self) -> &str {
        "proxy_pool"
    }

    fn priority(&self) -> i32 {
        90
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_round_robin() {
        let pool = ProxyPool::from_urls(
            vec!["http://proxy1:8080", "http://proxy2:8080"],
            RotationStrategy::RoundRobin,
        );

        let p1 = pool.next_proxy(None).await.unwrap();
        let p2 = pool.next_proxy(None).await.unwrap();
        assert_ne!(p1.url, p2.url);
    }

    #[tokio::test]
    async fn test_sticky_session() {
        let pool = ProxyPool::from_urls(
            vec!["http://proxy1:8080", "http://proxy2:8080"],
            RotationStrategy::StickySession,
        );

        let p1 = pool.next_proxy(Some("example.com")).await.unwrap();
        let p2 = pool.next_proxy(Some("example.com")).await.unwrap();
        assert_eq!(p1.url, p2.url); // Same domain = same proxy
    }

    #[tokio::test]
    async fn test_mark_unhealthy() {
        let pool = ProxyPool::from_urls(
            vec!["http://proxy1:8080"],
            RotationStrategy::RoundRobin,
        );

        assert_eq!(pool.healthy_count().await, 1);
        pool.mark_unhealthy("http://proxy1:8080").await;
        assert_eq!(pool.healthy_count().await, 0);
    }
}
