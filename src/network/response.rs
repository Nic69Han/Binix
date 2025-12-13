//! HTTP response types

use std::collections::HashMap;

/// HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

impl Response {
    /// Create a new response
    pub fn new(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    /// Create a new response with headers
    pub fn with_headers(
        status: u16,
        body: impl Into<String>,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            status,
            headers,
            body: body.into(),
        }
    }

    /// Get the status code
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Check if the response was successful (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if the response is a redirect (3xx)
    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status)
    }

    /// Check if the response is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if the response is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// Get the response body
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Get the content type from headers
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(|s| s.as_str())
    }

    /// Get content length from headers
    pub fn content_length(&self) -> Option<usize> {
        self.headers
            .get("content-length")
            .and_then(|s| s.parse().ok())
    }

    /// Get response headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get a specific header
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    /// Add a header
    pub fn add_header(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.headers.insert(key.into(), value.into());
    }
}
