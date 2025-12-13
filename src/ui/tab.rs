//! Tab management

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};

/// Unique tab identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(u64);

impl TabId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Rendered content for display
#[derive(Debug, Clone)]
pub struct PageContent {
    /// Page title
    pub title: String,
    /// Rendered text elements with positions
    pub elements: Vec<RenderElement>,
    /// Error message if loading failed
    pub error: Option<String>,
}

impl Default for PageContent {
    fn default() -> Self {
        Self {
            title: "New Tab".to_string(),
            elements: Vec::new(),
            error: None,
        }
    }
}

/// A rendered element
#[derive(Debug, Clone)]
pub struct RenderElement {
    /// Element type
    pub kind: ElementKind,
    /// Text content
    pub text: String,
    /// Y position (line number)
    pub y: f32,
    /// Indentation level
    pub indent: u32,
    /// Is this a link?
    pub is_link: bool,
    /// Link URL if any
    pub href: Option<String>,
}

/// Kind of element
#[derive(Debug, Clone, PartialEq)]
pub enum ElementKind {
    Heading1,
    Heading2,
    Heading3,
    Paragraph,
    Link,
    ListItem,
    Code,
    Text,
}

/// A browser tab
pub struct Tab {
    id: TabId,
    title: String,
    url: String,
    loading: bool,
    /// Rendered page content
    content: Arc<Mutex<PageContent>>,
    /// Channel for receiving loaded content
    content_receiver: Option<Receiver<PageContent>>,
}

impl Tab {
    /// Create a new tab
    pub fn new(id: TabId) -> Self {
        Self {
            id,
            title: "New Tab".to_string(),
            url: String::new(),
            loading: false,
            content: Arc::new(Mutex::new(PageContent::default())),
            content_receiver: None,
        }
    }

    /// Get the tab ID
    pub fn id(&self) -> TabId {
        self.id
    }

    /// Get the tab title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the tab title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Get the current URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Navigate to a URL
    pub fn navigate(&mut self, url: impl Into<String>) {
        let url_str: String = url.into();

        // Normalize URL
        let normalized_url = if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
            format!("https://{}", url_str)
        } else {
            url_str
        };

        self.url = normalized_url.clone();
        self.loading = true;

        // Create channel for async content loading
        let (tx, rx) = channel::<PageContent>();
        self.content_receiver = Some(rx);

        // Spawn async task to fetch and parse the page
        let content = Arc::clone(&self.content);
        std::thread::spawn(move || {
            let result = fetch_and_parse(&normalized_url);
            let _ = tx.send(result);
        });
    }

    /// Check if the tab is loading
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Mark loading as complete
    pub fn finish_loading(&mut self) {
        self.loading = false;
    }

    /// Check for loaded content and update
    pub fn poll_content(&mut self) {
        if let Some(ref rx) = self.content_receiver {
            if let Ok(new_content) = rx.try_recv() {
                self.title = new_content.title.clone();
                if let Ok(mut content) = self.content.lock() {
                    *content = new_content;
                }
                self.loading = false;
                self.content_receiver = None;
            }
        }
    }

    /// Get page content
    pub fn content(&self) -> PageContent {
        self.content.lock().map(|c| c.clone()).unwrap_or_default()
    }
}

/// Fetch and parse a web page
fn fetch_and_parse(url: &str) -> PageContent {
    // Use blocking reqwest for simplicity in thread
    let client = reqwest::blocking::Client::builder()
        .user_agent("Binix/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => return PageContent {
            title: "Error".to_string(),
            elements: vec![],
            error: Some(format!("Failed to create client: {}", e)),
        },
    };

    let response = match client.get(url).send() {
        Ok(r) => r,
        Err(e) => return PageContent {
            title: "Error".to_string(),
            elements: vec![],
            error: Some(format!("Failed to fetch: {}", e)),
        },
    };

    let html = match response.text() {
        Ok(t) => t,
        Err(e) => return PageContent {
            title: "Error".to_string(),
            elements: vec![],
            error: Some(format!("Failed to read response: {}", e)),
        },
    };

    // Parse HTML and extract content
    parse_html_to_content(&html, url)
}

/// Parse HTML to renderable content
fn parse_html_to_content(html: &str, _url: &str) -> PageContent {
    use html5ever::tendril::TendrilSink;
    use html5ever::{parse_document, ParseOpts};
    use markup5ever_rcdom::RcDom;

    let opts = ParseOpts::default();
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .expect("Failed to parse HTML");

    let mut elements = Vec::new();
    let mut title = String::new();
    let mut y: f32 = 0.0;

    // Recursively extract content
    extract_content(&dom.document, &mut elements, &mut title, &mut y, 0);

    if title.is_empty() {
        title = "Untitled".to_string();
    }

    PageContent {
        title,
        elements,
        error: None,
    }
}

use markup5ever_rcdom::Handle;

fn extract_content(
    handle: &Handle,
    elements: &mut Vec<RenderElement>,
    title: &mut String,
    y: &mut f32,
    indent: u32,
) {
    use markup5ever_rcdom::NodeData;

    match &handle.data {
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.as_ref();
            let attrs = attrs.borrow();

            // Extract href for links
            let href = attrs.iter()
                .find(|a| a.name.local.as_ref() == "href")
                .map(|a| a.value.to_string());

            match tag {
                "title" => {
                    // Extract title text
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Text { contents } = &child.data {
                            *title = contents.borrow().to_string().trim().to_string();
                        }
                    }
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let kind = match tag {
                            "h1" => ElementKind::Heading1,
                            "h2" => ElementKind::Heading2,
                            _ => ElementKind::Heading3,
                        };
                        elements.push(RenderElement {
                            kind,
                            text,
                            y: *y,
                            indent,
                            is_link: false,
                            href: None,
                        });
                        *y += 1.0;
                    }
                }
                "p" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        elements.push(RenderElement {
                            kind: ElementKind::Paragraph,
                            text,
                            y: *y,
                            indent,
                            is_link: false,
                            href: None,
                        });
                        *y += 1.0;
                    }
                }
                "a" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        elements.push(RenderElement {
                            kind: ElementKind::Link,
                            text,
                            y: *y,
                            indent,
                            is_link: true,
                            href,
                        });
                        *y += 1.0;
                    }
                    return; // Don't recurse into links
                }
                "li" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        elements.push(RenderElement {
                            kind: ElementKind::ListItem,
                            text: format!("• {}", text),
                            y: *y,
                            indent: indent + 1,
                            is_link: false,
                            href: None,
                        });
                        *y += 1.0;
                    }
                    return;
                }
                "script" | "style" | "noscript" | "meta" | "link" | "head" => {
                    return; // Skip these elements
                }
                _ => {}
            }

            // Recurse into children
            for child in handle.children.borrow().iter() {
                extract_content(child, elements, title, y, indent);
            }
        }
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            let text = text.trim();
            if !text.is_empty() && text.len() > 1 {
                elements.push(RenderElement {
                    kind: ElementKind::Text,
                    text: text.to_string(),
                    y: *y,
                    indent,
                    is_link: false,
                    href: None,
                });
                *y += 1.0;
            }
        }
        NodeData::Document => {
            for child in handle.children.borrow().iter() {
                extract_content(child, elements, title, y, indent);
            }
        }
        _ => {}
    }
}

fn extract_text(handle: &Handle) -> String {
    use markup5ever_rcdom::NodeData;

    let mut text = String::new();

    fn collect(handle: &Handle, text: &mut String) {
        match &handle.data {
            NodeData::Text { contents } => {
                let s = contents.borrow().to_string();
                let s = s.trim();
                if !s.is_empty() {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(s);
                }
            }
            NodeData::Element { name, .. } => {
                let tag = name.local.as_ref();
                if tag != "script" && tag != "style" {
                    for child in handle.children.borrow().iter() {
                        collect(child, text);
                    }
                }
            }
            _ => {
                for child in handle.children.borrow().iter() {
                    collect(child, text);
                }
            }
        }
    }

    collect(handle, &mut text);
    text
}

/// Manages multiple tabs
pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab: Option<TabId>,
    next_id: u64,
}

impl TabManager {
    /// Create a new tab manager
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            next_id: 1,
        }
    }

    /// Create a new tab
    pub fn create_tab(&mut self) -> TabId {
        let id = TabId::new(self.next_id);
        self.next_id += 1;

        let tab = Tab::new(id);
        self.tabs.push(tab);

        if self.active_tab.is_none() {
            self.active_tab = Some(id);
        }

        id
    }

    /// Close a tab
    pub fn close_tab(&mut self, id: TabId) {
        self.tabs.retain(|t| t.id != id);

        if self.active_tab == Some(id) {
            self.active_tab = self.tabs.first().map(|t| t.id);
        }
    }

    /// Get the active tab
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_tab
            .and_then(|id| self.tabs.iter().find(|t| t.id == id))
    }

    /// Get mutable reference to active tab
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab
            .and_then(|id| self.tabs.iter_mut().find(|t| t.id == id))
    }

    /// Set the active tab
    pub fn set_active(&mut self, id: TabId) {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_tab = Some(id);
        }
    }

    /// Get all tabs
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Get tab count
    pub fn count(&self) -> usize {
        self.tabs.len()
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

