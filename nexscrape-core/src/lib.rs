//! # NexScrape Core
//!
//! Next-generation high-performance web scraping engine built in Rust.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │            NexScrape Core                │
//! │  ┌──────────┐  ┌────────────────────┐   │
//! │  │ Scheduler│  │    HTTP Engine      │   │
//! │  └──────┬───┘  └────────┬───────────┘   │
//! │         │               │               │
//! │  ┌──────▼───────────────▼────────────┐  │
//! │  │       Middleware Pipeline          │  │
//! │  └──────────────────┬────────────────┘  │
//! │                     │                    │
//! │  ┌──────────────────▼────────────────┐  │
//! │  │         Parser Engine             │  │
//! │  └──────────────────┬────────────────┘  │
//! │                     │                    │
//! │  ┌──────────────────▼────────────────┐  │
//! │  │         Data Pipeline             │  │
//! │  └───────────────────────────────────┘  │
//! └──────────────────────────────────────────┘
//! ```

pub mod engine;
pub mod middleware;
pub mod parser;
pub mod anti_detection;
pub mod storage;

// Re-exports for convenience
pub use engine::http::{HttpEngine, EngineConfig};
pub use engine::scheduler::{Scheduler, SchedulerConfig, Priority};
pub use middleware::pipeline::{MiddlewarePipeline, Middleware};
pub use middleware::fingerprint::FingerprintRotator;
pub use middleware::proxy::{ProxyPool, ProxyConfig, RotationStrategy};
pub use middleware::ratelimit::RateLimiter;
pub use middleware::retry::RetryPolicy;
pub use middleware::cache::{Cache, CacheConfig};
pub use parser::html::HtmlParser;
pub use parser::json::JsonExtractor;
pub use parser::schema::{Schema, SchemaField, FieldType};
pub use anti_detection::stealth::StealthConfig;
pub use anti_detection::fingerprint::BrowserProfile;
pub use storage::bloom::BloomFilter;
pub use storage::export::{Exporter, ExportFormat};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Core error type for NexScrape operations.
#[derive(Error, Debug)]
pub enum NexError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("Parser error: {0}")]
    ParseError(String),

    #[error("Selector error: {0}")]
    SelectorError(String),

    #[error("Schema validation error: {0}")]
    SchemaError(String),

    #[error("Rate limited by target (HTTP {status})")]
    RateLimited { status: u16, retry_after: Option<u64> },

    #[error("All proxies exhausted")]
    ProxyExhausted,

    #[error("Request timeout after {0}s")]
    Timeout(u64),

    #[error("Max retries ({0}) exceeded")]
    MaxRetries(u32),

    #[error("Export failed: {0}")]
    ExportError(String),

    #[error("CAPTCHA detected on {0}")]
    CaptchaDetected(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, NexError>;

/// Represents a scraping request with all metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapRequest {
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
    pub meta: std::collections::HashMap<String, serde_json::Value>,
    pub priority: i32,
    pub dont_filter: bool,
    pub max_retries: u32,
    pub callback_name: Option<String>,
}

impl ScrapRequest {
    /// Create a new GET request.
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: "GET".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
            meta: std::collections::HashMap::new(),
            priority: 0,
            dont_filter: false,
            max_retries: 3,
            callback_name: None,
        }
    }

    /// Create a new POST request.
    pub fn post(url: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: "POST".to_string(),
            headers: std::collections::HashMap::new(),
            body: Some(body.into()),
            meta: std::collections::HashMap::new(),
            priority: 0,
            dont_filter: false,
            max_retries: 3,
            callback_name: None,
        }
    }

    /// Set a custom header.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set request priority (higher = first).
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set metadata for this request.
    pub fn meta(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.meta.insert(key.into(), value);
        self
    }
}

/// Represents a scraping response with parsed content.
#[derive(Debug, Clone)]
pub struct ScrapResponse {
    pub url: String,
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: bytes::Bytes,
    pub request: ScrapRequest,
}

impl ScrapResponse {
    /// Get the response body as a UTF-8 string.
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.to_vec())
            .map_err(|e| NexError::ParseError(format!("UTF-8 decode failed: {}", e)))
    }

    /// Parse the body as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body)
            .map_err(|e| NexError::ParseError(format!("JSON parse failed: {}", e)))
    }

    /// Create an HTML parser for CSS/XPath selection.
    pub fn html(&self) -> Result<HtmlParser> {
        let text = self.text()?;
        Ok(HtmlParser::new(&text, &self.url))
    }

    /// Check if the response indicates success (2xx).
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if the response indicates rate limiting.
    pub fn is_rate_limited(&self) -> bool {
        self.status == 429
    }
}

/// Extracted data item with named fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub fields: std::collections::HashMap<String, serde_json::Value>,
    pub source_url: String,
}

impl Item {
    pub fn new(source_url: impl Into<String>) -> Self {
        Self {
            fields: std::collections::HashMap::new(),
            source_url: source_url.into(),
        }
    }

    pub fn set(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.fields.get(key)
    }
}

/// Global configuration for a scraping session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexConfig {
    pub concurrency: usize,
    pub timeout_secs: u64,
    pub delay_min_ms: u64,
    pub delay_max_ms: u64,
    pub max_retries: u32,
    pub respect_robots_txt: bool,
    pub user_agent: String,
    pub follow_redirects: bool,
    pub max_redirects: usize,
    pub max_depth: Option<usize>,
}

impl Default for NexConfig {
    fn default() -> Self {
        Self {
            concurrency: 16,
            timeout_secs: 30,
            delay_min_ms: 500,
            delay_max_ms: 2000,
            max_retries: 3,
            respect_robots_txt: true,
            user_agent: format!("NexScrape/{} (+https://github.com/nexscrape/nexscrape)", env!("CARGO_PKG_VERSION")),
            follow_redirects: true,
            max_redirects: 10,
            max_depth: None,
        }
    }
}

/// Quick one-shot fetch — the simplest way to use NexScrape.
///
/// # Example
/// ```no_run
/// # tokio_test::block_on(async {
/// let response = nexscrape_core::fetch("https://example.com").await.unwrap();
/// let html = response.html().unwrap();
/// let title = html.select_text("title").unwrap();
/// println!("Title: {}", title);
/// # });
/// ```
pub async fn fetch(url: &str) -> Result<ScrapResponse> {
    let engine = HttpEngine::new(EngineConfig::default())?;
    let request = ScrapRequest::get(url);
    engine.execute(request).await
}

/// Run a full scraping session with config.
pub async fn run_session(
    config: NexConfig,
    start_urls: Vec<String>,
    selectors: std::collections::HashMap<String, String>,
) -> Result<Vec<Item>> {
    let engine = HttpEngine::new(EngineConfig::from(&config))?;
    let scheduler = Scheduler::new(SchedulerConfig::from(&config));
    let mut items = Vec::new();

    // Enqueue start URLs
    for url in start_urls {
        scheduler.enqueue(ScrapRequest::get(url)).await;
    }

    // Process queue
    while let Some(request) = scheduler.dequeue().await {
        match engine.execute(request.clone()).await {
            Ok(response) => {
                if let Ok(html) = response.html() {
                    let mut item = Item::new(&response.url);
                    for (field_name, selector) in &selectors {
                        if let Ok(text) = html.select_text(selector) {
                            item = item.set(field_name.clone(), serde_json::Value::String(text));
                        }
                    }
                    items.push(item);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch {}: {}", request.url, e);
            }
        }
    }

    Ok(items)
}
