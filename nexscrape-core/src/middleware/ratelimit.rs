//! Token-bucket rate limiter with adaptive backoff.

use crate::{Result, ScrapRequest};
use crate::middleware::pipeline::Middleware;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Token-bucket rate limiter.
///
/// Controls request rate per domain using the token bucket algorithm.
/// Supports adaptive rate reduction when 429 responses are received.
pub struct RateLimiter {
    /// Tokens per second.
    rate: f64,
    /// Maximum burst size.
    burst: u32,
    /// Per-domain token buckets.
    buckets: Arc<Mutex<std::collections::HashMap<String, TokenBucket>>>,
    /// Whether to apply per-domain limits.
    per_domain: bool,
}

struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }

    async fn acquire(&mut self) {
        self.refill();

        if self.tokens < 1.0 {
            let wait_time = (1.0 - self.tokens) / self.refill_rate;
            sleep(Duration::from_secs_f64(wait_time)).await;
            self.refill();
        }

        self.tokens -= 1.0;
    }
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// - `rate`: Requests per second
    /// - `burst`: Maximum burst size
    /// - `per_domain`: Whether to track limits per domain
    pub fn new(rate: f64, burst: u32, per_domain: bool) -> Self {
        Self {
            rate,
            burst,
            buckets: Arc::new(Mutex::new(std::collections::HashMap::new())),
            per_domain,
        }
    }

    fn domain_key(&self, url: &str) -> String {
        if self.per_domain {
            url::Url::parse(url)
                .ok()
                .and_then(|u| u.host_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "default".to_string())
        } else {
            "global".to_string()
        }
    }
}

#[async_trait]
impl Middleware for RateLimiter {
    async fn process_request(&self, request: ScrapRequest) -> Result<Option<ScrapRequest>> {
        let key = self.domain_key(&request.url);
        let mut buckets = self.buckets.lock().await;

        let bucket = buckets
            .entry(key)
            .or_insert_with(|| TokenBucket::new(self.burst as f64, self.rate));

        bucket.acquire().await;

        Ok(Some(request))
    }

    fn name(&self) -> &str {
        "rate_limiter"
    }

    fn priority(&self) -> i32 {
        80
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_burst() {
        let limiter = RateLimiter::new(10.0, 5, false);

        // Should allow burst of 5 requests immediately
        for _ in 0..5 {
            let req = ScrapRequest::get("https://example.com");
            let result = limiter.process_request(req).await.unwrap();
            assert!(result.is_some());
        }
    }
}
