//! Smart Prefetching - Predictive loading based on usage patterns
//!
//! Uses simple ML-like heuristics to predict which resources will be needed next
//! and prefetch them to reduce latency.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Strategy for prefetching resources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefetchStrategy {
    /// No prefetching
    None,
    /// Prefetch on hover
    OnHover,
    /// Prefetch based on viewport visibility
    OnVisible,
    /// Predictive prefetching using navigation patterns
    Predictive,
    /// Aggressive prefetching (all links)
    Aggressive,
}

/// A hint about what to prefetch
#[derive(Debug, Clone)]
pub struct PrefetchHint {
    /// URL to prefetch
    pub url: String,
    /// Priority (0-100, higher = more important)
    pub priority: u8,
    /// Type of resource
    pub resource_type: ResourceType,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
}

/// Type of resource to prefetch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Document,
    Script,
    Stylesheet,
    Image,
    Font,
    Fetch,
}

/// Navigation pattern entry
#[derive(Debug, Clone)]
struct NavigationEntry {
    from_url: String,
    to_url: String,
    count: u32,
    last_access: Instant,
}

/// Smart prefetcher using pattern analysis
pub struct Prefetcher {
    strategy: PrefetchStrategy,
    /// Navigation history: from_url -> list of (to_url, count)
    patterns: Arc<RwLock<HashMap<String, Vec<NavigationEntry>>>>,
    /// Minimum confidence threshold
    confidence_threshold: f64,
    /// Maximum number of prefetch hints
    max_hints: usize,
    /// Pattern decay time
    decay_duration: Duration,
}

impl Prefetcher {
    pub fn new(strategy: PrefetchStrategy) -> Self {
        Self {
            strategy,
            patterns: Arc::new(RwLock::new(HashMap::new())),
            confidence_threshold: 0.3,
            max_hints: 5,
            decay_duration: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Record a navigation event for pattern learning
    pub fn record_navigation(&self, from: &str, to: &str) {
        let mut patterns = self.patterns.write().unwrap();
        let entries = patterns.entry(from.to_string()).or_default();

        // Update existing or add new
        if let Some(entry) = entries.iter_mut().find(|e| e.to_url == to) {
            entry.count += 1;
            entry.last_access = Instant::now();
        } else {
            entries.push(NavigationEntry {
                from_url: from.to_string(),
                to_url: to.to_string(),
                count: 1,
                last_access: Instant::now(),
            });
        }
    }

    /// Get prefetch hints for the current URL
    pub fn get_hints(&self, current_url: &str) -> Vec<PrefetchHint> {
        if self.strategy == PrefetchStrategy::None {
            return Vec::new();
        }

        let patterns = self.patterns.read().unwrap();
        let mut hints = Vec::new();

        if let Some(entries) = patterns.get(current_url) {
            // Calculate total navigations from this URL
            let total: u32 = entries.iter().map(|e| e.count).sum();

            for entry in entries {
                // Skip old entries
                if entry.last_access.elapsed() > self.decay_duration {
                    continue;
                }

                let confidence = entry.count as f64 / total as f64;

                if confidence >= self.confidence_threshold {
                    hints.push(PrefetchHint {
                        url: entry.to_url.clone(),
                        priority: (confidence * 100.0) as u8,
                        resource_type: ResourceType::Document,
                        confidence,
                    });
                }
            }
        }

        // Sort by priority and limit
        hints.sort_by(|a, b| b.priority.cmp(&a.priority));
        hints.truncate(self.max_hints);
        hints
    }

    /// Get the current strategy
    pub fn strategy(&self) -> PrefetchStrategy {
        self.strategy
    }

    /// Set the strategy
    pub fn set_strategy(&mut self, strategy: PrefetchStrategy) {
        self.strategy = strategy;
    }

    /// Clear all learned patterns
    pub fn clear_patterns(&self) {
        self.patterns.write().unwrap().clear();
    }

    /// Number of learned patterns
    pub fn pattern_count(&self) -> usize {
        self.patterns.read().unwrap().len()
    }
}

impl Default for Prefetcher {
    fn default() -> Self {
        Self::new(PrefetchStrategy::Predictive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_strategy() {
        let prefetcher = Prefetcher::new(PrefetchStrategy::None);
        assert_eq!(prefetcher.strategy(), PrefetchStrategy::None);
        assert!(prefetcher.get_hints("http://example.com").is_empty());
    }

    #[test]
    fn test_record_navigation() {
        let prefetcher = Prefetcher::new(PrefetchStrategy::Predictive);
        prefetcher.record_navigation("http://a.com", "http://b.com");
        prefetcher.record_navigation("http://a.com", "http://b.com");
        prefetcher.record_navigation("http://a.com", "http://c.com");

        assert_eq!(prefetcher.pattern_count(), 1);
    }

    #[test]
    fn test_get_hints() {
        let prefetcher = Prefetcher::new(PrefetchStrategy::Predictive);

        // Record pattern: a.com -> b.com (80% of time)
        for _ in 0..8 {
            prefetcher.record_navigation("http://a.com", "http://b.com");
        }
        for _ in 0..2 {
            prefetcher.record_navigation("http://a.com", "http://c.com");
        }

        let hints = prefetcher.get_hints("http://a.com");
        assert!(!hints.is_empty());
        assert_eq!(hints[0].url, "http://b.com");
        assert!(hints[0].confidence > 0.7);
    }

    #[test]
    fn test_clear_patterns() {
        let prefetcher = Prefetcher::new(PrefetchStrategy::Predictive);
        prefetcher.record_navigation("http://a.com", "http://b.com");
        assert_eq!(prefetcher.pattern_count(), 1);

        prefetcher.clear_patterns();
        assert_eq!(prefetcher.pattern_count(), 0);
    }
}
