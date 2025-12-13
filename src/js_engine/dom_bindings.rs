//! DOM bindings for JavaScript
//!
//! Provides JavaScript access to the DOM tree.

use crate::renderer::{Document, Node};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Unique identifier for DOM nodes in JavaScript
pub type NodeId = u64;

/// DOM binding context that bridges JavaScript and the DOM
pub struct DomBindings {
    /// The document being manipulated
    document: Arc<Mutex<Document>>,
    /// Node ID counter
    next_node_id: u64,
    /// Map from node IDs to node paths (for retrieval)
    node_map: HashMap<NodeId, Vec<usize>>,
}

impl DomBindings {
    /// Create new DOM bindings for a document
    pub fn new(document: Document) -> Self {
        Self {
            document: Arc::new(Mutex::new(document)),
            next_node_id: 1,
            node_map: HashMap::new(),
        }
    }

    /// Get a reference to the document
    pub fn document(&self) -> Arc<Mutex<Document>> {
        Arc::clone(&self.document)
    }

    /// Get the document element (root)
    pub fn get_document_element(&mut self) -> Option<DomNode> {
        let doc = self.document.lock().ok()?;
        if !doc.root.children.is_empty() {
            let tag_name = doc.root.children[0]
                .as_element()
                .map(|e| e.tag_name.clone())
                .unwrap_or_default();
            drop(doc); // Release lock before mutable borrow
            let id = self.allocate_node_id(vec![0]);
            Some(DomNode {
                id,
                tag_name,
                node_type: DomNodeType::Element,
            })
        } else {
            None
        }
    }

    /// Query selector - find first matching element
    pub fn query_selector(&mut self, selector: &str) -> Option<DomNode> {
        // First, find matching nodes without modifying self
        let matches = {
            let doc = self.document.lock().ok()?;
            Self::find_matching_nodes(&doc.root, selector, vec![], true)
        };

        // Then allocate IDs
        if let Some((path, tag_name)) = matches.into_iter().next() {
            let id = self.allocate_node_id(path);
            Some(DomNode {
                id,
                tag_name,
                node_type: DomNodeType::Element,
            })
        } else {
            None
        }
    }

    /// Query selector all - find all matching elements
    pub fn query_selector_all(&mut self, selector: &str) -> Vec<DomNode> {
        // First, find matching nodes without modifying self
        let matches = {
            let doc = match self.document.lock() {
                Ok(d) => d,
                Err(_) => return vec![],
            };
            Self::find_matching_nodes(&doc.root, selector, vec![], false)
        };

        // Then allocate IDs
        matches
            .into_iter()
            .map(|(path, tag_name)| {
                let id = self.allocate_node_id(path);
                DomNode {
                    id,
                    tag_name,
                    node_type: DomNodeType::Element,
                }
            })
            .collect()
    }

    /// Get element by ID
    pub fn get_element_by_id(&mut self, id: &str) -> Option<DomNode> {
        self.query_selector(&format!("#{}", id))
    }

    /// Get elements by class name
    pub fn get_elements_by_class_name(&mut self, class_name: &str) -> Vec<DomNode> {
        self.query_selector_all(&format!(".{}", class_name))
    }

    /// Get elements by tag name
    pub fn get_elements_by_tag_name(&mut self, tag_name: &str) -> Vec<DomNode> {
        self.query_selector_all(tag_name)
    }

    /// Create a new element
    pub fn create_element(&mut self, tag_name: &str) -> DomNode {
        let id = self.next_node_id;
        self.next_node_id += 1;
        DomNode {
            id,
            tag_name: tag_name.to_string(),
            node_type: DomNodeType::Element,
        }
    }

    /// Create a text node
    pub fn create_text_node(&mut self, content: &str) -> DomNode {
        let id = self.next_node_id;
        self.next_node_id += 1;
        DomNode {
            id,
            tag_name: "#text".to_string(),
            node_type: DomNodeType::Text(content.to_string()),
        }
    }

    // Private helper methods

    fn allocate_node_id(&mut self, path: Vec<usize>) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.node_map.insert(id, path);
        id
    }

    /// Find matching nodes and return their paths and tag names (no self mutation)
    fn find_matching_nodes(
        node: &Node,
        selector: &str,
        path: Vec<usize>,
        first_only: bool,
    ) -> Vec<(Vec<usize>, String)> {
        let mut results = Vec::new();

        if Self::node_matches_selector(node, selector) {
            if let Some(elem) = node.as_element() {
                results.push((path.clone(), elem.tag_name.clone()));
                if first_only {
                    return results;
                }
            }
        }

        for (i, child) in node.children.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(i);
            let child_results = Self::find_matching_nodes(child, selector, child_path, first_only);
            results.extend(child_results);
            if first_only && !results.is_empty() {
                return results;
            }
        }

        results
    }

    fn node_matches_selector(node: &Node, selector: &str) -> bool {
        let elem = match node.as_element() {
            Some(e) => e,
            None => return false,
        };

        let selector = selector.trim();

        // ID selector: #id
        if let Some(id) = selector.strip_prefix('#') {
            return elem.id().map(|i| i == id).unwrap_or(false);
        }

        // Class selector: .class
        if let Some(class) = selector.strip_prefix('.') {
            return elem.classes().contains(&class);
        }

        // Tag selector
        elem.tag_name.eq_ignore_ascii_case(selector)
    }
}

/// A DOM node reference for JavaScript
#[derive(Debug, Clone)]
pub struct DomNode {
    /// Unique node identifier
    pub id: NodeId,
    /// Tag name (or #text for text nodes)
    pub tag_name: String,
    /// Node type
    pub node_type: DomNodeType,
}

/// Types of DOM nodes
#[derive(Debug, Clone)]
pub enum DomNodeType {
    /// Element node
    Element,
    /// Text node with content
    Text(String),
    /// Comment node
    Comment(String),
    /// Document node
    Document,
}

impl DomNode {
    /// Check if this is an element
    pub fn is_element(&self) -> bool {
        matches!(self.node_type, DomNodeType::Element)
    }

    /// Check if this is a text node
    pub fn is_text(&self) -> bool {
        matches!(self.node_type, DomNodeType::Text(_))
    }

    /// Get text content if this is a text node
    pub fn text_content(&self) -> Option<&str> {
        match &self.node_type {
            DomNodeType::Text(s) => Some(s),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::NodeType;

    fn create_test_document() -> Document {
        let mut doc = Document::new();
        let mut html = Node::element("html");

        let mut body = Node::element("body");
        let mut div = Node::element("div");
        if let NodeType::Element(ref mut data) = div.node_type {
            data.set_attribute("id", "main");
            data.set_attribute("class", "container active");
        }

        let p = Node::element("p");
        let text = Node::text("Hello World");

        div.add_child(p);
        div.add_child(text);
        body.add_child(div);
        html.add_child(body);
        doc.root.add_child(html);

        doc
    }

    #[test]
    fn test_get_element_by_id() {
        let doc = create_test_document();
        let mut bindings = DomBindings::new(doc);

        let result = bindings.get_element_by_id("main");
        assert!(result.is_some());
        let node = result.unwrap();
        assert_eq!(node.tag_name, "div");
    }

    #[test]
    fn test_query_selector_by_tag() {
        let doc = create_test_document();
        let mut bindings = DomBindings::new(doc);

        let result = bindings.query_selector("p");
        assert!(result.is_some());
        assert_eq!(result.unwrap().tag_name, "p");
    }

    #[test]
    fn test_query_selector_by_class() {
        let doc = create_test_document();
        let mut bindings = DomBindings::new(doc);

        let result = bindings.query_selector(".container");
        assert!(result.is_some());
        assert_eq!(result.unwrap().tag_name, "div");
    }

    #[test]
    fn test_get_elements_by_tag_name() {
        let doc = create_test_document();
        let mut bindings = DomBindings::new(doc);

        let results = bindings.get_elements_by_tag_name("div");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_create_element() {
        let doc = Document::new();
        let mut bindings = DomBindings::new(doc);

        let node = bindings.create_element("span");
        assert_eq!(node.tag_name, "span");
        assert!(node.is_element());
    }

    #[test]
    fn test_create_text_node() {
        let doc = Document::new();
        let mut bindings = DomBindings::new(doc);

        let node = bindings.create_text_node("Hello");
        assert!(node.is_text());
        assert_eq!(node.text_content(), Some("Hello"));
    }
}
