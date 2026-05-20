use async_openai::{
    types::{
        chat::{
            ChatCompletionTools,
            ChatCompletionTool,
            CreateChatCompletionRequest,
            CreateChatCompletionRequestArgs,
            ChatCompletionRequestMessage
        },
        assistants::{FunctionObject, ResponseFormat},
    }
};
use super::{
    READ_FILE_SCHEMA,
    LIST_FILES_SCHEMA,
    WRITE_FILE_SCHEMA,
    RM_FILE_SCHEMA,
    GREP_SCHEMA,
    GIT_SCHEMA,
    ASSISTANT_RESPONSE_SCHEMA,
};
use super::LLMErrors;


fn parse_function_schema(schema_str: &str) -> FunctionObject {
    serde_json::from_str(schema_str)
        .expect("Invalid JSON Schema for tool definition")
}

pub fn tools_available() -> Option<Vec<ChatCompletionTools>> {
    Some(vec![
        ChatCompletionTools::Function(
            ChatCompletionTool {
                function: parse_function_schema(READ_FILE_SCHEMA),
            }
        ),
        ChatCompletionTools::Function(
            ChatCompletionTool {
                function: parse_function_schema(LIST_FILES_SCHEMA),
            }
        ),
        ChatCompletionTools::Function(
            ChatCompletionTool {
                function: parse_function_schema(WRITE_FILE_SCHEMA),
            }
        ),
        ChatCompletionTools::Function(
            ChatCompletionTool {
                function: parse_function_schema(RM_FILE_SCHEMA),
            }
        ),
        ChatCompletionTools::Function(
            ChatCompletionTool {
                function: parse_function_schema(GREP_SCHEMA),
            }
        ),
        ChatCompletionTools::Function(
            ChatCompletionTool {
                function: parse_function_schema(GIT_SCHEMA),
            }
        ),
    ])
}

pub fn content_response_format() -> Option<ResponseFormat> {
    let response_json_schema = serde_json::from_str(ASSISTANT_RESPONSE_SCHEMA).expect("Invalid ASSISTANT_RESPONSE_SCHEMA JSON");
    Some(ResponseFormat::JsonSchema { json_schema:response_json_schema })
}

pub fn build_request(
    model: &String,
    messages: &Vec<ChatCompletionRequestMessage>,
    tools: &Option<Vec<ChatCompletionTools>>,
    response_format: &Option<ResponseFormat>,
) -> Result<CreateChatCompletionRequest, LLMErrors> {
    Ok(
        CreateChatCompletionRequestArgs::default()
            .model(model.clone())
            .messages(messages.clone())
            .tools(tools.as_ref().cloned().unwrap())
            .response_format(response_format.as_ref().cloned().unwrap())
            .build()?
    )
}