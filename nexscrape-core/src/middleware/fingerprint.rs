//! Browser fingerprint rotation to avoid detection.

use crate::{Result, ScrapRequest};
use crate::middleware::pipeline::Middleware;
use async_trait::async_trait;

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// Represents a realistic browser fingerprint profile.
#[derive(Debug, Clone)]
pub struct BrowserProfile {
    pub user_agent: String,
    pub accept: String,
    pub accept_language: String,
    pub accept_encoding: String,
    pub sec_ch_ua: String,
    pub sec_ch_ua_mobile: String,
    pub sec_ch_ua_platform: String,
}

impl BrowserProfile {
    /// Generate a set of built-in browser profiles.
    pub fn defaults() -> Vec<Self> {
        vec![
            // Chrome 124 Windows
            Self {
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".into(),
                accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".into(),
                accept_language: "en-US,en;q=0.9".into(),
                accept_encoding: "gzip, deflate, br".into(),
                sec_ch_ua: r#""Chromium";v="124", "Google Chrome";v="124", "Not-A.Brand";v="99""#.into(),
                sec_ch_ua_mobile: "?0".into(),
                sec_ch_ua_platform: r#""Windows""#.into(),
            },
            // Chrome 124 macOS
            Self {
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".into(),
                accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".into(),
                accept_language: "en-US,en;q=0.9".into(),
                accept_encoding: "gzip, deflate, br".into(),
                sec_ch_ua: r#""Chromium";v="124", "Google Chrome";v="124", "Not-A.Brand";v="99""#.into(),
                sec_ch_ua_mobile: "?0".into(),
                sec_ch_ua_platform: r#""macOS""#.into(),
            },
            // Firefox 125 Windows
            Self {
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:125.0) Gecko/20100101 Firefox/125.0".into(),
                accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".into(),
                accept_language: "en-US,en;q=0.5".into(),
                accept_encoding: "gzip, deflate, br".into(),
                sec_ch_ua: String::new(),
                sec_ch_ua_mobile: String::new(),
                sec_ch_ua_platform: String::new(),
            },
            // Safari macOS
            Self {
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_4_1) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15".into(),
                accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".into(),
                accept_language: "en-US,en;q=0.9".into(),
                accept_encoding: "gzip, deflate, br".into(),
                sec_ch_ua: String::new(),
                sec_ch_ua_mobile: String::new(),
                sec_ch_ua_platform: String::new(),
            },
            // Edge 124 Windows
            Self {
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0".into(),
                accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".into(),
                accept_language: "en-US,en;q=0.9".into(),
                accept_encoding: "gzip, deflate, br".into(),
                sec_ch_ua: r#""Microsoft Edge";v="124", "Chromium";v="124", "Not-A.Brand";v="99""#.into(),
                sec_ch_ua_mobile: "?0".into(),
                sec_ch_ua_platform: r#""Windows""#.into(),
            },
        ]
    }
}

/// Rotates browser fingerprints across requests to avoid detection.
pub struct FingerprintRotator {
    profiles: Vec<BrowserProfile>,
    current_index: AtomicUsize,
    request_count: AtomicU32,
    rotation_interval: u32,
}

impl FingerprintRotator {
    /// Create a new fingerprint rotator with default profiles.
    pub fn new(rotation_interval: u32) -> Self {
        Self {
            profiles: BrowserProfile::defaults(),
            current_index: AtomicUsize::new(0),
            request_count: AtomicU32::new(0),
            rotation_interval,
        }
    }

    /// Create a fingerprint rotator with custom profiles.
    pub fn with_profiles(profiles: Vec<BrowserProfile>, rotation_interval: u32) -> Self {
        Self {
            profiles,
            current_index: AtomicUsize::new(0),
            request_count: AtomicU32::new(0),
            rotation_interval,
        }
    }

    /// Get the current browser profile.
    pub fn current_profile(&self) -> &BrowserProfile {
        let idx = self.current_index.load(Ordering::Relaxed) % self.profiles.len();
        &self.profiles[idx]
    }

    fn maybe_rotate(&self) {
        let count = self.request_count.fetch_add(1, Ordering::Relaxed);
        if count > 0 && count % self.rotation_interval == 0 {
            self.current_index.fetch_add(1, Ordering::Relaxed);
            tracing::debug!("Rotated browser fingerprint");
        }
    }
}

#[async_trait]
impl Middleware for FingerprintRotator {
    async fn process_request(&self, mut request: ScrapRequest) -> Result<Option<ScrapRequest>> {
        self.maybe_rotate();
        let profile = self.current_profile();

        request.headers.insert("User-Agent".into(), profile.user_agent.clone());
        request.headers.insert("Accept".into(), profile.accept.clone());
        request.headers.insert("Accept-Language".into(), profile.accept_language.clone());
        request.headers.insert("Accept-Encoding".into(), profile.accept_encoding.clone());

        if !profile.sec_ch_ua.is_empty() {
            request.headers.insert("sec-ch-ua".into(), profile.sec_ch_ua.clone());
            request.headers.insert("sec-ch-ua-mobile".into(), profile.sec_ch_ua_mobile.clone());
            request.headers.insert("sec-ch-ua-platform".into(), profile.sec_ch_ua_platform.clone());
        }

        Ok(Some(request))
    }

    fn name(&self) -> &str {
        "fingerprint_rotator"
    }

    fn priority(&self) -> i32 {
        100 // Run early in the pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profiles() {
        let profiles = BrowserProfile::defaults();
        assert!(profiles.len() >= 4);
        for p in &profiles {
            assert!(!p.user_agent.is_empty());
            assert!(!p.accept.is_empty());
        }
    }

    #[test]
    fn test_rotation() {
        let rotator = FingerprintRotator::new(2);
        let first = rotator.current_profile().user_agent.clone();

        rotator.maybe_rotate(); // count=0, no rotation
        rotator.maybe_rotate(); // count=1, no rotation
        rotator.maybe_rotate(); // count=2, rotation!

        let second = rotator.current_profile().user_agent.clone();
        assert_ne!(first, second);
    }
}
