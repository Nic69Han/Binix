//! DOM bindings for JavaScript
//!
//! Provides JavaScript access to the DOM tree, including:
//! - Element creation and manipulation
//! - Event listeners
//! - innerHTML/textContent

use crate::renderer::{Document, Node, NodeType, ElementData};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Event listener callback type
pub type EventCallback = Box<dyn Fn(&Event) + Send + Sync>;

/// DOM Event
#[derive(Debug, Clone)]
pub struct Event {
    /// Event type (e.g., "click", "input")
    pub event_type: String,
    /// Target node ID
    pub target: NodeId,
    /// Whether propagation was stopped
    pub propagation_stopped: bool,
    /// Whether default was prevented
    pub default_prevented: bool,
}

impl Event {
    /// Create a new event
    pub fn new(event_type: &str, target: NodeId) -> Self {
        Self {
            event_type: event_type.to_string(),
            target,
            propagation_stopped: false,
            default_prevented: false,
        }
    }

    /// Stop event propagation
    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    /// Prevent default behavior
    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }
}

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
    /// Pending nodes (created but not attached to DOM)
    pending_nodes: HashMap<NodeId, Node>,
    /// Event listeners: (node_id, event_type) -> callbacks
    event_listeners: HashMap<(NodeId, String), Vec<usize>>,
    /// Stored callbacks (indexed by ID)
    callbacks: Vec<EventCallback>,
}

impl DomBindings {
    /// Create new DOM bindings for a document
    pub fn new(document: Document) -> Self {
        Self {
            document: Arc::new(Mutex::new(document)),
            next_node_id: 1,
            node_map: HashMap::new(),
            pending_nodes: HashMap::new(),
            event_listeners: HashMap::new(),
            callbacks: Vec::new(),
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

        // Store the actual DOM node for later attachment
        let node = Node::element(tag_name);
        self.pending_nodes.insert(id, node);

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

        // Store the actual DOM node
        let node = Node::text(content);
        self.pending_nodes.insert(id, node);

        DomNode {
            id,
            tag_name: "#text".to_string(),
            node_type: DomNodeType::Text(content.to_string()),
        }
    }

    /// Append a child node to a parent
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) -> bool {
        // Get the child node from pending or create a placeholder
        let child_node = match self.pending_nodes.remove(&child_id) {
            Some(node) => node,
            None => return false, // Child not found
        };

        // If parent is in DOM, add child to DOM
        if let Some(path) = self.node_map.get(&parent_id).cloned() {
            if let Ok(mut doc) = self.document.lock() {
                if let Some(parent) = Self::get_node_at_path_mut(&mut doc.root, &path) {
                    parent.add_child(child_node);
                    return true;
                }
            }
        }

        // If parent is pending, add child to pending parent
        if let Some(parent) = self.pending_nodes.get_mut(&parent_id) {
            parent.add_child(child_node);
            return true;
        }

        false
    }

    /// Remove a child node from a parent
    pub fn remove_child(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        if let Some(path) = self.node_map.get(&parent_id).cloned() {
            if let Ok(mut doc) = self.document.lock() {
                if let Some(parent) = Self::get_node_at_path_mut(&mut doc.root, &path) {
                    if child_index < parent.children.len() {
                        parent.children.remove(child_index);
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Set innerHTML of an element
    pub fn set_inner_html(&mut self, node_id: NodeId, html: &str) -> bool {
        use crate::renderer::HtmlParser;

        // Parse HTML fragment
        let parser = HtmlParser::new();
        let fragment = match parser.parse(&format!("<div>{}</div>", html)) {
            Ok(doc) => doc,
            Err(_) => return false,
        };

        // Get the parsed children
        let new_children: Vec<Node> = if !fragment.root.children.is_empty() {
            // Get children from the wrapper div
            if let Some(first_child) = fragment.root.children.first() {
                if !first_child.children.is_empty() {
                    first_child.children.clone()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Replace children of target node
        if let Some(path) = self.node_map.get(&node_id).cloned() {
            if let Ok(mut doc) = self.document.lock() {
                if let Some(node) = Self::get_node_at_path_mut(&mut doc.root, &path) {
                    node.children = new_children;
                    return true;
                }
            }
        }

        // Check pending nodes
        if let Some(node) = self.pending_nodes.get_mut(&node_id) {
            node.children = new_children;
            return true;
        }

        false
    }

    /// Get textContent of an element
    pub fn get_text_content(&self, node_id: NodeId) -> Option<String> {
        if let Some(path) = self.node_map.get(&node_id) {
            if let Ok(doc) = self.document.lock() {
                if let Some(node) = Self::get_node_at_path(&doc.root, path) {
                    return Some(Self::collect_text(node));
                }
            }
        }

        if let Some(node) = self.pending_nodes.get(&node_id) {
            return Some(Self::collect_text(node));
        }

        None
    }

    /// Set textContent of an element
    pub fn set_text_content(&mut self, node_id: NodeId, content: &str) -> bool {
        let text_node = Node::text(content);

        if let Some(path) = self.node_map.get(&node_id).cloned() {
            if let Ok(mut doc) = self.document.lock() {
                if let Some(node) = Self::get_node_at_path_mut(&mut doc.root, &path) {
                    node.children = vec![text_node];
                    return true;
                }
            }
        }

        if let Some(node) = self.pending_nodes.get_mut(&node_id) {
            node.children = vec![text_node];
            return true;
        }

        false
    }

    /// Set an attribute on an element
    pub fn set_attribute(&mut self, node_id: NodeId, name: &str, value: &str) -> bool {
        if let Some(path) = self.node_map.get(&node_id).cloned() {
            if let Ok(mut doc) = self.document.lock() {
                if let Some(node) = Self::get_node_at_path_mut(&mut doc.root, &path) {
                    if let NodeType::Element(ref mut data) = node.node_type {
                        data.set_attribute(name, value);
                        return true;
                    }
                }
            }
        }

        if let Some(node) = self.pending_nodes.get_mut(&node_id) {
            if let NodeType::Element(ref mut data) = node.node_type {
                data.set_attribute(name, value);
                return true;
            }
        }

        false
    }

    /// Get an attribute from an element
    pub fn get_attribute(&self, node_id: NodeId, name: &str) -> Option<String> {
        if let Some(path) = self.node_map.get(&node_id) {
            if let Ok(doc) = self.document.lock() {
                if let Some(node) = Self::get_node_at_path(&doc.root, path) {
                    if let Some(elem) = node.as_element() {
                        return elem.get_attribute(name).cloned();
                    }
                }
            }
        }

        if let Some(node) = self.pending_nodes.get(&node_id) {
            if let Some(elem) = node.as_element() {
                return elem.get_attribute(name).cloned();
            }
        }

        None
    }

    /// Add an event listener
    pub fn add_event_listener(&mut self, node_id: NodeId, event_type: &str, callback: EventCallback) {
        let callback_id = self.callbacks.len();
        self.callbacks.push(callback);

        let key = (node_id, event_type.to_string());
        self.event_listeners.entry(key).or_default().push(callback_id);
    }

    /// Remove event listeners for a node and event type
    pub fn remove_event_listeners(&mut self, node_id: NodeId, event_type: &str) {
        let key = (node_id, event_type.to_string());
        self.event_listeners.remove(&key);
    }

    /// Dispatch an event to listeners
    pub fn dispatch_event(&self, event: &Event) {
        let key = (event.target, event.event_type.clone());
        if let Some(callback_ids) = self.event_listeners.get(&key) {
            for &id in callback_ids {
                if let Some(callback) = self.callbacks.get(id) {
                    callback(event);
                }
            }
        }
    }

    // Private helper methods

    /// Get node at path (immutable)
    fn get_node_at_path<'a>(root: &'a Node, path: &[usize]) -> Option<&'a Node> {
        let mut current = root;
        for &index in path {
            current = current.children.get(index)?;
        }
        Some(current)
    }

    /// Get node at path (mutable)
    fn get_node_at_path_mut<'a>(root: &'a mut Node, path: &[usize]) -> Option<&'a mut Node> {
        let mut current = root;
        for &index in path {
            current = current.children.get_mut(index)?;
        }
        Some(current)
    }

    /// Collect all text from a node tree
    fn collect_text(node: &Node) -> String {
        match &node.node_type {
            NodeType::Text(content) => content.clone(),
            _ => {
                node.children.iter()
                    .map(Self::collect_text)
                    .collect::<Vec<_>>()
                    .join("")
            }
        }
    }

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
