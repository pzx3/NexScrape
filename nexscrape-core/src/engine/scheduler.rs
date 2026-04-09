//! Priority-based request scheduler with deduplication.

use crate::ScrapRequest;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Priority wrapper for scheduled requests.
#[derive(Debug, Clone)]
struct PrioritizedRequest {
    request: ScrapRequest,
    priority: i32,
    sequence: u64, // tie-breaking: FIFO order
}

impl Eq for PrioritizedRequest {}

impl PartialEq for PrioritizedRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl Ord for PrioritizedRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.sequence.cmp(&self.sequence)) // lower sequence = earlier
    }
}

impl PartialOrd for PrioritizedRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Scheduler configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Maximum queue size (0 = unlimited).
    pub max_queue_size: usize,
    /// Whether to deduplicate URLs.
    pub deduplicate: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 0,
            deduplicate: true,
        }
    }
}

impl From<&crate::NexConfig> for SchedulerConfig {
    fn from(_config: &crate::NexConfig) -> Self {
        Self::default()
    }
}

/// Request priority levels.
#[derive(Debug, Clone, Copy)]
pub enum Priority {
    Low = 0,
    Normal = 5,
    High = 10,
    Critical = 20,
}

/// Priority-based request scheduler.
///
/// Manages a priority queue of requests with optional URL deduplication
/// via a Bloom filter. Requests with higher priority are processed first.
pub struct Scheduler {
    queue: Arc<Mutex<BinaryHeap<PrioritizedRequest>>>,
    seen_urls: Arc<Mutex<std::collections::HashSet<String>>>,
    sequence: Arc<std::sync::atomic::AtomicU64>,
    config: SchedulerConfig,
}

impl Scheduler {
    /// Create a new scheduler with the given configuration.
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            seen_urls: Arc::new(Mutex::new(std::collections::HashSet::new())),
            sequence: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            config,
        }
    }

    /// Enqueue a request for processing.
    ///
    /// Returns `true` if the request was enqueued, `false` if it was a duplicate.
    pub async fn enqueue(&self, request: ScrapRequest) -> bool {
        // Dedup check
        if self.config.deduplicate && !request.dont_filter {
            let mut seen = self.seen_urls.lock().await;
            if seen.contains(&request.url) {
                debug!(url = %request.url, "Skipped duplicate URL");
                return false;
            }
            seen.insert(request.url.clone());
        }

        // Queue size limit check
        let mut queue = self.queue.lock().await;
        if self.config.max_queue_size > 0 && queue.len() >= self.config.max_queue_size {
            debug!("Queue full, dropping request: {}", request.url);
            return false;
        }

        let seq = self.sequence.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let priority = request.priority;

        queue.push(PrioritizedRequest {
            request,
            priority,
            sequence: seq,
        });

        true
    }

    /// Dequeue the next highest-priority request.
    pub async fn dequeue(&self) -> Option<ScrapRequest> {
        let mut queue = self.queue.lock().await;
        queue.pop().map(|pr| pr.request)
    }

    /// Get the number of pending requests.
    pub async fn pending_count(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Check if the queue is empty.
    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }

    /// Clear all pending requests.
    pub async fn clear(&self) {
        self.queue.lock().await.clear();
    }

    /// Get the number of seen (deduplicated) URLs.
    pub async fn seen_count(&self) -> usize {
        self.seen_urls.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_priority_ordering() {
        let scheduler = Scheduler::new(SchedulerConfig {
            deduplicate: false,
            ..Default::default()
        });

        scheduler.enqueue(ScrapRequest::get("https://low.com").priority(Priority::Low as i32)).await;
        scheduler.enqueue(ScrapRequest::get("https://high.com").priority(Priority::High as i32)).await;
        scheduler.enqueue(ScrapRequest::get("https://normal.com").priority(Priority::Normal as i32)).await;

        let first = scheduler.dequeue().await.unwrap();
        assert_eq!(first.url, "https://high.com");

        let second = scheduler.dequeue().await.unwrap();
        assert_eq!(second.url, "https://normal.com");

        let third = scheduler.dequeue().await.unwrap();
        assert_eq!(third.url, "https://low.com");
    }

    #[tokio::test]
    async fn test_scheduler_deduplication() {
        let scheduler = Scheduler::new(SchedulerConfig::default());

        assert!(scheduler.enqueue(ScrapRequest::get("https://example.com")).await);
        assert!(!scheduler.enqueue(ScrapRequest::get("https://example.com")).await); // duplicate

        assert_eq!(scheduler.pending_count().await, 1);
    }

    #[tokio::test]
    async fn test_scheduler_dont_filter() {
        let scheduler = Scheduler::new(SchedulerConfig::default());

        let mut req = ScrapRequest::get("https://example.com");
        req.dont_filter = true;

        assert!(scheduler.enqueue(req.clone()).await);
        assert!(scheduler.enqueue(req).await); // should pass despite duplicate URL

        assert_eq!(scheduler.pending_count().await, 2);
    }
}
