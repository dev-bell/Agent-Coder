use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum HistoryErrors {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Conversation not found: {0}")]
    ConversationNotFound(String),
    #[error("Message index out of range")]
    MessageIndexOutOfRange,
    #[error("File not found: {0}")]
    FileNotFound(PathBuf)
}