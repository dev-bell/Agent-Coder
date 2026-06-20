use async_openai::types::chat::{
    ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestToolMessage,
    ChatCompletionRequestToolMessageContent,
    ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent,
};
use async_openai::types::chat::ChatCompletionMessageToolCalls;
use crate::history::Conversation;
use crate::llm::LLMResponse;
use std::fs;

pub fn load_system_message() -> ChatCompletionRequestMessage {
    let content = fs::read_to_string("prompts/system.txt")
        .expect("Failed to read prompts/system.txt");
    let system_msg = ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text(content),
        name: None,
    };
    ChatCompletionRequestMessage::System(system_msg)
}

pub fn string_to_message(s: String) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(s),
        name: None,
    })
}

pub fn push_element(
    new_req: &mut Vec<ChatCompletionRequestMessage>,
    msgs: &mut Vec<ChatCompletionRequestMessage>,
    msg: ChatCompletionRequestMessage,
) {
    new_req.push(msg.clone());
    msgs.push(msg);
}

pub fn empty_elements(
    new_req: &mut Vec<ChatCompletionRequestMessage>,
    msgs: &mut Vec<ChatCompletionRequestMessage>,
) {
    let count = new_req.len();
    if count > 0 {
        let start = msgs.len().saturating_sub(count);
        msgs.truncate(start);
        new_req.clear();
    }
}

pub fn conversation_update(
    conv: &mut Conversation,
    new_req: &Vec<ChatCompletionRequestMessage>,
    resp: &LLMResponse,
) {
    for msg in new_req {
        conv.add_message(msg.clone());
    }

    let content_str = resp.content.to_string();
    let tool_calls_enum = resp.tool_calls.as_ref().map(|tc_vec| {
        tc_vec
            .iter()
            .map(|tc| ChatCompletionMessageToolCalls::Function(tc.clone()))
            .collect()
    });
    let assistant_msg = ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
        content: Some(ChatCompletionRequestAssistantMessageContent::Text(content_str)),
        tool_calls: tool_calls_enum,
        ..Default::default()
    });
    conv.add_message(assistant_msg);
}

pub fn build_tool_message(output: String, tool_call_id: String) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage::Tool(ChatCompletionRequestToolMessage {
        content: ChatCompletionRequestToolMessageContent::Text(output),
        tool_call_id,
    })
}

pub fn build_assistant_message(response: &LLMResponse) -> ChatCompletionRequestMessage {
    let content_str = response.content.to_string();
    let tool_calls_enum = response.tool_calls.as_ref().map(|tc_vec| {
        tc_vec
            .iter()
            .map(|tc| ChatCompletionMessageToolCalls::Function(tc.clone()))
            .collect()
    });

    ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
        content: Some(ChatCompletionRequestAssistantMessageContent::Text(content_str)),
        tool_calls: tool_calls_enum,
        ..Default::default()
    })
}