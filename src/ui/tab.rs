//! Tab management

use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

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

/// A rendered element with full styling
#[derive(Debug, Clone)]
pub struct RenderElement {
    /// Element type
    pub kind: ElementKind,
    /// Text content
    pub text: String,
    /// Bounding box (x, y, width, height)
    pub bounds: ElementBounds,
    /// Visual style
    pub style: ElementStyle,
    /// Is this a link?
    pub is_link: bool,
    /// Link URL if any
    pub href: Option<String>,
    /// Image source URL (for Image elements)
    pub src: Option<String>,
    /// Alt text (for Image elements)
    pub alt: Option<String>,
    /// Children elements (for nested rendering)
    pub children: Vec<RenderElement>,
}

/// Element bounding box
#[derive(Debug, Clone, Copy, Default)]
pub struct ElementBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Visual styling for elements
#[derive(Debug, Clone)]
pub struct ElementStyle {
    /// Text color (RGBA)
    pub color: [u8; 4],
    /// Background color (RGBA)
    pub background_color: Option<[u8; 4]>,
    /// Font size in pixels
    pub font_size: f32,
    /// Bold text
    pub font_weight_bold: bool,
    /// Italic text
    pub font_style_italic: bool,
    /// Text decoration (underline)
    pub text_decoration_underline: bool,
    /// Padding (top, right, bottom, left)
    pub padding: [f32; 4],
    /// Margin (top, right, bottom, left)
    pub margin: [f32; 4],
    /// Border color (RGBA)
    pub border_color: Option<[u8; 4]>,
    /// Border width (top, right, bottom, left)
    pub border_width: [f32; 4],
    /// Border radius
    pub border_radius: f32,
    /// Text alignment
    pub text_align: TextAlign,
    /// Line height multiplier (1.0 = normal)
    pub line_height: f32,
    /// Max width for text wrapping (0 = no wrap)
    pub max_width: f32,
}

impl Default for ElementStyle {
    fn default() -> Self {
        Self {
            color: [0, 0, 0, 255], // Black text
            background_color: None,
            font_size: 16.0,
            font_weight_bold: false,
            font_style_italic: false,
            text_decoration_underline: false,
            padding: [0.0; 4],
            margin: [0.0; 4],
            border_color: None,
            border_width: [0.0; 4],
            border_radius: 0.0,
            text_align: TextAlign::Left,
            line_height: 1.4,
            max_width: 0.0, // 0 means use container width
        }
    }
}

impl RenderElement {
    /// Create a new render element with default styling
    pub fn new(kind: ElementKind, text: String) -> Self {
        Self {
            kind,
            text,
            bounds: ElementBounds::default(),
            style: ElementStyle::default(),
            is_link: false,
            href: None,
            src: None,
            alt: None,
            children: Vec::new(),
        }
    }

    /// Create an image element
    pub fn image(src: String, alt: String) -> Self {
        Self {
            kind: ElementKind::Image,
            text: alt.clone(),
            bounds: ElementBounds::default(),
            style: ElementStyle::default(),
            is_link: false,
            href: None,
            src: Some(src),
            alt: Some(alt),
            children: Vec::new(),
        }
    }

    /// Legacy compatibility: get y position
    pub fn y(&self) -> f32 {
        self.bounds.y
    }

    /// Legacy compatibility: get indent level (approximated from x position)
    pub fn indent(&self) -> u32 {
        (self.bounds.x / 20.0) as u32
    }
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
    Image,
    Blockquote,
    Table,
    TableRow,
    TableCell,
    HorizontalRule,
    // Form elements
    Button,
    Input,
    Label,
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
        let normalized_url = if !url_str.starts_with("http://") && !url_str.starts_with("https://")
        {
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
    // Handle file:// protocol for local files
    if url.starts_with("file://") {
        return fetch_local_file(url);
    }

    // Use blocking reqwest for simplicity in thread
    let client = reqwest::blocking::Client::builder()
        .user_agent("Binix/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return PageContent {
                title: "Error".to_string(),
                elements: vec![],
                error: Some(format!("Failed to create client: {}", e)),
            };
        }
    };

    let response = match client.get(url).send() {
        Ok(r) => r,
        Err(e) => {
            return PageContent {
                title: "Error".to_string(),
                elements: vec![],
                error: Some(format!("Failed to fetch: {}", e)),
            };
        }
    };

    let html = match response.text() {
        Ok(t) => t,
        Err(e) => {
            return PageContent {
                title: "Error".to_string(),
                elements: vec![],
                error: Some(format!("Failed to read response: {}", e)),
            };
        }
    };

    // Parse HTML and extract content
    parse_html_to_content(&html, url)
}

/// Fetch a local file
fn fetch_local_file(url: &str) -> PageContent {
    use std::fs;

    // Remove file:// prefix
    let path = url.trim_start_matches("file://");

    match fs::read_to_string(path) {
        Ok(html) => parse_html_to_content(&html, url),
        Err(e) => PageContent {
            title: "Error".to_string(),
            elements: vec![],
            error: Some(format!("Failed to read file: {}", e)),
        },
    }
}

/// Collected CSS rules from stylesheets
#[derive(Debug, Clone, Default)]
struct CollectedStyles {
    rules: Vec<CssRule>,
}

/// A simplified CSS rule for matching
#[derive(Debug, Clone)]
struct CssRule {
    selector: CssSelector,
    properties: std::collections::HashMap<String, String>,
}

/// A simplified CSS selector
#[derive(Debug, Clone)]
struct CssSelector {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
}

impl CssSelector {
    /// Calculate specificity (id, class, tag)
    fn specificity(&self) -> (u32, u32, u32) {
        (
            if self.id.is_some() { 1 } else { 0 },
            self.classes.len() as u32,
            if self.tag.is_some() { 1 } else { 0 },
        )
    }

    /// Check if selector matches element attributes
    fn matches(&self, tag_name: &str, id: Option<&str>, classes: &[String]) -> bool {
        // Check tag
        if let Some(ref sel_tag) = self.tag {
            if sel_tag.to_lowercase() != tag_name.to_lowercase() {
                return false;
            }
        }

        // Check id
        if let Some(ref sel_id) = self.id {
            match id {
                Some(elem_id) if elem_id == sel_id => {}
                _ => return false,
            }
        }

        // Check classes
        for class in &self.classes {
            if !classes.iter().any(|c| c == class) {
                return false;
            }
        }

        true
    }
}

/// Parse a selector string into CssSelector
fn parse_css_selector(selector: &str) -> CssSelector {
    let selector = selector.trim();
    let mut tag = None;
    let mut id = None;
    let mut classes = Vec::new();

    let mut current = String::new();
    let mut mode = 't'; // t=tag, #=id, .=class

    for ch in selector.chars() {
        match ch {
            '#' => {
                if !current.is_empty() {
                    match mode {
                        't' => tag = Some(current.clone()),
                        '.' => classes.push(current.clone()),
                        '#' => id = Some(current.clone()),
                        _ => {}
                    }
                    current.clear();
                }
                mode = '#';
            }
            '.' => {
                if !current.is_empty() {
                    match mode {
                        't' => tag = Some(current.clone()),
                        '.' => classes.push(current.clone()),
                        '#' => id = Some(current.clone()),
                        _ => {}
                    }
                    current.clear();
                }
                mode = '.';
            }
            ' ' | '>' | '+' | '~' => {
                // Stop at combinators (simplified - only match the last part)
                if !current.is_empty() {
                    match mode {
                        't' => tag = Some(current.clone()),
                        '.' => classes.push(current.clone()),
                        '#' => id = Some(current.clone()),
                        _ => {}
                    }
                    current.clear();
                }
                // Reset for next part
                tag = None;
                id = None;
                classes.clear();
                mode = 't';
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Handle remaining
    if !current.is_empty() {
        match mode {
            't' => tag = Some(current),
            '.' => classes.push(current),
            '#' => id = Some(current),
            _ => {}
        }
    }

    CssSelector { tag, id, classes }
}

/// Parse CSS text into rules
fn parse_css_rules(css: &str) -> Vec<CssRule> {
    let mut rules = Vec::new();

    // Simple regex-free CSS parser
    let mut chars = css.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            chars.next();
        }

        // Read selector
        let mut selector = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == '{' {
                chars.next();
                break;
            }
            selector.push(ch);
            chars.next();
        }

        if selector.trim().is_empty() {
            break;
        }

        // Read properties until }
        let mut properties_str = String::new();
        let mut depth = 1;
        while let Some(ch) = chars.next() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            properties_str.push(ch);
        }

        // Parse properties
        let mut properties = std::collections::HashMap::new();
        for decl in properties_str.split(';') {
            let decl = decl.trim();
            if let Some(colon_pos) = decl.find(':') {
                let prop = decl[..colon_pos].trim().to_lowercase();
                let value = decl[colon_pos + 1..].trim().to_string();
                if !prop.is_empty() && !value.is_empty() {
                    properties.insert(prop, value);
                }
            }
        }

        // Parse selectors (comma-separated)
        for sel_str in selector.split(',') {
            let sel_str = sel_str.trim();
            if !sel_str.is_empty() {
                rules.push(CssRule {
                    selector: parse_css_selector(sel_str),
                    properties: properties.clone(),
                });
            }
        }
    }

    rules
}

/// Extract CSS from style elements
fn extract_stylesheets(handle: &Handle) -> String {
    use markup5ever_rcdom::NodeData;

    let mut css = String::new();

    match &handle.data {
        NodeData::Element { name, .. } => {
            if name.local.as_ref() == "style" {
                // Get text content
                for child in handle.children.borrow().iter() {
                    if let NodeData::Text { contents } = &child.data {
                        css.push_str(&contents.borrow());
                        css.push('\n');
                    }
                }
            }
        }
        _ => {}
    }

    // Recurse
    for child in handle.children.borrow().iter() {
        css.push_str(&extract_stylesheets(child));
    }

    css
}

/// Parse HTML to renderable content
fn parse_html_to_content(html: &str, base_url: &str) -> PageContent {
    use html5ever::tendril::TendrilSink;
    use html5ever::{ParseOpts, parse_document};
    use markup5ever_rcdom::RcDom;

    let opts = ParseOpts::default();
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .expect("Failed to parse HTML");

    // Extract CSS from <style> elements
    let css_text = extract_stylesheets(&dom.document);
    let css_rules = parse_css_rules(&css_text);

    let mut elements = Vec::new();
    let mut title = String::new();
    let mut y: f32 = 0.0;

    // Recursively extract content with CSS rules
    extract_content_with_css(&dom.document, &mut elements, &mut title, &mut y, 0, base_url, &css_rules);

    if title.is_empty() {
        title = "Untitled".to_string();
    }

    PageContent {
        title,
        elements,
        error: None,
    }
}

/// Resolve a relative URL against a base URL
fn resolve_url(relative: &str, base_url: &str) -> String {
    // Already absolute
    if relative.starts_with("http://") || relative.starts_with("https://") || relative.starts_with("file://") {
        return relative.to_string();
    }

    // Data URIs
    if relative.starts_with("data:") {
        return relative.to_string();
    }

    // Protocol-relative URL
    if relative.starts_with("//") {
        if base_url.starts_with("https://") {
            return format!("https:{}", relative);
        } else {
            return format!("http:{}", relative);
        }
    }

    // Parse base URL
    let base = if let Some(idx) = base_url.rfind('/') {
        if base_url[..idx].contains("://") {
            // Has scheme, check if path or host
            let after_scheme = &base_url[base_url.find("://").unwrap() + 3..];
            if after_scheme.contains('/') {
                &base_url[..idx + 1]
            } else {
                &base_url[..]
            }
        } else {
            base_url
        }
    } else {
        base_url
    };

    // Absolute path from root
    if relative.starts_with('/') {
        // Extract origin (scheme + host)
        if let Some(scheme_end) = base_url.find("://") {
            let after_scheme = &base_url[scheme_end + 3..];
            if let Some(path_start) = after_scheme.find('/') {
                let origin = &base_url[..scheme_end + 3 + path_start];
                return format!("{}{}", origin, relative);
            }
        }
        return format!("{}{}", base_url.trim_end_matches('/'), relative);
    }

    // Relative path
    if base.ends_with('/') {
        format!("{}{}", base, relative)
    } else {
        format!("{}/{}", base, relative)
    }
}

use markup5ever_rcdom::Handle;

/// Create a styled render element based on tag type
fn create_styled_element(kind: ElementKind, text: String, y: f32, indent: u32) -> RenderElement {
    let mut style = ElementStyle::default();

    // Apply default styles based on element type
    match kind {
        ElementKind::Heading1 => {
            style.font_size = 32.0;
            style.font_weight_bold = true;
            style.margin = [24.0, 0.0, 16.0, 0.0];
        }
        ElementKind::Heading2 => {
            style.font_size = 24.0;
            style.font_weight_bold = true;
            style.margin = [20.0, 0.0, 12.0, 0.0];
        }
        ElementKind::Heading3 => {
            style.font_size = 18.0;
            style.font_weight_bold = true;
            style.margin = [16.0, 0.0, 8.0, 0.0];
        }
        ElementKind::Paragraph => {
            style.font_size = 16.0;
            style.margin = [0.0, 0.0, 16.0, 0.0];
        }
        ElementKind::Link => {
            style.font_size = 16.0;
            style.color = [0, 102, 204, 255]; // Blue
            style.text_decoration_underline = true;
        }
        ElementKind::ListItem => {
            style.font_size = 16.0;
            style.margin = [4.0, 0.0, 4.0, 20.0];
        }
        ElementKind::Code => {
            style.font_size = 14.0;
            style.background_color = Some([245, 245, 245, 255]);
            style.padding = [2.0, 4.0, 2.0, 4.0];
        }
        ElementKind::Text => {
            style.font_size = 16.0;
        }
        ElementKind::Image => {
            style.margin = [8.0, 0.0, 8.0, 0.0];
        }
        ElementKind::Blockquote => {
            style.font_size = 16.0;
            style.font_style_italic = true;
            style.color = [100, 100, 100, 255];
            style.border_color = Some([200, 200, 200, 255]);
            style.border_width = [0.0, 0.0, 0.0, 4.0]; // Left border
            style.padding = [8.0, 16.0, 8.0, 16.0];
            style.margin = [16.0, 0.0, 16.0, 0.0];
        }
        ElementKind::Table => {
            style.margin = [16.0, 0.0, 16.0, 0.0];
        }
        ElementKind::TableRow => {
            style.border_color = Some([220, 220, 220, 255]);
            style.border_width = [0.0, 0.0, 1.0, 0.0]; // Bottom border
        }
        ElementKind::TableCell => {
            style.padding = [8.0, 12.0, 8.0, 12.0];
        }
        ElementKind::HorizontalRule => {
            style.background_color = Some([200, 200, 200, 255]);
            style.margin = [16.0, 0.0, 16.0, 0.0];
        }
        ElementKind::Button => {
            style.font_size = 14.0;
            style.background_color = Some([59, 130, 246, 255]); // Blue button
            style.color = [255, 255, 255, 255]; // White text
            style.padding = [8.0, 16.0, 8.0, 16.0];
            style.border_radius = 4.0;
        }
        ElementKind::Input => {
            style.font_size = 14.0;
            style.background_color = Some([255, 255, 255, 255]);
            style.border_color = Some([200, 200, 200, 255]);
            style.border_width = [1.0; 4];
            style.padding = [8.0, 12.0, 8.0, 12.0];
            style.border_radius = 4.0;
        }
        ElementKind::Label => {
            style.font_size = 14.0;
            style.margin = [4.0, 0.0, 4.0, 0.0];
        }
    }

    RenderElement {
        kind,
        text,
        bounds: ElementBounds {
            x: indent as f32 * 20.0,
            y: y * 24.0, // Line height
            width: 0.0,  // Will be calculated during layout
            height: style.font_size * 1.2,
        },
        style,
        is_link: false,
        href: None,
        src: None,
        alt: None,
        children: Vec::new(),
    }
}

/// Parse inline style attribute and apply to ElementStyle
fn parse_inline_style(style_attr: &str, style: &mut ElementStyle) {
    for declaration in style_attr.split(';') {
        let parts: Vec<&str> = declaration.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        let property = parts[0].trim().to_lowercase();
        let value = parts[1].trim();

        match property.as_str() {
            "color" => {
                if let Some(color) = parse_css_color(value) {
                    style.color = color;
                }
            }
            "background-color" | "background" => {
                if let Some(color) = parse_css_color(value) {
                    style.background_color = Some(color);
                }
            }
            "font-size" => {
                if let Some(size) = parse_css_size(value) {
                    style.font_size = size;
                }
            }
            "font-weight" => {
                style.font_weight_bold = value == "bold" || value == "700" || value == "800" || value == "900";
            }
            "font-style" => {
                style.font_style_italic = value == "italic" || value == "oblique";
            }
            "text-decoration" => {
                style.text_decoration_underline = value.contains("underline");
            }
            "padding" => {
                if let Some(size) = parse_css_size(value) {
                    style.padding = [size; 4];
                }
            }
            "padding-top" => {
                if let Some(size) = parse_css_size(value) {
                    style.padding[0] = size;
                }
            }
            "padding-right" => {
                if let Some(size) = parse_css_size(value) {
                    style.padding[1] = size;
                }
            }
            "padding-bottom" => {
                if let Some(size) = parse_css_size(value) {
                    style.padding[2] = size;
                }
            }
            "padding-left" => {
                if let Some(size) = parse_css_size(value) {
                    style.padding[3] = size;
                }
            }
            "margin" => {
                if let Some(size) = parse_css_size(value) {
                    style.margin = [size; 4];
                }
            }
            "margin-top" => {
                if let Some(size) = parse_css_size(value) {
                    style.margin[0] = size;
                }
            }
            "margin-right" => {
                if let Some(size) = parse_css_size(value) {
                    style.margin[1] = size;
                }
            }
            "margin-bottom" => {
                if let Some(size) = parse_css_size(value) {
                    style.margin[2] = size;
                }
            }
            "margin-left" => {
                if let Some(size) = parse_css_size(value) {
                    style.margin[3] = size;
                }
            }
            "border-radius" => {
                if let Some(size) = parse_css_size(value) {
                    style.border_radius = size;
                }
            }
            "border-color" => {
                if let Some(color) = parse_css_color(value) {
                    style.border_color = Some(color);
                }
            }
            "border-width" => {
                if let Some(size) = parse_css_size(value) {
                    style.border_width = [size; 4];
                }
            }
            _ => {}
        }
    }
}

/// Parse a CSS color value to RGBA
fn parse_css_color(value: &str) -> Option<[u8; 4]> {
    let value = value.trim().to_lowercase();

    // Named colors
    match value.as_str() {
        "black" => return Some([0, 0, 0, 255]),
        "white" => return Some([255, 255, 255, 255]),
        "red" => return Some([255, 0, 0, 255]),
        "green" => return Some([0, 128, 0, 255]),
        "blue" => return Some([0, 0, 255, 255]),
        "yellow" => return Some([255, 255, 0, 255]),
        "orange" => return Some([255, 165, 0, 255]),
        "purple" => return Some([128, 0, 128, 255]),
        "gray" | "grey" => return Some([128, 128, 128, 255]),
        "transparent" => return Some([0, 0, 0, 0]),
        _ => {}
    }

    // Hex colors
    if value.starts_with('#') {
        let hex = &value[1..];
        if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some([r, g, b, 255]);
        } else if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some([r, g, b, 255]);
        }
    }

    // rgb() and rgba()
    if value.starts_with("rgb") {
        let inner = value
            .trim_start_matches("rgba(")
            .trim_start_matches("rgb(")
            .trim_end_matches(')');
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            let a = if parts.len() >= 4 {
                (parts[3].trim().parse::<f32>().ok()? * 255.0) as u8
            } else {
                255
            };
            return Some([r, g, b, a]);
        }
    }

    None
}

/// Parse a CSS size value to pixels
fn parse_css_size(value: &str) -> Option<f32> {
    let value = value.trim().to_lowercase();

    if value.ends_with("px") {
        return value.trim_end_matches("px").parse().ok();
    } else if value.ends_with("rem") {
        // Check rem before em since rem contains em
        let rem: f32 = value.trim_end_matches("rem").parse().ok()?;
        return Some(rem * 16.0);
    } else if value.ends_with("em") {
        let em: f32 = value.trim_end_matches("em").parse().ok()?;
        return Some(em * 16.0); // Base font size
    } else if value.ends_with('%') {
        let pct: f32 = value.trim_end_matches('%').parse().ok()?;
        return Some(pct / 100.0 * 16.0);
    }

    // Try parsing as plain number
    value.parse().ok()
}

// Legacy function - replaced by extract_content_with_css
#[allow(dead_code)]
fn extract_content(
    handle: &Handle,
    elements: &mut Vec<RenderElement>,
    title: &mut String,
    y: &mut f32,
    indent: u32,
    base_url: &str,
) {
    use markup5ever_rcdom::NodeData;

    match &handle.data {
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.as_ref();
            let attrs = attrs.borrow();

            // Extract href for links (resolve relative URLs)
            let href = attrs
                .iter()
                .find(|a| a.name.local.as_ref() == "href")
                .map(|a| resolve_url(&a.value.to_string(), base_url));

            // Extract src for images (resolve relative URLs)
            let src = attrs
                .iter()
                .find(|a| a.name.local.as_ref() == "src")
                .map(|a| resolve_url(&a.value.to_string(), base_url));

            // Extract inline style
            let style_attr = attrs
                .iter()
                .find(|a| a.name.local.as_ref() == "style")
                .map(|a| a.value.to_string());

            // Helper to apply inline styles
            let apply_inline = |elem: &mut RenderElement| {
                if let Some(ref style_str) = style_attr {
                    parse_inline_style(style_str, &mut elem.style);
                }
            };

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
                        let mut elem = create_styled_element(kind, text, *y, indent);
                        apply_inline(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                }
                "p" | "div" | "span" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Paragraph, text, *y, indent);
                        apply_inline(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                }
                "strong" | "b" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Text, text, *y, indent);
                        elem.style.font_weight_bold = true;
                        apply_inline(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "em" | "i" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Text, text, *y, indent);
                        elem.style.font_style_italic = true;
                        apply_inline(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "a" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Link, text, *y, indent);
                        elem.is_link = true;
                        elem.href = href;
                        apply_inline(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return; // Don't recurse into links
                }
                "li" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(
                            ElementKind::ListItem,
                            format!("• {}", text),
                            *y,
                            indent + 1,
                        );
                        apply_inline(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "code" | "pre" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Code, text, *y, indent);
                        apply_inline(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "blockquote" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Blockquote, text, *y, indent);
                        apply_inline(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "img" => {
                    // Extract image attributes
                    let alt = attrs
                        .iter()
                        .find(|a| a.name.local.as_ref() == "alt")
                        .map(|a| a.value.to_string())
                        .unwrap_or_else(|| "[Image]".to_string());
                    // src is already resolved above
                    let mut elem = create_styled_element(ElementKind::Image, format!("🖼 {}", alt.clone()), *y, indent);
                    elem.src = src.clone();
                    elem.alt = Some(alt);
                    apply_inline(&mut elem);
                    elements.push(elem);
                    *y += 1.0;
                    return;
                }
                "table" => {
                    // Add table marker
                    let mut elem = create_styled_element(ElementKind::Table, "".to_string(), *y, indent);
                    apply_inline(&mut elem);
                    elements.push(elem);
                    // Recurse into table with increased indent
                    for child in handle.children.borrow().iter() {
                        extract_content(child, elements, title, y, indent + 1, base_url);
                    }
                    return;
                }
                "tr" => {
                    // Table row - recurse into cells
                    for child in handle.children.borrow().iter() {
                        extract_content(child, elements, title, y, indent, base_url);
                    }
                    *y += 1.0;
                    return;
                }
                "td" | "th" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::TableCell, text, *y, indent);
                        if tag == "th" {
                            elem.style.font_weight_bold = true;
                        }
                        apply_inline(&mut elem);
                        elements.push(elem);
                    }
                    return;
                }
                "button" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Button, text, *y, indent);
                        apply_inline(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "input" => {
                    let input_type = attrs
                        .iter()
                        .find(|a| a.name.local.as_ref() == "type")
                        .map(|a| a.value.to_string())
                        .unwrap_or_else(|| "text".to_string());
                    let placeholder = attrs
                        .iter()
                        .find(|a| a.name.local.as_ref() == "placeholder")
                        .map(|a| a.value.to_string())
                        .unwrap_or_default();
                    let value = attrs
                        .iter()
                        .find(|a| a.name.local.as_ref() == "value")
                        .map(|a| a.value.to_string())
                        .unwrap_or_default();

                    let display = if !value.is_empty() {
                        value
                    } else if !placeholder.is_empty() {
                        format!("[{}]", placeholder)
                    } else {
                        format!("[{} input]", input_type)
                    };

                    let mut elem = create_styled_element(ElementKind::Input, display, *y, indent);
                    apply_inline(&mut elem);
                    elements.push(elem);
                    *y += 1.0;
                    return;
                }
                "label" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Label, text, *y, indent);
                        apply_inline(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "script" | "style" | "noscript" | "meta" | "link" => {
                    return; // Skip these elements completely
                }
                "head" => {
                    // Only extract title from head, skip rendering other content
                    for child in handle.children.borrow().iter() {
                        extract_content(child, elements, title, y, indent, base_url);
                    }
                    return;
                }
                "br" => {
                    *y += 0.5; // Half line break
                }
                "hr" => {
                    let elem = create_styled_element(ElementKind::HorizontalRule, "───────────────────────────────".to_string(), *y, indent);
                    elements.push(elem);
                    *y += 1.0;
                }
                _ => {}
            }

            // Recurse into children
            for child in handle.children.borrow().iter() {
                extract_content(child, elements, title, y, indent, base_url);
            }
        }
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            let text = text.trim();
            if !text.is_empty() && text.len() > 1 {
                elements.push(create_styled_element(ElementKind::Text, text.to_string(), *y, indent));
                *y += 1.0;
            }
        }
        NodeData::Document => {
            for child in handle.children.borrow().iter() {
                extract_content(child, elements, title, y, indent, base_url);
            }
        }
        _ => {}
    }
}

/// Extract content with CSS rules applied
fn extract_content_with_css(
    handle: &Handle,
    elements: &mut Vec<RenderElement>,
    title: &mut String,
    y: &mut f32,
    indent: u32,
    base_url: &str,
    css_rules: &[CssRule],
) {
    use markup5ever_rcdom::NodeData;

    match &handle.data {
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.as_ref();
            let attrs_ref = attrs.borrow();

            // Extract id and classes for CSS matching
            let id = attrs_ref
                .iter()
                .find(|a| a.name.local.as_ref() == "id")
                .map(|a| a.value.to_string());

            let classes: Vec<String> = attrs_ref
                .iter()
                .find(|a| a.name.local.as_ref() == "class")
                .map(|a| a.value.to_string())
                .unwrap_or_default()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            // Extract href for links (resolve relative URLs)
            let href = attrs_ref
                .iter()
                .find(|a| a.name.local.as_ref() == "href")
                .map(|a| resolve_url(&a.value.to_string(), base_url));

            // Extract src for images (resolve relative URLs)
            let src = attrs_ref
                .iter()
                .find(|a| a.name.local.as_ref() == "src")
                .map(|a| resolve_url(&a.value.to_string(), base_url));

            // Extract inline style
            let style_attr = attrs_ref
                .iter()
                .find(|a| a.name.local.as_ref() == "style")
                .map(|a| a.value.to_string());

            // Find matching CSS rules and collect properties
            let mut matched_props: Vec<((u32, u32, u32), &std::collections::HashMap<String, String>)> = Vec::new();
            for rule in css_rules {
                if rule.selector.matches(tag, id.as_deref(), &classes) {
                    matched_props.push((rule.selector.specificity(), &rule.properties));
                }
            }
            // Sort by specificity
            matched_props.sort_by_key(|(spec, _)| *spec);

            // Helper to apply CSS rules then inline styles
            let apply_styles = |elem: &mut RenderElement| {
                // Apply matched CSS rules (lower specificity first)
                for (_, props) in &matched_props {
                    for (prop, value) in *props {
                        apply_css_property(&prop, &value, &mut elem.style);
                    }
                }
                // Apply inline style (highest priority)
                if let Some(ref style_str) = style_attr {
                    parse_inline_style(style_str, &mut elem.style);
                }
            };

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
                        let mut elem = create_styled_element(kind, text, *y, indent);
                        apply_styles(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                }
                "div" | "section" | "article" | "main" | "header" | "footer" | "nav" => {
                    // Container elements - always recurse into children
                    for child in handle.children.borrow().iter() {
                        extract_content_with_css(child, elements, title, y, indent, base_url, css_rules);
                    }
                    return;
                }
                "p" | "span" => {
                    // Check if contains links or block elements - recurse into them
                    let has_special_children = handle.children.borrow().iter().any(|child| {
                        if let NodeData::Element { name, .. } = &child.data {
                            matches!(name.local.as_ref(),
                                "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" |
                                "ul" | "ol" | "table" | "section" | "a" | "code" | "pre" |
                                "strong" | "b" | "em" | "i" | "img")
                        } else {
                            false
                        }
                    });

                    if has_special_children {
                        for child in handle.children.borrow().iter() {
                            extract_content_with_css(child, elements, title, y, indent, base_url, css_rules);
                        }
                    } else {
                        let text = extract_text(handle);
                        if !text.is_empty() {
                            let mut elem = create_styled_element(ElementKind::Paragraph, text, *y, indent);
                            apply_styles(&mut elem);
                            elements.push(elem);
                            *y += 1.0;
                        }
                    }
                    return;
                }
                "a" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Link, text, *y, indent);
                        elem.is_link = true;
                        elem.href = href;
                        apply_styles(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "strong" | "b" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Text, text, *y, indent);
                        elem.style.font_weight_bold = true;
                        apply_styles(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "em" | "i" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Text, text, *y, indent);
                        elem.style.font_style_italic = true;
                        apply_styles(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "ul" | "ol" => {
                    for child in handle.children.borrow().iter() {
                        extract_content_with_css(child, elements, title, y, indent + 1, base_url, css_rules);
                    }
                    return;
                }
                "li" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::ListItem, format!("• {}", text), *y, indent);
                        apply_styles(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "code" | "pre" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Code, text, *y, indent);
                        apply_styles(&mut elem);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "img" => {
                    let alt = attrs_ref
                        .iter()
                        .find(|a| a.name.local.as_ref() == "alt")
                        .map(|a| a.value.to_string())
                        .unwrap_or_else(|| "[Image]".to_string());
                    let mut elem = create_styled_element(ElementKind::Image, format!("🖼 {}", alt.clone()), *y, indent);
                    elem.src = src.clone();
                    elem.alt = Some(alt);
                    apply_styles(&mut elem);
                    elements.push(elem);
                    *y += 1.0;
                    return;
                }
                "script" | "style" | "noscript" | "meta" | "link" => {
                    return;
                }
                "head" => {
                    for child in handle.children.borrow().iter() {
                        extract_content_with_css(child, elements, title, y, indent, base_url, css_rules);
                    }
                    return;
                }
                "br" => {
                    *y += 0.5;
                }
                "hr" => {
                    let elem = create_styled_element(ElementKind::HorizontalRule, "───────────────────────────────".to_string(), *y, indent);
                    elements.push(elem);
                    *y += 1.0;
                }
                _ => {}
            }

            // Recurse into children
            for child in handle.children.borrow().iter() {
                extract_content_with_css(child, elements, title, y, indent, base_url, css_rules);
            }
        }
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            let text = text.trim();
            if !text.is_empty() && text.len() > 1 {
                elements.push(create_styled_element(ElementKind::Text, text.to_string(), *y, indent));
                *y += 1.0;
            }
        }
        NodeData::Document => {
            for child in handle.children.borrow().iter() {
                extract_content_with_css(child, elements, title, y, indent, base_url, css_rules);
            }
        }
        _ => {}
    }
}

/// Apply a single CSS property to ElementStyle
fn apply_css_property(property: &str, value: &str, style: &mut ElementStyle) {
    match property {
        "color" => {
            if let Some(color) = parse_css_color(value) {
                style.color = color;
            }
        }
        "background-color" | "background" => {
            if let Some(color) = parse_css_color(value) {
                style.background_color = Some(color);
            }
        }
        "font-size" => {
            if let Some(size) = parse_css_size(value) {
                style.font_size = size;
            }
        }
        "font-weight" => {
            style.font_weight_bold = value == "bold" || value == "700" || value == "800" || value == "900";
        }
        "font-style" => {
            style.font_style_italic = value == "italic" || value == "oblique";
        }
        "text-decoration" => {
            style.text_decoration_underline = value.contains("underline");
        }
        "text-align" => {
            style.text_align = match value.to_lowercase().as_str() {
                "center" => TextAlign::Center,
                "right" => TextAlign::Right,
                "justify" => TextAlign::Justify,
                _ => TextAlign::Left,
            };
        }
        "line-height" => {
            if let Some(size) = parse_css_size(value) {
                style.line_height = size / style.font_size;
            } else if let Ok(multiplier) = value.parse::<f32>() {
                style.line_height = multiplier;
            }
        }
        "max-width" => {
            if let Some(size) = parse_css_size(value) {
                style.max_width = size;
            }
        }
        "width" => {
            if let Some(size) = parse_css_size(value) {
                style.max_width = size;
            }
        }
        "padding" => {
            if let Some(size) = parse_css_size(value) {
                style.padding = [size; 4];
            }
        }
        "padding-top" => {
            if let Some(size) = parse_css_size(value) {
                style.padding[0] = size;
            }
        }
        "padding-right" => {
            if let Some(size) = parse_css_size(value) {
                style.padding[1] = size;
            }
        }
        "padding-bottom" => {
            if let Some(size) = parse_css_size(value) {
                style.padding[2] = size;
            }
        }
        "padding-left" => {
            if let Some(size) = parse_css_size(value) {
                style.padding[3] = size;
            }
        }
        "margin" => {
            if let Some(size) = parse_css_size(value) {
                style.margin = [size; 4];
            }
        }
        "margin-top" => {
            if let Some(size) = parse_css_size(value) {
                style.margin[0] = size;
            }
        }
        "margin-right" => {
            if let Some(size) = parse_css_size(value) {
                style.margin[1] = size;
            }
        }
        "margin-bottom" => {
            if let Some(size) = parse_css_size(value) {
                style.margin[2] = size;
            }
        }
        "margin-left" => {
            if let Some(size) = parse_css_size(value) {
                style.margin[3] = size;
            }
        }
        "border-radius" => {
            if let Some(size) = parse_css_size(value) {
                style.border_radius = size;
            }
        }
        "border" | "border-color" => {
            if let Some(color) = parse_css_color(value) {
                style.border_color = Some(color);
                if style.border_width == [0.0; 4] {
                    style.border_width = [1.0; 4];
                }
            }
        }
        "border-width" => {
            if let Some(size) = parse_css_size(value) {
                style.border_width = [size; 4];
            }
        }
        "opacity" => {
            if let Ok(opacity) = value.parse::<f32>() {
                style.color[3] = (opacity * 255.0) as u8;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_style_default() {
        let style = ElementStyle::default();
        assert_eq!(style.color, [0, 0, 0, 255]); // Black
        assert_eq!(style.font_size, 16.0);
        assert!(!style.font_weight_bold);
        assert!(!style.font_style_italic);
        assert!(!style.text_decoration_underline);
    }

    #[test]
    fn test_render_element_new() {
        let elem = RenderElement::new(ElementKind::Paragraph, "Test text".to_string());
        assert_eq!(elem.text, "Test text");
        assert_eq!(elem.kind, ElementKind::Paragraph);
        assert!(!elem.is_link);
        assert!(elem.href.is_none());
        assert!(elem.children.is_empty());
    }

    #[test]
    fn test_create_styled_heading() {
        let elem = create_styled_element(ElementKind::Heading1, "Title".to_string(), 0.0, 0);
        assert_eq!(elem.style.font_size, 32.0);
        assert!(elem.style.font_weight_bold);
        assert_eq!(elem.style.margin[0], 24.0); // Top margin
    }

    #[test]
    fn test_create_styled_link() {
        let elem = create_styled_element(ElementKind::Link, "Click me".to_string(), 1.0, 0);
        assert_eq!(elem.style.color, [0, 102, 204, 255]); // Blue
        assert!(elem.style.text_decoration_underline);
    }

    #[test]
    fn test_create_styled_code() {
        let elem = create_styled_element(ElementKind::Code, "let x = 1;".to_string(), 0.0, 0);
        assert_eq!(elem.style.font_size, 14.0);
        assert!(elem.style.background_color.is_some());
    }

    #[test]
    fn test_element_bounds() {
        let elem = create_styled_element(ElementKind::Paragraph, "Text".to_string(), 2.0, 1);
        assert_eq!(elem.bounds.x, 20.0); // indent * 20
        assert_eq!(elem.bounds.y, 48.0); // y * 24 (line height)
    }

    #[test]
    fn test_element_legacy_accessors() {
        let elem = create_styled_element(ElementKind::Text, "Test".to_string(), 3.0, 2);
        assert_eq!(elem.y(), 72.0); // 3.0 * 24
        assert_eq!(elem.indent(), 2); // 40.0 / 20.0
    }

    #[test]
    fn test_parse_css_color_named() {
        assert_eq!(parse_css_color("black"), Some([0, 0, 0, 255]));
        assert_eq!(parse_css_color("white"), Some([255, 255, 255, 255]));
        assert_eq!(parse_css_color("red"), Some([255, 0, 0, 255]));
        assert_eq!(parse_css_color("Blue"), Some([0, 0, 255, 255])); // Case insensitive
    }

    #[test]
    fn test_parse_css_color_hex() {
        assert_eq!(parse_css_color("#fff"), Some([255, 255, 255, 255]));
        assert_eq!(parse_css_color("#000"), Some([0, 0, 0, 255]));
        assert_eq!(parse_css_color("#ff0000"), Some([255, 0, 0, 255]));
        assert_eq!(parse_css_color("#00ff00"), Some([0, 255, 0, 255]));
    }

    #[test]
    fn test_parse_css_color_rgb() {
        assert_eq!(parse_css_color("rgb(255, 0, 0)"), Some([255, 0, 0, 255]));
        assert_eq!(parse_css_color("rgba(0, 255, 0, 0.5)"), Some([0, 255, 0, 127]));
    }

    #[test]
    fn test_parse_css_size() {
        assert_eq!(parse_css_size("16px"), Some(16.0));
        assert_eq!(parse_css_size("1.5em"), Some(24.0));
        assert_eq!(parse_css_size("2rem"), Some(32.0));
        assert_eq!(parse_css_size("100%"), Some(16.0));
    }

    #[test]
    fn test_parse_inline_style() {
        let mut style = ElementStyle::default();
        parse_inline_style("color: red; font-size: 24px; font-weight: bold", &mut style);

        assert_eq!(style.color, [255, 0, 0, 255]);
        assert_eq!(style.font_size, 24.0);
        assert!(style.font_weight_bold);
    }

    #[test]
    fn test_parse_inline_style_background() {
        let mut style = ElementStyle::default();
        parse_inline_style("background-color: #f0f0f0; text-decoration: underline", &mut style);

        assert_eq!(style.background_color, Some([240, 240, 240, 255]));
        assert!(style.text_decoration_underline);
    }

    #[test]
    fn test_parse_example_com() {
        let html = r#"<!doctype html><html lang="en"><head><title>Example Domain</title></head><body><div><h1>Example Domain</h1><p>This domain is for use in documentation examples.</p><p><a href="https://iana.org/domains/example">Learn more</a></p></div></body></html>"#;

        let content = parse_html_to_content(html, "https://example.com");

        assert_eq!(content.title, "Example Domain");
        assert!(content.error.is_none());
        assert!(!content.elements.is_empty(), "Should have parsed elements");

        // Check for heading
        let h1 = content.elements.iter().find(|e| matches!(e.kind, ElementKind::Heading1));
        assert!(h1.is_some(), "Should have h1 element");
        assert_eq!(h1.unwrap().text, "Example Domain");

        // Check for paragraph
        let p = content.elements.iter().find(|e| matches!(e.kind, ElementKind::Paragraph));
        assert!(p.is_some(), "Should have paragraph element");

        // Check for link
        let link = content.elements.iter().find(|e| matches!(e.kind, ElementKind::Link));
        assert!(link.is_some(), "Should have link element");
        assert_eq!(link.unwrap().text, "Learn more");
        assert_eq!(link.unwrap().href.as_deref(), Some("https://iana.org/domains/example"));
    }

    #[test]
    fn test_parse_list_items() {
        let html = r#"<html><body><ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul></body></html>"#;
        let content = parse_html_to_content(html, "");

        let items: Vec<_> = content.elements.iter()
            .filter(|e| matches!(e.kind, ElementKind::ListItem))
            .collect();
        assert_eq!(items.len(), 3, "Should have 3 list items");
        assert!(items[0].text.contains("Item 1"));
        assert!(items[1].text.contains("Item 2"));
        assert!(items[2].text.contains("Item 3"));
    }

    #[test]
    fn test_parse_multiple_headings() {
        let html = r#"<html><body><h1>Title</h1><h2>Subtitle</h2><h3>Section</h3></body></html>"#;
        let content = parse_html_to_content(html, "");

        let h1 = content.elements.iter().find(|e| matches!(e.kind, ElementKind::Heading1));
        let h2 = content.elements.iter().find(|e| matches!(e.kind, ElementKind::Heading2));
        let h3 = content.elements.iter().find(|e| matches!(e.kind, ElementKind::Heading3));

        assert!(h1.is_some(), "Should have h1");
        assert!(h2.is_some(), "Should have h2");
        assert!(h3.is_some(), "Should have h3");

        assert_eq!(h1.unwrap().text, "Title");
        assert_eq!(h2.unwrap().text, "Subtitle");
        assert_eq!(h3.unwrap().text, "Section");
    }

    #[test]
    fn test_parse_code_blocks() {
        let html = r#"<html><body><code>let x = 42;</code><pre>function test() {}</pre></body></html>"#;
        let content = parse_html_to_content(html, "");

        let code: Vec<_> = content.elements.iter()
            .filter(|e| matches!(e.kind, ElementKind::Code))
            .collect();
        assert_eq!(code.len(), 2, "Should have 2 code elements");
    }

    #[test]
    fn test_parse_image() {
        let html = r#"<html><body><img src="test.png" alt="Test Image" width="100" height="50"></body></html>"#;
        let content = parse_html_to_content(html, "https://example.com");

        let img = content.elements.iter().find(|e| matches!(e.kind, ElementKind::Image));
        assert!(img.is_some(), "Should have image element");
        let img = img.unwrap();
        // URL is resolved relative to base
        assert_eq!(img.src.as_deref(), Some("https://example.com/test.png"));
        assert_eq!(img.alt.as_deref(), Some("Test Image"));
    }

    #[test]
    fn test_resolve_url() {
        // Absolute URLs
        assert_eq!(resolve_url("https://other.com/page", "https://example.com"), "https://other.com/page");

        // Relative to root
        assert_eq!(resolve_url("/images/logo.png", "https://example.com/page"), "https://example.com/images/logo.png");

        // Relative to current path
        assert_eq!(resolve_url("image.png", "https://example.com/path/"), "https://example.com/path/image.png");

        // Protocol-relative
        assert_eq!(resolve_url("//cdn.example.com/lib.js", "https://example.com"), "https://cdn.example.com/lib.js");
    }

    #[test]
    #[ignore] // Integration test - requires network
    fn test_fetch_example_com() {
        let content = fetch_and_parse("https://example.com");

        assert!(content.error.is_none(), "Should not have error");
        assert_eq!(content.title, "Example Domain");
        assert!(!content.elements.is_empty(), "Should have elements");

        // Check for h1
        let has_h1 = content.elements.iter().any(|e| matches!(e.kind, ElementKind::Heading1));
        assert!(has_h1, "Should have h1");

        // Check for link
        let has_link = content.elements.iter().any(|e| matches!(e.kind, ElementKind::Link));
        assert!(has_link, "Should have link");
    }

    #[test]
    fn test_css_rules_parsing() {
        let css = "h1 { color: red; font-size: 24px; } .title { font-weight: bold; }";
        let rules = parse_css_rules(css);

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].selector.tag, Some("h1".to_string()));
        assert_eq!(rules[0].properties.get("color"), Some(&"red".to_string()));
        assert_eq!(rules[0].properties.get("font-size"), Some(&"24px".to_string()));

        assert!(rules[1].selector.classes.contains(&"title".to_string()));
        assert_eq!(rules[1].properties.get("font-weight"), Some(&"bold".to_string()));
    }

    #[test]
    fn test_css_selector_matching() {
        let selector = parse_css_selector("div.container#main");
        assert_eq!(selector.tag, Some("div".to_string()));
        assert_eq!(selector.id, Some("main".to_string()));
        assert!(selector.classes.contains(&"container".to_string()));

        // Test matching
        assert!(selector.matches("div", Some("main"), &["container".to_string()]));
        assert!(!selector.matches("div", Some("other"), &["container".to_string()]));
        assert!(!selector.matches("span", Some("main"), &["container".to_string()]));
    }

    #[test]
    fn test_css_styles_applied() {
        let html = r#"
            <html>
            <head>
                <style>
                    h1 { color: red; }
                    .highlight { background-color: yellow; }
                </style>
            </head>
            <body>
                <h1>Title</h1>
                <p class="highlight">Highlighted text</p>
            </body>
            </html>
        "#;
        let content = parse_html_to_content(html, "");

        // Find h1 element
        let h1 = content.elements.iter().find(|e| matches!(e.kind, ElementKind::Heading1));
        assert!(h1.is_some(), "Should have h1");
        let h1 = h1.unwrap();
        // Red color should be applied
        assert_eq!(h1.style.color, [255, 0, 0, 255], "h1 should have red color");

        // Find highlighted paragraph
        let p = content.elements.iter().find(|e| matches!(e.kind, ElementKind::Paragraph));
        assert!(p.is_some(), "Should have paragraph");
        let p = p.unwrap();
        // Yellow background should be applied
        assert_eq!(p.style.background_color, Some([255, 255, 0, 255]), "p should have yellow background");
    }
}
