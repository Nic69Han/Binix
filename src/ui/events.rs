//! Event handling system for the browser
//!
//! Provides a unified event system for DOM events, keyboard, mouse, and custom events.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Event types supported by the browser
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    // Mouse events
    Click,
    DoubleClick,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseEnter,
    MouseLeave,
    ContextMenu,

    // Keyboard events
    KeyDown,
    KeyUp,
    KeyPress,

    // Focus events
    Focus,
    Blur,
    FocusIn,
    FocusOut,

    // Form events
    Input,
    Change,
    Submit,
    Reset,

    // Scroll events
    Scroll,
    Wheel,

    // Touch events
    TouchStart,
    TouchMove,
    TouchEnd,
    TouchCancel,

    // Window events
    Resize,
    Load,
    Unload,
    BeforeUnload,

    // Custom event
    Custom(String),
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

/// Keyboard modifier keys
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

/// Mouse event data
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub x: f32,
    pub y: f32,
    pub client_x: f32,
    pub client_y: f32,
    pub page_x: f32,
    pub page_y: f32,
    pub button: MouseButton,
    pub buttons: u8,
    pub modifiers: Modifiers,
}

impl Default for MouseEvent {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            client_x: 0.0,
            client_y: 0.0,
            page_x: 0.0,
            page_y: 0.0,
            button: MouseButton::Left,
            buttons: 0,
            modifiers: Modifiers::default(),
        }
    }
}

/// Keyboard event data
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub key: String,
    pub code: String,
    pub modifiers: Modifiers,
    pub repeat: bool,
}

impl Default for KeyboardEvent {
    fn default() -> Self {
        Self {
            key: String::new(),
            code: String::new(),
            modifiers: Modifiers::default(),
            repeat: false,
        }
    }
}

/// Scroll event data
#[derive(Debug, Clone)]
pub struct ScrollEvent {
    pub delta_x: f32,
    pub delta_y: f32,
    pub delta_z: f32,
}

impl Default for ScrollEvent {
    fn default() -> Self {
        Self {
            delta_x: 0.0,
            delta_y: 0.0,
            delta_z: 0.0,
        }
    }
}

/// Touch point
#[derive(Debug, Clone)]
pub struct TouchPoint {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub force: f32,
}

/// Touch event data
#[derive(Debug, Clone, Default)]
pub struct TouchEvent {
    pub touches: Vec<TouchPoint>,
    pub changed_touches: Vec<TouchPoint>,
    pub target_touches: Vec<TouchPoint>,
}

/// Unified event data
#[derive(Debug, Clone)]
pub enum EventData {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Scroll(ScrollEvent),
    Touch(TouchEvent),
    Focus,
    Resize { width: u32, height: u32 },
    Custom(String),
    None,
}

/// A DOM event
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub data: EventData,
    pub target_id: Option<u64>,
    pub bubbles: bool,
    pub cancelable: bool,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    pub timestamp: u64,
}

impl Event {
    /// Create a new event
    pub fn new(event_type: EventType, data: EventData) -> Self {
        Self {
            event_type,
            data,
            target_id: None,
            bubbles: true,
            cancelable: true,
            default_prevented: false,
            propagation_stopped: false,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Create a click event
    pub fn click(x: f32, y: f32) -> Self {
        Self::new(
            EventType::Click,
            EventData::Mouse(MouseEvent {
                x,
                y,
                client_x: x,
                client_y: y,
                ..Default::default()
            }),
        )
    }

    /// Create a key down event
    pub fn key_down(key: &str, code: &str, modifiers: Modifiers) -> Self {
        Self::new(
            EventType::KeyDown,
            EventData::Keyboard(KeyboardEvent {
                key: key.to_string(),
                code: code.to_string(),
                modifiers,
                repeat: false,
            }),
        )
    }

    /// Create a scroll event
    pub fn scroll(delta_x: f32, delta_y: f32) -> Self {
        Self::new(
            EventType::Scroll,
            EventData::Scroll(ScrollEvent {
                delta_x,
                delta_y,
                delta_z: 0.0,
            }),
        )
    }

    /// Prevent default action
    pub fn prevent_default(&mut self) {
        if self.cancelable {
            self.default_prevented = true;
        }
    }

    /// Stop event propagation
    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }
}

/// Event handler callback type
pub type EventHandler = Arc<dyn Fn(&Event) + Send + Sync>;

/// Event listener registration
struct EventListener {
    handler: EventHandler,
    capture: bool,
}

/// Event dispatcher for managing event listeners
pub struct EventDispatcher {
    listeners: HashMap<EventType, Vec<EventListener>>,
    target_listeners: HashMap<u64, HashMap<EventType, Vec<EventListener>>>,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            target_listeners: HashMap::new(),
        }
    }

    /// Add a global event listener
    pub fn add_listener(&mut self, event_type: EventType, handler: EventHandler, capture: bool) {
        let listener = EventListener { handler, capture };
        self.listeners
            .entry(event_type)
            .or_default()
            .push(listener);
    }

    /// Add an event listener for a specific target
    pub fn add_target_listener(
        &mut self,
        target_id: u64,
        event_type: EventType,
        handler: EventHandler,
        capture: bool,
    ) {
        let listener = EventListener { handler, capture };
        self.target_listeners
            .entry(target_id)
            .or_default()
            .entry(event_type)
            .or_default()
            .push(listener);
    }

    /// Remove all listeners for a target
    pub fn remove_target_listeners(&mut self, target_id: u64) {
        self.target_listeners.remove(&target_id);
    }

    /// Dispatch an event
    pub fn dispatch(&self, event: &Event) {
        // Capture phase (global listeners with capture=true)
        if let Some(listeners) = self.listeners.get(&event.event_type) {
            for listener in listeners.iter().filter(|l| l.capture) {
                if event.propagation_stopped {
                    return;
                }
                (listener.handler)(event);
            }
        }

        // Target phase
        if let Some(target_id) = event.target_id {
            if let Some(target_listeners) = self.target_listeners.get(&target_id) {
                if let Some(listeners) = target_listeners.get(&event.event_type) {
                    for listener in listeners {
                        if event.propagation_stopped {
                            return;
                        }
                        (listener.handler)(event);
                    }
                }
            }
        }

        // Bubble phase (global listeners with capture=false)
        if event.bubbles {
            if let Some(listeners) = self.listeners.get(&event.event_type) {
                for listener in listeners.iter().filter(|l| !l.capture) {
                    if event.propagation_stopped {
                        return;
                    }
                    (listener.handler)(event);
                }
            }
        }
    }

    /// Get listener count for an event type
    pub fn listener_count(&self, event_type: &EventType) -> usize {
        self.listeners.get(event_type).map(|l| l.len()).unwrap_or(0)
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe event queue
pub struct EventQueue {
    events: Arc<Mutex<Vec<Event>>>,
}

impl EventQueue {
    /// Create a new event queue
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Push an event to the queue
    pub fn push(&self, event: Event) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    /// Pop all events from the queue
    pub fn drain(&self) -> Vec<Event> {
        if let Ok(mut events) = self.events.lock() {
            std::mem::take(&mut *events)
        } else {
            Vec::new()
        }
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.lock().map(|e| e.is_empty()).unwrap_or(true)
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.events.lock().map(|e| e.len()).unwrap_or(0)
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventQueue {
    fn clone(&self) -> Self {
        Self {
            events: Arc::clone(&self.events),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_creation() {
        let event = Event::click(100.0, 200.0);
        assert_eq!(event.event_type, EventType::Click);
        assert!(event.bubbles);
        assert!(event.cancelable);
    }

    #[test]
    fn test_prevent_default() {
        let mut event = Event::click(0.0, 0.0);
        assert!(!event.default_prevented);
        event.prevent_default();
        assert!(event.default_prevented);
    }

    #[test]
    fn test_stop_propagation() {
        let mut event = Event::click(0.0, 0.0);
        assert!(!event.propagation_stopped);
        event.stop_propagation();
        assert!(event.propagation_stopped);
    }

    #[test]
    fn test_event_dispatcher() {
        let mut dispatcher = EventDispatcher::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        dispatcher.add_listener(
            EventType::Click,
            Arc::new(move |_| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }),
            false,
        );

        let event = Event::click(0.0, 0.0);
        dispatcher.dispatch(&event);

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_event_queue() {
        let queue = EventQueue::new();
        assert!(queue.is_empty());

        queue.push(Event::click(0.0, 0.0));
        queue.push(Event::scroll(0.0, 10.0));

        assert_eq!(queue.len(), 2);

        let events = queue.drain();
        assert_eq!(events.len(), 2);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_keyboard_event() {
        let event = Event::key_down("a", "KeyA", Modifiers { ctrl: true, ..Default::default() });
        assert_eq!(event.event_type, EventType::KeyDown);
        if let EventData::Keyboard(kbd) = &event.data {
            assert_eq!(kbd.key, "a");
            assert!(kbd.modifiers.ctrl);
        } else {
            panic!("Expected keyboard event data");
        }
    }

    #[test]
    fn test_target_listeners() {
        let mut dispatcher = EventDispatcher::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        dispatcher.add_target_listener(
            42,
            EventType::Click,
            Arc::new(move |_| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }),
            false,
        );

        let mut event = Event::click(0.0, 0.0);
        event.target_id = Some(42);
        dispatcher.dispatch(&event);

        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // Event with different target should not trigger
        let mut event2 = Event::click(0.0, 0.0);
        event2.target_id = Some(99);
        dispatcher.dispatch(&event2);

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}

