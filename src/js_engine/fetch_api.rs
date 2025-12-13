//! Fetch API implementation for JavaScript
//!
//! Provides fetch() and XMLHttpRequest APIs for AJAX requests from JavaScript.

use crate::utils::Result;
use std::collections::HashMap;

/// HTTP methods supported by Fetch API
#[derive(Debug, Clone, PartialEq)]
pub enum FetchMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl Default for FetchMethod {
    fn default() -> Self {
        Self::Get
    }
}

impl FetchMethod {
    /// Parse method from string
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            "PATCH" => Self::Patch,
            "HEAD" => Self::Head,
            "OPTIONS" => Self::Options,
            _ => Self::Get,
        }
    }
}

/// Fetch request configuration
#[derive(Debug, Clone, Default)]
pub struct FetchRequest {
    /// URL to fetch
    pub url: String,
    /// HTTP method
    pub method: FetchMethod,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body (for POST/PUT/PATCH)
    pub body: Option<String>,
    /// Credentials mode
    pub credentials: CredentialsMode,
    /// CORS mode
    pub mode: CorsMode,
}

impl FetchRequest {
    /// Create a new GET request
    pub fn get(url: &str) -> Self {
        Self {
            url: url.to_string(),
            method: FetchMethod::Get,
            ..Default::default()
        }
    }

    /// Create a new POST request
    pub fn post(url: &str, body: &str) -> Self {
        Self {
            url: url.to_string(),
            method: FetchMethod::Post,
            body: Some(body.to_string()),
            ..Default::default()
        }
    }

    /// Set a header
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }
}

/// Credentials mode for fetch
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CredentialsMode {
    #[default]
    SameOrigin,
    Include,
    Omit,
}

/// CORS mode for fetch
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CorsMode {
    #[default]
    Cors,
    NoCors,
    SameOrigin,
}

/// Fetch response
#[derive(Debug, Clone)]
pub struct FetchResponse {
    /// HTTP status code
    pub status: u16,
    /// Status text
    pub status_text: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    body: String,
    /// Whether the response is OK (2xx)
    pub ok: bool,
    /// Response URL (after redirects)
    pub url: String,
}

impl FetchResponse {
    /// Create a new response
    pub fn new(status: u16, body: String) -> Self {
        Self {
            status,
            status_text: Self::status_text(status),
            headers: HashMap::new(),
            body,
            ok: (200..300).contains(&status),
            url: String::new(),
        }
    }

    /// Get status text for code
    fn status_text(code: u16) -> String {
        match code {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            304 => "Not Modified",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            _ => "Unknown",
        }.to_string()
    }

    /// Get response body as text
    pub fn text(&self) -> String {
        self.body.clone()
    }

    /// Get raw body bytes
    pub fn body(&self) -> &str {
        &self.body
    }
}

/// Fetch API client for JavaScript
pub struct FetchClient {
    client: reqwest::blocking::Client,
}

impl FetchClient {
    /// Create a new fetch client
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Binix/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Execute a fetch request (blocking)
    pub fn fetch(&self, request: FetchRequest) -> Result<FetchResponse> {
        let method = match request.method {
            FetchMethod::Get => reqwest::Method::GET,
            FetchMethod::Post => reqwest::Method::POST,
            FetchMethod::Put => reqwest::Method::PUT,
            FetchMethod::Delete => reqwest::Method::DELETE,
            FetchMethod::Patch => reqwest::Method::PATCH,
            FetchMethod::Head => reqwest::Method::HEAD,
            FetchMethod::Options => reqwest::Method::OPTIONS,
        };

        let mut req = self.client.request(method, &request.url);

        // Add headers
        for (name, value) in &request.headers {
            req = req.header(name.as_str(), value.as_str());
        }

        // Add body
        if let Some(body) = request.body {
            req = req.body(body);
        }

        // Execute request
        let response = req.send()
            .map_err(|e| crate::utils::error::NetworkError::ConnectionFailed(e.to_string()))?;

        let status = response.status().as_u16();
        let headers: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let url = response.url().to_string();

        let body = response.text()
            .map_err(|e| crate::utils::error::NetworkError::ConnectionFailed(e.to_string()))?;

        let mut resp = FetchResponse::new(status, body);
        resp.headers = headers;
        resp.url = url;

        Ok(resp)
    }
}

impl Default for FetchClient {
    fn default() -> Self {
        Self::new()
    }
}

/// XMLHttpRequest state
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ReadyState {
    #[default]
    Unsent = 0,
    Opened = 1,
    HeadersReceived = 2,
    Loading = 3,
    Done = 4,
}

/// XMLHttpRequest for legacy AJAX support
#[derive(Debug, Default)]
pub struct XmlHttpRequest {
    /// Ready state
    pub ready_state: ReadyState,
    /// Response status
    pub status: u16,
    /// Response status text
    pub status_text: String,
    /// Response text
    pub response_text: String,
    /// Response headers
    response_headers: HashMap<String, String>,
    /// Request method
    method: FetchMethod,
    /// Request URL
    url: String,
    /// Request headers
    headers: HashMap<String, String>,
}

impl XmlHttpRequest {
    /// Create a new XMLHttpRequest
    pub fn new() -> Self {
        Self::default()
    }

    /// Open a request
    pub fn open(&mut self, method: &str, url: &str) {
        self.method = FetchMethod::from_str(method);
        self.url = url.to_string();
        self.ready_state = ReadyState::Opened;
    }

    /// Set a request header
    pub fn set_request_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    /// Send the request (blocking)
    pub fn send(&mut self, body: Option<&str>) {
        let client = FetchClient::new();

        let request = FetchRequest {
            url: self.url.clone(),
            method: self.method.clone(),
            headers: self.headers.clone(),
            body: body.map(|s| s.to_string()),
            ..Default::default()
        };

        self.ready_state = ReadyState::Loading;

        match client.fetch(request) {
            Ok(response) => {
                self.status = response.status;
                self.status_text = response.status_text.clone();
                self.response_headers = response.headers.clone();
                self.response_text = response.text();
                self.ready_state = ReadyState::Done;
            }
            Err(_) => {
                self.status = 0;
                self.status_text = "Network Error".to_string();
                self.ready_state = ReadyState::Done;
            }
        }
    }

    /// Get a response header
    pub fn get_response_header(&self, name: &str) -> Option<String> {
        self.response_headers.get(name).cloned()
    }

    /// Get all response headers
    pub fn get_all_response_headers(&self) -> String {
        self.response_headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\r\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_method_parsing() {
        assert_eq!(FetchMethod::from_str("GET"), FetchMethod::Get);
        assert_eq!(FetchMethod::from_str("post"), FetchMethod::Post);
        assert_eq!(FetchMethod::from_str("PUT"), FetchMethod::Put);
        assert_eq!(FetchMethod::from_str("DELETE"), FetchMethod::Delete);
    }

    #[test]
    fn test_fetch_request_builder() {
        let req = FetchRequest::get("https://example.com")
            .header("Accept", "application/json");

        assert_eq!(req.url, "https://example.com");
        assert_eq!(req.method, FetchMethod::Get);
        assert_eq!(req.headers.get("Accept"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_fetch_response() {
        let resp = FetchResponse::new(200, "Hello".to_string());
        assert!(resp.ok);
        assert_eq!(resp.status, 200);
        assert_eq!(resp.text(), "Hello");
    }

    #[test]
    fn test_xhr_open() {
        let mut xhr = XmlHttpRequest::new();
        xhr.open("GET", "https://example.com");

        assert_eq!(xhr.ready_state, ReadyState::Opened);
        assert_eq!(xhr.url, "https://example.com");
    }
}