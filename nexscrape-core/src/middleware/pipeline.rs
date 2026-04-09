//! Middleware pipeline for request/response processing.

use crate::{Result, ScrapRequest, ScrapResponse};
use async_trait::async_trait;
use std::sync::Arc;

/// Trait for middleware components in the processing pipeline.
///
/// Middlewares can modify requests before they are sent and/or
/// modify responses after they are received.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process a request before it is sent.
    /// Return `None` to drop the request.
    async fn process_request(&self, request: ScrapRequest) -> Result<Option<ScrapRequest>> {
        Ok(Some(request))
    }

    /// Process a response after it is received.
    /// Return `None` to drop the response.
    async fn process_response(&self, response: ScrapResponse) -> Result<Option<ScrapResponse>> {
        Ok(Some(response))
    }

    /// Called when a request fails.
    async fn process_error(&self, request: &ScrapRequest, error: &crate::NexError) {
        let _ = (request, error);
    }

    /// Name of this middleware (for logging).
    fn name(&self) -> &str {
        "unnamed"
    }

    /// Priority of this middleware (higher = runs first).
    fn priority(&self) -> i32 {
        0
    }
}

/// Ordered pipeline of middleware components.
pub struct MiddlewarePipeline {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewarePipeline {
    /// Create an empty pipeline.
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add a middleware to the pipeline.
    pub fn add<M: Middleware + 'static>(&mut self, middleware: M) {
        self.middlewares.push(Arc::new(middleware));
        // Sort by priority (descending, so highest priority runs first)
        self.middlewares.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Process a request through all middlewares.
    pub async fn process_request(&self, mut request: ScrapRequest) -> Result<Option<ScrapRequest>> {
        for mw in &self.middlewares {
            match mw.process_request(request).await? {
                Some(r) => request = r,
                None => {
                    tracing::debug!(middleware = mw.name(), "Request dropped by middleware");
                    return Ok(None);
                }
            }
        }
        Ok(Some(request))
    }

    /// Process a response through all middlewares (reverse order).
    pub async fn process_response(
        &self,
        mut response: ScrapResponse,
    ) -> Result<Option<ScrapResponse>> {
        for mw in self.middlewares.iter().rev() {
            match mw.process_response(response).await? {
                Some(r) => response = r,
                None => {
                    tracing::debug!(middleware = mw.name(), "Response dropped by middleware");
                    return Ok(None);
                }
            }
        }
        Ok(Some(response))
    }

    /// Notify all middlewares of an error.
    pub async fn process_error(&self, request: &ScrapRequest, error: &crate::NexError) {
        for mw in &self.middlewares {
            mw.process_error(request, error).await;
        }
    }

    /// Get the number of middlewares in the pipeline.
    pub fn len(&self) -> usize {
        self.middlewares.len()
    }

    /// Check if the pipeline is empty.
    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }
}

impl Default for MiddlewarePipeline {
    fn default() -> Self {
        Self::new()
    }
}
