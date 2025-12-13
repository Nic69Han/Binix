//! Core browser engine orchestrating all components
//!
//! The BrowserEngine coordinates the rendering pipeline:
//! 1. Fetch HTML/CSS/JS resources via NetworkStack
//! 2. Parse HTML into DOM and CSS into stylesheets
//! 3. Apply styles and compute layout
//! 4. Paint and composite to screen via GPU

mod page;

pub use page::Page;

use crate::compositor::GpuCompositor;
use crate::js_engine::{DefaultJsEngine, JavaScriptEngine};
use crate::network::NetworkStack;
use crate::renderer::{DefaultRenderingEngine, RenderingEngine};
use crate::utils::Result;

/// The main browser engine coordinating all subsystems
pub struct BrowserEngine {
    /// Rendering engine for HTML/CSS
    renderer: Box<dyn RenderingEngine>,
    /// JavaScript engine
    js_engine: Box<dyn JavaScriptEngine>,
    /// Network stack for fetching resources
    network: NetworkStack,
    /// GPU compositor
    compositor: GpuCompositor,
}

impl BrowserEngine {
    /// Create a new browser engine with default components
    pub fn new() -> Self {
        Self {
            renderer: Box::new(DefaultRenderingEngine::new()),
            js_engine: Box::new(DefaultJsEngine::new()),
            network: NetworkStack::new(),
            compositor: GpuCompositor::new(),
        }
    }

    /// Process a page from URL
    pub async fn process_page(&mut self, url: &str) -> Result<Page> {
        // 1. Fetch the HTML content
        let response = self.network.fetch(url).await?;

        // 2. Parse HTML into DOM
        let dom = self.renderer.parse_html(response.body())?;

        // 3. Compute layout
        let layout = self.renderer.compute_layout(&dom)?;

        // 4. Composite to frame
        let frame = self.compositor.composite(layout).await?;

        Ok(Page::new(url.to_string(), dom, frame))
    }

    /// Get a reference to the network stack
    pub fn network(&self) -> &NetworkStack {
        &self.network
    }

    /// Get a mutable reference to the JS engine
    pub fn js_engine_mut(&mut self) -> &mut Box<dyn JavaScriptEngine> {
        &mut self.js_engine
    }
}

impl Default for BrowserEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let _engine = BrowserEngine::new();
        // Verify engine compiles and initializes
    }

    #[tokio::test]
    async fn test_engine_fetch() {
        let engine = BrowserEngine::new();
        let result = engine.network().fetch("https://example.com").await;
        assert!(result.is_ok());
    }
}
