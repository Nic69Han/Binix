//! Developer tools for Binix browser
//!
//! Provides debugging and inspection tools:
//! - Console: JavaScript console with logging
//! - DOM Inspector: View and modify DOM tree
//! - Network Inspector: Monitor network requests
//! - Performance Profiler: Analyze performance metrics

mod console;
mod dom_inspector;
mod network_inspector;
mod profiler;

pub use console::{Console, ConsoleMessage, LogLevel};
pub use dom_inspector::{DomInspector, DomNode};
pub use network_inspector::{NetworkInspector, NetworkRequest, RequestStatus};
pub use profiler::{PerformanceProfiler, ProfileEntry, ProfileMetric};

use std::sync::{Arc, Mutex};

/// Developer tools manager
pub struct DevTools {
    pub console: Arc<Mutex<Console>>,
    pub dom_inspector: Arc<Mutex<DomInspector>>,
    pub network_inspector: Arc<Mutex<NetworkInspector>>,
    pub profiler: Arc<Mutex<PerformanceProfiler>>,
    enabled: bool,
}

impl DevTools {
    /// Create new developer tools
    pub fn new() -> Self {
        Self {
            console: Arc::new(Mutex::new(Console::new())),
            dom_inspector: Arc::new(Mutex::new(DomInspector::new())),
            network_inspector: Arc::new(Mutex::new(NetworkInspector::new())),
            profiler: Arc::new(Mutex::new(PerformanceProfiler::new())),
            enabled: false,
        }
    }

    /// Enable/disable developer tools
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if developer tools are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Clear all developer tools data
    pub fn clear_all(&self) {
        if let Ok(mut console) = self.console.lock() {
            console.clear();
        }
        if let Ok(mut network) = self.network_inspector.lock() {
            network.clear();
        }
        if let Ok(mut profiler) = self.profiler.lock() {
            profiler.clear();
        }
    }
}

impl Default for DevTools {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devtools_creation() {
        let devtools = DevTools::new();
        assert!(!devtools.is_enabled());
    }

    #[test]
    fn test_devtools_enable_disable() {
        let mut devtools = DevTools::new();
        devtools.set_enabled(true);
        assert!(devtools.is_enabled());
        devtools.set_enabled(false);
        assert!(!devtools.is_enabled());
    }
}

