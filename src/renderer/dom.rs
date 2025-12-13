//! DOM (Document Object Model) implementation

use std::collections::HashMap;

/// Node types in the DOM
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    /// Document root
    Document,
    /// Element node (e.g., <div>)
    Element(ElementData),
    /// Text node
    Text(String),
    /// Comment node
    Comment(String),
}

/// Data for element nodes
#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    /// Tag name (e.g., "div", "span")
    pub tag_name: String,
    /// Element attributes
    pub attributes: HashMap<String, String>,
}

impl ElementData {
    /// Create a new element
    pub fn new(tag_name: impl Into<String>) -> Self {
        Self {
            tag_name: tag_name.into(),
            attributes: HashMap::new(),
        }
    }

    /// Get an attribute value
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    /// Set an attribute value
    pub fn set_attribute(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(name.into(), value.into());
    }

    /// Get the ID attribute
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    /// Get class names
    pub fn classes(&self) -> Vec<&str> {
        self.attributes
            .get("class")
            .map(|c| c.split_whitespace().collect())
            .unwrap_or_default()
    }
}

/// A node in the DOM tree
#[derive(Debug, Clone)]
pub struct Node {
    /// Node type and data
    pub node_type: NodeType,
    /// Child nodes
    pub children: Vec<Node>,
}

impl Node {
    /// Create a new node
    pub fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            children: Vec::new(),
        }
    }

    /// Create an element node
    pub fn element(tag_name: impl Into<String>) -> Self {
        Self::new(NodeType::Element(ElementData::new(tag_name)))
    }

    /// Create a text node
    pub fn text(content: impl Into<String>) -> Self {
        Self::new(NodeType::Text(content.into()))
    }

    /// Add a child node
    pub fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }

    /// Check if this is an element node
    pub fn is_element(&self) -> bool {
        matches!(self.node_type, NodeType::Element(_))
    }

    /// Get element data if this is an element
    pub fn as_element(&self) -> Option<&ElementData> {
        match &self.node_type {
            NodeType::Element(data) => Some(data),
            _ => None,
        }
    }
}

/// The DOM document
#[derive(Debug, Clone)]
pub struct Document {
    /// Root node
    pub root: Node,
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            root: Node::new(NodeType::Document),
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
