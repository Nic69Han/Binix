//! Network Inspector implementation

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}

/// Request status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestStatus {
    Pending,
    Complete,
    Failed,
    Cancelled,
}

/// Resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Document,
    Stylesheet,
    Script,
    Image,
    Font,
    Xhr,
    Fetch,
    WebSocket,
    Other,
}

/// A network request
#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub id: u64,
    pub url: String,
    pub method: HttpMethod,
    pub status: RequestStatus,
    pub status_code: Option<u16>,
    pub resource_type: ResourceType,
    pub request_headers: HashMap<String, String>,
    pub response_headers: HashMap<String, String>,
    pub request_size: usize,
    pub response_size: usize,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub duration: Option<Duration>,
    pub error_message: Option<String>,
}

impl NetworkRequest {
    /// Create a new network request
    pub fn new(id: u64, url: &str, method: HttpMethod, resource_type: ResourceType) -> Self {
        Self {
            id,
            url: url.to_string(),
            method,
            status: RequestStatus::Pending,
            status_code: None,
            resource_type,
            request_headers: HashMap::new(),
            response_headers: HashMap::new(),
            request_size: 0,
            response_size: 0,
            start_time: Some(Instant::now()),
            end_time: None,
            duration: None,
            error_message: None,
        }
    }

    /// Mark request as complete
    pub fn complete(&mut self, status_code: u16, response_size: usize) {
        self.status = RequestStatus::Complete;
        self.status_code = Some(status_code);
        self.response_size = response_size;
        self.end_time = Some(Instant::now());
        if let Some(start) = self.start_time {
            self.duration = Some(self.end_time.unwrap().duration_since(start));
        }
    }

    /// Mark request as failed
    pub fn fail(&mut self, error: &str) {
        self.status = RequestStatus::Failed;
        self.error_message = Some(error.to_string());
        self.end_time = Some(Instant::now());
        if let Some(start) = self.start_time {
            self.duration = Some(self.end_time.unwrap().duration_since(start));
        }
    }
}

/// Network Inspector for monitoring requests
pub struct NetworkInspector {
    requests: Vec<NetworkRequest>,
    next_id: u64,
    filter_type: Option<ResourceType>,
    preserve_log: bool,
}

impl NetworkInspector {
    /// Create a new network inspector
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            next_id: 1,
            filter_type: None,
            preserve_log: false,
        }
    }

    /// Start tracking a new request
    pub fn start_request(
        &mut self,
        url: &str,
        method: HttpMethod,
        resource_type: ResourceType,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let request = NetworkRequest::new(id, url, method, resource_type);
        self.requests.push(request);
        id
    }

    /// Complete a request
    pub fn complete_request(&mut self, id: u64, status_code: u16, response_size: usize) {
        if let Some(req) = self.requests.iter_mut().find(|r| r.id == id) {
            req.complete(status_code, response_size);
        }
    }

    /// Fail a request
    pub fn fail_request(&mut self, id: u64, error: &str) {
        if let Some(req) = self.requests.iter_mut().find(|r| r.id == id) {
            req.fail(error);
        }
    }

    /// Get all requests
    pub fn requests(&self) -> impl Iterator<Item = &NetworkRequest> {
        self.requests.iter().filter(|r| {
            self.filter_type
                .map(|f| r.resource_type == f)
                .unwrap_or(true)
        })
    }

    /// Get request by ID
    pub fn get_request(&self, id: u64) -> Option<&NetworkRequest> {
        self.requests.iter().find(|r| r.id == id)
    }

    /// Clear all requests
    pub fn clear(&mut self) {
        if !self.preserve_log {
            self.requests.clear();
        }
    }

    /// Set filter type
    pub fn set_filter(&mut self, filter: Option<ResourceType>) {
        self.filter_type = filter;
    }

    /// Set preserve log
    pub fn set_preserve_log(&mut self, preserve: bool) {
        self.preserve_log = preserve;
    }

    /// Get total request count
    pub fn request_count(&self) -> usize {
        self.requests.len()
    }

    /// Get total transferred size
    pub fn total_transferred(&self) -> usize {
        self.requests.iter().map(|r| r.response_size).sum()
    }
}

impl Default for NetworkInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_request_creation() {
        let req = NetworkRequest::new(
            1,
            "https://example.com",
            HttpMethod::Get,
            ResourceType::Document,
        );
        assert_eq!(req.status, RequestStatus::Pending);
        assert_eq!(req.url, "https://example.com");
    }

    #[test]
    fn test_network_request_complete() {
        let mut req = NetworkRequest::new(
            1,
            "https://example.com",
            HttpMethod::Get,
            ResourceType::Document,
        );
        req.complete(200, 1024);
        assert_eq!(req.status, RequestStatus::Complete);
        assert_eq!(req.status_code, Some(200));
        assert_eq!(req.response_size, 1024);
    }

    #[test]
    fn test_network_inspector_track() {
        let mut inspector = NetworkInspector::new();
        let id = inspector.start_request(
            "https://example.com",
            HttpMethod::Get,
            ResourceType::Document,
        );
        assert_eq!(inspector.request_count(), 1);
        inspector.complete_request(id, 200, 512);
        let req = inspector.get_request(id).unwrap();
        assert_eq!(req.status, RequestStatus::Complete);
    }

    #[test]
    fn test_network_inspector_fail() {
        let mut inspector = NetworkInspector::new();
        let id = inspector.start_request(
            "https://example.com",
            HttpMethod::Get,
            ResourceType::Document,
        );
        inspector.fail_request(id, "Connection refused");
        let req = inspector.get_request(id).unwrap();
        assert_eq!(req.status, RequestStatus::Failed);
    }

    #[test]
    fn test_http_method_str() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
    }
}
