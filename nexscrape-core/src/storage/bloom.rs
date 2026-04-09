//! Space-efficient Bloom filter for URL deduplication.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A probabilistic Bloom filter for space-efficient set membership testing.
///
/// Used for URL deduplication — can test if a URL has *probably* been seen
/// before with a configurable false-positive rate.
pub struct BloomFilter {
    bits: Vec<bool>,
    num_hashes: usize,
    size: usize,
    count: usize,
}

impl BloomFilter {
    /// Create a new Bloom filter sized for a given capacity and false-positive rate.
    ///
    /// # Arguments
    /// - `capacity`: Expected number of elements.
    /// - `fp_rate`: Desired false-positive rate (e.g., 0.01 for 1%).
    pub fn new(capacity: usize, fp_rate: f64) -> Self {
        let size = Self::optimal_size(capacity, fp_rate);
        let num_hashes = Self::optimal_hashes(size, capacity);

        Self {
            bits: vec![false; size],
            num_hashes,
            size,
            count: 0,
        }
    }

    /// Create a Bloom filter with explicit size and hash count.
    pub fn with_params(size: usize, num_hashes: usize) -> Self {
        Self {
            bits: vec![false; size],
            num_hashes,
            size,
            count: 0,
        }
    }

    /// Insert an item into the filter.
    pub fn insert(&mut self, item: &str) {
        for i in 0..self.num_hashes {
            let idx = self.hash(item, i) % self.size;
            self.bits[idx] = true;
        }
        self.count += 1;
    }

    /// Check if an item is possibly in the filter.
    ///
    /// Returns `true` if the item *might* be in the set (with possible false positives).
    /// Returns `false` if the item is *definitely* not in the set.
    pub fn contains(&self, item: &str) -> bool {
        for i in 0..self.num_hashes {
            let idx = self.hash(item, i) % self.size;
            if !self.bits[idx] {
                return false;
            }
        }
        true
    }

    /// Insert and return whether the item was already present.
    pub fn insert_check(&mut self, item: &str) -> bool {
        let was_present = self.contains(item);
        self.insert(item);
        was_present
    }

    /// Get the number of items inserted.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Get the filter's bit array size.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Estimate the current false-positive rate.
    pub fn estimated_fp_rate(&self) -> f64 {
        let bits_set: f64 = self.bits.iter().filter(|&&b| b).count() as f64;
        let ratio = bits_set / self.size as f64;
        ratio.powi(self.num_hashes as i32)
    }

    /// Clear the filter.
    pub fn clear(&mut self) {
        self.bits.fill(false);
        self.count = 0;
    }

    fn hash(&self, item: &str, seed: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn optimal_size(capacity: usize, fp_rate: f64) -> usize {
        let ln2_sq = std::f64::consts::LN_2 * std::f64::consts::LN_2;
        let size = -(capacity as f64 * fp_rate.ln()) / ln2_sq;
        size.ceil() as usize
    }

    fn optimal_hashes(size: usize, capacity: usize) -> usize {
        let k = (size as f64 / capacity as f64) * std::f64::consts::LN_2;
        std::cmp::max(k.ceil() as usize, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_basic() {
        let mut bf = BloomFilter::new(1000, 0.01);

        bf.insert("https://example.com");
        assert!(bf.contains("https://example.com"));
        assert!(!bf.contains("https://nothere.com"));
    }

    #[test]
    fn test_bloom_filter_insert_check() {
        let mut bf = BloomFilter::new(1000, 0.01);

        assert!(!bf.insert_check("https://first.com"));
        assert!(bf.insert_check("https://first.com")); // already present
    }

    #[test]
    fn test_bloom_filter_count() {
        let mut bf = BloomFilter::new(1000, 0.01);

        bf.insert("a");
        bf.insert("b");
        bf.insert("c");

        assert_eq!(bf.count(), 3);
    }

    #[test]
    fn test_bloom_filter_clear() {
        let mut bf = BloomFilter::new(1000, 0.01);

        bf.insert("test");
        assert!(bf.contains("test"));

        bf.clear();
        assert!(!bf.contains("test"));
        assert_eq!(bf.count(), 0);
    }

    #[test]
    fn test_bloom_filter_false_positive_rate() {
        // With a large filter and few insertions, FP rate should be very low
        let mut bf = BloomFilter::new(100_000, 0.001);

        for i in 0..100 {
            bf.insert(&format!("url_{}", i));
        }

        let mut false_positives = 0;
        for i in 1000..2000 {
            if bf.contains(&format!("url_{}", i)) {
                false_positives += 1;
            }
        }

        // Should be very few false positives (< 1%)
        assert!(false_positives < 10, "Too many false positives: {}", false_positives);
    }
}
