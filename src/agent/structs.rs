use async_openai::types::chat::ChatCompletionRequestMessage;
use std::path::PathBuf;
use crate::llm::LLMClient;
use crate::history::Conversation;

pub struct ToolsForExecute {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

pub struct Agent {
    pub llm: LLMClient,
    pub selected_history: Vec<ChatCompletionRequestMessage>,
    pub query: String,
    pub conversation: Conversation,
    pub project_root: PathBuf,
}
