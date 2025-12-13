//! GPU compositor for hardware-accelerated rendering
//!
//! Uses wgpu for cross-platform GPU acceleration (Vulkan/Metal/DX12/WebGPU).

mod gpu;
mod layer;
mod painter;

pub use gpu::{GpuContext, GpuError, GpuRenderer, RenderTarget};
pub use layer::{Layer, LayerTree};
pub use painter::Painter;

use crate::renderer::LayoutBox;
use crate::utils::Result;

/// GPU compositor for compositing layers
pub struct GpuCompositor {
    layers: LayerTree,
    painter: Painter,
    gpu_context: GpuContext,
}

impl GpuCompositor {
    /// Create a new GPU compositor
    pub fn new() -> Self {
        Self {
            layers: LayerTree::new(),
            painter: Painter::new(),
            gpu_context: GpuContext::new(),
        }
    }

    /// Initialize GPU resources (async)
    pub async fn initialize_gpu(&mut self) -> std::result::Result<(), GpuError> {
        self.gpu_context.initialize().await
    }

    /// Check if GPU is available
    pub fn is_gpu_available(&self) -> bool {
        self.gpu_context.is_initialized()
    }

    /// Composite the layout into pixels
    pub async fn composite(&mut self, layout: LayoutBox) -> Result<Frame> {
        // Build layer tree from layout
        self.layers.build_from_layout(&layout);

        // Paint layers (using GPU if available, fallback to software)
        let frame = self.painter.paint(&self.layers)?;

        Ok(frame)
    }

    /// Force a repaint of dirty regions
    pub fn repaint_dirty(&mut self) -> Result<Frame> {
        self.painter.paint(&self.layers)
    }

    /// Get GPU context reference
    pub fn gpu_context(&self) -> &GpuContext {
        &self.gpu_context
    }
}

impl Default for GpuCompositor {
    fn default() -> Self {
        Self::new()
    }
}

/// A rendered frame
#[derive(Debug, Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Frame {
    /// Create a new frame
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize; // RGBA
        Self {
            width,
            height,
            pixels: vec![0; size],
        }
    }

    /// Get pixel at (x, y)
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ]
    }

    /// Set pixel at (x, y)
    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        let idx = ((y * self.width + x) * 4) as usize;
        self.pixels[idx] = rgba[0];
        self.pixels[idx + 1] = rgba[1];
        self.pixels[idx + 2] = rgba[2];
        self.pixels[idx + 3] = rgba[3];
    }
}
