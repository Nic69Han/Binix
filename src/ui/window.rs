//! Browser window management

use super::{TabManager, Theme, UiConfig};

/// Browser window
pub struct Window {
    config: UiConfig,
    tabs: TabManager,
    fullscreen: bool,
}

impl Window {
    /// Create a new window with default config
    pub fn new() -> Self {
        Self::with_config(UiConfig::default())
    }

    /// Create a new window with custom config
    pub fn with_config(config: UiConfig) -> Self {
        Self {
            config,
            tabs: TabManager::new(),
            fullscreen: false,
        }
    }

    /// Get window width
    pub fn width(&self) -> u32 {
        self.config.window_width
    }

    /// Get window height
    pub fn height(&self) -> u32 {
        self.config.window_height
    }

    /// Resize the window
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.window_width = width;
        self.config.window_height = height;
    }

    /// Get tab manager
    pub fn tabs(&self) -> &TabManager {
        &self.tabs
    }

    /// Get mutable tab manager
    pub fn tabs_mut(&mut self) -> &mut TabManager {
        &mut self.tabs
    }

    /// Toggle fullscreen mode
    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
    }

    /// Check if in fullscreen mode
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Get current zoom level
    pub fn zoom(&self) -> f32 {
        self.config.default_zoom
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f32) {
        self.config.default_zoom = zoom.clamp(0.25, 5.0);
    }

    /// Get current theme
    pub fn theme(&self) -> Theme {
        self.config.theme
    }

    /// Set theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.config.theme = theme;
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}
