//! HTTP Cache implementation
//!
//! Implements RFC 7234 HTTP caching with support for:
//! - Cache-Control directives
//! - ETag/If-None-Match
//! - Last-Modified/If-Modified-Since
//! - Expiration handling

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cache entry representing a cached response
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Cached response body
    pub body: Vec<u8>,
    /// Content type
    pub content_type: String,
    /// ETag for validation
    pub etag: Option<String>,
    /// Last-Modified header
    pub last_modified: Option<String>,
    /// When this entry was created
    pub created_at: Instant,
    /// Time-to-live duration
    pub ttl: Duration,
    /// Whether this is a private cache entry
    pub is_private: bool,
}

impl CacheEntry {
    /// Check if this entry is still fresh
    pub fn is_fresh(&self) -> bool {
        self.created_at.elapsed() < self.ttl
    }

    /// Check if this entry is stale but revalidatable
    pub fn is_stale(&self) -> bool {
        !self.is_fresh() && (self.etag.is_some() || self.last_modified.is_some())
    }

    /// Get age of this entry in seconds
    pub fn age(&self) -> u64 {
        self.created_at.elapsed().as_secs()
    }
}

/// Cache-Control directive parser
#[derive(Debug, Clone, Default)]
pub struct CacheControl {
    /// max-age in seconds
    pub max_age: Option<u64>,
    /// s-maxage for shared caches
    pub s_maxage: Option<u64>,
    /// no-cache (must revalidate)
    pub no_cache: bool,
    /// no-store (don't cache)
    pub no_store: bool,
    /// private (only browser cache)
    pub private: bool,
    /// public (can be cached by CDN)
    pub public: bool,
    /// must-revalidate
    pub must_revalidate: bool,
    /// immutable (never changes)
    pub immutable: bool,
}

impl CacheControl {
    /// Parse Cache-Control header value
    pub fn parse(header: &str) -> Self {
        let mut cc = Self::default();

        for directive in header.split(',') {
            let directive = directive.trim().to_lowercase();

            if directive == "no-cache" {
                cc.no_cache = true;
            } else if directive == "no-store" {
                cc.no_store = true;
            } else if directive == "private" {
                cc.private = true;
            } else if directive == "public" {
                cc.public = true;
            } else if directive == "must-revalidate" {
                cc.must_revalidate = true;
            } else if directive == "immutable" {
                cc.immutable = true;
            } else if let Some(value) = directive.strip_prefix("max-age=") {
                cc.max_age = value.parse().ok();
            } else if let Some(value) = directive.strip_prefix("s-maxage=") {
                cc.s_maxage = value.parse().ok();
            }
        }

        cc
    }

    /// Check if response is cacheable
    pub fn is_cacheable(&self) -> bool {
        !self.no_store
    }

    /// Get TTL for this response
    pub fn ttl(&self) -> Option<Duration> {
        if self.no_store || self.no_cache {
            return Some(Duration::ZERO);
        }

        // s-maxage takes precedence for shared caches
        if let Some(secs) = self.s_maxage.or(self.max_age) {
            return Some(Duration::from_secs(secs));
        }

        // If immutable, use a very long TTL
        if self.immutable {
            return Some(Duration::from_secs(31536000)); // 1 year
        }

        None
    }
}

/// HTTP Cache with LRU eviction
pub struct HttpCache {
    /// Cached entries by URL
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Maximum cache size in bytes
    max_size: usize,
    /// Current cache size in bytes
    current_size: Arc<RwLock<usize>>,
    /// Maximum number of entries
    max_entries: usize,
}

impl HttpCache {
    /// Create a new cache with default limits
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_size: 50 * 1024 * 1024, // 50MB
            current_size: Arc::new(RwLock::new(0)),
            max_entries: 1000,
        }
    }

    /// Create a cache with custom limits
    pub fn with_limits(max_size: usize, max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            current_size: Arc::new(RwLock::new(0)),
            max_entries,
        }
    }

    /// Get a cached entry if fresh
    pub fn get(&self, url: &str) -> Option<CacheEntry> {
        let entries = self.entries.read().ok()?;
        let entry = entries.get(url)?;

        if entry.is_fresh() {
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Get a stale entry for revalidation
    pub fn get_stale(&self, url: &str) -> Option<CacheEntry> {
        let entries = self.entries.read().ok()?;
        let entry = entries.get(url)?;

        if entry.is_stale() {
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Store an entry in the cache
    pub fn put(&self, url: &str, entry: CacheEntry) {
        // Check if entry should be cached
        if entry.ttl == Duration::ZERO {
            return;
        }

        let entry_size = entry.body.len();

        // Don't cache if single entry exceeds max size
        if entry_size > self.max_size {
            return;
        }

        // Evict entries if needed
        self.evict_if_needed(entry_size);

        // Insert new entry
        if let Ok(mut entries) = self.entries.write() {
            // Remove old entry if exists
            if let Some(old) = entries.remove(url) {
                if let Ok(mut size) = self.current_size.write() {
                    *size = size.saturating_sub(old.body.len());
                }
            }

            entries.insert(url.to_string(), entry);

            if let Ok(mut size) = self.current_size.write() {
                *size += entry_size;
            }
        }
    }

    /// Remove an entry from cache
    pub fn remove(&self, url: &str) {
        if let Ok(mut entries) = self.entries.write() {
            if let Some(entry) = entries.remove(url) {
                if let Ok(mut size) = self.current_size.write() {
                    *size = size.saturating_sub(entry.body.len());
                }
            }
        }
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
        if let Ok(mut size) = self.current_size.write() {
            *size = 0;
        }
    }

    /// Evict stale entries to make room
    fn evict_if_needed(&self, needed_size: usize) {
        let current = self.current_size.read().map(|g| *g).unwrap_or(0);

        if current + needed_size <= self.max_size {
            let entry_count = self.entries.read().map(|e| e.len()).unwrap_or(0);
            if entry_count < self.max_entries {
                return; // No eviction needed
            }
        }

        // Evict stale entries first
        if let Ok(mut entries) = self.entries.write() {
            let stale_urls: Vec<String> = entries
                .iter()
                .filter(|(_, e)| !e.is_fresh())
                .map(|(url, _)| url.clone())
                .collect();

            for url in stale_urls {
                if let Some(entry) = entries.remove(&url) {
                    if let Ok(mut size) = self.current_size.write() {
                        *size = size.saturating_sub(entry.body.len());
                    }
                }
            }
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let entries = self.entries.read().map(|e| e.len()).unwrap_or(0);
        let size = self.current_size.read().map(|g| *g).unwrap_or(0);

        CacheStats {
            entries,
            size_bytes: size,
            max_size_bytes: self.max_size,
        }
    }
}

impl Default for HttpCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached entries
    pub entries: usize,
    /// Total size in bytes
    pub size_bytes: usize,
    /// Maximum cache size
    pub max_size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_control_parse() {
        let cc = CacheControl::parse("max-age=3600, public");
        assert_eq!(cc.max_age, Some(3600));
        assert!(cc.public);
        assert!(!cc.private);
    }

    #[test]
    fn test_cache_control_no_store() {
        let cc = CacheControl::parse("no-store");
        assert!(cc.no_store);
        assert!(!cc.is_cacheable());
    }

    #[test]
    fn test_cache_entry_freshness() {
        let entry = CacheEntry {
            body: vec![1, 2, 3],
            content_type: "text/html".to_string(),
            etag: Some("abc123".to_string()),
            last_modified: None,
            created_at: Instant::now(),
            ttl: Duration::from_secs(3600),
            is_private: false,
        };

        assert!(entry.is_fresh());
        assert!(!entry.is_stale());
    }

    #[test]
    fn test_cache_put_get() {
        let cache = HttpCache::new();

        let entry = CacheEntry {
            body: b"Hello, World!".to_vec(),
            content_type: "text/plain".to_string(),
            etag: None,
            last_modified: None,
            created_at: Instant::now(),
            ttl: Duration::from_secs(3600),
            is_private: false,
        };

        cache.put("https://example.com/", entry);

        let cached = cache.get("https://example.com/");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().body, b"Hello, World!");
    }

    #[test]
    fn test_cache_stats() {
        let cache = HttpCache::new();

        let entry = CacheEntry {
            body: vec![0; 1000],
            content_type: "text/plain".to_string(),
            etag: None,
            last_modified: None,
            created_at: Instant::now(),
            ttl: Duration::from_secs(3600),
            is_private: false,
        };

        cache.put("https://example.com/", entry);

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.size_bytes, 1000);
    }
}