//! Layer tree for compositing

use crate::renderer::LayoutBox;

/// A compositing layer
#[derive(Debug, Clone)]
pub struct Layer {
    /// Layer bounds
    pub bounds: LayerBounds,
    /// Whether this layer needs repainting
    pub dirty: bool,
    /// Layer content type
    pub content_type: LayerContentType,
    /// Child layers
    pub children: Vec<Layer>,
}

/// Layer bounds
#[derive(Debug, Clone, Copy, Default)]
pub struct LayerBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Type of content in a layer
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayerContentType {
    /// Normal painted content
    Normal,
    /// Scrollable content
    Scrollable,
    /// Fixed position content
    Fixed,
    /// Video content (hardware decoded)
    Video,
    /// WebGL/Canvas content
    Canvas,
}

impl Layer {
    /// Create a new layer
    pub fn new(bounds: LayerBounds) -> Self {
        Self {
            bounds,
            dirty: true,
            content_type: LayerContentType::Normal,
            children: Vec::new(),
        }
    }

    /// Mark this layer as dirty
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark this layer as clean
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Add a child layer
    pub fn add_child(&mut self, child: Layer) {
        self.children.push(child);
    }
}

/// Tree of compositing layers
#[derive(Debug, Clone)]
pub struct LayerTree {
    pub root: Option<Layer>,
}

impl LayerTree {
    /// Create a new empty layer tree
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Build layer tree from layout
    pub fn build_from_layout(&mut self, layout: &LayoutBox) {
        let bounds = LayerBounds {
            x: layout.dimensions.content.x,
            y: layout.dimensions.content.y,
            width: layout.dimensions.content.width,
            height: layout.dimensions.content.height,
        };

        let mut root = Layer::new(bounds);

        // Build child layers recursively
        for child_layout in &layout.children {
            self.build_layer_recursive(child_layout, &mut root);
        }

        self.root = Some(root);
    }

    /// Recursively build layers from layout boxes
    fn build_layer_recursive(&self, layout: &LayoutBox, parent: &mut Layer) {
        let bounds = LayerBounds {
            x: layout.dimensions.content.x,
            y: layout.dimensions.content.y,
            width: layout.dimensions.content.width,
            height: layout.dimensions.content.height,
        };

        let mut layer = Layer::new(bounds);

        for child_layout in &layout.children {
            self.build_layer_recursive(child_layout, &mut layer);
        }

        parent.add_child(layer);
    }

    /// Mark all layers as dirty
    pub fn mark_all_dirty(&mut self) {
        if let Some(ref mut root) = self.root {
            Self::mark_dirty_recursive(root);
        }
    }

    fn mark_dirty_recursive(layer: &mut Layer) {
        layer.mark_dirty();
        for child in &mut layer.children {
            Self::mark_dirty_recursive(child);
        }
    }
}

impl Default for LayerTree {
    fn default() -> Self {
        Self::new()
    }
}

