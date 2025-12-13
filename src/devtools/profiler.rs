//! Performance Profiler implementation

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileMetric {
    /// Time to first byte
    Ttfb,
    /// DOM content loaded
    DomContentLoaded,
    /// Page load complete
    LoadComplete,
    /// First paint
    FirstPaint,
    /// First contentful paint
    FirstContentfulPaint,
    /// Largest contentful paint
    LargestContentfulPaint,
    /// Time to interactive
    TimeToInteractive,
    /// Layout time
    Layout,
    /// Paint time
    Paint,
    /// Script execution time
    Script,
    /// Style calculation
    Style,
    /// Custom metric
    Custom,
}

impl ProfileMetric {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProfileMetric::Ttfb => "TTFB",
            ProfileMetric::DomContentLoaded => "DOMContentLoaded",
            ProfileMetric::LoadComplete => "Load",
            ProfileMetric::FirstPaint => "FP",
            ProfileMetric::FirstContentfulPaint => "FCP",
            ProfileMetric::LargestContentfulPaint => "LCP",
            ProfileMetric::TimeToInteractive => "TTI",
            ProfileMetric::Layout => "Layout",
            ProfileMetric::Paint => "Paint",
            ProfileMetric::Script => "Script",
            ProfileMetric::Style => "Style",
            ProfileMetric::Custom => "Custom",
        }
    }
}

/// A profile entry
#[derive(Debug, Clone)]
pub struct ProfileEntry {
    pub metric: ProfileMetric,
    pub name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub duration: Option<Duration>,
    pub metadata: HashMap<String, String>,
}

impl ProfileEntry {
    /// Create a new profile entry
    pub fn new(metric: ProfileMetric, name: impl Into<String>) -> Self {
        Self {
            metric,
            name: name.into(),
            start_time: Instant::now(),
            end_time: None,
            duration: None,
            metadata: HashMap::new(),
        }
    }

    /// End the profile entry
    pub fn end(&mut self) {
        self.end_time = Some(Instant::now());
        self.duration = Some(self.end_time.unwrap().duration_since(self.start_time));
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> Option<f64> {
        self.duration.map(|d| d.as_secs_f64() * 1000.0)
    }
}

/// Performance profiler
pub struct PerformanceProfiler {
    entries: Vec<ProfileEntry>,
    active_entries: HashMap<String, usize>,
    page_start: Option<Instant>,
    metrics: HashMap<ProfileMetric, Duration>,
}

impl PerformanceProfiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            active_entries: HashMap::new(),
            page_start: None,
            metrics: HashMap::new(),
        }
    }

    /// Start timing a page load
    pub fn start_page_load(&mut self) {
        self.page_start = Some(Instant::now());
        self.entries.clear();
        self.active_entries.clear();
        self.metrics.clear();
    }

    /// Start a profile entry
    pub fn start(&mut self, metric: ProfileMetric, name: &str) -> usize {
        let entry = ProfileEntry::new(metric, name);
        let idx = self.entries.len();
        self.entries.push(entry);
        self.active_entries.insert(name.to_string(), idx);
        idx
    }

    /// End a profile entry by name
    pub fn end(&mut self, name: &str) {
        if let Some(idx) = self.active_entries.remove(name) {
            if let Some(entry) = self.entries.get_mut(idx) {
                entry.end();
                if let Some(duration) = entry.duration {
                    self.metrics
                        .entry(entry.metric)
                        .and_modify(|d| *d += duration)
                        .or_insert(duration);
                }
            }
        }
    }

    /// Record a metric
    pub fn record_metric(&mut self, metric: ProfileMetric, duration: Duration) {
        self.metrics.insert(metric, duration);
    }

    /// Get metric value
    pub fn get_metric(&self, metric: ProfileMetric) -> Option<Duration> {
        self.metrics.get(&metric).copied()
    }

    /// Get all entries
    pub fn entries(&self) -> &[ProfileEntry] {
        &self.entries
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.entries.clear();
        self.active_entries.clear();
        self.metrics.clear();
    }

    /// Get summary
    pub fn summary(&self) -> HashMap<ProfileMetric, f64> {
        self.metrics
            .iter()
            .map(|(k, v)| (*k, v.as_secs_f64() * 1000.0))
            .collect()
    }

    /// Get time since page load started
    pub fn time_since_start(&self) -> Option<Duration> {
        self.page_start.map(|s| s.elapsed())
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiler_creation() {
        let profiler = PerformanceProfiler::new();
        assert!(profiler.entries().is_empty());
    }

    #[test]
    fn test_profiler_start_end() {
        let mut profiler = PerformanceProfiler::new();
        profiler.start(ProfileMetric::Layout, "layout");
        thread::sleep(Duration::from_millis(10));
        profiler.end("layout");

        assert_eq!(profiler.entries().len(), 1);
        assert!(profiler.entries()[0].duration.is_some());
    }

    #[test]
    fn test_profiler_page_load() {
        let mut profiler = PerformanceProfiler::new();
        profiler.start_page_load();
        thread::sleep(Duration::from_millis(5));
        assert!(profiler.time_since_start().is_some());
    }

    #[test]
    fn test_profiler_record_metric() {
        let mut profiler = PerformanceProfiler::new();
        profiler.record_metric(ProfileMetric::Ttfb, Duration::from_millis(100));
        assert!(profiler.get_metric(ProfileMetric::Ttfb).is_some());
    }

    #[test]
    fn test_profile_metric_str() {
        assert_eq!(ProfileMetric::Ttfb.as_str(), "TTFB");
        assert_eq!(ProfileMetric::FirstContentfulPaint.as_str(), "FCP");
    }

    #[test]
    fn test_profiler_clear() {
        let mut profiler = PerformanceProfiler::new();
        profiler.start(ProfileMetric::Script, "script");
        profiler.end("script");
        profiler.clear();
        assert!(profiler.entries().is_empty());
    }
}
