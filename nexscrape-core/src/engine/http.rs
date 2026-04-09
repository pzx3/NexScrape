//! High-performance async HTTP engine with connection pooling and HTTP/2 support.

use crate::{NexError, Result, ScrapRequest, ScrapResponse};
use reqwest::{Client, ClientBuilder, Method};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Configuration for the HTTP engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Maximum number of concurrent requests.
    pub concurrency: usize,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum idle connections per host.
    pub pool_idle_per_host: usize,
    /// TCP keepalive interval in seconds.
    pub tcp_keepalive_secs: u64,
    /// Default User-Agent string.
    pub user_agent: String,
    /// Whether to follow redirects.
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow.
    pub max_redirects: usize,
    /// Whether to accept invalid TLS certificates (use with caution).
    pub danger_accept_invalid_certs: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            concurrency: 16,
            timeout_secs: 30,
            pool_idle_per_host: 10,
            tcp_keepalive_secs: 30,
            user_agent: format!(
                "NexScrape/{} (+https://github.com/nexscrape/nexscrape)",
                env!("CARGO_PKG_VERSION")
            ),
            follow_redirects: true,
            max_redirects: 10,
            danger_accept_invalid_certs: false,
        }
    }
}

impl From<&crate::NexConfig> for EngineConfig {
    fn from(config: &crate::NexConfig) -> Self {
        Self {
            concurrency: config.concurrency,
            timeout_secs: config.timeout_secs,
            user_agent: config.user_agent.clone(),
            follow_redirects: config.follow_redirects,
            max_redirects: config.max_redirects,
            ..Default::default()
        }
    }
}

/// High-performance async HTTP engine.
///
/// Uses connection pooling, HTTP/2 multiplexing, and a semaphore-based
/// concurrency limiter to achieve maximum throughput without overwhelming
/// target servers.
pub struct HttpEngine {
    client: Client,
    semaphore: Arc<Semaphore>,
    config: EngineConfig,
}

impl HttpEngine {
    /// Create a new HTTP engine with the given configuration.
    pub fn new(config: EngineConfig) -> Result<Self> {
        let mut builder = ClientBuilder::new()
            .tcp_keepalive(Duration::from_secs(config.tcp_keepalive_secs))
            .pool_max_idle_per_host(config.pool_idle_per_host)
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .gzip(true)
            .brotli(true)
            .deflate(true);

        if config.follow_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::limited(config.max_redirects));
        } else {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        }

        if config.danger_accept_invalid_certs {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build()?;

        info!(
            concurrency = config.concurrency,
            timeout = config.timeout_secs,
            "HttpEngine initialized"
        );

        Ok(Self {
            client,
            semaphore: Arc::new(Semaphore::new(config.concurrency)),
            config,
        })
    }

    /// Execute a single scraping request.
    pub async fn execute(&self, request: ScrapRequest) -> Result<ScrapResponse> {
        // Acquire concurrency permit
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            NexError::Other(format!("Semaphore error: {}", e))
        })?;

        debug!(url = %request.url, method = %request.method, "Executing request");

        let method: Method = request.method.parse().map_err(|_| {
            NexError::Other(format!("Invalid HTTP method: {}", request.method))
        })?;

        let mut req_builder = self.client.request(method, &request.url);

        // Apply custom headers
        for (key, value) in &request.headers {
            req_builder = req_builder.header(key.as_str(), value.as_str());
        }

        // Apply body if present
        if let Some(ref body) = request.body {
            req_builder = req_builder.body(body.clone());
        }

        let response = req_builder.send().await?;

        let status = response.status().as_u16();
        let url = response.url().to_string();

        // Convert response headers
        let mut headers = std::collections::HashMap::new();
        for (key, value) in response.headers().iter() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        // Check for rate limiting
        if status == 429 {
            let retry_after = headers
                .get("retry-after")
                .and_then(|v| v.parse::<u64>().ok());
            warn!(url = %url, retry_after = ?retry_after, "Rate limited");
            return Err(NexError::RateLimited { status, retry_after });
        }

        let body = response.bytes().await?;

        debug!(
            url = %url,
            status = status,
            body_size = body.len(),
            "Request completed"
        );

        Ok(ScrapResponse {
            url,
            status,
            headers,
            body,
            request,
        })
    }

    /// Execute multiple requests in parallel.
    pub async fn execute_batch(
        &self,
        requests: Vec<ScrapRequest>,
    ) -> Vec<Result<ScrapResponse>> {
        let futures: Vec<_> = requests
            .into_iter()
            .map(|req| self.execute(req))
            .collect();

        futures::future::join_all(futures).await
    }

    /// Get current engine configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Get the number of available permits (= free concurrency slots).
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert_eq!(config.concurrency, 16);
        assert_eq!(config.timeout_secs, 30);
        assert!(config.follow_redirects);
    }

    #[test]
    fn test_engine_creation() {
        let config = EngineConfig::default();
        let engine = HttpEngine::new(config).unwrap();
        assert_eq!(engine.available_permits(), 16);
    }

    #[test]
    fn test_scrap_request_builder() {
        let req = ScrapRequest::get("https://example.com")
            .header("Accept", "text/html")
            .priority(5);

        assert_eq!(req.url, "https://example.com");
        assert_eq!(req.method, "GET");
        assert_eq!(req.priority, 5);
        assert_eq!(req.headers.get("Accept").unwrap(), "text/html");
    }
}
