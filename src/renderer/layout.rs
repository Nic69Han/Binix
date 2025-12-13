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

/// Flex direction
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Flex wrap
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Justify content (main axis alignment)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Align items (cross axis alignment)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AlignItems {
    #[default]
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

/// Flexbox properties
#[derive(Debug, Clone, Copy, Default)]
pub struct FlexProperties {
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    /// Flex grow factor
    pub flex_grow: f32,
    /// Flex shrink factor
    pub flex_shrink: f32,
    /// Flex basis (initial size)
    pub flex_basis: Option<f32>,
    /// Gap between items
    pub gap: f32,
}

/// Grid properties
#[derive(Debug, Clone, Default)]
pub struct GridProperties {
    /// Column template (sizes for each column)
    pub template_columns: Vec<GridTrackSize>,
    /// Row template (sizes for each row)
    pub template_rows: Vec<GridTrackSize>,
    /// Column gap
    pub column_gap: f32,
    /// Row gap
    pub row_gap: f32,
    /// Grid item column position
    pub column: Option<GridPosition>,
    /// Grid item row position
    pub row: Option<GridPosition>,
}

/// Grid track size (column or row)
#[derive(Debug, Clone, Copy)]
pub enum GridTrackSize {
    /// Fixed size in pixels
    Px(f32),
    /// Fraction of available space
    Fr(f32),
    /// Auto size based on content
    Auto,
    /// Min-content
    MinContent,
    /// Max-content
    MaxContent,
}

impl Default for GridTrackSize {
    fn default() -> Self {
        GridTrackSize::Auto
    }
}

/// Grid item position
#[derive(Debug, Clone, Copy)]
pub struct GridPosition {
    /// Start line (1-based)
    pub start: i32,
    /// End line (1-based), or span count if negative
    pub end: i32,
}

impl GridPosition {
    pub fn line(n: i32) -> Self {
        Self { start: n, end: n + 1 }
    }

    pub fn span(start: i32, span: i32) -> Self {
        Self { start, end: start + span }
    }
}

/// Layout box in the render tree
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// Box dimensions
    pub dimensions: Dimensions,
    /// Display type
    pub display: DisplayType,
    /// Flexbox properties (if display is Flex)
    pub flex: FlexProperties,
    /// Grid properties (if display is Grid)
    pub grid: GridProperties,
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
            flex: FlexProperties::default(),
            grid: GridProperties::default(),
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
            flex: FlexProperties::default(),
            grid: GridProperties::default(),
            children: Vec::new(),
            tag_name: tag_name.to_string(),
            text_content: None,
        }
    }

    /// Create a flex container
    pub fn flex_container(direction: FlexDirection) -> Self {
        Self {
            dimensions: Dimensions::default(),
            display: DisplayType::Flex,
            flex: FlexProperties {
                direction,
                ..Default::default()
            },
            grid: GridProperties::default(),
            children: Vec::new(),
            tag_name: String::new(),
            text_content: None,
        }
    }

    /// Create a grid container
    pub fn grid_container(columns: Vec<GridTrackSize>, rows: Vec<GridTrackSize>) -> Self {
        Self {
            dimensions: Dimensions::default(),
            display: DisplayType::Grid,
            flex: FlexProperties::default(),
            grid: GridProperties {
                template_columns: columns,
                template_rows: rows,
                ..Default::default()
            },
            children: Vec::new(),
            tag_name: String::new(),
            text_content: None,
        }
    }

    /// Create a layout box for text
    pub fn text(content: &str) -> Self {
        Self {
            dimensions: Dimensions::default(),
            display: DisplayType::Inline,
            flex: FlexProperties::default(),
            grid: GridProperties::default(),
            children: Vec::new(),
            tag_name: "#text".to_string(),
            text_content: Some(content.to_string()),
        }
    }

    /// Add a child box
    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    /// Set flex properties
    pub fn with_flex(mut self, flex: FlexProperties) -> Self {
        self.flex = flex;
        self
    }

    /// Set grid properties
    pub fn with_grid(mut self, grid: GridProperties) -> Self {
        self.grid = grid;
        self
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
                DisplayType::Grid => {
                    self.layout_grid(child, &child_containing);
                    child_containing.content.height += child.dimensions.margin_box().height;
                }
                DisplayType::None => {}
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

    /// Layout flex container with full flexbox algorithm
    fn layout_flex(&self, layout_box: &mut LayoutBox, containing_block: &Dimensions) {
        self.calculate_block_width(layout_box, containing_block);
        self.calculate_block_position(layout_box, containing_block);

        let child_count = layout_box.children.len();
        if child_count == 0 {
            return;
        }

        let flex = layout_box.flex;
        let is_row = matches!(flex.direction, FlexDirection::Row | FlexDirection::RowReverse);
        let is_reverse = matches!(flex.direction, FlexDirection::RowReverse | FlexDirection::ColumnReverse);

        // Main axis size
        let main_size = if is_row {
            layout_box.dimensions.content.width
        } else {
            layout_box.dimensions.content.height
        };

        // Calculate total flex grow/shrink and initial sizes
        let mut total_flex_grow: f32 = 0.0;
        let mut total_flex_shrink: f32 = 0.0;
        let mut total_basis: f32 = 0.0;
        let gap_total = flex.gap * (child_count as f32 - 1.0).max(0.0);

        for child in &layout_box.children {
            total_flex_grow += child.flex.flex_grow;
            total_flex_shrink += child.flex.flex_shrink;
            let basis = child.flex.flex_basis.unwrap_or(0.0);
            total_basis += basis;
        }

        // Calculate free space
        let free_space = main_size - total_basis - gap_total;

        // Calculate item sizes
        let mut item_sizes: Vec<f32> = Vec::with_capacity(child_count);
        for child in &layout_box.children {
            let basis = child.flex.flex_basis.unwrap_or(0.0);
            let size = if free_space > 0.0 && total_flex_grow > 0.0 {
                basis + (child.flex.flex_grow / total_flex_grow) * free_space
            } else if free_space < 0.0 && total_flex_shrink > 0.0 {
                basis + (child.flex.flex_shrink / total_flex_shrink) * free_space
            } else if total_flex_grow == 0.0 && total_basis == 0.0 {
                // Equal distribution when no flex basis set
                (main_size - gap_total) / child_count as f32
            } else {
                basis
            };
            item_sizes.push(size.max(0.0));
        }

        // Apply justify-content
        let total_item_size: f32 = item_sizes.iter().sum();
        let remaining_space = main_size - total_item_size - gap_total;

        let (mut main_offset, spacing) = match flex.justify_content {
            JustifyContent::FlexStart => (0.0, flex.gap),
            JustifyContent::FlexEnd => (remaining_space, flex.gap),
            JustifyContent::Center => (remaining_space / 2.0, flex.gap),
            JustifyContent::SpaceBetween => {
                if child_count > 1 {
                    (0.0, remaining_space / (child_count as f32 - 1.0) + flex.gap)
                } else {
                    (0.0, flex.gap)
                }
            }
            JustifyContent::SpaceAround => {
                let item_spacing = remaining_space / child_count as f32;
                (item_spacing / 2.0, item_spacing + flex.gap)
            }
            JustifyContent::SpaceEvenly => {
                let item_spacing = remaining_space / (child_count as f32 + 1.0);
                (item_spacing, item_spacing + flex.gap)
            }
        };

        // Cross axis size
        let cross_size = if is_row {
            layout_box.dimensions.content.height
        } else {
            layout_box.dimensions.content.width
        };

        // Layout each child
        let indices: Vec<usize> = if is_reverse {
            (0..child_count).rev().collect()
        } else {
            (0..child_count).collect()
        };

        for (pos, &i) in indices.iter().enumerate() {
            let child = &mut layout_box.children[i];
            let item_main_size = item_sizes[i];

            if is_row {
                child.dimensions.content.width = item_main_size;
                child.dimensions.content.x = layout_box.dimensions.content.x + main_offset;
                child.dimensions.content.y = layout_box.dimensions.content.y;

                // Apply align-items
                match flex.align_items {
                    AlignItems::Stretch => {
                        child.dimensions.content.height = cross_size;
                    }
                    AlignItems::FlexStart => {}
                    AlignItems::FlexEnd => {
                        child.dimensions.content.y += cross_size - child.dimensions.content.height;
                    }
                    AlignItems::Center => {
                        child.dimensions.content.y += (cross_size - child.dimensions.content.height) / 2.0;
                    }
                    AlignItems::Baseline => {} // Simplified
                }
            } else {
                child.dimensions.content.height = item_main_size;
                child.dimensions.content.y = layout_box.dimensions.content.y + main_offset;
                child.dimensions.content.x = layout_box.dimensions.content.x;

                // Apply align-items for column
                match flex.align_items {
                    AlignItems::Stretch => {
                        child.dimensions.content.width = cross_size;
                    }
                    AlignItems::FlexStart => {}
                    AlignItems::FlexEnd => {
                        child.dimensions.content.x += cross_size - child.dimensions.content.width;
                    }
                    AlignItems::Center => {
                        child.dimensions.content.x += (cross_size - child.dimensions.content.width) / 2.0;
                    }
                    AlignItems::Baseline => {}
                }
            }

            self.layout_children(child);
            self.calculate_block_height(child);

            main_offset += item_main_size;
            if pos < child_count - 1 {
                main_offset += spacing;
            }
        }

        // Calculate container height from children
        if is_row {
            let max_height = layout_box
                .children
                .iter()
                .map(|c| c.dimensions.margin_box().height)
                .fold(0.0f32, |a, b| a.max(b));
            layout_box.dimensions.content.height = max_height;
        } else {
            layout_box.dimensions.content.height = main_offset;
        }
    }

    /// Layout grid container
    fn layout_grid(&self, layout_box: &mut LayoutBox, containing_block: &Dimensions) {
        self.calculate_block_width(layout_box, containing_block);
        self.calculate_block_position(layout_box, containing_block);

        let child_count = layout_box.children.len();
        if child_count == 0 {
            return;
        }

        let grid = &layout_box.grid;
        let available_width = layout_box.dimensions.content.width;
        let available_height = layout_box.dimensions.content.height;

        // Calculate column widths
        let column_widths = self.resolve_grid_tracks(&grid.template_columns, available_width, grid.column_gap);
        let num_cols = column_widths.len().max(1);

        // Calculate row heights (auto for now)
        let num_rows = (child_count + num_cols - 1) / num_cols;
        let row_height = if !grid.template_rows.is_empty() {
            self.resolve_grid_tracks(&grid.template_rows, available_height, grid.row_gap)
        } else {
            vec![30.0; num_rows] // Default row height
        };

        // Position each child in the grid
        let container_x = layout_box.dimensions.content.x;
        let container_y = layout_box.dimensions.content.y;

        for (i, child) in layout_box.children.iter_mut().enumerate() {
            // Calculate grid position (auto-placement if not specified)
            let (col, row) = if let Some(ref pos) = child.grid.column {
                let c = (pos.start - 1).max(0) as usize;
                let r = child.grid.row.map(|p| (p.start - 1).max(0) as usize).unwrap_or(i / num_cols);
                (c, r)
            } else {
                (i % num_cols, i / num_cols)
            };

            // Calculate position
            let x_offset: f32 = column_widths.iter().take(col).sum::<f32>() + col as f32 * grid.column_gap;
            let y_offset: f32 = row_height.iter().take(row).sum::<f32>() + row as f32 * grid.row_gap;

            child.dimensions.content.x = container_x + x_offset;
            child.dimensions.content.y = container_y + y_offset;
            child.dimensions.content.width = column_widths.get(col).copied().unwrap_or(available_width / num_cols as f32);
            child.dimensions.content.height = row_height.get(row).copied().unwrap_or(30.0);

            self.layout_children(child);
        }

        // Calculate container height
        let total_height: f32 = row_height.iter().sum::<f32>() + (num_rows.saturating_sub(1)) as f32 * grid.row_gap;
        layout_box.dimensions.content.height = total_height;
    }

    /// Resolve grid track sizes to actual pixel values
    fn resolve_grid_tracks(&self, tracks: &[GridTrackSize], available: f32, gap: f32) -> Vec<f32> {
        if tracks.is_empty() {
            return vec![available];
        }

        let total_gap = gap * (tracks.len() as f32 - 1.0).max(0.0);
        let remaining = available - total_gap;

        // First pass: calculate fixed sizes and count fr units
        let mut sizes = Vec::with_capacity(tracks.len());
        let mut total_fr = 0.0;
        let mut used_space = 0.0;

        for track in tracks {
            match track {
                GridTrackSize::Px(px) => {
                    sizes.push(*px);
                    used_space += *px;
                }
                GridTrackSize::Fr(fr) => {
                    sizes.push(0.0); // Placeholder
                    total_fr += *fr;
                }
                GridTrackSize::Auto => {
                    sizes.push(0.0); // Placeholder
                    total_fr += 1.0; // Treat auto as 1fr for simplicity
                }
                GridTrackSize::MinContent | GridTrackSize::MaxContent => {
                    sizes.push(50.0); // Default for content-based sizing
                    used_space += 50.0;
                }
            }
        }

        // Second pass: distribute remaining space to fr units
        let fr_space = (remaining - used_space).max(0.0);
        let fr_unit = if total_fr > 0.0 { fr_space / total_fr } else { 0.0 };

        for (i, track) in tracks.iter().enumerate() {
            match track {
                GridTrackSize::Fr(fr) => {
                    sizes[i] = fr * fr_unit;
                }
                GridTrackSize::Auto => {
                    sizes[i] = fr_unit;
                }
                _ => {}
            }
        }

        sizes
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

    #[test]
    fn test_flex_properties_default() {
        let flex = FlexProperties::default();
        assert_eq!(flex.direction, FlexDirection::Row);
        assert_eq!(flex.wrap, FlexWrap::NoWrap);
        assert_eq!(flex.justify_content, JustifyContent::FlexStart);
        assert_eq!(flex.align_items, AlignItems::Stretch);
        assert_eq!(flex.flex_grow, 0.0);
        assert_eq!(flex.flex_shrink, 0.0);
        assert_eq!(flex.gap, 0.0);
    }

    #[test]
    fn test_flex_container_creation() {
        let container = LayoutBox::flex_container(FlexDirection::Row);
        assert_eq!(container.display, DisplayType::Flex);
        assert_eq!(container.flex.direction, FlexDirection::Row);
    }

    #[test]
    fn test_flex_layout_row() {
        let engine = LayoutEngine::new();

        let mut container = LayoutBox::flex_container(FlexDirection::Row);
        container.dimensions.content.x = 0.0;
        container.dimensions.content.y = 0.0;

        // Add 3 children with flex-grow
        for _ in 0..3 {
            let mut child = LayoutBox::element("div", DisplayType::Block);
            child.flex.flex_grow = 1.0;
            container.add_child(child);
        }

        let containing = Dimensions {
            content: Rect {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 300.0,
            },
            ..Default::default()
        };

        engine.layout_flex(&mut container, &containing);

        // Container width is set from containing block (300px)
        // Each child should get 100px width (300 / 3) due to equal flex-grow
        assert_eq!(container.children.len(), 3);
        let expected_width = 100.0;
        for child in &container.children {
            assert!((child.dimensions.content.width - expected_width).abs() < 0.1,
                "Expected width ~{}, got {}", expected_width, child.dimensions.content.width);
        }
    }

    #[test]
    fn test_flex_justify_center() {
        let mut engine = LayoutEngine::new();
        engine.set_viewport(400.0, 300.0);

        let mut container = LayoutBox::flex_container(FlexDirection::Row);
        container.flex.justify_content = JustifyContent::Center;
        container.dimensions.content.width = 300.0;
        container.dimensions.content.height = 100.0;

        // Add 2 children with fixed basis
        for _ in 0..2 {
            let mut child = LayoutBox::element("div", DisplayType::Block);
            child.flex.flex_basis = Some(50.0);
            container.add_child(child);
        }

        let containing = Dimensions {
            content: Rect {
                x: 0.0,
                y: 0.0,
                width: 400.0,
                height: 300.0,
            },
            ..Default::default()
        };

        engine.layout_flex(&mut container, &containing);

        // Children should be centered: (300 - 100) / 2 = 100 offset
        assert!(container.children[0].dimensions.content.x >= 99.0);
    }
}

/// Convert a LayoutBox tree to RenderElements for UI rendering
pub fn layout_to_render_elements(layout: &LayoutBox) -> Vec<crate::ui::RenderElement> {
    let mut elements = Vec::new();
    collect_render_elements(layout, &mut elements);
    elements
}

fn collect_render_elements(layout: &LayoutBox, elements: &mut Vec<crate::ui::RenderElement>) {
    use crate::ui::{ElementBounds, ElementKind, ElementStyle, RenderElement};

    // Determine element kind from tag name
    let kind = match layout.tag_name.as_str() {
        "h1" => ElementKind::Heading1,
        "h2" => ElementKind::Heading2,
        "h3" | "h4" | "h5" | "h6" => ElementKind::Heading3,
        "p" => ElementKind::Paragraph,
        "a" => ElementKind::Link,
        "li" => ElementKind::ListItem,
        "code" | "pre" => ElementKind::Code,
        "img" => ElementKind::Image,
        "blockquote" => ElementKind::Blockquote,
        "table" => ElementKind::Table,
        "tr" => ElementKind::TableRow,
        "td" | "th" => ElementKind::TableCell,
        "hr" => ElementKind::HorizontalRule,
        "#text" => ElementKind::Text,
        _ => ElementKind::Text,
    };

    // Only create element if there's text content
    if let Some(text) = &layout.text_content {
        if !text.trim().is_empty() {
            let dims = &layout.dimensions;

            // Create style from layout dimensions
            let mut style = ElementStyle::default();
            style.padding = [
                dims.padding.top,
                dims.padding.right,
                dims.padding.bottom,
                dims.padding.left,
            ];
            style.margin = [
                dims.margin.top,
                dims.margin.right,
                dims.margin.bottom,
                dims.margin.left,
            ];

            // Apply default styles based on kind
            match kind {
                ElementKind::Heading1 => {
                    style.font_size = 32.0;
                    style.font_weight_bold = true;
                }
                ElementKind::Heading2 => {
                    style.font_size = 24.0;
                    style.font_weight_bold = true;
                }
                ElementKind::Heading3 => {
                    style.font_size = 18.0;
                    style.font_weight_bold = true;
                }
                ElementKind::Link => {
                    style.color = [0, 102, 204, 255];
                    style.text_decoration_underline = true;
                }
                ElementKind::Code => {
                    style.font_size = 14.0;
                    style.background_color = Some([245, 245, 245, 255]);
                }
                _ => {}
            }

            let is_link = kind == ElementKind::Link;

            let element = RenderElement {
                kind,
                text: text.clone(),
                bounds: ElementBounds {
                    x: dims.content.x,
                    y: dims.content.y,
                    width: dims.content.width,
                    height: dims.content.height,
                },
                style,
                is_link,
                href: None, // Would need to be extracted from attributes
                src: None,
                alt: None,
                children: Vec::new(),
            };

            elements.push(element);
        }
    }

    // Recurse into children
    for child in &layout.children {
        collect_render_elements(child, elements);
    }
}
