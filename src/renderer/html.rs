//! HTML5 parser implementation using html5ever

use super::dom::{Document, ElementData, Node, NodeType};
use crate::utils::Result;
use html5ever::parse_document;
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{local_name, namespace_url, ns, ParseOpts, QualName};
use markup5ever::interface::tree_builder::NodeOrText;
use markup5ever::interface::tree_builder::TreeSink;
use markup5ever::Attribute;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

/// Handle type for DOM nodes (index into the arena)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle(usize);

/// DOM sink that builds our Document structure
pub struct DomSink {
    /// Arena of nodes
    nodes: RefCell<Vec<SinkNode>>,
    /// QualNames for elements (stored separately to allow returning references)
    qual_names: RefCell<Vec<QualName>>,
    /// Document handle
    document_handle: Handle,
}

/// Internal node representation during parsing
struct SinkNode {
    node_type: SinkNodeType,
    children: Vec<Handle>,
    /// Index into qual_names for elements
    qual_name_idx: Option<usize>,
}

enum SinkNodeType {
    Document,
    Element { name: String, attrs: HashMap<String, String> },
    Text(String),
    Comment(String),
    Doctype,
    ProcessingInstruction,
}

impl DomSink {
    fn new() -> Self {
        let nodes = vec![SinkNode {
            node_type: SinkNodeType::Document,
            children: Vec::new(),
            qual_name_idx: None,
        }];
        Self {
            nodes: RefCell::new(nodes),
            qual_names: RefCell::new(Vec::new()),
            document_handle: Handle(0),
        }
    }

    fn new_handle(&self, node: SinkNode) -> Handle {
        let mut nodes = self.nodes.borrow_mut();
        let handle = Handle(nodes.len());
        nodes.push(node);
        handle
    }

    fn add_qual_name(&self, qn: QualName) -> usize {
        let mut qual_names = self.qual_names.borrow_mut();
        let idx = qual_names.len();
        qual_names.push(qn);
        idx
    }

    /// Convert to our Document format
    fn into_document(self) -> Document {
        let nodes = self.nodes.into_inner();
        let mut document = Document::new();

        // Get document children
        if let Some(doc_node) = nodes.first() {
            for child_handle in &doc_node.children {
                if let Some(node) = Self::convert_node(&nodes, *child_handle) {
                    document.root.add_child(node);
                }
            }
        }

        document
    }

    fn convert_node(nodes: &[SinkNode], handle: Handle) -> Option<Node> {
        let sink_node = nodes.get(handle.0)?;

        match &sink_node.node_type {
            SinkNodeType::Document => None,
            SinkNodeType::Element { name, attrs } => {
                let mut elem_data = ElementData::new(name.clone());
                for (k, v) in attrs {
                    elem_data.set_attribute(k.clone(), v.clone());
                }
                let mut node = Node::new(NodeType::Element(elem_data));
                for child_handle in &sink_node.children {
                    if let Some(child) = Self::convert_node(nodes, *child_handle) {
                        node.add_child(child);
                    }
                }
                Some(node)
            }
            SinkNodeType::Text(text) => {
                if text.trim().is_empty() {
                    None
                } else {
                    Some(Node::text(text.clone()))
                }
            }
            SinkNodeType::Comment(text) => Some(Node::new(NodeType::Comment(text.clone()))),
            SinkNodeType::Doctype | SinkNodeType::ProcessingInstruction => None,
        }
    }
}

impl TreeSink for DomSink {
    type Handle = Handle;
    type Output = Self;
    type ElemName<'a> = &'a QualName where Self: 'a;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {}

    fn get_document(&self) -> Handle {
        self.document_handle
    }

    fn elem_name<'a>(&'a self, target: &'a Handle) -> Self::ElemName<'a> {
        let nodes = self.nodes.borrow();
        if let Some(node) = nodes.get(target.0) {
            if let Some(idx) = node.qual_name_idx {
                let qual_names = self.qual_names.borrow();
                // Safety: We're returning a reference that outlives the borrow
                // This is a workaround - in production code we'd use a different approach
                let qn_ptr = &qual_names[idx] as *const QualName;
                return unsafe { &*qn_ptr };
            }
        }
        // Return a static reference for non-elements
        static DEFAULT_QNAME: std::sync::OnceLock<QualName> = std::sync::OnceLock::new();
        DEFAULT_QNAME.get_or_init(|| QualName::new(None, ns!(html), local_name!("")))
    }

    fn create_element(&self, name: QualName, attrs: Vec<Attribute>, _flags: html5ever::tree_builder::ElementFlags) -> Handle {
        let mut attr_map = HashMap::new();
        for attr in attrs {
            attr_map.insert(attr.name.local.to_string(), attr.value.to_string());
        }
        let qual_name_idx = Some(self.add_qual_name(name.clone()));
        self.new_handle(SinkNode {
            node_type: SinkNodeType::Element {
                name: name.local.to_string(),
                attrs: attr_map,
            },
            children: Vec::new(),
            qual_name_idx,
        })
    }

    fn create_comment(&self, text: StrTendril) -> Handle {
        self.new_handle(SinkNode {
            node_type: SinkNodeType::Comment(text.to_string()),
            children: Vec::new(),
            qual_name_idx: None,
        })
    }

    fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> Handle {
        self.new_handle(SinkNode {
            node_type: SinkNodeType::ProcessingInstruction,
            qual_name_idx: None,
            children: Vec::new(),
        })
    }

    fn append(&self, parent: &Handle, child: NodeOrText<Handle>) {
        let child_handle = match child {
            NodeOrText::AppendNode(h) => h,
            NodeOrText::AppendText(text) => self.new_handle(SinkNode {
                node_type: SinkNodeType::Text(text.to_string()),
                children: Vec::new(),
                qual_name_idx: None,
            }),
        };
        self.nodes.borrow_mut()[parent.0].children.push(child_handle);
    }

    fn append_based_on_parent_node(&self, _element: &Handle, prev: &Handle, child: NodeOrText<Handle>) {
        self.append(prev, child);
    }

    fn append_doctype_to_document(&self, _name: StrTendril, _public: StrTendril, _system: StrTendril) {
        let doctype = self.new_handle(SinkNode {
            node_type: SinkNodeType::Doctype,
            children: Vec::new(),
            qual_name_idx: None,
        });
        self.nodes.borrow_mut()[0].children.push(doctype);
    }

    fn get_template_contents(&self, target: &Handle) -> Handle {
        *target
    }

    fn same_node(&self, x: &Handle, y: &Handle) -> bool {
        x.0 == y.0
    }

    fn set_quirks_mode(&self, _mode: html5ever::tree_builder::QuirksMode) {}

    fn append_before_sibling(&self, sibling: &Handle, new_node: NodeOrText<Handle>) {
        // Find parent and insert before sibling
        let nodes = self.nodes.borrow();
        for (idx, node) in nodes.iter().enumerate() {
            if let Some(pos) = node.children.iter().position(|h| h.0 == sibling.0) {
                drop(nodes);
                let child_handle = match new_node {
                    NodeOrText::AppendNode(h) => h,
                    NodeOrText::AppendText(text) => self.new_handle(SinkNode {
                        node_type: SinkNodeType::Text(text.to_string()),
                        children: Vec::new(),
                        qual_name_idx: None,
                    }),
                };
                self.nodes.borrow_mut()[idx].children.insert(pos, child_handle);
                return;
            }
        }
    }

    fn add_attrs_if_missing(&self, target: &Handle, attrs: Vec<Attribute>) {
        let mut nodes = self.nodes.borrow_mut();
        if let Some(node) = nodes.get_mut(target.0) {
            if let SinkNodeType::Element { attrs: existing, .. } = &mut node.node_type {
                for attr in attrs {
                    existing.entry(attr.name.local.to_string())
                        .or_insert_with(|| attr.value.to_string());
                }
            }
        }
    }

    fn remove_from_parent(&self, target: &Handle) {
        let mut nodes = self.nodes.borrow_mut();
        for node in nodes.iter_mut() {
            node.children.retain(|h| h.0 != target.0);
        }
    }

    fn reparent_children(&self, node: &Handle, new_parent: &Handle) {
        let mut nodes = self.nodes.borrow_mut();
        let children: Vec<Handle> = nodes[node.0].children.drain(..).collect();
        nodes[new_parent.0].children.extend(children);
    }
}

/// HTML5 parser using html5ever
pub struct HtmlParser {
    opts: ParseOpts,
}

impl HtmlParser {
    /// Create a new HTML parser
    pub fn new() -> Self {
        Self {
            opts: ParseOpts {
                tree_builder: TreeBuilderOpts {
                    drop_doctype: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    /// Parse HTML content into a DOM document
    pub fn parse(&self, content: &str) -> Result<Document> {
        if content.trim().is_empty() {
            return Ok(Document::new());
        }

        let sink = DomSink::new();
        let dom = parse_document(sink, self.opts.clone())
            .from_utf8()
            .read_from(&mut content.as_bytes())
            .expect("Failed to parse HTML");

        Ok(dom.into_document())
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_html() {
        let parser = HtmlParser::new();
        let doc = parser.parse("").unwrap();
        assert!(doc.root.children.is_empty());
    }

    #[test]
    fn test_parse_simple_html() {
        let parser = HtmlParser::new();
        let doc = parser.parse("<html><body>Hello</body></html>").unwrap();
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_parse_with_attributes() {
        let parser = HtmlParser::new();
        let doc = parser.parse(r#"<div id="main" class="container">Content</div>"#).unwrap();
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_parse_nested_elements() {
        let parser = HtmlParser::new();
        let doc = parser.parse(r#"
            <html>
                <head><title>Test</title></head>
                <body>
                    <div>
                        <p>Paragraph 1</p>
                        <p>Paragraph 2</p>
                    </div>
                </body>
            </html>
        "#).unwrap();
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_parse_malformed_html() {
        // html5ever should handle malformed HTML gracefully
        let parser = HtmlParser::new();
        let doc = parser.parse("<p>Unclosed paragraph<div>Another").unwrap();
        assert!(!doc.root.children.is_empty());
    }
}
