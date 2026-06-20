use async_openai::types::chat::{
    CreateChatCompletionResponse,
    ChatCompletionMessageToolCalls,
};
use serde_json::Value;
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

    let content_str = message.content.unwrap_or_default();

    let tool_calls = message.tool_calls.map(|tc_vec| {
        tc_vec
            .into_iter()
            .filter_map(|tc| match tc {
                ChatCompletionMessageToolCalls::Function(tc) => Some(tc),
                ChatCompletionMessageToolCalls::Custom(_) => None,
            })
            .collect()
    });

    let (status, content) = match serde_json::from_str::<Value>(&content_str) {
        Ok(json_value) => {
            if let Some(obj) = json_value.as_object() {
                let has_final = obj
                    .get("Final Answer")
                    .map(|v| !v.is_null())
                    .unwrap_or(false);

                if has_final {
                    ("Final Answer Detected".to_string(), json_value.clone())
                } else {
                    ("Final Answer Not Detected".to_string(), json_value.clone())
                }
            } else {
                ("Not JSON".to_string(), Value::String(content_str))
            }
        }
        Err(_) => {
            ("Not JSON".to_string(), Value::String(content_str))
        }
    };

    Ok(LLMResponse {
        status,
        content,
        tool_calls,
    })
}