use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinixError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Layout error: {0}")]
    Layout(String),
    #[error("Security violation: {0}")]
    Security(String),
}
pub type Result<T> = std::result::Result<T, BinixError>;