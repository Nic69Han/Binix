//! Network stack for Binix browser
//!
//! Implements HTTP/3 with connection pooling and predictive loading.

mod client;
mod dns;
mod http3;
mod request;
mod response;

pub use client::NetworkClient;
pub use http3::{Http3Client, Http3Config, Http3Connection};
pub use request::Request;
pub use response::Response;

use crate::utils::Result;

/// Network stack handling all HTTP communications
pub struct NetworkStack {
    client: NetworkClient,
}

impl NetworkStack {
    /// Create a new network stack
    pub fn new() -> Self {
        Self {
            client: NetworkClient::new(),
        }
    }

    /// Fetch a resource from the given URL
    pub async fn fetch(&self, url: &str) -> Result<Response> {
        let request = Request::get(url)?;
        self.client.execute(request).await
    }

    /// Prefetch resources for predictive loading
    pub async fn prefetch(&self, urls: &[&str]) -> Vec<Result<Response>> {
        let futures: Vec<_> = urls.iter().map(|url| self.fetch(url)).collect();
        futures::future::join_all(futures).await
    }
}

impl Default for NetworkStack {
    fn default() -> Self {
        Self::new()
    }
}

