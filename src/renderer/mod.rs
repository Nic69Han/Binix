//! Rendering engine for Binix browser
//!
//! Handles HTML/CSS parsing and layout computation with parallel processing.

pub mod css;
mod dirty_tracking;
mod dom;
pub mod html;
mod layout;
mod layout_batch;
mod streaming;
mod style;

pub use css::CssParser;
pub use dirty_tracking::{DirtyTracker, LayoutChange, Rect};
pub use dom::{Document, ElementData, Node, NodeType};
pub use html::HtmlParser;
pub use layout::{LayoutBox, LayoutEngine};
pub use layout_batch::{BatchConfig, BatchResult, LayoutBatcher};
pub use streaming::{ParsedChunk, ParserState, StreamingParser};
pub use style::StyleEngine;

use crate::utils::Result;

/// Trait for rendering engines
pub trait RenderingEngine: Send + Sync {
    /// Parse HTML content into a DOM tree
    fn parse_html(&self, content: &str) -> Result<Document>;

    /// Parse CSS content
    fn parse_css(&self, content: &str) -> Result<css::Stylesheet>;

    /// Compute layout for a document
    fn compute_layout(&self, document: &Document) -> Result<LayoutBox>;
}

/// Default rendering engine implementation
pub struct DefaultRenderingEngine {
    html_parser: HtmlParser,
    css_parser: CssParser,
    layout_engine: LayoutEngine,
    style_engine: StyleEngine,
}

impl DefaultRenderingEngine {
    /// Create a new rendering engine
    pub fn new() -> Self {
        Self {
            html_parser: HtmlParser::new(),
            css_parser: CssParser::new(),
            layout_engine: LayoutEngine::new(),
            style_engine: StyleEngine::new(),
        }
    }
}

impl Default for DefaultRenderingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderingEngine for DefaultRenderingEngine {
    fn parse_html(&self, content: &str) -> Result<Document> {
        self.html_parser.parse(content)
    }

    fn parse_css(&self, content: &str) -> Result<css::Stylesheet> {
        self.css_parser.parse(content)
    }

    fn compute_layout(&self, document: &Document) -> Result<LayoutBox> {
        self.layout_engine.compute(document)
    }
}
