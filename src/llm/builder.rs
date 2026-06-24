use async_openai::{
    types::{
        chat::{
            ChatCompletionTools,
            ChatCompletionTool,
            CreateChatCompletionRequest,
            CreateChatCompletionRequestArgs,
            ChatCompletionRequestMessage
        },
        assistants::{FunctionObject},
    }
};
use super::{
    READ_FILE_SCHEMA,
    LIST_FILES_SCHEMA,
    WRITE_FILE_SCHEMA,
    RM_FILE_SCHEMA,
    GREP_SCHEMA,
    GIT_SCHEMA,
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

pub fn build_request(
    model: &String,
    messages: &Vec<ChatCompletionRequestMessage>,
    tools: &Option<Vec<ChatCompletionTools>>,
) -> Result<CreateChatCompletionRequest, LLMErrors> {
    Ok(
        CreateChatCompletionRequestArgs::default()
            .model(model.clone())
            .messages(messages.clone())
            .tools(tools.as_ref().cloned().unwrap())
            .build()?
    )
}