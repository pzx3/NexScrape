//! Human behavior simulation for stealth scraping.

use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

/// Simulates human-like browsing behavior.
pub struct HumanSimulator {
    /// Minimum delay between actions (ms).
    pub min_delay_ms: u64,
    /// Maximum delay between actions (ms).
    pub max_delay_ms: u64,
}

impl Default for HumanSimulator {
    fn default() -> Self {
        Self {
            min_delay_ms: 500,
            max_delay_ms: 3000,
        }
    }
}

impl HumanSimulator {
    /// Create a new human simulator with custom delay range.
    pub fn new(min_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            min_delay_ms,
            max_delay_ms,
        }
    }

    /// Wait for a random duration simulating human reading time.
    pub async fn random_delay(&self) {
        let mut rng = rand::thread_rng();
        let delay = rng.gen_range(self.min_delay_ms..=self.max_delay_ms);
        sleep(Duration::from_millis(delay)).await;
    }

    /// Generate random mouse coordinates within a viewport.
    pub fn random_mouse_position(&self, width: u32, height: u32) -> (u32, u32) {
        let mut rng = rand::thread_rng();
        (rng.gen_range(0..width), rng.gen_range(0..height))
    }

    /// Generate a realistic typing delay per character (ms).
    pub fn typing_delay_ms(&self) -> u64 {
        let mut rng = rand::thread_rng();
        // Average human types 40-80 WPM = ~120-300ms per character
        rng.gen_range(80..250)
    }

    /// Generate random scroll amount.
    pub fn random_scroll(&self) -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(100..600)
    }

    /// Generate a realistic viewport size.
    pub fn random_viewport(&self) -> (u32, u32) {
        let viewports = vec![
            (1920, 1080),
            (1366, 768),
            (1440, 900),
            (1536, 864),
            (1280, 720),
            (1600, 900),
            (2560, 1440),
        ];
        let mut rng = rand::thread_rng();
        viewports[rng.gen_range(0..viewports.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_viewport() {
        let sim = HumanSimulator::default();
        let (w, h) = sim.random_viewport();
        assert!(w >= 1280);
        assert!(h >= 720);
    }

    #[test]
    fn test_random_mouse_position() {
        let sim = HumanSimulator::default();
        let (x, y) = sim.random_mouse_position(1920, 1080);
        assert!(x < 1920);
        assert!(y < 1080);
    }

    #[test]
    fn test_typing_delay() {
        let sim = HumanSimulator::default();
        let delay = sim.typing_delay_ms();
        assert!(delay >= 80 && delay < 250);
    }
}
