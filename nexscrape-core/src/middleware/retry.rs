//! Retry policy with exponential backoff and jitter.

use crate::{Result, ScrapResponse};
use crate::middleware::pipeline::Middleware;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Retry policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Initial backoff delay in milliseconds.
    pub initial_backoff_ms: u64,
    /// Maximum backoff delay in milliseconds.
    pub max_backoff_ms: u64,
    /// Backoff multiplier.
    pub multiplier: f64,
    /// Whether to add random jitter.
    pub jitter: bool,
    /// HTTP status codes to retry on.
    pub retry_on_status: Vec<u16>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 30000,
            multiplier: 2.0,
            jitter: true,
            retry_on_status: vec![429, 500, 502, 503, 504, 408],
        }
    }
}

impl RetryPolicy {
    /// Calculate the backoff duration for a given attempt.
    pub fn backoff_duration(&self, attempt: u32) -> Duration {
        let base = self.initial_backoff_ms as f64 * self.multiplier.powi(attempt as i32);
        let capped = base.min(self.max_backoff_ms as f64);

        let duration = if self.jitter {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let jitter = rng.gen_range(0.0..=capped);
            (capped + jitter) / 2.0
        } else {
            capped
        };

        Duration::from_millis(duration as u64)
    }

    /// Check if a given status code should trigger a retry.
    pub fn should_retry_status(&self, status: u16) -> bool {
        self.retry_on_status.contains(&status)
    }
}

/// Retry middleware that wraps the retry policy.
pub struct RetryMiddleware {
    policy: RetryPolicy,
}

impl RetryMiddleware {
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl Middleware for RetryMiddleware {
    async fn process_response(&self, response: ScrapResponse) -> Result<Option<ScrapResponse>> {
        if self.policy.should_retry_status(response.status) {
            tracing::warn!(
                url = %response.url,
                status = response.status,
                "Retryable status code received"
            );
            // In a full implementation, this would re-enqueue the request.
            // For now, we pass it through and let the engine handle retries.
        }
        Ok(Some(response))
    }

    fn name(&self) -> &str {
        "retry"
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_increases() {
        let policy = RetryPolicy {
            jitter: false,
            ..Default::default()
        };

        let d0 = policy.backoff_duration(0);
        let d1 = policy.backoff_duration(1);
        let d2 = policy.backoff_duration(2);

        assert!(d1 > d0);
        assert!(d2 > d1);
    }

    #[test]
    fn test_backoff_capped() {
        let policy = RetryPolicy {
            initial_backoff_ms: 1000,
            max_backoff_ms: 5000,
            multiplier: 10.0,
            jitter: false,
            ..Default::default()
        };

        let d = policy.backoff_duration(10);
        assert!(d.as_millis() <= 5000);
    }

    #[test]
    fn test_retry_on_status() {
        let policy = RetryPolicy::default();
        assert!(policy.should_retry_status(429));
        assert!(policy.should_retry_status(503));
        assert!(!policy.should_retry_status(200));
        assert!(!policy.should_retry_status(404));
    }
}
