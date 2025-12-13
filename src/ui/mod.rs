//! User interface components for Binix browser

mod app;
mod tab;
mod window;

pub use app::{run, BrowserApp};
pub use tab::{Tab, TabId, TabManager};
pub use window::Window;

/// UI configuration
#[derive(Debug, Clone)]
pub struct UiConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub default_zoom: f32,
    pub theme: Theme,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            default_zoom: 1.0,
            theme: Theme::System,
        }
    }
}

/// UI theme
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

