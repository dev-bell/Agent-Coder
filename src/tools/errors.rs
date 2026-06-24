use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolErrors {
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Git error: {0}")]
    Git(String),
    #[error("Pattern error: {0}")]
    Pattern(String),
    #[error("Grep error: {0}")]
    Grep(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}