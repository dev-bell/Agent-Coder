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

    let (status, extra_fields, unformatted_fields, content) = match serde_json::from_str::<Value>(&content_str) {
        Ok(json_value) => {
            if let Some(obj) = json_value.as_object() {
                let allowed = ["Thought", "Action", "Final Answer"];

                let has_action = obj.contains_key("Action");
                let has_final = obj.contains_key("Final Answer");
                if has_action && has_final {
                    return Ok(LLMResponse {
                        status: "Both Action and Final Answer are detected".to_string(),
                        extra_fields: None,
                        unformatted_fields: None,
                        content: json_value.clone(),
                        tool_calls,
                    });
                }

                let extra: Vec<String> = obj
                    .keys()
                    .filter(|k| !allowed.contains(&k.as_str()))
                    .map(|s| s.to_string())
                    .collect();
                if !extra.is_empty() {
                    return Ok(LLMResponse {
                        status: "Extra Field".to_string(),
                        extra_fields: Some(extra),
                        unformatted_fields: None,
                        content: json_value.clone(),
                        tool_calls,
                    });
                }

                if obj.is_empty() {
                    return Ok(LLMResponse {
                        status: "No fields".to_string(),
                        extra_fields: None,
                        unformatted_fields: None,
                        content: json_value.clone(),
                        tool_calls,
                    });
                }

                if !obj.contains_key("Thought") {
                    return Ok(LLMResponse {
                        status: "Missing Thought".to_string(),
                        extra_fields: None,
                        unformatted_fields: None,
                        content: json_value.clone(),
                        tool_calls,
                    });
                }

                let has_action = obj.contains_key("Action");
                let has_final = obj.contains_key("Final Answer");
                if !has_action && !has_final {
                    return Ok(LLMResponse {
                        status: "Missing Action or Final Answer".to_string(),
                        extra_fields: None,
                        unformatted_fields: None,
                        content: json_value.clone(),
                        tool_calls,
                    });
                }

                let mut unformatted = Vec::new();
                for field in allowed.iter() {
                    if let Some(value) = obj.get(*field) {
                        if !value.is_string() {
                            unformatted.push(field.to_string());
                        }
                    }
                }
                if !unformatted.is_empty() {
                    return Ok(LLMResponse {
                        status: "Unformatted Fields".to_string(),
                        extra_fields: None,
                        unformatted_fields: Some(unformatted),
                        content: json_value.clone(),
                        tool_calls,
                    });
                }

                (
                    "Well-formatted".to_string(),
                    None,
                    None,
                    json_value.clone(),
                )
            } else {
                ("Not JSON".to_string(), None, None, Value::String(content_str))
            }
        }
        Err(_) => {
            ("Not JSON".to_string(), None, None, Value::String(content_str))
        }
    };

    Ok(LLMResponse {
        status,
        extra_fields,
        unformatted_fields,
        content,
        tool_calls,
    })
}