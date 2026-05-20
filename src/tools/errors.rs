use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolErrors {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Git error: {0}")]
    Git(String),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing argument: {0}")]
    MissingArgument(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}