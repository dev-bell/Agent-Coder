use async_openai::types::chat::ChatCompletionMessageToolCall;
use async_openai::{Client, config::OpenAIConfig};
use async_openai::types::{
    chat::ChatCompletionTools,
};

pub struct LLMClient {
    pub client: Client<OpenAIConfig>,
    pub model: String,
    pub tools: Option<Vec<ChatCompletionTools>>, 
}

pub struct LLMResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
}