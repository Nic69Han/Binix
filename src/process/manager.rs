//! Process manager for multi-process architecture

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::ipc::{IpcChannel, IpcMessage, IpcMessageType};

/// Process types in the browser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessType {
    /// Main browser process
    Browser,
    /// Renderer process for a site
    Renderer,
    /// GPU process
    Gpu,
    /// Network process
    Network,
    /// Extension process
    Extension,
    /// Utility process
    Utility,
}

impl ProcessType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessType::Browser => "Browser",
            ProcessType::Renderer => "Renderer",
            ProcessType::Gpu => "GPU",
            ProcessType::Network => "Network",
            ProcessType::Extension => "Extension",
            ProcessType::Utility => "Utility",
        }
    }
}

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Starting,
    Running,
    Suspended,
    Crashed,
    Terminated,
}

/// Information about a process
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub id: u32,
    pub process_type: ProcessType,
    pub state: ProcessState,
    pub site_origin: Option<String>,
    pub memory_usage: usize,
    pub cpu_usage: f32,
    pub start_time: Instant,
}

impl ProcessInfo {
    /// Create a new process info
    pub fn new(id: u32, process_type: ProcessType) -> Self {
        Self {
            id,
            process_type,
            state: ProcessState::Starting,
            site_origin: None,
            memory_usage: 0,
            cpu_usage: 0.0,
            start_time: Instant::now(),
        }
    }

    /// Set site origin for renderer process
    pub fn with_origin(mut self, origin: &str) -> Self {
        self.site_origin = Some(origin.to_string());
        self
    }

    /// Get uptime
    pub fn uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Process manager
pub struct ProcessManager {
    processes: HashMap<u32, ProcessInfo>,
    site_to_process: HashMap<String, u32>,
    next_id: u32,
    max_renderer_processes: usize,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            site_to_process: HashMap::new(),
            next_id: 1,
            max_renderer_processes: 20,
        }
    }

    /// Spawn a new process
    pub fn spawn(&mut self, process_type: ProcessType) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let mut info = ProcessInfo::new(id, process_type);
        info.state = ProcessState::Running;
        self.processes.insert(id, info);

        id
    }

    /// Spawn a renderer process for a site
    pub fn spawn_renderer_for_site(&mut self, origin: &str) -> u32 {
        // Check if we already have a renderer for this site
        if let Some(&id) = self.site_to_process.get(origin) {
            if self
                .processes
                .get(&id)
                .map(|p| p.state == ProcessState::Running)
                .unwrap_or(false)
            {
                return id;
            }
        }

        let id = self.spawn(ProcessType::Renderer);
        if let Some(info) = self.processes.get_mut(&id) {
            info.site_origin = Some(origin.to_string());
        }
        self.site_to_process.insert(origin.to_string(), id);

        id
    }

    /// Get process info
    pub fn get_process(&self, id: u32) -> Option<&ProcessInfo> {
        self.processes.get(&id)
    }

    /// Get all processes
    pub fn all_processes(&self) -> impl Iterator<Item = &ProcessInfo> {
        self.processes.values()
    }

    /// Get processes by type
    pub fn processes_by_type(&self, process_type: ProcessType) -> Vec<&ProcessInfo> {
        self.processes
            .values()
            .filter(|p| p.process_type == process_type)
            .collect()
    }

    /// Terminate a process
    pub fn terminate(&mut self, id: u32) {
        if let Some(info) = self.processes.get_mut(&id) {
            info.state = ProcessState::Terminated;
            if let Some(origin) = &info.site_origin {
                self.site_to_process.remove(origin);
            }
        }
    }

    /// Mark process as crashed
    pub fn mark_crashed(&mut self, id: u32) {
        if let Some(info) = self.processes.get_mut(&id) {
            info.state = ProcessState::Crashed;
        }
    }

    /// Get process count
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }

    /// Get renderer count
    pub fn renderer_count(&self) -> usize {
        self.processes_by_type(ProcessType::Renderer).len()
    }

    /// Update process stats
    pub fn update_stats(&mut self, id: u32, memory: usize, cpu: f32) {
        if let Some(info) = self.processes.get_mut(&id) {
            info.memory_usage = memory;
            info.cpu_usage = cpu;
        }
    }

    /// Set max renderer processes
    pub fn set_max_renderers(&mut self, max: usize) {
        self.max_renderer_processes = max;
    }

    /// Clean up terminated processes
    pub fn cleanup(&mut self) {
        self.processes
            .retain(|_, p| p.state != ProcessState::Terminated);
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_manager_spawn() {
        let mut manager = ProcessManager::new();
        let id = manager.spawn(ProcessType::Renderer);
        assert!(manager.get_process(id).is_some());
    }

    #[test]
    fn test_process_manager_spawn_for_site() {
        let mut manager = ProcessManager::new();
        let id1 = manager.spawn_renderer_for_site("https://example.com");
        let id2 = manager.spawn_renderer_for_site("https://example.com");
        // Same site should reuse process
        assert_eq!(id1, id2);

        let id3 = manager.spawn_renderer_for_site("https://other.com");
        // Different site should get new process
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_process_type_str() {
        assert_eq!(ProcessType::Browser.as_str(), "Browser");
        assert_eq!(ProcessType::Renderer.as_str(), "Renderer");
    }

    #[test]
    fn test_process_terminate() {
        let mut manager = ProcessManager::new();
        let id = manager.spawn(ProcessType::Renderer);
        manager.terminate(id);
        assert_eq!(
            manager.get_process(id).unwrap().state,
            ProcessState::Terminated
        );
    }

    #[test]
    fn test_process_count() {
        let mut manager = ProcessManager::new();
        manager.spawn(ProcessType::Renderer);
        manager.spawn(ProcessType::Renderer);
        manager.spawn(ProcessType::Gpu);
        assert_eq!(manager.process_count(), 3);
        assert_eq!(manager.renderer_count(), 2);
    }

    #[test]
    fn test_process_cleanup() {
        let mut manager = ProcessManager::new();
        let id = manager.spawn(ProcessType::Renderer);
        manager.terminate(id);
        manager.cleanup();
        assert_eq!(manager.process_count(), 0);
    }
}
