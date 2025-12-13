//! Tab management

/// Unique tab identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(u64);

impl TabId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// A browser tab
pub struct Tab {
    id: TabId,
    title: String,
    url: String,
    loading: bool,
    // Each tab has its own isolated engine instance
    // engine: BrowserEngine,
}

impl Tab {
    /// Create a new tab
    pub fn new(id: TabId) -> Self {
        Self {
            id,
            title: "New Tab".to_string(),
            url: String::new(),
            loading: false,
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
        self.url = url.into();
        self.loading = true;
    }

    /// Check if the tab is loading
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Mark loading as complete
    pub fn finish_loading(&mut self) {
        self.loading = false;
    }
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

