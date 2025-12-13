//! Error types for Binix browser

use std::fmt;

/// Main error type for Binix browser operations
#[derive(Debug)]
pub enum BinixError {
    /// Network-related errors
    Network(NetworkError),
    /// Rendering/parsing errors
    Render(RenderError),
    /// JavaScript execution errors
    JavaScript(JsError),
    /// I/O errors
    Io(std::io::Error),
    /// Generic error with message
    Other(String),
}

/// Network-specific errors
#[derive(Debug)]
pub enum NetworkError {
    /// DNS resolution failed
    DnsResolution(String),
    /// Connection timed out
    Timeout,
    /// TLS/SSL error
    Tls(String),
    /// HTTP error with status code
    Http(u16, String),
    /// Connection refused
    ConnectionRefused,
    /// Invalid URL
    InvalidUrl(String),
}

/// Rendering-specific errors
#[derive(Debug)]
pub enum RenderError {
    /// HTML parsing error
    HtmlParse(String),
    /// CSS parsing error
    CssParse(String),
    /// Layout computation error
    Layout(String),
    /// Resource loading error
    ResourceLoad(String),
}

/// JavaScript-specific errors
#[derive(Debug)]
pub enum JsError {
    /// Script execution error
    Execution(String),
    /// Script compilation error
    Compilation(String),
    /// Runtime error
    Runtime(String),
}

impl fmt::Display for BinixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {:?}", e),
            Self::Render(e) => write!(f, "Render error: {:?}", e),
            Self::JavaScript(e) => write!(f, "JavaScript error: {:?}", e),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for BinixError {}

impl From<std::io::Error> for BinixError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<NetworkError> for BinixError {
    fn from(err: NetworkError) -> Self {
        Self::Network(err)
    }
}

impl From<RenderError> for BinixError {
    fn from(err: RenderError) -> Self {
        Self::Render(err)
    }
}

impl From<JsError> for BinixError {
    fn from(err: JsError) -> Self {
        Self::JavaScript(err)
    }
}

/// Convenience Result type for Binix operations
pub type Result<T> = std::result::Result<T, BinixError>;

