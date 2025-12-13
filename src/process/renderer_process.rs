//! Renderer process implementation

use std::collections::HashMap;

use super::sandbox::{Sandbox, SandboxPolicy};

/// Renderer process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererState {
    Idle,
    Loading,
    Parsing,
    Rendering,
    Interactive,
}

/// Renderer process
pub struct RendererProcess {
    id: u32,
    origin: String,
    state: RendererState,
    sandbox: Sandbox,
    documents: HashMap<u64, DocumentState>,
    current_document: Option<u64>,
}

/// Document state in renderer
#[derive(Debug, Clone)]
pub struct DocumentState {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub ready_state: ReadyState,
}

/// Document ready state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadyState {
    Loading,
    Interactive,
    Complete,
}

impl RendererProcess {
    /// Create a new renderer process
    pub fn new(id: u32, origin: &str) -> Self {
        Self {
            id,
            origin: origin.to_string(),
            state: RendererState::Idle,
            sandbox: Sandbox::new(SandboxPolicy::strict()),
            documents: HashMap::new(),
            current_document: None,
        }
    }

    /// Get process ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get origin
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// Get current state
    pub fn state(&self) -> RendererState {
        self.state
    }

    /// Start loading a document
    pub fn load_document(&mut self, url: &str) -> u64 {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let doc_id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let doc = DocumentState {
            id: doc_id,
            url: url.to_string(),
            title: String::new(),
            ready_state: ReadyState::Loading,
        };

        self.documents.insert(doc_id, doc);
        self.current_document = Some(doc_id);
        self.state = RendererState::Loading;

        doc_id
    }

    /// Update document state
    pub fn update_document_state(&mut self, doc_id: u64, ready_state: ReadyState) {
        if let Some(doc) = self.documents.get_mut(&doc_id) {
            doc.ready_state = ready_state;
        }

        self.state = match ready_state {
            ReadyState::Loading => RendererState::Loading,
            ReadyState::Interactive => RendererState::Interactive,
            ReadyState::Complete => RendererState::Idle,
        };
    }

    /// Set document title
    pub fn set_document_title(&mut self, doc_id: u64, title: &str) {
        if let Some(doc) = self.documents.get_mut(&doc_id) {
            doc.title = title.to_string();
        }
    }

    /// Get current document
    pub fn current_document(&self) -> Option<&DocumentState> {
        self.current_document.and_then(|id| self.documents.get(&id))
    }

    /// Check if same origin
    pub fn is_same_origin(&self, url: &str) -> bool {
        // Simplified origin check
        if let Some(origin) = extract_origin(url) {
            origin == self.origin
        } else {
            false
        }
    }

    /// Get sandbox
    pub fn sandbox(&self) -> &Sandbox {
        &self.sandbox
    }
}

/// Extract origin from URL
fn extract_origin(url: &str) -> Option<String> {
    // Simple extraction: protocol://host:port
    let url = url.trim();
    if let Some(proto_end) = url.find("://") {
        let rest = &url[proto_end + 3..];
        let host_end = rest.find('/').unwrap_or(rest.len());
        let host_port = &rest[..host_end];
        Some(format!("{}://{}", &url[..proto_end], host_port))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = RendererProcess::new(1, "https://example.com");
        assert_eq!(renderer.id(), 1);
        assert_eq!(renderer.origin(), "https://example.com");
    }

    #[test]
    fn test_load_document() {
        let mut renderer = RendererProcess::new(1, "https://example.com");
        let doc_id = renderer.load_document("https://example.com/page");
        assert!(renderer.current_document().is_some());
        assert_eq!(renderer.state(), RendererState::Loading);
    }

    #[test]
    fn test_same_origin() {
        let renderer = RendererProcess::new(1, "https://example.com");
        assert!(renderer.is_same_origin("https://example.com/page"));
        assert!(!renderer.is_same_origin("https://other.com/page"));
    }

    #[test]
    fn test_extract_origin() {
        assert_eq!(
            extract_origin("https://example.com/page"),
            Some("https://example.com".to_string())
        );
        assert_eq!(
            extract_origin("http://localhost:8080/api"),
            Some("http://localhost:8080".to_string())
        );
    }
}
