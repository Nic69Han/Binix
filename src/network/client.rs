//! HTTP client implementation using reqwest

use super::{Request, Response, request::Method};
use crate::utils::{Result, error::NetworkError};
use reqwest::Client;
use std::time::Duration;

/// HTTP client with connection pooling and HTTPS support
pub struct NetworkClient {
    client: Client,
    timeout: Duration,
}

impl NetworkClient {
    /// Create a new HTTP client with default settings
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(30))
    }

    /// Create a new HTTP client with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .user_agent("Binix/0.1.0 (Rust Web Browser)")
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, timeout }
    }

    /// Execute an HTTP request
    pub async fn execute(&self, request: Request) -> Result<Response> {
        let method = match request.method() {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
            Method::Patch => reqwest::Method::PATCH,
        };

        let mut req_builder = self.client.request(method, request.url());

        // Add headers
        for (key, value) in request.headers() {
            req_builder = req_builder.header(key, value);
        }

        // Add body if present
        if let Some(body) = request.body_bytes() {
            req_builder = req_builder.body(body.to_vec());
        }

        // Execute request
        let response = req_builder
            .send()
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

        // Convert to our Response type
        let status = response.status().as_u16();
        let headers: std::collections::HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body: String = response
            .text()
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

        Ok(Response::with_headers(status, body, headers))
    }

    /// Execute a GET request
    pub async fn get(&self, url: &str) -> Result<Response> {
        let request = Request::get(url)?;
        self.execute(request).await
    }

    /// Execute a POST request with body
    pub async fn post(&self, url: &str, body: Vec<u8>) -> Result<Response> {
        let request = Request::post(url)?.body(body);
        self.execute(request).await
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Default for NetworkClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = NetworkClient::new();
        assert_eq!(client.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_client_with_timeout() {
        let client = NetworkClient::with_timeout(Duration::from_secs(60));
        assert_eq!(client.timeout(), Duration::from_secs(60));
    }

    #[tokio::test]
    #[ignore] // Requires network access - run with `cargo test -- --ignored`
    async fn test_get_request() {
        let client = NetworkClient::new();
        // Test with a known working URL
        let result = client.get("https://httpbin.org/get").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let request = Request::get("not-a-valid-url");
        assert!(request.is_err());
    }

    #[tokio::test]
    async fn test_connection_error() {
        let client = NetworkClient::with_timeout(Duration::from_millis(100));
        let result = client
            .get("https://invalid.domain.that.does.not.exist/")
            .await;
        assert!(result.is_err());
    }
}
