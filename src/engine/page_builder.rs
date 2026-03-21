//! Page fetching pipeline
//!
//! Bridges the async NetworkStack to the synchronous thread context used by
//! Tab::navigate(). All HTTP traffic is routed through NetworkStack instead
//! of raw reqwest::blocking, enabling caching, redirect handling, and
//! future HTTP/3 support with zero changes to callers.

use crate::network::{HttpCache, NetworkStack, CacheEntry};
use crate::network::cache::CacheControl;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Maximum redirects to follow before giving up
const MAX_REDIRECTS: usize = 10;

/// Default TTL for cached pages without Cache-Control headers (5 minutes)
const DEFAULT_PAGE_TTL: Duration = Duration::from_secs(300);

/// Default TTL for cached CSS/JS assets without Cache-Control (1 hour)
const DEFAULT_ASSET_TTL: Duration = Duration::from_secs(3600);

/// A fetched resource ready for parsing
#[derive(Debug)]
pub struct FetchedPage {
    /// Final URL after following redirects
    pub url: String,
    /// Response body (HTML, CSS, JS, etc.)
    pub body: String,
    /// Whether the response was served from cache
    pub from_cache: bool,
    /// HTTP status code
    pub status: u16,
    /// Response headers (lowercase keys)
    pub headers: HashMap<String, String>,
}

/// Page fetcher backed by the proper NetworkStack with HTTP caching.
///
/// This replaces all `reqwest::blocking` usage in Tab and provides:
/// - Async NetworkStack under the hood (tokio runtime per fetch)
/// - In-memory HTTP cache (RFC 7234 compliant via HttpCache)
/// - Redirect following (up to MAX_REDIRECTS)
/// - External CSS / JS asset fetching
pub struct PageFetcher {
    cache: Arc<Mutex<HttpCache>>,
}

impl PageFetcher {
    /// Create a new PageFetcher with a fresh cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HttpCache::new())),
        }
    }

    /// Fetch a page URL, following redirects, using the NetworkStack.
    /// Returns an error string on failure.
    pub fn fetch(&self, url: &str) -> Result<FetchedPage, String> {
        // Serve from cache if fresh
        if let Ok(cache) = self.cache.lock() {
            if let Some(entry) = cache.get(url) {
                let body = String::from_utf8_lossy(&entry.body).into_owned();
                return Ok(FetchedPage {
                    url: url.to_string(),
                    body,
                    from_cache: true,
                    status: 200,
                    headers: HashMap::new(),
                });
            }
        }

        // Build a tokio runtime for this thread
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create async runtime: {}", e))?;

        let cache = Arc::clone(&self.cache);
        let url_owned = url.to_string();

        rt.block_on(async move {
            let network = NetworkStack::new();
            fetch_with_redirects(&network, &url_owned, MAX_REDIRECTS, cache).await
        })
    }

    /// Fetch an external CSS asset. Returns `None` on failure.
    pub fn fetch_css(&self, url: &str) -> Option<String> {
        self.fetch_css_for_page(url, "")
    }

    /// Fetch an external CSS asset with mixed-content check against page URL.
    pub fn fetch_css_for_page(&self, url: &str, page_url: &str) -> Option<String> {
        if !page_url.is_empty() && is_mixed_content(page_url, url) {
            log::warn!("Mixed content blocked: CSS {} on page {}", url, page_url);
            return None;
        }
        self.fetch_asset(url, DEFAULT_ASSET_TTL)
    }

    /// Fetch an external JavaScript asset. Returns `None` on failure.
    pub fn fetch_script(&self, url: &str) -> Option<String> {
        self.fetch_script_for_page(url, "")
    }

    /// Fetch an external JavaScript asset with mixed-content check.
    pub fn fetch_script_for_page(&self, url: &str, page_url: &str) -> Option<String> {
        if !page_url.is_empty() && is_mixed_content(page_url, url) {
            log::warn!("Mixed content blocked: script {} on page {}", url, page_url);
            return None;
        }
        self.fetch_asset(url, DEFAULT_ASSET_TTL)
    }

    /// Generic asset fetch with cache support
    fn fetch_asset(&self, url: &str, fallback_ttl: Duration) -> Option<String> {
        // Cache hit
        if let Ok(cache) = self.cache.lock() {
            if let Some(entry) = cache.get(url) {
                return Some(String::from_utf8_lossy(&entry.body).into_owned());
            }
        }

        // Validate URL scheme (no javascript: or data: resources via network)
        if url.starts_with("javascript:") || url.starts_with("data:") {
            log::warn!("Blocked fetch of scheme: {}", url);
            return None;
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?;

        let cache = Arc::clone(&self.cache);
        let url_owned = url.to_string();

        rt.block_on(async move {
            let network = NetworkStack::new();
            match network.fetch(&url_owned).await {
                Ok(resp) if resp.is_success() => {
                    let body = resp.body().to_string();
                    // Store in cache
                    let ttl = resp
                        .header("cache-control")
                        .and_then(|cc| CacheControl::parse(cc).ttl())
                        .unwrap_or(fallback_ttl);

                    let entry = CacheEntry {
                        body: body.as_bytes().to_vec(),
                        content_type: resp
                            .content_type()
                            .unwrap_or("text/plain")
                            .to_string(),
                        etag: resp.header("etag").cloned(),
                        last_modified: resp.header("last-modified").cloned(),
                        created_at: std::time::Instant::now(),
                        ttl,
                        is_private: false,
                    };
                    if let Ok(cache) = cache.lock() {
                        cache.put(&url_owned, entry);
                    }
                    Some(body)
                }
                Ok(resp) => {
                    log::warn!(
                        "Asset fetch failed: HTTP {} for {}",
                        resp.status(),
                        url_owned
                    );
                    None
                }
                Err(e) => {
                    log::warn!("Asset fetch error for {}: {}", url_owned, e);
                    None
                }
            }
        })
    }
}

impl Default for PageFetcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Follow redirects up to `max_redirects`, using the provided NetworkStack.
async fn fetch_with_redirects(
    network: &NetworkStack,
    url: &str,
    max_redirects: usize,
    cache: Arc<Mutex<HttpCache>>,
) -> Result<FetchedPage, String> {
    let mut current_url = url.to_string();
    let mut redirects = 0;

    loop {
        log::debug!("Fetching: {}", current_url);

        let response = network
            .fetch(&current_url)
            .await
            .map_err(|e| format!("Network error fetching {}: {}", current_url, e))?;

        let status = response.status();

        // Follow 3xx redirects
        if response.is_redirect() {
            if redirects >= max_redirects {
                return Err(format!(
                    "Too many redirects (max {}) for {}",
                    max_redirects, url
                ));
            }

            let location = response
                .header("location")
                .or_else(|| response.header("Location"))
                .ok_or_else(|| {
                    format!("HTTP {} redirect with no Location header", status)
                })?
                .clone();

            let next = resolve_url(&location, &current_url);
            log::debug!("Redirect {} → {}", current_url, next);
            current_url = next;
            redirects += 1;
            continue;
        }

        if !response.is_success() {
            return Err(format!("HTTP {} for {}", status, current_url));
        }

        let headers = response.headers().clone();
        let body = response.body().to_string();

        // Determine TTL and store in cache
        let cc = headers
            .get("cache-control")
            .map(|v| CacheControl::parse(v))
            .unwrap_or_default();

        let ttl = cc.ttl().unwrap_or(DEFAULT_PAGE_TTL);

        let entry = CacheEntry {
            body: body.as_bytes().to_vec(),
            content_type: headers
                .get("content-type")
                .cloned()
                .unwrap_or_else(|| "text/html".to_string()),
            etag: headers.get("etag").cloned(),
            last_modified: headers.get("last-modified").cloned(),
            created_at: std::time::Instant::now(),
            ttl,
            is_private: cc.private,
        };

        if let Ok(cache) = cache.lock() {
            cache.put(url, entry); // Cache under original URL (pre-redirect)
        }

        return Ok(FetchedPage {
            url: current_url,
            body,
            from_cache: false,
            status,
            headers,
        });
    }
}

/// Resolve a URL (possibly relative) against a base URL
pub fn resolve_url(url: &str, base: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }

    if url.starts_with("//") {
        // Protocol-relative
        let scheme = if base.starts_with("https") { "https" } else { "http" };
        return format!("{}:{}", scheme, url);
    }

    if url.starts_with('/') {
        // Absolute path — extract origin
        let end_of_scheme = base.find("://").map(|i| i + 3).unwrap_or(0);
        let path_start = base[end_of_scheme..]
            .find('/')
            .map(|i| i + end_of_scheme)
            .unwrap_or(base.len());
        return format!("{}{}", &base[..path_start], url);
    }

    // Relative path
    let last_slash = base.rfind('/').unwrap_or(base.len());
    let base_dir = &base[..=last_slash];
    format!("{}{}", base_dir, url)
}

/// Check if loading `resource_url` from an `https://` page would be mixed content
pub fn is_mixed_content(page_url: &str, resource_url: &str) -> bool {
    // Only HTTPS pages enforce mixed content blocking
    if !page_url.starts_with("https://") {
        return false;
    }
    // HTTP resources on HTTPS pages = mixed content
    resource_url.starts_with("http://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_absolute_url() {
        assert_eq!(
            resolve_url("https://other.com/page", "https://example.com/"),
            "https://other.com/page"
        );
    }

    #[test]
    fn test_resolve_absolute_path() {
        assert_eq!(
            resolve_url("/styles.css", "https://example.com/page/index.html"),
            "https://example.com/styles.css"
        );
    }

    #[test]
    fn test_resolve_relative_path() {
        assert_eq!(
            resolve_url("style.css", "https://example.com/page/index.html"),
            "https://example.com/page/style.css"
        );
    }

    #[test]
    fn test_resolve_protocol_relative() {
        assert_eq!(
            resolve_url("//cdn.example.com/lib.js", "https://example.com/"),
            "https://cdn.example.com/lib.js"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CSP + SRI enforcement helpers
// ─────────────────────────────────────────────────────────────────────────────

use crate::security::csp::{ContentSecurityPolicy, CspDirective};
use crate::security::sri::SubresourceIntegrity;

/// Enforce CSP on a fetched asset URL, given the policy parsed from the page headers.
/// Returns `Ok(())` if the resource is allowed, `Err(reason)` if blocked.
pub fn csp_check_asset(
    csp: &mut ContentSecurityPolicy,
    directive: CspDirective,
    url: &str,
    document_url: &str,
) -> Result<(), String> {
    if csp.check_and_report(directive, url, document_url) {
        Ok(())
    } else {
        Err(format!("CSP blocked {} for directive {:?}", url, directive))
    }
}

/// Parse CSP header from response headers. Returns None if no CSP header present.
pub fn parse_csp_from_headers(headers: &std::collections::HashMap<String, String>) -> Option<ContentSecurityPolicy> {
    headers
        .get("content-security-policy")
        .or_else(|| headers.get("Content-Security-Policy"))
        .map(|v| ContentSecurityPolicy::parse(v))
}

/// Verify a fetched asset's body against its SRI integrity attribute.
/// Returns `Ok(())` if integrity passes or no integrity specified.
/// Returns `Err(reason)` if integrity check fails.
pub fn sri_verify(integrity_attr: &str, content: &[u8]) -> Result<(), String> {
    if integrity_attr.is_empty() {
        return Ok(());
    }
    let sri = SubresourceIntegrity::parse(integrity_attr);
    if sri.verify(content) {
        Ok(())
    } else {
        Err(format!("SRI integrity check failed for attribute: {}", integrity_attr))
    }
}
