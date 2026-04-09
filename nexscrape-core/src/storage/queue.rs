//! In-memory request queue.

use crate::ScrapRequest;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Thread-safe in-memory request queue.
pub struct RequestQueue {
    inner: Arc<Mutex<VecDeque<ScrapRequest>>>,
    max_size: Option<usize>,
}

impl RequestQueue {
    /// Create a new unbounded queue.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
            max_size: None,
        }
    }

    /// Create a new bounded queue.
    pub fn bounded(max_size: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size: Some(max_size),
        }
    }

    /// Push a request to the back of the queue.
    pub async fn push(&self, request: ScrapRequest) -> bool {
        let mut queue = self.inner.lock().await;
        if let Some(max) = self.max_size {
            if queue.len() >= max {
                return false;
            }
        }
        queue.push_back(request);
        true
    }

    /// Pop a request from the front of the queue.
    pub async fn pop(&self) -> Option<ScrapRequest> {
        self.inner.lock().await.pop_front()
    }

    /// Get the number of pending requests.
    pub async fn len(&self) -> usize {
        self.inner.lock().await.len()
    }

    /// Check if the queue is empty.
    pub async fn is_empty(&self) -> bool {
        self.inner.lock().await.is_empty()
    }

    /// Clear all requests.
    pub async fn clear(&self) {
        self.inner.lock().await.clear();
    }
}

impl Default for RequestQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queue_push_pop() {
        let queue = RequestQueue::new();

        queue.push(ScrapRequest::get("https://a.com")).await;
        queue.push(ScrapRequest::get("https://b.com")).await;

        let first = queue.pop().await.unwrap();
        assert_eq!(first.url, "https://a.com");

        let second = queue.pop().await.unwrap();
        assert_eq!(second.url, "https://b.com");

        assert!(queue.pop().await.is_none());
    }

    #[tokio::test]
    async fn test_bounded_queue() {
        let queue = RequestQueue::bounded(2);

        assert!(queue.push(ScrapRequest::get("https://a.com")).await);
        assert!(queue.push(ScrapRequest::get("https://b.com")).await);
        assert!(!queue.push(ScrapRequest::get("https://c.com")).await); // full
    }
}
