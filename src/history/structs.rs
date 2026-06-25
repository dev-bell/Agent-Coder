use async_openai::types::chat::ChatCompletionRequestMessage;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub query: String,
    pub messages: Vec<ChatCompletionRequestMessage>,
}

impl Conversation {
    pub fn new(query: &str) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let start_time = Utc::now();
        Self {
            id,
            start_time,
            query: query.to_string(),
            messages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub conversations: Vec<Conversation>,
    pub path: PathBuf,
}

impl History {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            conversations: Vec::new(),
            path,
        }
    }
}