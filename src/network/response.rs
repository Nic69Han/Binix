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

    /// Get the status code
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Check if the response was successful (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Get the response body
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Get response headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get a specific header
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}

