//! Page representation

use crate::compositor::Frame;
use crate::renderer::Document;

/// A fully processed web page
#[derive(Debug, Clone)]
pub struct Page {
    /// The page URL
    url: String,
    /// The DOM document
    document: Document,
    /// The rendered frame
    frame: Frame,
}

impl Page {
    /// Create a new page
    pub fn new(url: String, document: Document, frame: Frame) -> Self {
        Self {
            url,
            document,
            frame,
        }
    }

    /// Get the page URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the DOM document
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Get the rendered frame
    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    /// Get frame dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.frame.width, self.frame.height)
    }
}

