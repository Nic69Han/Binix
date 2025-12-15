//! User interface components for Binix browser

mod app;
mod events;
mod images;
mod tab;
pub mod taffy_layout;
mod window;

pub use app::{BrowserApp, run};
pub use events::{
    Event, EventData, EventDispatcher, EventHandler, EventQueue, EventType, KeyboardEvent,
    Modifiers, MouseButton, MouseEvent, ScrollEvent, TouchEvent, TouchPoint,
};
pub use images::{ImageCache, ImageState, LoadedImage, SharedImageCache, create_shared_cache, decode_image};
pub use tab::{
    AlignItems, DisplayMode, ElementBounds, ElementKind, ElementStyle, FlexDirection,
    FlexProperties, FlexWrap, FormAttributes, JustifyContent, PageContent, Position,
    RenderElement, Tab, TabId, TabManager, TextAlign,
};
pub use taffy_layout::{TaffyLayoutContext, LayoutResult};
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
