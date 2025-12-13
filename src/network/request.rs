//! HTTP request types

use crate::utils::{error::NetworkError, Result};
use std::collections::HashMap;

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

/// HTTP request
#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    url: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl Request {
    /// Create a new request
    pub fn new(method: Method, url: impl Into<String>) -> Result<Self> {
        let url = url.into();
        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(NetworkError::InvalidUrl(url).into());
        }
        Ok(Self {
            method,
            url,
            headers: HashMap::new(),
            body: None,
        })
    }

    /// Create a GET request
    pub fn get(url: impl Into<String>) -> Result<Self> {
        Self::new(Method::Get, url)
    }

    /// Create a POST request
    pub fn post(url: impl Into<String>) -> Result<Self> {
        Self::new(Method::Post, url)
    }

    /// Add a header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the request body
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Get the URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the method
    pub fn method(&self) -> Method {
        self.method
    }

    /// Get headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

