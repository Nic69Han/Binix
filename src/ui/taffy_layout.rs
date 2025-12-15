//! Taffy-based CSS layout engine for accurate flexbox/grid support

use taffy::prelude::*;
use std::collections::HashMap;

use super::{
    AlignItems, DisplayMode, ElementStyle, FlexDirection, FlexWrap,
    JustifyContent, RenderElement,
};

/// Helper to create length values
fn len(val: f32) -> LengthPercentage {
    LengthPercentage::length(val)
}

fn len_auto(val: f32) -> LengthPercentageAuto {
    LengthPercentageAuto::length(val)
}

fn dim(val: f32) -> Dimension {
    Dimension::length(val)
}

/// Layout context using Taffy
pub struct TaffyLayoutContext {
    taffy: TaffyTree<()>,
    node_map: HashMap<usize, NodeId>,
}

impl TaffyLayoutContext {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_map: HashMap::new(),
        }
    }

    /// Convert our DisplayMode to Taffy Display
    fn to_taffy_display(mode: &DisplayMode) -> Display {
        match mode {
            DisplayMode::Flex => Display::Flex,
            DisplayMode::Grid => Display::Grid,
            DisplayMode::None => Display::None,
            _ => Display::Block,
        }
    }

    /// Convert our FlexDirection to Taffy
    fn to_taffy_flex_direction(dir: &FlexDirection) -> taffy::FlexDirection {
        match dir {
            FlexDirection::Row => taffy::FlexDirection::Row,
            FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
            FlexDirection::Column => taffy::FlexDirection::Column,
            FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
        }
    }

    /// Convert our FlexWrap to Taffy
    fn to_taffy_flex_wrap(wrap: &FlexWrap) -> taffy::FlexWrap {
        match wrap {
            FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
            FlexWrap::Wrap => taffy::FlexWrap::Wrap,
            FlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
        }
    }

    /// Convert our JustifyContent to Taffy
    fn to_taffy_justify_content(jc: &JustifyContent) -> Option<taffy::JustifyContent> {
        Some(match jc {
            JustifyContent::FlexStart => taffy::JustifyContent::FlexStart,
            JustifyContent::FlexEnd => taffy::JustifyContent::FlexEnd,
            JustifyContent::Center => taffy::JustifyContent::Center,
            JustifyContent::SpaceBetween => taffy::JustifyContent::SpaceBetween,
            JustifyContent::SpaceAround => taffy::JustifyContent::SpaceAround,
            JustifyContent::SpaceEvenly => taffy::JustifyContent::SpaceEvenly,
        })
    }

    /// Convert our AlignItems to Taffy
    fn to_taffy_align_items(ai: &AlignItems) -> Option<taffy::AlignItems> {
        Some(match ai {
            AlignItems::Stretch => taffy::AlignItems::Stretch,
            AlignItems::FlexStart => taffy::AlignItems::FlexStart,
            AlignItems::FlexEnd => taffy::AlignItems::FlexEnd,
            AlignItems::Center => taffy::AlignItems::Center,
            AlignItems::Baseline => taffy::AlignItems::Baseline,
        })
    }

    /// Convert ElementStyle to Taffy Style
    fn element_to_taffy_style(style: &ElementStyle) -> Style {
        let flex = &style.flex;

        Style {
            display: Self::to_taffy_display(&style.display),
            flex_direction: Self::to_taffy_flex_direction(&flex.direction),
            flex_wrap: Self::to_taffy_flex_wrap(&flex.wrap),
            justify_content: Self::to_taffy_justify_content(&flex.justify_content),
            align_items: Self::to_taffy_align_items(&flex.align_items),
            gap: Size {
                width: len(flex.gap),
                height: len(flex.gap),
            },
            padding: Rect {
                top: len(style.padding[0]),
                right: len(style.padding[1]),
                bottom: len(style.padding[2]),
                left: len(style.padding[3]),
            },
            margin: Rect {
                top: len_auto(style.margin[0]),
                right: len_auto(style.margin[1]),
                bottom: len_auto(style.margin[2]),
                left: len_auto(style.margin[3]),
            },
            flex_grow: flex.flex_grow,
            flex_shrink: if flex.flex_shrink > 0.0 { flex.flex_shrink } else { 1.0 },
            ..Default::default()
        }
    }

    /// Build a Taffy tree from RenderElement and compute layout
    pub fn compute_layout(&mut self, element: &RenderElement, available_width: f32, available_height: f32) -> LayoutResult {
        // Clear previous layout
        self.taffy = TaffyTree::new();
        self.node_map.clear();

        // Build tree recursively
        let root = self.build_node(element, 0);
        
        // Compute layout
        let size = Size {
            width: AvailableSpace::Definite(available_width),
            height: AvailableSpace::Definite(available_height),
        };
        
        if self.taffy.compute_layout(root, size).is_err() {
            return LayoutResult::default();
        }

        // Extract results
        self.extract_layout(root)
    }

    fn build_node(&mut self, element: &RenderElement, index: usize) -> NodeId {
        let style = Self::element_to_taffy_style(&element.style);
        
        // Build children first
        let child_nodes: Vec<NodeId> = element.children
            .iter()
            .enumerate()
            .map(|(i, child)| self.build_node(child, index * 100 + i + 1))
            .collect();

        // Create node with children
        let node = if child_nodes.is_empty() {
            // Leaf node - set minimum size based on text content
            let text_width = element.text.len() as f32 * 8.0; // Approximate
            let text_height = 20.0;

            let mut leaf_style = style;
            leaf_style.min_size = Size {
                width: dim(text_width.max(20.0)),
                height: dim(text_height),
            };
            self.taffy.new_leaf(leaf_style).unwrap()
        } else {
            self.taffy.new_with_children(style, &child_nodes).unwrap()
        };

        self.node_map.insert(index, node);
        node
    }

    fn extract_layout(&self, node: NodeId) -> LayoutResult {
        let layout = self.taffy.layout(node).unwrap();

        LayoutResult {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
            children: self.taffy.children(node)
                .unwrap_or_default()
                .iter()
                .map(|&child| self.extract_layout(child))
                .collect(),
        }
    }
}

impl Default for TaffyLayoutContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of layout computation
#[derive(Debug, Clone, Default)]
pub struct LayoutResult {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub children: Vec<LayoutResult>,
}

impl LayoutResult {
    /// Get child layout at index
    pub fn child(&self, index: usize) -> Option<&LayoutResult> {
        self.children.get(index)
    }
}

