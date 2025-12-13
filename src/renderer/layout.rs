//! Layout engine for computing element positions and sizes

use super::dom::{Document, Node, NodeType};
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

    /// Get the padding box rectangle
    pub fn padding_box(&self) -> Rect {
        Rect {
            x: self.content.x - self.padding.left,
            y: self.content.y - self.padding.top,
            width: self.content.width + self.padding.left + self.padding.right,
            height: self.content.height + self.padding.top + self.padding.bottom,
        }
    }

    /// Get the border box rectangle
    pub fn border_box(&self) -> Rect {
        let padding = self.padding_box();
        Rect {
            x: padding.x - self.border.left,
            y: padding.y - self.border.top,
            width: padding.width + self.border.left + self.border.right,
            height: padding.height + self.border.top + self.border.bottom,
        }
    }

    /// Get the margin box rectangle
    pub fn margin_box(&self) -> Rect {
        let border = self.border_box();
        Rect {
            x: border.x - self.margin.left,
            y: border.y - self.margin.top,
            width: border.width + self.margin.left + self.margin.right,
            height: border.height + self.margin.top + self.margin.bottom,
        }
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
    /// Box dimensions
    pub dimensions: Dimensions,
    /// Display type
    pub display: DisplayType,
    /// Child layout boxes
    pub children: Vec<LayoutBox>,
    /// Tag name (for debugging)
    pub tag_name: String,
    /// Text content (for text nodes)
    pub text_content: Option<String>,
}

impl LayoutBox {
    /// Create a new layout box
    pub fn new(display: DisplayType) -> Self {
        Self {
            dimensions: Dimensions::default(),
            display,
            children: Vec::new(),
            tag_name: String::new(),
            text_content: None,
        }
    }

    /// Create a layout box for an element
    pub fn element(tag_name: &str, display: DisplayType) -> Self {
        Self {
            dimensions: Dimensions::default(),
            display,
            children: Vec::new(),
            tag_name: tag_name.to_string(),
            text_content: None,
        }
    }

    /// Create a layout box for text
    pub fn text(content: &str) -> Self {
        Self {
            dimensions: Dimensions::default(),
            display: DisplayType::Inline,
            children: Vec::new(),
            tag_name: "#text".to_string(),
            text_content: Some(content.to_string()),
        }
    }

    /// Add a child box
    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }
}

/// Default font metrics for text layout
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    /// Font size in pixels
    pub size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Average character width (approximation)
    pub char_width: f32,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            size: 16.0,
            line_height: 1.2,
            char_width: 8.0, // Approximate for monospace
        }
    }
}

/// Layout engine for computing the layout tree
pub struct LayoutEngine {
    viewport_width: f32,
    viewport_height: f32,
    font_metrics: FontMetrics,
}

impl LayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        Self {
            viewport_width: 1920.0,
            viewport_height: 1080.0,
            font_metrics: FontMetrics::default(),
        }
    }

    /// Set viewport dimensions
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Set font metrics
    pub fn set_font_metrics(&mut self, metrics: FontMetrics) {
        self.font_metrics = metrics;
    }

    /// Compute layout for a document
    pub fn compute(&self, document: &Document) -> Result<LayoutBox> {
        let containing_block = Dimensions {
            content: Rect {
                x: 0.0,
                y: 0.0,
                width: self.viewport_width,
                height: self.viewport_height,
            },
            ..Default::default()
        };

        let mut root = self.build_layout_tree(&document.root);
        self.layout_block(&mut root, &containing_block);
        Ok(root)
    }

    /// Build layout tree from DOM
    fn build_layout_tree(&self, node: &Node) -> LayoutBox {
        match &node.node_type {
            NodeType::Element(data) => {
                let display = self.get_display_type(&data.tag_name);
                let mut layout_box = LayoutBox::element(&data.tag_name, display);

                for child in &node.children {
                    let child_box = self.build_layout_tree(child);
                    if child_box.display != DisplayType::None {
                        layout_box.add_child(child_box);
                    }
                }

                layout_box
            }
            NodeType::Text(content) => {
                let trimmed = content.trim();
                if trimmed.is_empty() {
                    LayoutBox::new(DisplayType::None)
                } else {
                    LayoutBox::text(trimmed)
                }
            }
            NodeType::Comment(_) => LayoutBox::new(DisplayType::None),
            NodeType::Document => {
                let mut root = LayoutBox::element("#document", DisplayType::Block);
                for child in &node.children {
                    let child_box = self.build_layout_tree(child);
                    if child_box.display != DisplayType::None {
                        root.add_child(child_box);
                    }
                }
                root
            }
        }
    }

    /// Get display type for an element
    fn get_display_type(&self, tag_name: &str) -> DisplayType {
        match tag_name.to_lowercase().as_str() {
            // Block elements
            "html" | "body" | "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ul"
            | "ol" | "li" | "table" | "tr" | "form" | "header" | "footer" | "main" | "nav"
            | "section" | "article" | "aside" | "blockquote" | "pre" | "hr" | "address" => {
                DisplayType::Block
            }
            // Inline elements
            "span" | "a" | "strong" | "em" | "b" | "i" | "u" | "code" | "small" | "sub" | "sup"
            | "label" | "abbr" | "cite" | "q" => DisplayType::Inline,
            // Inline-block elements
            "img" | "button" | "input" | "select" | "textarea" => DisplayType::InlineBlock,
            // Hidden elements
            "head" | "meta" | "title" | "link" | "style" | "script" | "noscript" => {
                DisplayType::None
            }
            // Default to block
            _ => DisplayType::Block,
        }
    }

    /// Layout a block-level element
    fn layout_block(&self, layout_box: &mut LayoutBox, containing_block: &Dimensions) {
        // Calculate width
        self.calculate_block_width(layout_box, containing_block);

        // Calculate position
        self.calculate_block_position(layout_box, containing_block);

        // Layout children
        self.layout_children(layout_box);

        // Calculate height based on children
        self.calculate_block_height(layout_box);
    }

    /// Calculate block width
    fn calculate_block_width(&self, layout_box: &mut LayoutBox, containing_block: &Dimensions) {
        // Default: fill available width
        let available_width = containing_block.content.width
            - layout_box.dimensions.margin.left
            - layout_box.dimensions.margin.right
            - layout_box.dimensions.border.left
            - layout_box.dimensions.border.right
            - layout_box.dimensions.padding.left
            - layout_box.dimensions.padding.right;

        layout_box.dimensions.content.width = available_width.max(0.0);
    }

    /// Calculate block position
    fn calculate_block_position(&self, layout_box: &mut LayoutBox, containing_block: &Dimensions) {
        layout_box.dimensions.content.x = containing_block.content.x
            + layout_box.dimensions.margin.left
            + layout_box.dimensions.border.left
            + layout_box.dimensions.padding.left;

        layout_box.dimensions.content.y = containing_block.content.y
            + containing_block.content.height
            + layout_box.dimensions.margin.top
            + layout_box.dimensions.border.top
            + layout_box.dimensions.padding.top;
    }

    /// Layout children of a block
    fn layout_children(&self, layout_box: &mut LayoutBox) {
        let mut child_containing = layout_box.dimensions;
        child_containing.content.height = 0.0;

        for child in &mut layout_box.children {
            match child.display {
                DisplayType::Block => {
                    self.layout_block(child, &child_containing);
                    child_containing.content.height += child.dimensions.margin_box().height;
                }
                DisplayType::Inline | DisplayType::InlineBlock => {
                    self.layout_inline(child, &child_containing);
                    child_containing.content.height += child.dimensions.margin_box().height;
                }
                DisplayType::Flex => {
                    self.layout_flex(child, &child_containing);
                    child_containing.content.height += child.dimensions.margin_box().height;
                }
                DisplayType::Grid | DisplayType::None => {}
            }
        }
    }

    /// Calculate block height
    fn calculate_block_height(&self, layout_box: &mut LayoutBox) {
        // Height is sum of children heights
        let children_height: f32 = layout_box
            .children
            .iter()
            .map(|c| c.dimensions.margin_box().height)
            .sum();

        layout_box.dimensions.content.height = children_height;

        // Text nodes have height based on content
        if let Some(ref text) = layout_box.text_content {
            let lines = self.calculate_text_lines(text, layout_box.dimensions.content.width);
            layout_box.dimensions.content.height =
                lines as f32 * self.font_metrics.size * self.font_metrics.line_height;
        }
    }

    /// Layout inline element
    fn layout_inline(&self, layout_box: &mut LayoutBox, containing_block: &Dimensions) {
        // For text, calculate width based on content
        if let Some(ref text) = layout_box.text_content {
            let text_width = text.len() as f32 * self.font_metrics.char_width;
            layout_box.dimensions.content.width = text_width.min(containing_block.content.width);
            layout_box.dimensions.content.height =
                self.font_metrics.size * self.font_metrics.line_height;
        } else {
            // Inline elements shrink to fit content
            layout_box.dimensions.content.width = 0.0;
            for child in &layout_box.children {
                layout_box.dimensions.content.width += child.dimensions.margin_box().width;
            }
        }

        layout_box.dimensions.content.x = containing_block.content.x;
        layout_box.dimensions.content.y =
            containing_block.content.y + containing_block.content.height;
    }

    /// Layout flex container
    fn layout_flex(&self, layout_box: &mut LayoutBox, containing_block: &Dimensions) {
        // Simple flex layout: distribute space evenly
        self.calculate_block_width(layout_box, containing_block);
        self.calculate_block_position(layout_box, containing_block);

        let child_count = layout_box.children.len();
        if child_count == 0 {
            return;
        }

        let child_width = layout_box.dimensions.content.width / child_count as f32;
        let mut x_offset = 0.0;

        for child in &mut layout_box.children {
            child.dimensions.content.width = child_width;
            child.dimensions.content.x = layout_box.dimensions.content.x + x_offset;
            child.dimensions.content.y = layout_box.dimensions.content.y;
            x_offset += child_width;

            self.layout_children(child);
            self.calculate_block_height(child);
        }

        // Height is max of children
        let max_height = layout_box
            .children
            .iter()
            .map(|c| c.dimensions.margin_box().height)
            .fold(0.0f32, |a, b| a.max(b));
        layout_box.dimensions.content.height = max_height;
    }

    /// Calculate number of text lines
    fn calculate_text_lines(&self, text: &str, available_width: f32) -> usize {
        if available_width <= 0.0 {
            return 1;
        }
        let chars_per_line = (available_width / self.font_metrics.char_width).floor() as usize;
        if chars_per_line == 0 {
            return 1;
        }
        (text.len() + chars_per_line - 1) / chars_per_line
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimensions_total_width() {
        let mut dims = Dimensions::default();
        dims.content.width = 100.0;
        dims.padding.left = 10.0;
        dims.padding.right = 10.0;
        dims.border.left = 1.0;
        dims.border.right = 1.0;
        dims.margin.left = 5.0;
        dims.margin.right = 5.0;

        assert_eq!(dims.total_width(), 132.0);
    }

    #[test]
    fn test_dimensions_total_height() {
        let mut dims = Dimensions::default();
        dims.content.height = 50.0;
        dims.padding.top = 5.0;
        dims.padding.bottom = 5.0;
        dims.border.top = 1.0;
        dims.border.bottom = 1.0;
        dims.margin.top = 2.0;
        dims.margin.bottom = 2.0;

        assert_eq!(dims.total_height(), 66.0);
    }

    #[test]
    fn test_layout_box_creation() {
        let layout_box = LayoutBox::element("div", DisplayType::Block);
        assert_eq!(layout_box.tag_name, "div");
        assert_eq!(layout_box.display, DisplayType::Block);
        assert!(layout_box.children.is_empty());
    }

    #[test]
    fn test_text_layout_box() {
        let text_box = LayoutBox::text("Hello World");
        assert_eq!(text_box.tag_name, "#text");
        assert_eq!(text_box.display, DisplayType::Inline);
        assert_eq!(text_box.text_content, Some("Hello World".to_string()));
    }

    #[test]
    fn test_layout_engine_viewport() {
        let mut engine = LayoutEngine::new();
        engine.set_viewport(800.0, 600.0);
        assert_eq!(engine.viewport_width, 800.0);
        assert_eq!(engine.viewport_height, 600.0);
    }

    #[test]
    fn test_compute_simple_layout() {
        let engine = LayoutEngine::new();
        let doc = Document::new();
        let result = engine.compute(&doc);
        assert!(result.is_ok());
        let layout = result.unwrap();
        assert_eq!(layout.dimensions.content.width, 1920.0);
    }

    #[test]
    fn test_display_type_detection() {
        let engine = LayoutEngine::new();
        assert_eq!(engine.get_display_type("div"), DisplayType::Block);
        assert_eq!(engine.get_display_type("span"), DisplayType::Inline);
        assert_eq!(engine.get_display_type("img"), DisplayType::InlineBlock);
        assert_eq!(engine.get_display_type("script"), DisplayType::None);
    }

    #[test]
    fn test_text_lines_calculation() {
        let engine = LayoutEngine::new();
        // With default char_width of 8.0, 100px fits 12 chars
        assert_eq!(engine.calculate_text_lines("Hello", 100.0), 1);
        assert_eq!(engine.calculate_text_lines("Hello World Test", 100.0), 2);
    }

    #[test]
    fn test_margin_box() {
        let mut dims = Dimensions::default();
        dims.content = Rect {
            x: 20.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        dims.padding = EdgeSizes {
            top: 5.0,
            right: 5.0,
            bottom: 5.0,
            left: 5.0,
        };
        dims.border = EdgeSizes {
            top: 1.0,
            right: 1.0,
            bottom: 1.0,
            left: 1.0,
        };
        dims.margin = EdgeSizes {
            top: 10.0,
            right: 10.0,
            bottom: 10.0,
            left: 10.0,
        };

        let margin_box = dims.margin_box();
        assert_eq!(margin_box.width, 132.0); // 100 + 10 + 10 + 2 + 2 + 4 + 4
        assert_eq!(margin_box.height, 82.0); // 50 + 10 + 10 + 2 + 2 + 4 + 4
    }
}
