//! Multi-process architecture for site isolation
//!
//! Provides process-based isolation for security:
//! - Browser process: Main UI and coordination
//! - Renderer processes: One per site for isolation
//! - Network process: Handles all network requests
//! - GPU process: Handles GPU operations

mod ipc;
mod manager;
mod renderer_process;
mod sandbox;

pub use ipc::{IpcChannel, IpcMessage, IpcMessageType};
pub use manager::{ProcessManager, ProcessInfo, ProcessType};
pub use renderer_process::RendererProcess;
pub use sandbox::{Sandbox, SandboxPolicy};

