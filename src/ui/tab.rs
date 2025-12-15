//! Tab management

use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use crate::js_engine::JsRuntime;
use markup5ever_rcdom::Handle;

/// User-Agent string that mimics a real browser for better site compatibility
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

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
    /// Console output from JavaScript
    pub console_output: Vec<String>,
    /// JavaScript errors
    pub js_errors: Vec<String>,
}

impl Default for PageContent {
    fn default() -> Self {
        Self {
            title: "New Tab".to_string(),
            elements: Vec::new(),
            error: None,
            console_output: Vec::new(),
            js_errors: Vec::new(),
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
    /// Form element attributes
    pub form_attrs: Option<FormAttributes>,
    /// Is this an inline element? (should flow horizontally)
    pub is_inline: bool,
}

/// Form element attributes
#[derive(Debug, Clone, Default)]
pub struct FormAttributes {
    /// Input type (text, password, submit, etc.)
    pub input_type: String,
    /// Element name attribute
    pub name: String,
    /// Element id attribute
    pub id: String,
    /// Placeholder text
    pub placeholder: String,
    /// Current value
    pub value: String,
    /// Is the element disabled?
    pub disabled: bool,
    /// Is the element checked? (checkbox/radio)
    pub checked: bool,
    /// Options for select elements
    pub options: Vec<(String, String)>, // (value, label)
    /// Form action URL (from parent form)
    pub form_action: Option<String>,
    /// Form method (GET/POST)
    pub form_method: String,
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

/// Display mode for CSS display property
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DisplayMode {
    #[default]
    Block,
    Inline,
    InlineBlock,
    Flex,
    Grid,
    None,
}

/// CSS position property
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

/// Flex direction for flexbox layout
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Flex wrap mode
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
    /// Flex direction
    pub direction: FlexDirection,
    /// Flex wrap
    pub wrap: FlexWrap,
    /// Justify content (main axis)
    pub justify_content: JustifyContent,
    /// Align items (cross axis)
    pub align_items: AlignItems,
    /// Gap between items
    pub gap: f32,
    /// Flex grow factor (for children)
    pub flex_grow: f32,
    /// Flex shrink factor (for children)
    pub flex_shrink: f32,
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
    /// Display mode (block, inline, none, etc.)
    pub display: DisplayMode,
    /// Visibility (hidden elements still take space)
    pub visible: bool,
    /// Flexbox properties (when display: flex)
    pub flex: FlexProperties,
    /// CSS position property
    pub position: Position,
    /// Top offset (for positioned elements)
    pub top: Option<f32>,
    /// Right offset (for positioned elements)
    pub right: Option<f32>,
    /// Bottom offset (for positioned elements)
    pub bottom: Option<f32>,
    /// Left offset (for positioned elements)
    pub left: Option<f32>,
    /// Z-index for stacking order
    pub z_index: i32,
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
            max_width: 0.0,
            display: DisplayMode::Block,
            visible: true,
            flex: FlexProperties::default(),
            position: Position::Static,
            top: None,
            right: None,
            bottom: None,
            left: None,
            z_index: 0,
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
            form_attrs: None,
            is_inline: false,
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
            form_attrs: None,
            is_inline: false,
        }
    }

    /// Create a form input element
    pub fn input(input_type: &str, name: &str, placeholder: &str, value: &str) -> Self {
        let kind = match input_type {
            "submit" | "button" => ElementKind::Button,
            "checkbox" | "radio" => ElementKind::Input,
            _ => ElementKind::Input,
        };
        Self {
            kind,
            text: value.to_string(),
            bounds: ElementBounds::default(),
            style: ElementStyle::default(),
            is_link: false,
            href: None,
            src: None,
            alt: None,
            children: Vec::new(),
            form_attrs: Some(FormAttributes {
                input_type: input_type.to_string(),
                name: name.to_string(),
                id: String::new(),
                placeholder: placeholder.to_string(),
                value: value.to_string(),
                disabled: false,
                checked: false,
                options: Vec::new(),
                form_action: None,
                form_method: String::new(),
            }),
            is_inline: true,
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
    Textarea,
    Select,
    Checkbox,
    Radio,
    Label,
    Form,
    // Layout containers
    Container,
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
    /// Navigation history (past URLs)
    history_back: Vec<String>,
    /// Forward history (URLs we went back from)
    history_forward: Vec<String>,
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
            history_back: Vec::new(),
            history_forward: Vec::new(),
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

        // Normalize URL - don't add https:// if already has a protocol
        let normalized_url = if url_str.starts_with("http://")
            || url_str.starts_with("https://")
            || url_str.starts_with("file://")
        {
            url_str
        } else {
            format!("https://{}", url_str)
        };

        // Save current URL to history if we have one
        if !self.url.is_empty() {
            self.history_back.push(self.url.clone());
            // Clear forward history when navigating to a new page
            self.history_forward.clear();
        }

        self.url = normalized_url.clone();
        self.loading = true;

        // Create channel for async content loading
        let (tx, rx) = channel::<PageContent>();
        self.content_receiver = Some(rx);

        // Spawn async task to fetch and parse the page
        let _content = Arc::clone(&self.content);
        std::thread::spawn(move || {
            eprintln!("[DEBUG] Starting fetch for: {}", normalized_url);
            let result = fetch_and_parse(&normalized_url);
            eprintln!("[DEBUG] Fetch complete. Title: '{}', Elements: {}, Error: {:?}",
                      result.title, result.elements.len(), result.error);
            if let Err(e) = tx.send(result) {
                eprintln!("[DEBUG] Failed to send result: {}", e);
            }
        });
    }

    /// Check if we can go back in history
    pub fn can_go_back(&self) -> bool {
        !self.history_back.is_empty()
    }

    /// Check if we can go forward in history
    pub fn can_go_forward(&self) -> bool {
        !self.history_forward.is_empty()
    }

    /// Go back in history
    pub fn go_back(&mut self) {
        if let Some(prev_url) = self.history_back.pop() {
            // Save current URL to forward history
            if !self.url.is_empty() {
                self.history_forward.push(self.url.clone());
            }

            // Navigate without adding to back history
            self.navigate_internal(&prev_url);
        }
    }

    /// Go forward in history
    pub fn go_forward(&mut self) {
        if let Some(next_url) = self.history_forward.pop() {
            // Save current URL to back history
            if !self.url.is_empty() {
                self.history_back.push(self.url.clone());
            }

            // Navigate without adding to back history
            self.navigate_internal(&next_url);
        }
    }

    /// Reload the current page
    pub fn reload(&mut self) {
        if !self.url.is_empty() {
            let url = self.url.clone();
            self.navigate_internal(&url);
        }
    }

    /// Internal navigation that doesn't modify history
    fn navigate_internal(&mut self, url: &str) {
        self.url = url.to_string();
        self.loading = true;

        let (tx, rx) = channel::<PageContent>();
        self.content_receiver = Some(rx);

        let normalized_url = url.to_string();
        std::thread::spawn(move || {
            eprintln!("[DEBUG] Starting fetch for: {}", normalized_url);
            let result = fetch_and_parse(&normalized_url);
            eprintln!("[DEBUG] Fetch complete. Title: '{}', Elements: {}, Error: {:?}",
                      result.title, result.elements.len(), result.error);
            if let Err(e) = tx.send(result) {
                eprintln!("[DEBUG] Failed to send result: {}", e);
            }
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
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return PageContent {
                title: "Error".to_string(),
                elements: vec![],
                error: Some(format!("Failed to create client: {}", e)),
                console_output: Vec::new(),
                js_errors: Vec::new(),
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
                console_output: Vec::new(),
                js_errors: Vec::new(),
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
                console_output: Vec::new(),
                js_errors: Vec::new(),
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
            console_output: Vec::new(),
            js_errors: Vec::new(),
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

/// Combinator types for CSS selectors
#[derive(Debug, Clone, PartialEq)]
enum Combinator {
    /// No combinator (single selector)
    None,
    /// Descendant selector (space): div p
    Descendant,
    /// Child selector (>): ul > li
    Child,
    /// Adjacent sibling (+): h1 + p
    AdjacentSibling,
    /// General sibling (~): h1 ~ p
    GeneralSibling,
}

/// Pseudo-class types
#[derive(Debug, Clone, PartialEq)]
enum PseudoClass {
    Hover,
    Focus,
    Active,
    Visited,
    Link,
    FirstChild,
    LastChild,
    NthChild(i32),      // :nth-child(n)
    NthLastChild(i32),  // :nth-last-child(n)
    FirstOfType,
    LastOfType,
    Empty,
    Not(Box<SimpleSelector>),
}

/// A simple selector part (tag, id, classes, pseudo-classes)
#[derive(Debug, Clone, PartialEq)]
struct SimpleSelector {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    pseudo_classes: Vec<PseudoClass>,
}

impl SimpleSelector {
    fn new() -> Self {
        Self { tag: None, id: None, classes: Vec::new(), pseudo_classes: Vec::new() }
    }

    fn is_empty(&self) -> bool {
        self.tag.is_none() && self.id.is_none() && self.classes.is_empty() && self.pseudo_classes.is_empty()
    }

    /// Check if this simple selector matches an element
    /// Additional context for pseudo-classes
    fn matches_with_context(
        &self,
        tag_name: &str,
        id: Option<&str>,
        classes: &[String],
        is_first_child: bool,
        is_last_child: bool,
        child_index: usize,
        sibling_count: usize,
        has_children: bool,
    ) -> bool {
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

        // Check pseudo-classes
        for pseudo in &self.pseudo_classes {
            match pseudo {
                PseudoClass::FirstChild => {
                    if !is_first_child { return false; }
                }
                PseudoClass::LastChild => {
                    if !is_last_child { return false; }
                }
                PseudoClass::NthChild(n) => {
                    if child_index as i32 + 1 != *n { return false; }
                }
                PseudoClass::NthLastChild(n) => {
                    let from_end = sibling_count as i32 - child_index as i32;
                    if from_end != *n { return false; }
                }
                PseudoClass::Empty => {
                    if has_children { return false; }
                }
                // Interactive pseudo-classes (hover, focus, etc.) need runtime state
                // For static rendering, we skip them
                PseudoClass::Hover | PseudoClass::Focus | PseudoClass::Active => {
                    // These require interactive state - skip for now
                    return false;
                }
                PseudoClass::Link => {
                    // Link matches unvisited <a> elements
                    if tag_name.to_lowercase() != "a" { return false; }
                }
                PseudoClass::Visited => {
                    // We don't track visited state
                    return false;
                }
                _ => {}
            }
        }

        true
    }

    /// Simple match without pseudo-class context (for backward compatibility)
    fn matches(&self, tag_name: &str, id: Option<&str>, classes: &[String]) -> bool {
        // If no pseudo-classes, do simple match
        if self.pseudo_classes.is_empty() {
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
        } else {
            // Use default context for pseudo-classes
            self.matches_with_context(tag_name, id, classes, false, false, 0, 1, true)
        }
    }

    /// Calculate specificity contribution
    fn specificity(&self) -> (u32, u32, u32) {
        (
            if self.id.is_some() { 1 } else { 0 },
            self.classes.len() as u32 + self.pseudo_classes.len() as u32,
            if self.tag.is_some() { 1 } else { 0 },
        )
    }
}

/// A compound selector with optional combinators
#[derive(Debug, Clone)]
struct CssSelector {
    /// Parts of the selector chain
    parts: Vec<(Combinator, SimpleSelector)>,
}

impl CssSelector {
    /// Calculate total specificity (id, class, tag)
    fn specificity(&self) -> (u32, u32, u32) {
        let mut total = (0u32, 0u32, 0u32);
        for (_, part) in &self.parts {
            let s = part.specificity();
            total.0 += s.0;
            total.1 += s.1;
            total.2 += s.2;
        }
        total
    }

    /// Check if selector matches element (simplified - only checks the last part for now)
    /// Full combinator support requires DOM tree context
    fn matches(&self, tag_name: &str, id: Option<&str>, classes: &[String]) -> bool {
        // For simple selectors, just check the last part
        if let Some((combinator, last_part)) = self.parts.last() {
            if *combinator == Combinator::None || self.parts.len() == 1 {
                return last_part.matches(tag_name, id, classes);
            }
            // For compound selectors, we still match the last part
            // Full combinator matching requires ancestry info (future enhancement)
            last_part.matches(tag_name, id, classes)
        } else {
            false
        }
    }
}

/// Parse a pseudo-class string into PseudoClass enum
fn parse_pseudo_class(s: &str) -> Option<PseudoClass> {
    let s = s.to_lowercase();
    match s.as_str() {
        "hover" => Some(PseudoClass::Hover),
        "focus" => Some(PseudoClass::Focus),
        "active" => Some(PseudoClass::Active),
        "visited" => Some(PseudoClass::Visited),
        "link" => Some(PseudoClass::Link),
        "first-child" => Some(PseudoClass::FirstChild),
        "last-child" => Some(PseudoClass::LastChild),
        "first-of-type" => Some(PseudoClass::FirstOfType),
        "last-of-type" => Some(PseudoClass::LastOfType),
        "empty" => Some(PseudoClass::Empty),
        _ if s.starts_with("nth-child(") && s.ends_with(')') => {
            let inner = &s[10..s.len()-1];
            inner.trim().parse::<i32>().ok().map(PseudoClass::NthChild)
        }
        _ if s.starts_with("nth-last-child(") && s.ends_with(')') => {
            let inner = &s[15..s.len()-1];
            inner.trim().parse::<i32>().ok().map(PseudoClass::NthLastChild)
        }
        _ => None,
    }
}

/// Parse a simple selector part (tag#id.class1.class2:pseudo)
fn parse_simple_selector(s: &str) -> SimpleSelector {
    let s = s.trim();
    let mut selector = SimpleSelector::new();

    if s.is_empty() {
        return selector;
    }

    let mut current = String::new();
    let mut mode = 't'; // t=tag, #=id, .=class, :=pseudo

    for ch in s.chars() {
        match ch {
            '#' => {
                if !current.is_empty() {
                    match mode {
                        't' => selector.tag = Some(current.clone()),
                        '.' => selector.classes.push(current.clone()),
                        '#' => selector.id = Some(current.clone()),
                        ':' => {
                            if let Some(pseudo) = parse_pseudo_class(&current) {
                                selector.pseudo_classes.push(pseudo);
                            }
                        }
                        _ => {}
                    }
                    current.clear();
                }
                mode = '#';
            }
            '.' => {
                if !current.is_empty() {
                    match mode {
                        't' => selector.tag = Some(current.clone()),
                        '.' => selector.classes.push(current.clone()),
                        '#' => selector.id = Some(current.clone()),
                        ':' => {
                            if let Some(pseudo) = parse_pseudo_class(&current) {
                                selector.pseudo_classes.push(pseudo);
                            }
                        }
                        _ => {}
                    }
                    current.clear();
                }
                mode = '.';
            }
            ':' => {
                if !current.is_empty() {
                    match mode {
                        't' => selector.tag = Some(current.clone()),
                        '.' => selector.classes.push(current.clone()),
                        '#' => selector.id = Some(current.clone()),
                        ':' => {
                            if let Some(pseudo) = parse_pseudo_class(&current) {
                                selector.pseudo_classes.push(pseudo);
                            }
                        }
                        _ => {}
                    }
                    current.clear();
                }
                mode = ':';
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Handle remaining
    if !current.is_empty() {
        match mode {
            't' => selector.tag = Some(current),
            '.' => selector.classes.push(current),
            '#' => selector.id = Some(current),
            ':' => {
                if let Some(pseudo) = parse_pseudo_class(&current) {
                    selector.pseudo_classes.push(pseudo);
                }
            }
            _ => {}
        }
    }

    selector
}

/// Parse a selector string into CssSelector with combinator support
fn parse_css_selector(selector: &str) -> CssSelector {
    let selector = selector.trim();
    let mut parts: Vec<(Combinator, SimpleSelector)> = Vec::new();

    // Tokenize selector into parts separated by combinators
    let mut current_part = String::new();
    let mut pending_combinator = Combinator::None;
    let mut chars = selector.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ' ' => {
                // Could be descendant or just whitespace around another combinator
                if !current_part.is_empty() {
                    let simple = parse_simple_selector(&current_part);
                    if !simple.is_empty() {
                        parts.push((pending_combinator, simple));
                        pending_combinator = Combinator::Descendant;
                    }
                    current_part.clear();
                }
                // Skip additional whitespace and check for explicit combinator
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            '>' => {
                if !current_part.is_empty() {
                    let simple = parse_simple_selector(&current_part);
                    if !simple.is_empty() {
                        parts.push((pending_combinator, simple));
                    }
                    current_part.clear();
                }
                pending_combinator = Combinator::Child;
                // Skip whitespace after combinator
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            '+' => {
                if !current_part.is_empty() {
                    let simple = parse_simple_selector(&current_part);
                    if !simple.is_empty() {
                        parts.push((pending_combinator, simple));
                    }
                    current_part.clear();
                }
                pending_combinator = Combinator::AdjacentSibling;
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            '~' => {
                if !current_part.is_empty() {
                    let simple = parse_simple_selector(&current_part);
                    if !simple.is_empty() {
                        parts.push((pending_combinator, simple));
                    }
                    current_part.clear();
                }
                pending_combinator = Combinator::GeneralSibling;
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            _ => {
                current_part.push(ch);
            }
        }
    }

    // Handle remaining part
    if !current_part.is_empty() {
        let simple = parse_simple_selector(&current_part);
        if !simple.is_empty() {
            parts.push((pending_combinator, simple));
        }
    }

    CssSelector { parts }
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

/// Extract external stylesheet URLs from <link rel="stylesheet"> elements
fn extract_external_stylesheets(handle: &Handle, base_url: &str) -> Vec<String> {
    use markup5ever_rcdom::NodeData;

    let mut urls = Vec::new();

    match &handle.data {
        NodeData::Element { name, attrs, .. } => {
            if name.local.as_ref() == "link" {
                let attrs = attrs.borrow();

                // Check if it's a stylesheet link
                let is_stylesheet = attrs.iter().any(|a| {
                    a.name.local.as_ref() == "rel" &&
                    a.value.to_string().to_lowercase().contains("stylesheet")
                });

                if is_stylesheet {
                    // Get the href attribute
                    if let Some(href_attr) = attrs.iter().find(|a| a.name.local.as_ref() == "href") {
                        let href = href_attr.value.to_string();
                        let resolved = resolve_url(&href, base_url);
                        urls.push(resolved);
                    }
                }
            }
        }
        _ => {}
    }

    // Recurse
    for child in handle.children.borrow().iter() {
        urls.extend(extract_external_stylesheets(child, base_url));
    }

    urls
}

/// Fetch external CSS file (blocking)
fn fetch_external_css(url: &str) -> Option<String> {
    // Skip non-HTTP URLs for now
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }

    // Use blocking reqwest for simplicity
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent(USER_AGENT)
        .build()
        .ok()?;

    match client.get(url).send() {
        Ok(response) => {
            if response.status().is_success() {
                response.text().ok()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Parse HTML to renderable content
fn parse_html_to_content(html: &str, base_url: &str) -> PageContent {
    use html5ever::tendril::TendrilSink;
    use html5ever::{ParseOpts, parse_document};
    use markup5ever_rcdom::RcDom;

    // Try YouTube-specific parsing first
    if let Some(youtube_elements) = try_parse_youtube(html, base_url) {
        let title = if base_url.contains("youtube.com") {
            "YouTube".to_string()
        } else {
            "Video".to_string()
        };
        return PageContent {
            title,
            elements: youtube_elements,
            error: None,
            console_output: Vec::new(),
            js_errors: Vec::new(),
        };
    }

    let opts = ParseOpts::default();
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .expect("Failed to parse HTML");

    // Collect all CSS: external stylesheets first, then inline styles
    let mut all_css = String::new();

    // 1. Extract and fetch external stylesheets
    let external_urls = extract_external_stylesheets(&dom.document, base_url);
    for url in &external_urls {
        if let Some(css) = fetch_external_css(url) {
            all_css.push_str(&css);
            all_css.push('\n');
        }
    }

    // 2. Extract inline <style> elements (these have higher precedence)
    let inline_css = extract_stylesheets(&dom.document);
    all_css.push_str(&inline_css);

    // Parse all CSS rules
    let css_rules = parse_css_rules(&all_css);

    let mut elements = Vec::new();
    let mut title = String::new();
    let mut y: f32 = 0.0;

    // Recursively extract content with CSS rules
    extract_content_with_css(&dom.document, &mut elements, &mut title, &mut y, 0, base_url, &css_rules);

    if title.is_empty() {
        title = "Untitled".to_string();
    }

    // 3. Extract and execute JavaScript
    let (console_output, js_errors) = execute_page_scripts(&dom.document, base_url);

    PageContent {
        title,
        elements,
        error: None,
        console_output,
        js_errors,
    }
}

/// Extract and execute JavaScript from the page
fn execute_page_scripts(handle: &Handle, base_url: &str) -> (Vec<String>, Vec<String>) {
    use std::sync::{Arc, Mutex};

    let console_output = Arc::new(Mutex::new(Vec::new()));
    let js_errors = Arc::new(Mutex::new(Vec::new()));

    // Extract all script contents
    let scripts = extract_scripts(handle, base_url);

    log::info!("Found {} JavaScript scripts to execute", scripts.len());

    if scripts.is_empty() {
        return (Vec::new(), Vec::new());
    }

    // Create JavaScript runtime with URL context
    let mut runtime = JsRuntime::with_url(base_url);

    // Execute each script
    for (i, script) in scripts.iter().enumerate() {
        log::info!("Executing script {} ({} bytes)", i + 1, script.len());
        log::debug!("Script content: {}", &script[..script.len().min(200)]);

        match runtime.execute(script) {
            Ok(result) => {
                log::info!("Script {} executed successfully, result: {:?}", i + 1, result);
            }
            Err(e) => {
                let error_msg = format!("JS Error in script {}: {}", i + 1, e);
                log::warn!("{}", error_msg);
                if let Ok(mut errors) = js_errors.lock() {
                    errors.push(error_msg);
                }
            }
        }
    }

    let output = console_output.lock().map(|o| o.clone()).unwrap_or_default();
    let errors = js_errors.lock().map(|e| e.clone()).unwrap_or_default();

    (output, errors)
}

/// Extract script contents from the DOM
fn extract_scripts(handle: &Handle, base_url: &str) -> Vec<String> {
    use markup5ever_rcdom::NodeData;

    let mut scripts = Vec::new();
    extract_scripts_recursive(handle, &mut scripts, base_url);
    scripts
}

fn extract_scripts_recursive(handle: &Handle, scripts: &mut Vec<String>, base_url: &str) {
    use markup5ever_rcdom::NodeData;

    match &handle.data {
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.as_ref();

            if tag == "script" {
                let attrs_ref = attrs.borrow();

                // Check for src attribute (external script)
                let src = attrs_ref
                    .iter()
                    .find(|a| a.name.local.as_ref() == "src")
                    .map(|a| a.value.to_string());

                // Check script type
                let script_type = attrs_ref
                    .iter()
                    .find(|a| a.name.local.as_ref() == "type")
                    .map(|a| a.value.to_string())
                    .unwrap_or_else(|| "text/javascript".to_string());

                // Only process JavaScript (not modules or other types for now)
                if script_type.contains("javascript") || script_type.is_empty() || script_type == "text/javascript" {
                    if let Some(src_url) = src {
                        // External script - fetch it
                        let full_url = resolve_url(&src_url, base_url);
                        if let Some(script_content) = fetch_external_script(&full_url) {
                            scripts.push(script_content);
                        }
                    } else {
                        // Inline script - extract text content
                        let script_text = extract_script_text(handle);
                        if !script_text.trim().is_empty() {
                            scripts.push(script_text);
                        }
                    }
                }
                return; // Don't recurse into script tags
            }
        }
        _ => {}
    }

    // Recurse into children
    for child in handle.children.borrow().iter() {
        extract_scripts_recursive(child, scripts, base_url);
    }
}

/// Extract text content from a script element
fn extract_script_text(handle: &Handle) -> String {
    use markup5ever_rcdom::NodeData;

    let mut text = String::new();
    for child in handle.children.borrow().iter() {
        if let NodeData::Text { contents } = &child.data {
            text.push_str(&contents.borrow().to_string());
        }
    }
    text
}

/// Fetch an external JavaScript file
fn fetch_external_script(url: &str) -> Option<String> {
    log::info!("Fetching external script: {}", url);

    // Skip data URLs and javascript: URLs
    if url.starts_with("data:") || url.starts_with("javascript:") {
        return None;
    }

    // Handle local files
    if url.starts_with("file://") {
        let path = url.trim_start_matches("file://");
        log::info!("Reading local script file: {}", path);
        match std::fs::read_to_string(path) {
            Ok(content) => {
                log::info!("Successfully read local script ({} bytes)", content.len());
                return Some(content);
            }
            Err(e) => {
                log::warn!("Failed to read local script {}: {}", path, e);
                return None;
            }
        }
    }

    // Use blocking reqwest for HTTP(S) URLs
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent(USER_AGENT)
        .build()
        .ok()?;

    let response = client.get(url).send().ok()?;

    if response.status().is_success() {
        response.text().ok()
    } else {
        log::warn!("Failed to fetch script {}: {}", url, response.status());
        None
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
        ElementKind::Textarea => {
            style.font_size = 14.0;
            style.background_color = Some([255, 255, 255, 255]);
            style.border_color = Some([200, 200, 200, 255]);
            style.border_width = [1.0; 4];
            style.padding = [8.0, 12.0, 8.0, 12.0];
            style.border_radius = 4.0;
            style.margin = [4.0, 0.0, 4.0, 0.0];
        }
        ElementKind::Select => {
            style.font_size = 14.0;
            style.background_color = Some([255, 255, 255, 255]);
            style.border_color = Some([200, 200, 200, 255]);
            style.border_width = [1.0; 4];
            style.padding = [8.0, 12.0, 8.0, 12.0];
            style.border_radius = 4.0;
        }
        ElementKind::Checkbox | ElementKind::Radio => {
            style.font_size = 14.0;
            style.margin = [4.0, 8.0, 4.0, 0.0];
        }
        ElementKind::Label => {
            style.font_size = 14.0;
            style.margin = [4.0, 0.0, 4.0, 0.0];
        }
        ElementKind::Form => {
            style.margin = [8.0, 0.0, 8.0, 0.0];
        }
        ElementKind::Container => {
            // Container has no default styling - it inherits from CSS
        }
    }

    // Determine if element is inline based on kind
    let is_inline = matches!(kind,
        ElementKind::Label | ElementKind::Input | ElementKind::Button |
        ElementKind::Checkbox | ElementKind::Radio | ElementKind::Select |
        ElementKind::Link | ElementKind::Text
    );

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
        form_attrs: None,
        is_inline,
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

/// Form context for passing form action/method to child inputs
#[derive(Clone, Default)]
struct FormContext {
    action: Option<String>,
    method: String,
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
    extract_content_inner(handle, elements, title, y, indent, base_url, &FormContext::default())
}

fn extract_content_inner(
    handle: &Handle,
    elements: &mut Vec<RenderElement>,
    title: &mut String,
    y: &mut f32,
    indent: u32,
    base_url: &str,
    form_ctx: &FormContext,
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
                    let btn_text = if text.is_empty() { "Button".to_string() } else { text };
                    let mut elem = create_styled_element(ElementKind::Button, btn_text.clone(), *y, indent);
                    elem.form_attrs = Some(FormAttributes {
                        input_type: "button".to_string(),
                        name: get_attr(&attrs, "name"),
                        id: get_attr(&attrs, "id"),
                        value: btn_text,
                        ..Default::default()
                    });
                    apply_inline(&mut elem);
                    elements.push(elem);
                    *y += 1.0;
                    return;
                }
                "input" => {
                    let input_type = get_attr(&attrs, "type");
                    let input_type = if input_type.is_empty() { "text".to_string() } else { input_type };
                    let placeholder = get_attr(&attrs, "placeholder");
                    let value = get_attr(&attrs, "value");
                    let name = get_attr(&attrs, "name");
                    let id = get_attr(&attrs, "id");
                    let checked = attrs.iter().any(|a| a.name.local.as_ref() == "checked");
                    let disabled = attrs.iter().any(|a| a.name.local.as_ref() == "disabled");

                    let (kind, display) = match input_type.as_str() {
                        "submit" | "button" => (ElementKind::Button, if value.is_empty() { "Submit".to_string() } else { value.clone() }),
                        "checkbox" => (ElementKind::Checkbox, format!("[{}]", if checked { "x" } else { " " })),
                        "radio" => (ElementKind::Radio, format!("({})", if checked { "•" } else { " " })),
                        "hidden" => return, // Skip hidden inputs
                        _ => {
                            let display = if !placeholder.is_empty() {
                                placeholder.clone()
                            } else {
                                format!("{} input", input_type)
                            };
                            (ElementKind::Input, display)
                        }
                    };

                    let mut elem = create_styled_element(kind, display, *y, indent);
                    elem.form_attrs = Some(FormAttributes {
                        input_type: input_type.clone(),
                        name,
                        id,
                        placeholder,
                        value,
                        disabled,
                        checked,
                        options: Vec::new(),
                        form_action: form_ctx.action.clone(),
                        form_method: form_ctx.method.clone(),
                    });
                    apply_inline(&mut elem);
                    elements.push(elem);
                    *y += 1.0;
                    return;
                }
                "textarea" => {
                    let text = extract_text(handle);
                    let placeholder = get_attr(&attrs, "placeholder");
                    let name = get_attr(&attrs, "name");
                    let display = if !text.is_empty() { text.clone() } else if !placeholder.is_empty() { placeholder.clone() } else { "Enter text...".to_string() };

                    let mut elem = create_styled_element(ElementKind::Textarea, display, *y, indent);
                    elem.form_attrs = Some(FormAttributes {
                        input_type: "textarea".to_string(),
                        name,
                        id: get_attr(&attrs, "id"),
                        placeholder,
                        value: text,
                        form_action: form_ctx.action.clone(),
                        form_method: form_ctx.method.clone(),
                        ..Default::default()
                    });
                    apply_inline(&mut elem);
                    elements.push(elem);
                    *y += 2.0; // Textarea takes more space
                    return;
                }
                "select" => {
                    let name = get_attr(&attrs, "name");
                    let mut options: Vec<(String, String)> = Vec::new();

                    // Extract options
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name: child_name, attrs: child_attrs, .. } = &child.data {
                            if child_name.local.as_ref() == "option" {
                                let child_attrs = child_attrs.borrow();
                                let opt_value = child_attrs.iter()
                                    .find(|a| a.name.local.as_ref() == "value")
                                    .map(|a| a.value.to_string())
                                    .unwrap_or_default();
                                let opt_text = extract_text(child);
                                options.push((opt_value, opt_text));
                            }
                        }
                    }

                    let display = options.first().map(|(_, label)| label.clone()).unwrap_or_else(|| "Select...".to_string());
                    let mut elem = create_styled_element(ElementKind::Select, format!("▼ {}", display), *y, indent);
                    elem.form_attrs = Some(FormAttributes {
                        input_type: "select".to_string(),
                        name,
                        id: get_attr(&attrs, "id"),
                        options,
                        ..Default::default()
                    });
                    apply_inline(&mut elem);
                    elements.push(elem);
                    *y += 1.0;
                    return;
                }
                "form" => {
                    // Extract form action and method
                    let action = get_attr(&attrs, "action");
                    let method = get_attr(&attrs, "method").to_uppercase();
                    let method = if method.is_empty() { "GET".to_string() } else { method };

                    let new_form_ctx = FormContext {
                        action: if action.is_empty() { None } else { Some(resolve_url(&action, base_url)) },
                        method,
                    };

                    // Recurse into form with form context
                    for child in handle.children.borrow().iter() {
                        extract_content_inner(child, elements, title, y, indent, base_url, &new_form_ctx);
                    }
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
                        extract_content_inner(child, elements, title, y, indent, base_url, form_ctx);
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
                extract_content_inner(child, elements, title, y, indent, base_url, form_ctx);
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
                extract_content_inner(child, elements, title, y, indent, base_url, form_ctx);
            }
        }
        _ => {}
    }
}

/// Maximum recursion depth to prevent stack overflow on complex pages
const MAX_RECURSION_DEPTH: u32 = 100;

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
    extract_content_with_css_inner(handle, elements, title, y, indent, base_url, css_rules, &FormContext::default(), 0)
}

fn extract_content_with_css_inner(
    handle: &Handle,
    elements: &mut Vec<RenderElement>,
    title: &mut String,
    y: &mut f32,
    indent: u32,
    base_url: &str,
    css_rules: &[CssRule],
    form_ctx: &FormContext,
    depth: u32,
) {
    // Prevent stack overflow on deeply nested pages
    if depth > MAX_RECURSION_DEPTH {
        return;
    }

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
                    return; // Don't process children - text already extracted
                }
                "div" | "section" | "article" | "main" | "header" | "footer" | "nav" => {
                    // Create a temporary element to compute styles
                    let mut temp_elem = create_styled_element(ElementKind::Container, String::new(), *y, indent);
                    apply_styles(&mut temp_elem);

                    // Check if this is a flex/grid container
                    let is_flex_container = matches!(temp_elem.style.display, DisplayMode::Flex | DisplayMode::Grid);

                    if is_flex_container {
                        // Use the already styled container
                        let mut container = temp_elem;

                        // Extract children into the container - each child div becomes a flex item
                        let mut child_elements: Vec<RenderElement> = Vec::new();
                        for child in handle.children.borrow().iter() {
                            extract_content_with_css_inner(child, &mut child_elements, title, y, indent, base_url, css_rules, form_ctx, depth + 1);
                        }
                        container.children = child_elements;
                        elements.push(container);
                    } else {
                        // Check if this div has only text content (no nested elements)
                        let has_element_children = handle.children.borrow().iter().any(|child| {
                            matches!(&child.data, NodeData::Element { .. })
                        });

                        // Check if this div has any styling (background, border, padding)
                        let has_visual_styling = temp_elem.style.background_color.is_some()
                            || temp_elem.style.border_color.is_some()
                            || temp_elem.style.padding.iter().any(|&p| p > 0.0);

                        if !has_element_children && has_visual_styling {
                            // This is a styled leaf div with text - render it as a styled container
                            let text = extract_text(handle);
                            let mut elem = create_styled_element(ElementKind::Text, text, *y, indent);
                            apply_styles(&mut elem);
                            elements.push(elem);
                            *y += 1.0;
                        } else {
                            // Regular container - recurse into children
                            for child in handle.children.borrow().iter() {
                                extract_content_with_css_inner(child, elements, title, y, indent, base_url, css_rules, form_ctx, depth + 1);
                            }
                        }
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
                            extract_content_with_css_inner(child, elements, title, y, indent, base_url, css_rules, form_ctx, depth + 1);
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
                        extract_content_with_css_inner(child, elements, title, y, indent + 1, base_url, css_rules, form_ctx, depth + 1);
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
                "script" | "style" | "noscript" | "meta" | "link" | "template" => {
                    return;
                }
                "svg" => {
                    // SVG support - for now show placeholder or title
                    let title_text = extract_svg_title(handle);
                    if !title_text.is_empty() {
                        let elem = create_styled_element(ElementKind::Image, format!("🖼 {}", title_text), *y, indent);
                        elements.push(elem);
                        *y += 1.0;
                    }
                    return;
                }
                "iframe" | "object" | "embed" | "canvas" => {
                    // Skip embedded content
                    return;
                }
                "head" => {
                    for child in handle.children.borrow().iter() {
                        extract_content_with_css_inner(child, elements, title, y, indent, base_url, css_rules, form_ctx, depth + 1);
                    }
                    return;
                }
                "br" => {
                    *y += 0.5;
                    return;
                }
                "hr" => {
                    let elem = create_styled_element(ElementKind::HorizontalRule, "───────────────────────────────".to_string(), *y, indent);
                    elements.push(elem);
                    *y += 1.0;
                    return;
                }
                // Form elements
                "button" => {
                    let text = extract_text(handle);
                    let btn_text = if text.is_empty() { "Button".to_string() } else { text };
                    let mut elem = create_styled_element(ElementKind::Button, btn_text.clone(), *y, indent);
                    elem.is_inline = true;
                    elem.form_attrs = Some(FormAttributes {
                        input_type: "button".to_string(),
                        name: get_attr(&attrs_ref, "name"),
                        id: get_attr(&attrs_ref, "id"),
                        value: btn_text,
                        form_action: form_ctx.action.clone(),
                        form_method: form_ctx.method.clone(),
                        ..Default::default()
                    });
                    apply_styles(&mut elem);
                    elements.push(elem);
                    // No y increment for inline elements
                    return;
                }
                "input" => {
                    let input_type = get_attr(&attrs_ref, "type");
                    let input_type = if input_type.is_empty() { "text".to_string() } else { input_type };
                    let placeholder = get_attr(&attrs_ref, "placeholder");
                    let value = get_attr(&attrs_ref, "value");
                    let name = get_attr(&attrs_ref, "name");
                    let id = get_attr(&attrs_ref, "id");
                    let checked = attrs_ref.iter().any(|a| a.name.local.as_ref() == "checked");
                    let disabled = attrs_ref.iter().any(|a| a.name.local.as_ref() == "disabled");

                    let (kind, display) = match input_type.as_str() {
                        "submit" | "button" => (ElementKind::Button, if value.is_empty() { "Submit".to_string() } else { value.clone() }),
                        "checkbox" => (ElementKind::Checkbox, String::new()),
                        "radio" => (ElementKind::Radio, String::new()),
                        "hidden" => return,
                        _ => {
                            let display = if !placeholder.is_empty() {
                                placeholder.clone()
                            } else {
                                String::new()
                            };
                            (ElementKind::Input, display)
                        }
                    };

                    let mut elem = create_styled_element(kind, display, *y, indent);
                    elem.is_inline = true; // All inputs are inline
                    elem.form_attrs = Some(FormAttributes {
                        input_type: input_type.clone(),
                        name,
                        id,
                        placeholder,
                        value,
                        disabled,
                        checked,
                        options: Vec::new(),
                        form_action: form_ctx.action.clone(),
                        form_method: form_ctx.method.clone(),
                    });
                    apply_styles(&mut elem);
                    elements.push(elem);
                    // No y increment for inline elements
                    return;
                }
                "textarea" => {
                    let text = extract_text(handle);
                    let placeholder = get_attr(&attrs_ref, "placeholder");
                    let name = get_attr(&attrs_ref, "name");

                    let mut elem = create_styled_element(ElementKind::Textarea, String::new(), *y, indent);
                    elem.form_attrs = Some(FormAttributes {
                        input_type: "textarea".to_string(),
                        name,
                        id: get_attr(&attrs_ref, "id"),
                        placeholder,
                        value: text,
                        form_action: form_ctx.action.clone(),
                        form_method: form_ctx.method.clone(),
                        ..Default::default()
                    });
                    apply_styles(&mut elem);
                    elements.push(elem);
                    *y += 2.0;
                    return;
                }
                "select" => {
                    let name = get_attr(&attrs_ref, "name");
                    let mut options: Vec<(String, String)> = Vec::new();

                    for child in handle.children.borrow().iter() {
                        if let NodeData::Element { name: child_name, attrs: child_attrs, .. } = &child.data {
                            if child_name.local.as_ref() == "option" {
                                let child_attrs = child_attrs.borrow();
                                let opt_value = child_attrs.iter()
                                    .find(|a| a.name.local.as_ref() == "value")
                                    .map(|a| a.value.to_string())
                                    .unwrap_or_default();
                                let opt_text = extract_text(child);
                                options.push((opt_value, opt_text));
                            }
                        }
                    }

                    let mut elem = create_styled_element(ElementKind::Select, String::new(), *y, indent);
                    elem.is_inline = true;
                    elem.form_attrs = Some(FormAttributes {
                        input_type: "select".to_string(),
                        name,
                        id: get_attr(&attrs_ref, "id"),
                        options,
                        form_action: form_ctx.action.clone(),
                        form_method: form_ctx.method.clone(),
                        ..Default::default()
                    });
                    apply_styles(&mut elem);
                    elements.push(elem);
                    // No y increment for inline elements
                    return;
                }
                "form" => {
                    // Extract form action and method
                    let action = get_attr(&attrs_ref, "action");
                    let method = get_attr(&attrs_ref, "method").to_uppercase();
                    let method = if method.is_empty() { "GET".to_string() } else { method };

                    let new_form_ctx = FormContext {
                        action: if action.is_empty() { None } else { Some(resolve_url(&action, base_url)) },
                        method,
                    };

                    for child in handle.children.borrow().iter() {
                        extract_content_with_css_inner(child, elements, title, y, indent, base_url, css_rules, &new_form_ctx, depth + 1);
                    }
                    return;
                }
                "label" => {
                    let text = extract_text(handle);
                    if !text.is_empty() {
                        let mut elem = create_styled_element(ElementKind::Label, text, *y, indent);
                        elem.is_inline = true;
                        apply_styles(&mut elem);
                        elements.push(elem);
                        // No y increment for inline elements
                    }
                    return;
                }
                _ => {}
            }

            // Recurse into children
            for child in handle.children.borrow().iter() {
                extract_content_with_css_inner(child, elements, title, y, indent, base_url, css_rules, form_ctx, depth + 1);
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
                extract_content_with_css_inner(child, elements, title, y, indent, base_url, css_rules, form_ctx, depth + 1);
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
        "display" => {
            style.display = match value.to_lowercase().as_str() {
                "none" => DisplayMode::None,
                "inline" => DisplayMode::Inline,
                "inline-block" => DisplayMode::InlineBlock,
                "flex" => DisplayMode::Flex,
                "grid" => DisplayMode::Grid,
                "block" => DisplayMode::Block,
                _ => DisplayMode::Block,
            };
        }
        "visibility" => {
            style.visible = value.to_lowercase() != "hidden";
        }
        // Flexbox properties
        "flex-direction" => {
            style.flex.direction = match value.to_lowercase().as_str() {
                "row" => FlexDirection::Row,
                "row-reverse" => FlexDirection::RowReverse,
                "column" => FlexDirection::Column,
                "column-reverse" => FlexDirection::ColumnReverse,
                _ => FlexDirection::Row,
            };
        }
        "flex-wrap" => {
            style.flex.wrap = match value.to_lowercase().as_str() {
                "wrap" => FlexWrap::Wrap,
                "wrap-reverse" => FlexWrap::WrapReverse,
                "nowrap" => FlexWrap::NoWrap,
                _ => FlexWrap::NoWrap,
            };
        }
        "justify-content" => {
            style.flex.justify_content = match value.to_lowercase().as_str() {
                "flex-start" | "start" => JustifyContent::FlexStart,
                "flex-end" | "end" => JustifyContent::FlexEnd,
                "center" => JustifyContent::Center,
                "space-between" => JustifyContent::SpaceBetween,
                "space-around" => JustifyContent::SpaceAround,
                "space-evenly" => JustifyContent::SpaceEvenly,
                _ => JustifyContent::FlexStart,
            };
        }
        "align-items" => {
            style.flex.align_items = match value.to_lowercase().as_str() {
                "stretch" => AlignItems::Stretch,
                "flex-start" | "start" => AlignItems::FlexStart,
                "flex-end" | "end" => AlignItems::FlexEnd,
                "center" => AlignItems::Center,
                "baseline" => AlignItems::Baseline,
                _ => AlignItems::Stretch,
            };
        }
        "gap" | "grid-gap" => {
            if let Some(size) = parse_css_size(value) {
                style.flex.gap = size;
            }
        }
        "flex-grow" => {
            if let Ok(grow) = value.parse::<f32>() {
                style.flex.flex_grow = grow;
            }
        }
        "flex-shrink" => {
            if let Ok(shrink) = value.parse::<f32>() {
                style.flex.flex_shrink = shrink;
            }
        }
        "flex" => {
            // Parse shorthand: flex: grow shrink? basis?
            let parts: Vec<&str> = value.split_whitespace().collect();
            if let Some(first) = parts.first() {
                if let Ok(grow) = first.parse::<f32>() {
                    style.flex.flex_grow = grow;
                }
            }
            if let Some(second) = parts.get(1) {
                if let Ok(shrink) = second.parse::<f32>() {
                    style.flex.flex_shrink = shrink;
                }
            }
        }
        // CSS positioning properties
        "position" => {
            style.position = match value.to_lowercase().as_str() {
                "static" => Position::Static,
                "relative" => Position::Relative,
                "absolute" => Position::Absolute,
                "fixed" => Position::Fixed,
                "sticky" => Position::Sticky,
                _ => Position::Static,
            };
        }
        "top" => {
            if let Some(size) = parse_css_size(value) {
                style.top = Some(size);
            }
        }
        "right" => {
            if let Some(size) = parse_css_size(value) {
                style.right = Some(size);
            }
        }
        "bottom" => {
            if let Some(size) = parse_css_size(value) {
                style.bottom = Some(size);
            }
        }
        "left" => {
            if let Some(size) = parse_css_size(value) {
                style.left = Some(size);
            }
        }
        "z-index" => {
            if let Ok(z) = value.parse::<i32>() {
                style.z_index = z;
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

/// Helper to get an attribute value from attrs
fn get_attr(attrs: &[html5ever::Attribute], name: &str) -> String {
    attrs.iter()
        .find(|a| a.name.local.as_ref() == name)
        .map(|a| a.value.to_string())
        .unwrap_or_default()
}

/// Extract title from SVG element
fn extract_svg_title(handle: &Handle) -> String {
    use markup5ever_rcdom::NodeData;

    for child in handle.children.borrow().iter() {
        if let NodeData::Element { name, .. } = &child.data {
            if name.local.as_ref() == "title" {
                return extract_text(child);
            }
        }
    }
    String::new()
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
        // First rule: h1 selector
        assert!(rules[0].selector.matches("h1", None, &[]));
        assert_eq!(rules[0].properties.get("color"), Some(&"red".to_string()));
        assert_eq!(rules[0].properties.get("font-size"), Some(&"24px".to_string()));

        // Second rule: .title selector
        assert!(rules[1].selector.matches("div", None, &["title".to_string()]));
        assert_eq!(rules[1].properties.get("font-weight"), Some(&"bold".to_string()));
    }

    #[test]
    fn test_css_selector_matching() {
        let selector = parse_css_selector("div.container#main");

        // Test matching
        assert!(selector.matches("div", Some("main"), &["container".to_string()]));
        assert!(!selector.matches("div", Some("other"), &["container".to_string()]));
        assert!(!selector.matches("span", Some("main"), &["container".to_string()]));
    }

    #[test]
    fn test_css_combinator_parsing() {
        // Test descendant selector
        let selector = parse_css_selector("ul li");
        assert_eq!(selector.parts.len(), 2);

        // Test child selector
        let selector = parse_css_selector("ul > li");
        assert_eq!(selector.parts.len(), 2);

        // Test adjacent sibling selector
        let selector = parse_css_selector("h1 + p");
        assert_eq!(selector.parts.len(), 2);
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

// ============================================================================
// YouTube-specific content extraction
// ============================================================================

/// Extract ytInitialData JSON from YouTube HTML
fn extract_yt_initial_data(html: &str) -> Option<serde_json::Value> {
    let start_marker = "ytInitialData = ";
    let start = html.find(start_marker);
    if start.is_none() {
        log::warn!("ytInitialData marker not found in HTML");
        return None;
    }
    let start = start.unwrap();
    let json_start = start + start_marker.len();
    let remaining = &html[json_start..];

    // Find the end by looking for }; or }; patterns
    // YouTube typically ends with }; or };</script>
    let end_markers = ["};", "};</script>", "};\n"];
    let mut end_pos = None;

    for marker in &end_markers {
        if let Some(pos) = remaining.find(marker) {
            let candidate = pos + 1; // Include the }
            if end_pos.is_none() || candidate < end_pos.unwrap() {
                end_pos = Some(candidate);
            }
        }
    }

    let end_pos = match end_pos {
        Some(p) => p,
        None => {
            log::warn!("Could not find end of ytInitialData JSON");
            return None;
        }
    };

    log::info!("Found ytInitialData JSON ({} bytes)", end_pos);
    let json_str = &remaining[..end_pos];

    match serde_json::from_str(json_str) {
        Ok(v) => Some(v),
        Err(e) => {
            log::warn!("Failed to parse ytInitialData: {}", e);
            // Try to show where the error is
            if end_pos > 100 {
                log::debug!("JSON end: ...{}", &json_str[end_pos.saturating_sub(100)..]);
            }
            None
        }
    }
}

/// YouTube video data
struct YouTubeVideo {
    video_id: String,
    title: String,
    thumbnail_url: String,
    channel_name: String,
    view_count: String,
    duration: String,
}

/// Extract videos from ytInitialData by recursively searching for videoRenderer
fn extract_youtube_videos(data: &serde_json::Value) -> Vec<YouTubeVideo> {
    let mut videos = Vec::new();
    find_video_renderers(data, &mut videos);
    videos
}

/// Recursively find all videoRenderer objects in the JSON
fn find_video_renderers(value: &serde_json::Value, videos: &mut Vec<YouTubeVideo>) {
    match value {
        serde_json::Value::Object(map) => {
            // Check if this object contains a videoRenderer
            if let Some(renderer) = map.get("videoRenderer") {
                if let Some(video) = extract_video_from_renderer(renderer) {
                    videos.push(video);
                }
            }
            // Also check for richItemRenderer -> content -> videoRenderer
            if let Some(rich_item) = map.get("richItemRenderer") {
                if let Some(content) = rich_item.get("content") {
                    if let Some(renderer) = content.get("videoRenderer") {
                        if let Some(video) = extract_video_from_renderer(renderer) {
                            videos.push(video);
                        }
                    }
                }
            }
            // Recurse into all values
            for v in map.values() {
                find_video_renderers(v, videos);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                find_video_renderers(v, videos);
            }
        }
        _ => {}
    }
}

fn extract_video_from_renderer(renderer: &serde_json::Value) -> Option<YouTubeVideo> {

    let video_id = renderer.get("videoId")?.as_str()?.to_string();
    let title = renderer.get("title")
        .and_then(|t| t.get("runs"))
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .and_then(|r| r.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("Untitled").to_string();
    let thumbnail_url = renderer.get("thumbnail")
        .and_then(|t| t.get("thumbnails"))
        .and_then(|t| t.as_array())
        .and_then(|a| a.last())
        .and_then(|t| t.get("url"))
        .and_then(|u| u.as_str())
        .unwrap_or("").to_string();
    let channel_name = renderer.get("ownerText")
        .and_then(|o| o.get("runs"))
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .and_then(|r| r.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("").to_string();
    let view_count = renderer.get("viewCountText")
        .and_then(|v| v.get("simpleText"))
        .and_then(|t| t.as_str())
        .or_else(|| renderer.get("shortViewCountText")
            .and_then(|v| v.get("simpleText"))
            .and_then(|t| t.as_str()))
        .unwrap_or("").to_string();
    let duration = renderer.get("lengthText")
        .and_then(|l| l.get("simpleText"))
        .and_then(|t| t.as_str())
        .unwrap_or("").to_string();

    Some(YouTubeVideo { video_id, title, thumbnail_url, channel_name, view_count, duration })
}

/// Try to parse YouTube page and return video elements
fn try_parse_youtube(html: &str, url: &str) -> Option<Vec<RenderElement>> {
    if !url.contains("youtube.com") && !url.contains("youtu.be") {
        return None;
    }

    log::info!("Parsing YouTube page...");
    let data = extract_yt_initial_data(html)?;

    // Check if this is a video watch page
    if url.contains("/watch") || url.contains("youtu.be/") {
        return try_parse_youtube_watch_page(&data, url);
    }

    let videos = extract_youtube_videos(&data);
    log::info!("Found {} YouTube videos", videos.len());

    // If no videos found and this is the homepage, show a helpful message
    if videos.is_empty() {
        if url.contains("youtube.com") && !url.contains("/results") && !url.contains("/watch") && !url.contains("/@") && !url.contains("/channel") {
            return Some(create_youtube_homepage_placeholder());
        }
        return None;
    }

    let mut elements = Vec::new();
    let mut y: f32 = 10.0;

    // Helper to create element
    fn make_element(kind: ElementKind, text: String, bounds: ElementBounds, style: ElementStyle) -> RenderElement {
        RenderElement {
            kind,
            text,
            bounds,
            style,
            is_link: false,
            href: None,
            src: None,
            alt: None,
            children: Vec::new(),
            form_attrs: None,
            is_inline: false,
        }
    }

    // Header
    let mut header_style = ElementStyle::default();
    header_style.font_size = 28.0;
    header_style.font_weight_bold = true;
    header_style.color = [255, 0, 0, 255]; // YouTube red
    elements.push(make_element(
        ElementKind::Heading1,
        "📺 YouTube".to_string(),
        ElementBounds { x: 20.0, y, width: 800.0, height: 40.0 },
        header_style,
    ));
    y += 50.0;

    for video in videos.iter().take(20) {
        // Thumbnail
        if !video.thumbnail_url.is_empty() {
            let mut thumb = make_element(
                ElementKind::Image,
                String::new(),
                ElementBounds { x: 20.0, y, width: 320.0, height: 180.0 },
                ElementStyle::default(),
            );
            thumb.src = Some(video.thumbnail_url.clone());
            thumb.alt = Some(video.title.clone());
            elements.push(thumb);
        }

        // Duration badge
        if !video.duration.is_empty() {
            let mut dur_style = ElementStyle::default();
            dur_style.font_size = 12.0;
            dur_style.color = [255, 255, 255, 255];
            dur_style.background_color = Some([0, 0, 0, 200]);
            elements.push(make_element(
                ElementKind::Text,
                format!(" {} ", video.duration),
                ElementBounds { x: 280.0, y: y + 150.0, width: 60.0, height: 20.0 },
                dur_style,
            ));
        }

        // Video title link
        let mut title_style = ElementStyle::default();
        title_style.font_size = 16.0;
        title_style.font_weight_bold = true;
        title_style.color = [30, 30, 30, 255];
        let mut title_elem = make_element(
            ElementKind::Link,
            video.title.clone(),
            ElementBounds { x: 350.0, y, width: 500.0, height: 24.0 },
            title_style,
        );
        title_elem.is_link = true;
        title_elem.href = Some(format!("https://www.youtube.com/watch?v={}", video.video_id));
        elements.push(title_elem);

        // Channel name
        if !video.channel_name.is_empty() {
            let mut chan_style = ElementStyle::default();
            chan_style.font_size = 13.0;
            chan_style.color = [96, 96, 96, 255];
            elements.push(make_element(
                ElementKind::Text,
                video.channel_name.clone(),
                ElementBounds { x: 350.0, y: y + 28.0, width: 400.0, height: 18.0 },
                chan_style,
            ));
        }

        // View count
        if !video.view_count.is_empty() {
            let mut view_style = ElementStyle::default();
            view_style.font_size = 13.0;
            view_style.color = [96, 96, 96, 255];
            elements.push(make_element(
                ElementKind::Text,
                video.view_count.clone(),
                ElementBounds { x: 350.0, y: y + 48.0, width: 400.0, height: 18.0 },
                view_style,
            ));
        }

        y += 200.0;
    }

    Some(elements)
}

/// Parse a YouTube watch page (video page)
fn try_parse_youtube_watch_page(data: &serde_json::Value, url: &str) -> Option<Vec<RenderElement>> {
    // Extract video ID from URL
    let video_id = if url.contains("watch?v=") {
        url.split("watch?v=").nth(1).and_then(|s| s.split('&').next())
    } else if url.contains("youtu.be/") {
        url.split("youtu.be/").nth(1).and_then(|s| s.split('?').next())
    } else {
        None
    }?;

    // Find video info
    let mut title = String::new();
    let mut channel = String::new();
    let mut description = String::new();
    let mut view_count = String::new();
    let mut like_count = String::new();

    // Search for videoPrimaryInfoRenderer and videoSecondaryInfoRenderer
    find_video_info(data, &mut title, &mut channel, &mut description, &mut view_count, &mut like_count);

    if title.is_empty() {
        log::warn!("Could not find video title in watch page");
        return None;
    }

    log::info!("YouTube video: {} by {}", title, channel);

    let mut elements = Vec::new();
    let mut y: f32 = 10.0;

    // Helper to create element
    fn make_element(kind: ElementKind, text: String, bounds: ElementBounds, style: ElementStyle) -> RenderElement {
        RenderElement {
            kind, text, bounds, style,
            is_link: false, href: None, src: None, alt: None,
            children: Vec::new(), form_attrs: None, is_inline: false,
        }
    }

    // Video player placeholder (thumbnail)
    let thumbnail_url = format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", video_id);
    let mut thumb = make_element(
        ElementKind::Image,
        String::new(),
        ElementBounds { x: 20.0, y, width: 854.0, height: 480.0 },
        ElementStyle::default(),
    );
    thumb.src = Some(thumbnail_url);
    thumb.alt = Some(title.clone());
    elements.push(thumb);
    y += 490.0;

    // Play button overlay indicator
    let mut play_style = ElementStyle::default();
    play_style.font_size = 16.0;
    play_style.color = [100, 100, 100, 255];
    elements.push(make_element(
        ElementKind::Text,
        "▶ Video playback requires external player".to_string(),
        ElementBounds { x: 20.0, y, width: 400.0, height: 24.0 },
        play_style,
    ));
    y += 40.0;

    // Video title
    let mut title_style = ElementStyle::default();
    title_style.font_size = 22.0;
    title_style.font_weight_bold = true;
    title_style.color = [30, 30, 30, 255];
    elements.push(make_element(
        ElementKind::Heading1,
        title,
        ElementBounds { x: 20.0, y, width: 854.0, height: 30.0 },
        title_style,
    ));
    y += 40.0;

    // Channel name
    if !channel.is_empty() {
        let mut chan_style = ElementStyle::default();
        chan_style.font_size = 15.0;
        chan_style.font_weight_bold = true;
        chan_style.color = [30, 30, 30, 255];
        elements.push(make_element(
            ElementKind::Text,
            channel,
            ElementBounds { x: 20.0, y, width: 400.0, height: 22.0 },
            chan_style,
        ));
        y += 30.0;
    }

    // Stats (views and likes)
    if !view_count.is_empty() || !like_count.is_empty() {
        let stats = format!("{}{}{}",
            view_count,
            if !view_count.is_empty() && !like_count.is_empty() { " • " } else { "" },
            like_count
        );
        let mut stats_style = ElementStyle::default();
        stats_style.font_size = 13.0;
        stats_style.color = [96, 96, 96, 255];
        elements.push(make_element(
            ElementKind::Text,
            stats,
            ElementBounds { x: 20.0, y, width: 400.0, height: 20.0 },
            stats_style,
        ));
        y += 30.0;
    }

    // Description
    if !description.is_empty() {
        y += 10.0;
        let mut desc_style = ElementStyle::default();
        desc_style.font_size = 14.0;
        desc_style.color = [50, 50, 50, 255];
        desc_style.max_width = 854.0;
        // Truncate description if too long
        let desc_text = if description.len() > 500 {
            format!("{}...", &description[..500])
        } else {
            description
        };
        elements.push(make_element(
            ElementKind::Text,
            desc_text,
            ElementBounds { x: 20.0, y, width: 854.0, height: 100.0 },
            desc_style,
        ));
    }

    Some(elements)
}

/// Recursively find video info in ytInitialData
fn find_video_info(
    value: &serde_json::Value,
    title: &mut String,
    channel: &mut String,
    description: &mut String,
    view_count: &mut String,
    like_count: &mut String,
) {
    match value {
        serde_json::Value::Object(map) => {
            // videoPrimaryInfoRenderer contains title and view count
            if let Some(renderer) = map.get("videoPrimaryInfoRenderer") {
                if let Some(t) = renderer.get("title")
                    .and_then(|t| t.get("runs"))
                    .and_then(|r| r.as_array())
                    .and_then(|a| a.first())
                    .and_then(|r| r.get("text"))
                    .and_then(|t| t.as_str())
                {
                    if title.is_empty() { *title = t.to_string(); }
                }
                if let Some(vc) = renderer.get("viewCount")
                    .and_then(|v| v.get("videoViewCountRenderer"))
                    .and_then(|r| r.get("viewCount"))
                    .and_then(|v| v.get("simpleText"))
                    .and_then(|t| t.as_str())
                {
                    if view_count.is_empty() { *view_count = vc.to_string(); }
                }
            }

            // videoSecondaryInfoRenderer contains channel and description
            if let Some(renderer) = map.get("videoSecondaryInfoRenderer") {
                if let Some(c) = renderer.get("owner")
                    .and_then(|o| o.get("videoOwnerRenderer"))
                    .and_then(|r| r.get("title"))
                    .and_then(|t| t.get("runs"))
                    .and_then(|r| r.as_array())
                    .and_then(|a| a.first())
                    .and_then(|r| r.get("text"))
                    .and_then(|t| t.as_str())
                {
                    if channel.is_empty() { *channel = c.to_string(); }
                }
                if let Some(d) = renderer.get("attributedDescription")
                    .and_then(|d| d.get("content"))
                    .and_then(|c| c.as_str())
                {
                    if description.is_empty() { *description = d.to_string(); }
                }
            }

            // Recurse
            for v in map.values() {
                find_video_info(v, title, channel, description, view_count, like_count);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                find_video_info(v, title, channel, description, view_count, like_count);
            }
        }
        _ => {}
    }
}

/// Create a placeholder for YouTube homepage when no videos are available
fn create_youtube_homepage_placeholder() -> Vec<RenderElement> {
    let mut elements = Vec::new();
    let mut y: f32 = 50.0;

    fn make_element(kind: ElementKind, text: String, bounds: ElementBounds, style: ElementStyle) -> RenderElement {
        RenderElement {
            kind, text, bounds, style,
            is_link: false, href: None, src: None, alt: None,
            children: Vec::new(), form_attrs: None, is_inline: false,
        }
    }

    // YouTube logo/header
    let mut header_style = ElementStyle::default();
    header_style.font_size = 32.0;
    header_style.font_weight_bold = true;
    header_style.color = [255, 0, 0, 255];
    elements.push(make_element(
        ElementKind::Heading1,
        "📺 YouTube".to_string(),
        ElementBounds { x: 50.0, y, width: 400.0, height: 40.0 },
        header_style,
    ));
    y += 60.0;

    // Explanation
    let mut text_style = ElementStyle::default();
    text_style.font_size = 16.0;
    text_style.color = [60, 60, 60, 255];
    elements.push(make_element(
        ElementKind::Text,
        "YouTube homepage requires login to show personalized videos.".to_string(),
        ElementBounds { x: 50.0, y, width: 600.0, height: 24.0 },
        text_style.clone(),
    ));
    y += 40.0;

    elements.push(make_element(
        ElementKind::Text,
        "Try one of these instead:".to_string(),
        ElementBounds { x: 50.0, y, width: 400.0, height: 24.0 },
        text_style.clone(),
    ));
    y += 40.0;

    // Suggestions
    let suggestions = [
        ("🔍 Search for videos", "https://www.youtube.com/results?search_query="),
        ("🎵 Music videos", "https://www.youtube.com/results?search_query=music+videos"),
        ("💻 Programming tutorials", "https://www.youtube.com/results?search_query=programming+tutorial"),
        ("🎮 Gaming", "https://www.youtube.com/results?search_query=gaming"),
        ("📰 News", "https://www.youtube.com/results?search_query=news+today"),
    ];

    for (label, href) in suggestions {
        let mut link_style = ElementStyle::default();
        link_style.font_size = 15.0;
        link_style.color = [30, 100, 200, 255];
        link_style.text_decoration_underline = true;
        let mut link = make_element(
            ElementKind::Link,
            label.to_string(),
            ElementBounds { x: 70.0, y, width: 400.0, height: 24.0 },
            link_style,
        );
        link.is_link = true;
        link.href = Some(href.to_string());
        elements.push(link);
        y += 32.0;
    }

    y += 30.0;

    // Tip
    let mut tip_style = ElementStyle::default();
    tip_style.font_size = 13.0;
    tip_style.color = [100, 100, 100, 255];
    tip_style.font_style_italic = true;
    elements.push(make_element(
        ElementKind::Text,
        "Tip: Type a YouTube search URL in the address bar, or paste a video link directly.".to_string(),
        ElementBounds { x: 50.0, y, width: 700.0, height: 20.0 },
        tip_style,
    ));

    elements
}
