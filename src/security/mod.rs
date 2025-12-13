//! Security module for Binix browser
//!
//! Implements security features:
//! - Content Security Policy (CSP)
//! - Subresource Integrity (SRI)
//! - Cross-Origin Resource Sharing (CORS)
//! - Mixed content blocking

mod cors;
pub mod csp;
mod sri;

pub use cors::{CorsPolicy, CorsRequest, CorsResult};
pub use csp::{ContentSecurityPolicy, CspDirective, CspViolation};
pub use sri::{SriAlgorithm, SriHash, SubresourceIntegrity};

/// Security manager coordinating all security features
pub struct SecurityManager {
    csp_enabled: bool,
    sri_enabled: bool,
    cors_enabled: bool,
    mixed_content_blocking: bool,
}

impl SecurityManager {
    /// Create a new security manager with all features enabled
    pub fn new() -> Self {
        Self {
            csp_enabled: true,
            sri_enabled: true,
            cors_enabled: true,
            mixed_content_blocking: true,
        }
    }

    /// Check if CSP is enabled
    pub fn csp_enabled(&self) -> bool {
        self.csp_enabled
    }

    /// Check if SRI is enabled
    pub fn sri_enabled(&self) -> bool {
        self.sri_enabled
    }

    /// Check if CORS is enabled
    pub fn cors_enabled(&self) -> bool {
        self.cors_enabled
    }

    /// Check if mixed content blocking is enabled
    pub fn mixed_content_blocking(&self) -> bool {
        self.mixed_content_blocking
    }

    /// Enable/disable CSP
    pub fn set_csp_enabled(&mut self, enabled: bool) {
        self.csp_enabled = enabled;
    }

    /// Enable/disable SRI
    pub fn set_sri_enabled(&mut self, enabled: bool) {
        self.sri_enabled = enabled;
    }

    /// Check if a URL is secure (HTTPS)
    pub fn is_secure_url(url: &str) -> bool {
        url.starts_with("https://") || url.starts_with("wss://")
    }

    /// Check for mixed content
    pub fn check_mixed_content(&self, page_url: &str, resource_url: &str) -> bool {
        if !self.mixed_content_blocking {
            return true;
        }

        // If page is HTTPS, resource must also be HTTPS
        if Self::is_secure_url(page_url) && !Self::is_secure_url(resource_url) {
            return false;
        }

        true
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_manager_creation() {
        let manager = SecurityManager::new();
        assert!(manager.csp_enabled());
        assert!(manager.sri_enabled());
        assert!(manager.cors_enabled());
    }

    #[test]
    fn test_is_secure_url() {
        assert!(SecurityManager::is_secure_url("https://example.com"));
        assert!(SecurityManager::is_secure_url("wss://example.com"));
        assert!(!SecurityManager::is_secure_url("http://example.com"));
        assert!(!SecurityManager::is_secure_url("ws://example.com"));
    }

    #[test]
    fn test_mixed_content() {
        let manager = SecurityManager::new();

        // HTTPS page loading HTTPS resource - OK
        assert!(
            manager.check_mixed_content("https://example.com", "https://cdn.example.com/script.js")
        );

        // HTTPS page loading HTTP resource - blocked
        assert!(
            !manager.check_mixed_content("https://example.com", "http://cdn.example.com/script.js")
        );

        // HTTP page loading HTTP resource - OK
        assert!(
            manager.check_mixed_content("http://example.com", "http://cdn.example.com/script.js")
        );
    }
}
