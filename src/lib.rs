//! # Binix - Ultra-High-Performance Web Browser
//!
//! A next-generation web browser engine written in Rust, designed for
//! maximum performance, memory efficiency, and security.
//!
//! ## Architecture
//!
//! The browser is organized into the following core modules:
//!
//! - **engine**: Core browser engine orchestrating all components
//! - **renderer**: HTML/CSS parsing and layout computation
//! - **network**: HTTP/3 networking with connection pooling
//! - **js_engine**: JavaScript runtime integration
//! - **wasm**: WebAssembly runtime with SIMD and threading support
//! - **compositor**: GPU-accelerated compositing
//! - **process**: Multi-process architecture with site isolation
//! - **security**: CSP, SRI, CORS, and mixed content blocking
//! - **ui**: User interface components
//! - **devtools**: Developer tools (console, inspector, profiler)
//! - **utils**: Shared utilities and error types

pub mod compositor;
pub mod devtools;
pub mod engine;
pub mod js_engine;
pub mod memory;
pub mod network;
pub mod process;
pub mod renderer;
pub mod security;
pub mod ui;
pub mod utils;
pub mod wasm;
pub mod wpt;

// Re-export main types for convenience
pub use engine::BrowserEngine;
pub use utils::error::{BinixError, Result};

/// Browser version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "Binix";

/// Performance target constants (from CDC)
pub mod performance_targets {
    /// Target page load time in milliseconds
    pub const PAGE_LOAD_MS: u64 = 1500;
    /// Maximum memory per tab in MB
    pub const MAX_TAB_MEMORY_MB: u64 = 150;
    /// Target memory reduction vs Chrome (%)
    pub const MEMORY_REDUCTION_PERCENT: u8 = 30;
    /// Target CPU efficiency gain (%)
    pub const CPU_EFFICIENCY_GAIN_PERCENT: u8 = 25;
    /// Target battery improvement (%)
    pub const BATTERY_IMPROVEMENT_PERCENT: u8 = 20;
}
