use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{ChatCompletionRequestMessage};
use std::env;
use super::{build_request, tools_available, content_response_format, parse_response, LLMClient, LLMResponse, LLMErrors};

impl LLMClient {
    pub fn new() -> Self {
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set in environment");
        let base_url = env::var("OPENAI_BASE_URL").expect("OPENAI_BASE_URL must be set in environment");
        let model = env::var("OPENAI_MODEL_NAME").expect("OPENAI_MODEL_NAME must be set in environment");
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);
        let client = Client::with_config(config);
        let tools = tools_available();
        let response_format = content_response_format();
        Self { client, model, tools, response_format }
    }

    pub async fn chat(
        &self,
        messages: &Vec<ChatCompletionRequestMessage>,
    ) -> Result<LLMResponse, LLMErrors> {
        let request = build_request(&self.model, messages, &self.tools, &self.response_format)?;
        let response = self.client
            .chat()
            .create(request)
            .await?;
        parse_response(response)
    }
}