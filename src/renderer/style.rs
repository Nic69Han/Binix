//! Style computation and cascade

use super::css::{Selector, Stylesheet, Value};
use super::dom::{Document, Node};
use std::collections::HashMap;

/// Computed styles for an element
#[derive(Debug, Clone, Default)]
pub struct ComputedStyle {
    properties: HashMap<String, Value>,
}

impl ComputedStyle {
    /// Get a property value
    pub fn get(&self, property: &str) -> Option<&Value> {
        self.properties.get(property)
    }

    /// Set a property value
    pub fn set(&mut self, property: impl Into<String>, value: Value) {
        self.properties.insert(property.into(), value);
    }
}

/// Style tree node (DOM node + computed style)
#[derive(Debug, Clone)]
pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub style: ComputedStyle,
    pub children: Vec<StyledNode<'a>>,
}

/// Style engine for computing styles
pub struct StyleEngine {
    user_agent_stylesheet: Stylesheet,
}

impl StyleEngine {
    /// Create a new style engine
    pub fn new() -> Self {
        Self {
            user_agent_stylesheet: Stylesheet::default(),
        }
    }

    /// Compute styles for a document
    pub fn compute_styles<'a>(
        &self,
        document: &'a Document,
        stylesheets: &[Stylesheet],
    ) -> StyledNode<'a> {
        self.style_node(&document.root, stylesheets)
    }

    /// Compute styles for a single node
    fn style_node<'a>(&self, node: &'a Node, stylesheets: &[Stylesheet]) -> StyledNode<'a> {
        let style = self.compute_node_style(node, stylesheets);
        let children = node
            .children
            .iter()
            .map(|child| self.style_node(child, stylesheets))
            .collect();

        StyledNode {
            node,
            style,
            children,
        }
    }

    /// Compute style for a single node
    fn compute_node_style(&self, node: &Node, stylesheets: &[Stylesheet]) -> ComputedStyle {
        // TODO: Implement proper cascade algorithm
        ComputedStyle::default()
    }

    /// Check if a selector matches a node
    fn selector_matches(&self, selector: &Selector, node: &Node) -> bool {
        if let Some(element) = node.as_element() {
            // Check tag name
            if let Some(ref tag) = selector.tag_name {
                if tag != &element.tag_name {
                    return false;
                }
            }

            // Check ID
            if let Some(ref id) = selector.id {
                if element.id() != Some(id) {
                    return false;
                }
            }

            // Check classes
            let element_classes: Vec<&str> = element.classes();
            for class in &selector.classes {
                if !element_classes.contains(&class.as_str()) {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }
}

impl Default for StyleEngine {
    fn default() -> Self {
        Self::new()
    }
}
