use thiserror::Error;
use async_openai::error::OpenAIError;

#[derive(Debug, Error)]
pub enum LLMErrors {
    #[error("OpenAI Error: {0}")]
    OpenAIError(#[from] OpenAIError),
    #[error("LLM refused to answer for safety: {0}")]
    SafetyRefusal(String),
}