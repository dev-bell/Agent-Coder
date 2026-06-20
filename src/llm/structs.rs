use async_openai::types::chat::ChatCompletionMessageToolCall;
use async_openai::{Client, config::OpenAIConfig};
use async_openai::types::{
    chat::ChatCompletionTools,
    assistants::ResponseFormat,
};

pub struct LLMClient {
    pub client: Client<OpenAIConfig>,
    pub model: String,
    pub tools: Option<Vec<ChatCompletionTools>>, 
    pub response_format: Option<ResponseFormat>
}

pub struct LLMResponse {
    pub status: String,
    pub content: serde_json::Value,
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
}