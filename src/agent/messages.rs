use async_openai::types::chat::{
    ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestToolMessage,
    ChatCompletionRequestToolMessageContent,
    ChatCompletionRequestAssistantMessageArgs,
};
use async_openai::types::chat::ChatCompletionMessageToolCalls;
use crate::history::Conversation;
use crate::llm::LLMResponse;

pub fn build_messages_to_be_passed(
    selected_history: &[ChatCompletionRequestMessage],
    conversation: &Conversation,
    new_request_messages: &[ChatCompletionRequestMessage],
) -> Vec<ChatCompletionRequestMessage> {
    let mut messages = Vec::new();
    messages.extend_from_slice(selected_history);
    messages.extend(conversation.messages.clone());
    messages.extend_from_slice(new_request_messages);
    messages
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
        name: None,
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

pub fn build_refusal_message(refusal: &str) -> ChatCompletionRequestMessage {
    let assistant_msg = ChatCompletionRequestAssistantMessageArgs::default()
        .refusal(refusal.to_string())
        .build()
        .expect("Failed to build refusal message");
    ChatCompletionRequestMessage::Assistant(assistant_msg)
}