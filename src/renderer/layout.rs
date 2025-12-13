//! Layout engine for computing element positions and sizes

use super::dom::Document;
use crate::utils::Result;

/// Box dimensions
#[derive(Debug, Clone, Copy, Default)]
pub struct Dimensions {
    /// Content area
    pub content: Rect,
    /// Padding
    pub padding: EdgeSizes,
    /// Border
    pub border: EdgeSizes,
    /// Margin
    pub margin: EdgeSizes,
}

impl Dimensions {
    /// Get the total width including padding, border, and margin
    pub fn total_width(&self) -> f32 {
        self.content.width
            + self.padding.left
            + self.padding.right
            + self.border.left
            + self.border.right
            + self.margin.left
            + self.margin.right
    }

    /// Get the total height including padding, border, and margin
    pub fn total_height(&self) -> f32 {
        self.content.height
            + self.padding.top
            + self.padding.bottom
            + self.border.top
            + self.border.bottom
            + self.margin.top
            + self.margin.bottom
    }
}

/// Rectangle for positioning
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Edge sizes for padding, border, margin
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// Display type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayType {
    Block,
    Inline,
    InlineBlock,
    Flex,
    Grid,
    None,
}

/// Layout box in the render tree
#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub dimensions: Dimensions,
    pub display: DisplayType,
    pub children: Vec<LayoutBox>,
}

impl LayoutBox {
    /// Create a new layout box
    pub fn new(display: DisplayType) -> Self {
        Self {
            dimensions: Dimensions::default(),
            display,
            children: Vec::new(),
        }
    }
}

/// Layout engine for computing the layout tree
pub struct LayoutEngine {
    viewport_width: f32,
    viewport_height: f32,
}

impl LayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        Self {
            viewport_width: 1920.0,
            viewport_height: 1080.0,
        }
    }

    /// Set viewport dimensions
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Compute layout for a document
    pub fn compute(&self, document: &Document) -> Result<LayoutBox> {
        // TODO: Implement proper layout algorithm
        // For now, return a simple block layout
        let mut root = LayoutBox::new(DisplayType::Block);
        root.dimensions.content.width = self.viewport_width;
        root.dimensions.content.height = self.viewport_height;
        Ok(root)
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

