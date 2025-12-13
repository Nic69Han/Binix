//! JavaScript console implementation

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Log level for console messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Log,
    Info,
    Warn,
    Error,
    Debug,
    Trace,
}

impl LogLevel {
    /// Get the display string for the log level
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Log => "LOG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

/// A console message
#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
    pub source: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl ConsoleMessage {
    /// Create a new console message
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            level,
            message: message.into(),
            timestamp,
            source: None,
            line: None,
            column: None,
        }
    }

    /// Set source location
    pub fn with_source(mut self, source: &str, line: u32, column: u32) -> Self {
        self.source = Some(source.to_string());
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

/// JavaScript console
pub struct Console {
    messages: VecDeque<ConsoleMessage>,
    max_messages: usize,
    filter_level: Option<LogLevel>,
}

impl Console {
    /// Create a new console
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 1000,
            filter_level: None,
        }
    }

    /// Log a message
    pub fn log(&mut self, message: impl Into<String>) {
        self.add_message(ConsoleMessage::new(LogLevel::Log, message));
    }

    /// Log an info message
    pub fn info(&mut self, message: impl Into<String>) {
        self.add_message(ConsoleMessage::new(LogLevel::Info, message));
    }

    /// Log a warning message
    pub fn warn(&mut self, message: impl Into<String>) {
        self.add_message(ConsoleMessage::new(LogLevel::Warn, message));
    }

    /// Log an error message
    pub fn error(&mut self, message: impl Into<String>) {
        self.add_message(ConsoleMessage::new(LogLevel::Error, message));
    }

    /// Log a debug message
    pub fn debug(&mut self, message: impl Into<String>) {
        self.add_message(ConsoleMessage::new(LogLevel::Debug, message));
    }

    /// Add a message to the console
    fn add_message(&mut self, message: ConsoleMessage) {
        if self.messages.len() >= self.max_messages {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get all messages
    pub fn messages(&self) -> impl Iterator<Item = &ConsoleMessage> {
        self.messages
            .iter()
            .filter(|m| self.filter_level.map(|f| m.level == f).unwrap_or(true))
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Set filter level
    pub fn set_filter(&mut self, level: Option<LogLevel>) {
        self.filter_level = level;
    }

    /// Set max messages
    pub fn set_max_messages(&mut self, max: usize) {
        self.max_messages = max;
        while self.messages.len() > max {
            self.messages.pop_front();
        }
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_log() {
        let mut console = Console::new();
        console.log("Hello, world!");
        assert_eq!(console.message_count(), 1);
    }

    #[test]
    fn test_console_levels() {
        let mut console = Console::new();
        console.log("log");
        console.info("info");
        console.warn("warn");
        console.error("error");
        console.debug("debug");
        assert_eq!(console.message_count(), 5);
    }

    #[test]
    fn test_console_clear() {
        let mut console = Console::new();
        console.log("message");
        console.clear();
        assert_eq!(console.message_count(), 0);
    }

    #[test]
    fn test_console_max_messages() {
        let mut console = Console::new();
        console.set_max_messages(3);
        for i in 0..5 {
            console.log(format!("Message {}", i));
        }
        assert_eq!(console.message_count(), 3);
    }

    #[test]
    fn test_log_level_str() {
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
    }
}
