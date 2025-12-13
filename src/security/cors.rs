//! Cross-Origin Resource Sharing (CORS) implementation

use std::collections::HashSet;

/// CORS request type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorsRequestType {
    /// Simple request (GET, HEAD, POST with simple headers)
    Simple,
    /// Preflight required
    Preflight,
}

/// CORS request information
#[derive(Debug, Clone)]
pub struct CorsRequest {
    pub origin: String,
    pub method: String,
    pub headers: Vec<String>,
    pub request_type: CorsRequestType,
}

impl CorsRequest {
    /// Create a new CORS request
    pub fn new(origin: &str, method: &str) -> Self {
        let request_type = if Self::is_simple_method(method) {
            CorsRequestType::Simple
        } else {
            CorsRequestType::Preflight
        };

        Self {
            origin: origin.to_string(),
            method: method.to_string(),
            headers: Vec::new(),
            request_type,
        }
    }

    /// Check if method is a simple method
    fn is_simple_method(method: &str) -> bool {
        matches!(method.to_uppercase().as_str(), "GET" | "HEAD" | "POST")
    }

    /// Add a custom header
    pub fn add_header(&mut self, header: &str) {
        self.headers.push(header.to_string());
        // Custom headers require preflight
        if !Self::is_simple_header(header) {
            self.request_type = CorsRequestType::Preflight;
        }
    }

    /// Check if header is a simple header
    fn is_simple_header(header: &str) -> bool {
        let simple_headers = [
            "accept",
            "accept-language",
            "content-language",
            "content-type",
        ];
        simple_headers.contains(&header.to_lowercase().as_str())
    }
}

/// CORS check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorsResult {
    /// Request is allowed
    Allowed,
    /// Request is blocked
    Blocked(String),
    /// Preflight required
    PreflightRequired,
}

/// CORS policy for a resource
#[derive(Debug, Clone, Default)]
pub struct CorsPolicy {
    /// Allowed origins (* for any)
    allowed_origins: HashSet<String>,
    /// Allow any origin
    allow_any_origin: bool,
    /// Allowed methods
    allowed_methods: HashSet<String>,
    /// Allowed headers
    allowed_headers: HashSet<String>,
    /// Exposed headers (accessible to JS)
    exposed_headers: HashSet<String>,
    /// Allow credentials
    allow_credentials: bool,
    /// Max age for preflight cache (seconds)
    max_age: Option<u32>,
}

/// Preflight response for CORS
#[derive(Debug, Clone)]
pub struct PreflightResponse {
    /// Whether the preflight is allowed
    pub allowed: bool,
    /// Access-Control-Allow-Origin header
    pub allow_origin: Option<String>,
    /// Access-Control-Allow-Methods header
    pub allow_methods: Vec<String>,
    /// Access-Control-Allow-Headers header
    pub allow_headers: Vec<String>,
    /// Access-Control-Expose-Headers header
    pub expose_headers: Vec<String>,
    /// Access-Control-Allow-Credentials header
    pub allow_credentials: bool,
    /// Access-Control-Max-Age header
    pub max_age: Option<u32>,
}

impl CorsPolicy {
    /// Create a new CORS policy
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a permissive policy (allow all)
    pub fn permissive() -> Self {
        Self {
            allow_any_origin: true,
            allowed_methods: ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_headers: HashSet::new(),
            allow_credentials: false,
            max_age: Some(86400),
            ..Default::default()
        }
    }

    /// Create a restrictive policy (same-origin only)
    pub fn restrictive() -> Self {
        Self::default()
    }

    /// Allow a specific origin
    pub fn allow_origin(&mut self, origin: &str) {
        if origin == "*" {
            self.allow_any_origin = true;
        } else {
            self.allowed_origins.insert(origin.to_string());
        }
    }

    /// Allow a method
    pub fn allow_method(&mut self, method: &str) {
        self.allowed_methods.insert(method.to_uppercase());
    }

    /// Allow a header
    pub fn allow_header(&mut self, header: &str) {
        self.allowed_headers.insert(header.to_lowercase());
    }

    /// Set allow credentials
    pub fn set_allow_credentials(&mut self, allow: bool) {
        self.allow_credentials = allow;
    }

    /// Check if a request is allowed
    pub fn check(&self, request: &CorsRequest) -> CorsResult {
        // Check origin
        if !self.is_origin_allowed(&request.origin) {
            return CorsResult::Blocked(format!("Origin '{}' not allowed", request.origin));
        }

        // Check method
        if !self.is_method_allowed(&request.method) {
            return CorsResult::Blocked(format!("Method '{}' not allowed", request.method));
        }

        // Check headers
        for header in &request.headers {
            if !self.is_header_allowed(header) {
                return CorsResult::Blocked(format!("Header '{}' not allowed", header));
            }
        }

        CorsResult::Allowed
    }

    /// Check if origin is allowed
    fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allow_any_origin || self.allowed_origins.contains(origin)
    }

    /// Check if method is allowed
    fn is_method_allowed(&self, method: &str) -> bool {
        self.allowed_methods.is_empty() || self.allowed_methods.contains(&method.to_uppercase())
    }

    /// Check if header is allowed
    fn is_header_allowed(&self, header: &str) -> bool {
        self.allowed_headers.is_empty() || self.allowed_headers.contains(&header.to_lowercase())
    }

    /// Add an exposed header (accessible to JS)
    pub fn expose_header(&mut self, header: &str) {
        self.exposed_headers.insert(header.to_lowercase());
    }

    /// Handle preflight request
    pub fn preflight(&self, request: &CorsRequest) -> PreflightResponse {
        let allowed = matches!(self.check(request), CorsResult::Allowed);

        PreflightResponse {
            allowed,
            allow_origin: if allowed {
                if self.allow_any_origin {
                    Some("*".to_string())
                } else {
                    Some(request.origin.clone())
                }
            } else {
                None
            },
            allow_methods: self.allowed_methods.iter().cloned().collect(),
            allow_headers: self.allowed_headers.iter().cloned().collect(),
            expose_headers: self.exposed_headers.iter().cloned().collect(),
            allow_credentials: self.allow_credentials,
            max_age: self.max_age,
        }
    }

    /// Parse CORS policy from response headers
    pub fn from_headers(headers: &[(String, String)]) -> Self {
        let mut policy = Self::new();

        for (name, value) in headers {
            match name.to_lowercase().as_str() {
                "access-control-allow-origin" => {
                    policy.allow_origin(value);
                }
                "access-control-allow-methods" => {
                    for method in value.split(',') {
                        policy.allow_method(method.trim());
                    }
                }
                "access-control-allow-headers" => {
                    for header in value.split(',') {
                        policy.allow_header(header.trim());
                    }
                }
                "access-control-expose-headers" => {
                    for header in value.split(',') {
                        policy.expose_header(header.trim());
                    }
                }
                "access-control-allow-credentials" => {
                    policy.allow_credentials = value.to_lowercase() == "true";
                }
                "access-control-max-age" => {
                    policy.max_age = value.parse().ok();
                }
                _ => {}
            }
        }

        policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_request_simple() {
        let request = CorsRequest::new("https://example.com", "GET");
        assert_eq!(request.request_type, CorsRequestType::Simple);
    }

    #[test]
    fn test_cors_request_preflight() {
        let request = CorsRequest::new("https://example.com", "DELETE");
        assert_eq!(request.request_type, CorsRequestType::Preflight);
    }

    #[test]
    fn test_cors_policy_permissive() {
        let policy = CorsPolicy::permissive();
        let request = CorsRequest::new("https://any-origin.com", "POST");
        assert_eq!(policy.check(&request), CorsResult::Allowed);
    }

    #[test]
    fn test_cors_policy_restrictive() {
        let policy = CorsPolicy::restrictive();
        let request = CorsRequest::new("https://other.com", "GET");
        assert!(matches!(policy.check(&request), CorsResult::Blocked(_)));
    }

    #[test]
    fn test_cors_policy_specific_origin() {
        let mut policy = CorsPolicy::new();
        policy.allow_origin("https://trusted.com");
        policy.allow_method("GET");

        let allowed = CorsRequest::new("https://trusted.com", "GET");
        let blocked = CorsRequest::new("https://untrusted.com", "GET");

        assert_eq!(policy.check(&allowed), CorsResult::Allowed);
        assert!(matches!(policy.check(&blocked), CorsResult::Blocked(_)));
    }
}
