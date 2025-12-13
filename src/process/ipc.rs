//! Inter-process communication

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

/// IPC message types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcMessageType {
    /// Navigate to URL
    Navigate,
    /// Page loaded
    PageLoaded,
    /// Execute JavaScript
    ExecuteScript,
    /// Script result
    ScriptResult,
    /// DOM update
    DomUpdate,
    /// Network request
    NetworkRequest,
    /// Network response
    NetworkResponse,
    /// Render frame
    RenderFrame,
    /// Frame rendered
    FrameRendered,
    /// Process crash
    ProcessCrash,
    /// Shutdown
    Shutdown,
}

/// IPC message
#[derive(Debug, Clone)]
pub struct IpcMessage {
    pub msg_type: IpcMessageType,
    pub source_process: u32,
    pub target_process: u32,
    pub payload: Vec<u8>,
    pub sequence_id: u64,
}

impl IpcMessage {
    /// Create a new IPC message
    pub fn new(msg_type: IpcMessageType, source: u32, target: u32, payload: Vec<u8>) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        Self {
            msg_type,
            source_process: source,
            target_process: target,
            payload,
            sequence_id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        }
    }

    /// Create a navigate message
    pub fn navigate(source: u32, target: u32, url: &str) -> Self {
        Self::new(
            IpcMessageType::Navigate,
            source,
            target,
            url.as_bytes().to_vec(),
        )
    }

    /// Create a shutdown message
    pub fn shutdown(source: u32, target: u32) -> Self {
        Self::new(IpcMessageType::Shutdown, source, target, Vec::new())
    }

    /// Get payload as string
    pub fn payload_str(&self) -> Option<String> {
        String::from_utf8(self.payload.clone()).ok()
    }
}

/// IPC channel for communication between processes
pub struct IpcChannel {
    id: u32,
    sender: Sender<IpcMessage>,
    receiver: Arc<Mutex<Receiver<IpcMessage>>>,
}

impl IpcChannel {
    /// Create a new IPC channel pair
    pub fn pair(id1: u32, id2: u32) -> (Self, Self) {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        let channel1 = Self {
            id: id1,
            sender: tx2,
            receiver: Arc::new(Mutex::new(rx1)),
        };

        let channel2 = Self {
            id: id2,
            sender: tx1,
            receiver: Arc::new(Mutex::new(rx2)),
        };

        (channel1, channel2)
    }

    /// Send a message
    pub fn send(&self, message: IpcMessage) -> Result<(), String> {
        self.sender
            .send(message)
            .map_err(|e| format!("Failed to send IPC message: {}", e))
    }

    /// Receive a message (blocking)
    pub fn recv(&self) -> Result<IpcMessage, String> {
        self.receiver
            .lock()
            .map_err(|_| "Failed to lock receiver".to_string())?
            .recv()
            .map_err(|e| format!("Failed to receive IPC message: {}", e))
    }

    /// Try to receive a message (non-blocking)
    pub fn try_recv(&self) -> Option<IpcMessage> {
        self.receiver.lock().ok()?.try_recv().ok()
    }

    /// Get channel ID
    pub fn id(&self) -> u32 {
        self.id
    }
}

/// IPC router for managing multiple channels
pub struct IpcRouter {
    channels: HashMap<u32, Sender<IpcMessage>>,
}

impl IpcRouter {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    /// Register a channel
    pub fn register(&mut self, process_id: u32, sender: Sender<IpcMessage>) {
        self.channels.insert(process_id, sender);
    }

    /// Unregister a channel
    pub fn unregister(&mut self, process_id: u32) {
        self.channels.remove(&process_id);
    }

    /// Route a message to target process
    pub fn route(&self, message: IpcMessage) -> Result<(), String> {
        if let Some(sender) = self.channels.get(&message.target_process) {
            sender
                .send(message)
                .map_err(|e| format!("Failed to route message: {}", e))
        } else {
            Err(format!("No channel for process {}", message.target_process))
        }
    }
}

impl Default for IpcRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_message_creation() {
        let msg = IpcMessage::navigate(1, 2, "https://example.com");
        assert_eq!(msg.msg_type, IpcMessageType::Navigate);
        assert_eq!(msg.source_process, 1);
        assert_eq!(msg.target_process, 2);
    }

    #[test]
    fn test_ipc_message_payload() {
        let msg = IpcMessage::navigate(1, 2, "https://example.com");
        assert_eq!(msg.payload_str(), Some("https://example.com".to_string()));
    }

    #[test]
    fn test_ipc_channel_pair() {
        let (ch1, ch2) = IpcChannel::pair(1, 2);
        assert_eq!(ch1.id(), 1);
        assert_eq!(ch2.id(), 2);
    }

    #[test]
    fn test_ipc_send_recv() {
        let (ch1, ch2) = IpcChannel::pair(1, 2);
        let msg = IpcMessage::navigate(1, 2, "https://example.com");

        ch1.send(msg.clone()).unwrap();
        let received = ch2.recv().unwrap();

        assert_eq!(received.msg_type, IpcMessageType::Navigate);
        assert_eq!(
            received.payload_str(),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_ipc_try_recv() {
        let (ch1, ch2) = IpcChannel::pair(1, 2);

        // Should be None when no message
        assert!(ch2.try_recv().is_none());

        ch1.send(IpcMessage::shutdown(1, 2)).unwrap();

        // Should have message now
        assert!(ch2.try_recv().is_some());
    }
}
