//! Stealth configuration for anti-detection.

use serde::{Deserialize, Serialize};

/// Stealth mode configuration.
///
/// Controls various anti-detection measures when scraping
/// sites that employ bot protection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthConfig {
    /// Enable browser fingerprint rotation.
    pub fingerprint_rotation: bool,
    /// Rotate fingerprint every N requests.
    pub rotation_interval: u32,
    /// Enable human-like behavior simulation.
    pub human_simulation: bool,
    /// Random delay range (min, max) in milliseconds.
    pub random_delay: (u64, u64),
    /// Enable proxy usage.
    pub use_proxies: bool,
    /// Automatically solve CAPTCHAs.
    pub auto_captcha: bool,
    /// CAPTCHA solver provider.
    pub captcha_provider: Option<String>,
    /// CAPTCHA solver API key.
    pub captcha_api_key: Option<String>,
    /// Simulate realistic Referer headers.
    pub fake_referer: bool,
    /// Randomize viewport dimensions.
    pub randomize_viewport: bool,
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            fingerprint_rotation: true,
            rotation_interval: 10,
            human_simulation: false,
            random_delay: (500, 2000),
            use_proxies: false,
            auto_captcha: false,
            captcha_provider: None,
            captcha_api_key: None,
            fake_referer: true,
            randomize_viewport: false,
        }
    }
}

impl StealthConfig {
    /// Create a minimal stealth config (just fingerprint rotation).
    pub fn minimal() -> Self {
        Self::default()
    }

    /// Create a full stealth config (all protections enabled).
    pub fn full() -> Self {
        Self {
            fingerprint_rotation: true,
            rotation_interval: 5,
            human_simulation: true,
            random_delay: (1000, 5000),
            use_proxies: true,
            auto_captcha: true,
            captcha_provider: None,
            captcha_api_key: None,
            fake_referer: true,
            randomize_viewport: true,
        }
    }
}
