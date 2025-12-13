//! Secure Cookie Management
//!
//! Implements RFC 6265 cookies with modern security attributes.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// SameSite attribute for CSRF protection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SameSite {
    /// Strict - only same-site requests
    Strict,
    /// Lax - same-site + top-level navigation
    #[default]
    Lax,
    /// None - cross-site allowed (requires Secure)
    None,
}

impl SameSite {
    /// Parse from string
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "strict" => Self::Strict,
            "none" => Self::None,
            _ => Self::Lax,
        }
    }
}

/// A parsed cookie with security attributes
#[derive(Debug, Clone)]
pub struct Cookie {
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Domain the cookie applies to
    pub domain: Option<String>,
    /// Path the cookie applies to
    pub path: String,
    /// Expiration time (None = session cookie)
    pub expires: Option<SystemTime>,
    /// Max-Age in seconds
    pub max_age: Option<Duration>,
    /// Secure flag (HTTPS only)
    pub secure: bool,
    /// HttpOnly flag (no JS access)
    pub http_only: bool,
    /// SameSite attribute
    pub same_site: SameSite,
    /// Creation time
    pub created_at: SystemTime,
    /// Last access time
    pub last_accessed: SystemTime,
}

impl Cookie {
    /// Create a new session cookie
    pub fn new(name: &str, value: &str) -> Self {
        let now = SystemTime::now();
        Self {
            name: name.to_string(),
            value: value.to_string(),
            domain: None,
            path: "/".to_string(),
            expires: None,
            max_age: None,
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            created_at: now,
            last_accessed: now,
        }
    }

    /// Parse a Set-Cookie header
    pub fn parse(header: &str) -> Option<Self> {
        let mut parts = header.split(';');
        let name_value = parts.next()?.trim();

        let (name, value) = name_value.split_once('=')?;
        let mut cookie = Self::new(name.trim(), value.trim());

        for part in parts {
            let part = part.trim();
            if let Some((attr, val)) = part.split_once('=') {
                match attr.to_lowercase().as_str() {
                    "domain" => cookie.domain = Some(val.trim_start_matches('.').to_string()),
                    "path" => cookie.path = val.to_string(),
                    "max-age" => {
                        if let Ok(secs) = val.parse::<u64>() {
                            cookie.max_age = Some(Duration::from_secs(secs));
                        }
                    }
                    "samesite" => cookie.same_site = SameSite::parse(val),
                    "expires" => {
                        // Simplified date parsing
                        cookie.expires = Self::parse_expires(val);
                    }
                    _ => {}
                }
            } else {
                match part.to_lowercase().as_str() {
                    "secure" => cookie.secure = true,
                    "httponly" => cookie.http_only = true,
                    _ => {}
                }
            }
        }

        // SameSite=None requires Secure
        if cookie.same_site == SameSite::None && !cookie.secure {
            cookie.same_site = SameSite::Lax;
        }

        Some(cookie)
    }

    /// Parse expires date (simplified)
    fn parse_expires(_date_str: &str) -> Option<SystemTime> {
        // In production, use chrono or time crate for proper parsing
        // For now, treat as session cookie
        None
    }

    /// Check if cookie is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now();

        // Check max-age first
        if let Some(max_age) = self.max_age {
            if let Ok(elapsed) = now.duration_since(self.created_at) {
                return elapsed > max_age;
            }
        }

        // Check expires
        if let Some(expires) = self.expires {
            return now > expires;
        }

        false
    }

    /// Check if cookie should be sent for request
    pub fn should_send(&self, url: &str, is_same_site: bool, is_top_level: bool) -> bool {
        // Check expiration
        if self.is_expired() {
            return false;
        }

        // Check secure flag
        if self.secure && !url.starts_with("https://") {
            return false;
        }

        // Check SameSite
        if !is_same_site {
            match self.same_site {
                SameSite::Strict => return false,
                SameSite::Lax => {
                    if !is_top_level {
                        return false;
                    }
                }
                SameSite::None => {} // Allow if Secure
            }
        }

        true
    }
}

/// Cookie jar for storing and managing cookies
#[derive(Default)]
pub struct CookieJar {
    /// Cookies indexed by domain
    cookies: HashMap<String, Vec<Cookie>>,
    /// Maximum cookies per domain
    max_per_domain: usize,
    /// Maximum total cookies
    max_total: usize,
}

impl CookieJar {
    /// Create a new cookie jar
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
            max_per_domain: 50,
            max_total: 3000,
        }
    }

    /// Add or update a cookie
    pub fn set(&mut self, cookie: Cookie, request_domain: &str) {
        let domain = cookie.domain.clone()
            .unwrap_or_else(|| request_domain.to_string());

        // Validate domain (prevent cookie injection)
        if !Self::is_valid_domain(&domain, request_domain) {
            return;
        }

        let domain_cookies = self.cookies.entry(domain).or_default();

        // Remove existing cookie with same name/path
        domain_cookies.retain(|c| c.name != cookie.name || c.path != cookie.path);

        // Check limits
        if domain_cookies.len() >= self.max_per_domain {
            // Remove oldest cookie
            if let Some(oldest_idx) = domain_cookies.iter()
                .enumerate()
                .min_by_key(|(_, c)| c.last_accessed)
                .map(|(i, _)| i)
            {
                domain_cookies.remove(oldest_idx);
            }
        }

        domain_cookies.push(cookie);
    }

    /// Get cookies for a request
    pub fn get_for_request(&mut self, url: &str, is_same_site: bool, is_top_level: bool) -> Vec<&Cookie> {
        let domain = Self::extract_domain(url);

        // Collect matching cookies
        let mut result = Vec::new();

        for (cookie_domain, cookies) in &mut self.cookies {
            if Self::domain_matches(cookie_domain, &domain) {
                for cookie in cookies.iter_mut() {
                    if cookie.should_send(url, is_same_site, is_top_level) {
                        cookie.last_accessed = SystemTime::now();
                    }
                }

                result.extend(
                    cookies.iter()
                        .filter(|c| c.should_send(url, is_same_site, is_top_level))
                );
            }
        }

        result
    }

    /// Build Cookie header value
    pub fn build_header(&mut self, url: &str, is_same_site: bool, is_top_level: bool) -> Option<String> {
        let cookies = self.get_for_request(url, is_same_site, is_top_level);

        if cookies.is_empty() {
            return None;
        }

        Some(cookies.iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; "))
    }

    /// Remove expired cookies
    pub fn cleanup(&mut self) {
        for cookies in self.cookies.values_mut() {
            cookies.retain(|c| !c.is_expired());
        }

        self.cookies.retain(|_, v| !v.is_empty());
    }

    /// Clear all cookies for a domain
    pub fn clear_domain(&mut self, domain: &str) {
        self.cookies.remove(domain);
    }

    /// Clear all cookies
    pub fn clear_all(&mut self) {
        self.cookies.clear();
    }

    /// Validate domain for cookie setting
    fn is_valid_domain(cookie_domain: &str, request_domain: &str) -> bool {
        // Cookie domain must be a suffix of request domain
        request_domain.ends_with(cookie_domain) ||
        request_domain == cookie_domain
    }

    /// Check if cookie domain matches request domain
    fn domain_matches(cookie_domain: &str, request_domain: &str) -> bool {
        request_domain == cookie_domain ||
        request_domain.ends_with(&format!(".{}", cookie_domain))
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> String {
        url.trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("")
            .to_string()
    }

    /// Get total cookie count
    pub fn count(&self) -> usize {
        self.cookies.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_parse() {
        let cookie = Cookie::parse("session=abc123; Secure; HttpOnly; SameSite=Strict").unwrap();

        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert_eq!(cookie.same_site, SameSite::Strict);
    }

    #[test]
    fn test_samesite_none_requires_secure() {
        let cookie = Cookie::parse("test=value; SameSite=None").unwrap();

        // Should be downgraded to Lax because Secure is missing
        assert_eq!(cookie.same_site, SameSite::Lax);
    }

    #[test]
    fn test_cookie_jar() {
        let mut jar = CookieJar::new();

        let cookie = Cookie::parse("user=john; Path=/").unwrap();
        jar.set(cookie, "example.com");

        assert_eq!(jar.count(), 1);

        let header = jar.build_header("https://example.com/page", true, true);
        assert_eq!(header, Some("user=john".to_string()));
    }

    #[test]
    fn test_secure_cookie_https_only() {
        let mut jar = CookieJar::new();

        let cookie = Cookie::parse("secure=value; Secure").unwrap();
        jar.set(cookie, "example.com");

        // Should not be sent over HTTP
        let http_header = jar.build_header("http://example.com/", true, true);
        assert!(http_header.is_none());

        // Should be sent over HTTPS
        let https_header = jar.build_header("https://example.com/", true, true);
        assert!(https_header.is_some());
    }
}