//! Browser fingerprint data for anti_detection module.
//!
//! This module re-exports from the middleware fingerprint module
//! and adds anti-detection specific utilities.

pub use crate::middleware::fingerprint::BrowserProfile;

/// Generate a randomized browser fingerprint.
pub fn random_profile() -> BrowserProfile {
    use rand::seq::SliceRandom;
    let profiles = BrowserProfile::defaults();
    let mut rng = rand::thread_rng();
    profiles.choose(&mut rng).cloned().unwrap_or_else(|| profiles[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_profile() {
        let profile = random_profile();
        assert!(!profile.user_agent.is_empty());
    }
}
