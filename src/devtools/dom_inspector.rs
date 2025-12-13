//! DOM Inspector implementation

use std::collections::HashMap;

/// Simplified DOM node for inspection
#[derive(Debug, Clone)]
pub struct DomNode {
    pub id: u64,
    pub node_type: DomNodeType,
    pub tag_name: Option<String>,
    pub attributes: HashMap<String, String>,
    pub text_content: Option<String>,
    pub children: Vec<u64>,
    pub parent: Option<u64>,
    pub computed_styles: HashMap<String, String>,
    pub bounding_box: Option<BoundingBox>,
}

/// DOM node types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomNodeType {
    Element,
    Text,
    Comment,
    Document,
    DocumentType,
}

/// Bounding box for element
#[derive(Debug, Clone, Copy, Default)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl DomNode {
    /// Create a new element node
    pub fn element(id: u64, tag_name: &str) -> Self {
        Self {
            id,
            node_type: DomNodeType::Element,
            tag_name: Some(tag_name.to_string()),
            attributes: HashMap::new(),
            text_content: None,
            children: Vec::new(),
            parent: None,
            computed_styles: HashMap::new(),
            bounding_box: None,
        }
    }

    /// Create a new text node
    pub fn text(id: u64, content: &str) -> Self {
        Self {
            id,
            node_type: DomNodeType::Text,
            tag_name: None,
            attributes: HashMap::new(),
            text_content: Some(content.to_string()),
            children: Vec::new(),
            parent: None,
            computed_styles: HashMap::new(),
            bounding_box: None,
        }
    }

    /// Set an attribute
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }

    /// Get an attribute
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }
}

/// DOM Inspector for viewing and modifying DOM tree
pub struct DomInspector {
    nodes: HashMap<u64, DomNode>,
    root_id: Option<u64>,
    selected_node: Option<u64>,
    highlighted_node: Option<u64>,
}

impl DomInspector {
    /// Create a new DOM inspector
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_id: None,
            selected_node: None,
            highlighted_node: None,
        }
    }

    /// Set the DOM tree from nodes
    pub fn set_tree(&mut self, nodes: Vec<DomNode>, root_id: u64) {
        self.nodes.clear();
        for node in nodes {
            self.nodes.insert(node.id, node);
        }
        self.root_id = Some(root_id);
    }

    /// Add a node
    pub fn add_node(&mut self, node: DomNode) {
        if self.root_id.is_none() {
            self.root_id = Some(node.id);
        }
        self.nodes.insert(node.id, node);
    }

    /// Get a node by ID
    pub fn get_node(&self, id: u64) -> Option<&DomNode> {
        self.nodes.get(&id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: u64) -> Option<&mut DomNode> {
        self.nodes.get_mut(&id)
    }

    /// Select a node
    pub fn select_node(&mut self, id: u64) {
        if self.nodes.contains_key(&id) {
            self.selected_node = Some(id);
        }
    }

    /// Get selected node
    pub fn selected(&self) -> Option<&DomNode> {
        self.selected_node.and_then(|id| self.nodes.get(&id))
    }

    /// Highlight a node (for hover)
    pub fn highlight_node(&mut self, id: Option<u64>) {
        self.highlighted_node = id;
    }

    /// Get highlighted node
    pub fn highlighted(&self) -> Option<u64> {
        self.highlighted_node
    }

    /// Get root node
    pub fn root(&self) -> Option<&DomNode> {
        self.root_id.and_then(|id| self.nodes.get(&id))
    }

    /// Get children of a node
    pub fn children(&self, id: u64) -> Vec<&DomNode> {
        self.nodes
            .get(&id)
            .map(|n| {
                n.children
                    .iter()
                    .filter_map(|child_id| self.nodes.get(child_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for DomInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_node_element() {
        let node = DomNode::element(1, "div");
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert_eq!(node.node_type, DomNodeType::Element);
    }

    #[test]
    fn test_dom_node_text() {
        let node = DomNode::text(1, "Hello");
        assert_eq!(node.text_content, Some("Hello".to_string()));
        assert_eq!(node.node_type, DomNodeType::Text);
    }

    #[test]
    fn test_dom_inspector_add_node() {
        let mut inspector = DomInspector::new();
        inspector.add_node(DomNode::element(1, "html"));
        assert_eq!(inspector.node_count(), 1);
    }

    #[test]
    fn test_dom_inspector_select() {
        let mut inspector = DomInspector::new();
        inspector.add_node(DomNode::element(1, "div"));
        inspector.select_node(1);
        assert!(inspector.selected().is_some());
    }

    #[test]
    fn test_dom_node_attributes() {
        let mut node = DomNode::element(1, "div");
        node.set_attribute("class", "container");
        assert_eq!(node.get_attribute("class"), Some(&"container".to_string()));
    }
}

