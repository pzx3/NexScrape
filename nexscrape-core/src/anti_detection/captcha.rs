//! CAPTCHA solver integration.

use crate::{NexError, Result};
use serde::{Deserialize, Serialize};

/// Supported CAPTCHA solver providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptchaProvider {
    TwoCaptcha,
    AntiCaptcha,
    CapSolver,
    Custom(String),
}

/// CAPTCHA solver configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaSolverConfig {
    pub provider: CaptchaProvider,
    pub api_key: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

/// CAPTCHA solver client.
///
/// Integrates with external CAPTCHA-solving services
/// to automatically bypass CAPTCHA challenges.
pub struct CaptchaSolver {
    config: CaptchaSolverConfig,
}

impl CaptchaSolver {
    /// Create a new CAPTCHA solver.
    pub fn new(config: CaptchaSolverConfig) -> Self {
        Self { config }
    }

    /// Solve a reCAPTCHA v2 challenge.
    pub async fn solve_recaptcha_v2(
        &self,
        _site_key: &str,
        page_url: &str,
    ) -> Result<String> {
        tracing::info!(
            provider = ?self.config.provider,
            url = %page_url,
            "Solving reCAPTCHA v2"
        );

        // In a full implementation, this would call the external API.
        // Placeholder for the integration point.
        Err(NexError::CaptchaDetected(format!(
            "CAPTCHA solving requires API key configuration. Site: {}",
            page_url
        )))
    }

    /// Solve a hCaptcha challenge.
    pub async fn solve_hcaptcha(
        &self,
        _site_key: &str,
        page_url: &str,
    ) -> Result<String> {
        tracing::info!(
            provider = ?self.config.provider,
            url = %page_url,
            "Solving hCaptcha"
        );

        Err(NexError::CaptchaDetected(format!(
            "hCaptcha solving requires API key configuration. Site: {}",
            page_url
        )))
    }

    /// Detect CAPTCHA presence in HTML.
    pub fn detect_captcha(html: &str) -> Option<CaptchaType> {
        if html.contains("g-recaptcha") || html.contains("recaptcha") {
            Some(CaptchaType::ReCaptchaV2)
        } else if html.contains("h-captcha") || html.contains("hcaptcha") {
            Some(CaptchaType::HCaptcha)
        } else if html.contains("cf-turnstile") {
            Some(CaptchaType::Turnstile)
        } else {
            None
        }
    }
}

/// Types of CAPTCHA challenges.
#[derive(Debug, Clone)]
pub enum CaptchaType {
    ReCaptchaV2,
    ReCaptchaV3,
    HCaptcha,
    Turnstile,
    ImageCaptcha,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_recaptcha() {
        let html = r#"<div class="g-recaptcha" data-sitekey="xyz"></div>"#;
        let result = CaptchaSolver::detect_captcha(html);
        assert!(matches!(result, Some(CaptchaType::ReCaptchaV2)));
    }

    #[test]
    fn test_detect_hcaptcha() {
        let html = r#"<div class="h-captcha" data-sitekey="abc"></div>"#;
        let result = CaptchaSolver::detect_captcha(html);
        assert!(matches!(result, Some(CaptchaType::HCaptcha)));
    }

    #[test]
    fn test_no_captcha() {
        let html = "<html><body><h1>Hello</h1></body></html>";
        assert!(CaptchaSolver::detect_captcha(html).is_none());
    }
}
