//! Painter for rendering layers to pixels

use super::{Frame, Layer, LayerTree};
use crate::utils::Result;

/// Painter for rendering content
pub struct Painter {
    viewport_width: u32,
    viewport_height: u32,
}

impl Painter {
    /// Create a new painter
    pub fn new() -> Self {
        Self {
            viewport_width: 1920,
            viewport_height: 1080,
        }
    }

    /// Set viewport size
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Paint the layer tree to a frame
    pub fn paint(&self, layer_tree: &LayerTree) -> Result<Frame> {
        let mut frame = Frame::new(self.viewport_width, self.viewport_height);

        if let Some(ref root) = layer_tree.root {
            self.paint_layer(root, &mut frame);
        }

        Ok(frame)
    }

    /// Paint a single layer
    fn paint_layer(&self, layer: &Layer, frame: &mut Frame) {
        // TODO: Implement actual painting with GPU acceleration
        // For now, just fill with a placeholder color

        let x_start = layer.bounds.x.max(0.0) as u32;
        let y_start = layer.bounds.y.max(0.0) as u32;
        let x_end = ((layer.bounds.x + layer.bounds.width) as u32).min(frame.width);
        let y_end = ((layer.bounds.y + layer.bounds.height) as u32).min(frame.height);

        // Fill with white background
        for y in y_start..y_end {
            for x in x_start..x_end {
                frame.set_pixel(x, y, [255, 255, 255, 255]);
            }
        }

        // Paint child layers on top
        for child in &layer.children {
            self.paint_layer(child, frame);
        }
    }
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}
