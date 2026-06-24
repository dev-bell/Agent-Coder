use async_openai::types::chat::{
    CreateChatCompletionResponse,
    ChatCompletionMessageToolCalls,
};
use super::{LLMResponse, LLMErrors};

pub fn parse_response(response: CreateChatCompletionResponse) -> Result<LLMResponse, LLMErrors> {
    let choice = response
        .choices
        .into_iter()
        .next()
        .unwrap();

    let message = choice.message;

    if let Some(refusal) = message.refusal {
        return Err(LLMErrors::SafetyRefusal(refusal));
    }

    let content = message.content.unwrap_or_default();

    let tool_calls = message.tool_calls.map(|tc_vec| {
        tc_vec
            .into_iter()
            .filter_map(|tc| match tc {
                ChatCompletionMessageToolCalls::Function(tc) => Some(tc),
                ChatCompletionMessageToolCalls::Custom(_) => None,
            })
            .collect()
    });

    Ok(LLMResponse {
        content,
        tool_calls,
    })
}