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
use crate::llm::LLMResponse;
use std::fs;
use super::structs::Agent;

pub fn load_system_message() -> ChatCompletionRequestMessage {
    let content = fs::read_to_string("prompts/system.txt")
        .expect("Failed to read prompts/system.txt");
    let system_msg = ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text(content),
        name: None,
    };
    ChatCompletionRequestMessage::System(system_msg)
}

pub fn build_user_message(s: String) -> ChatCompletionRequestMessage {
    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(s),
        name: None,
    })
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

impl Agent {
    pub fn rollback(
        &mut self,
        new_req: &mut Vec<ChatCompletionRequestMessage>,
        msgs: &mut Vec<ChatCompletionRequestMessage>,
    ) {
        let count = new_req.len();
        if count > 0 {
            let start_msgs = msgs.len().saturating_sub(count);
            msgs.truncate(start_msgs);

            let start_conv = self.conversation.messages.len().saturating_sub(count);
            self.conversation.messages.truncate(start_conv);

            new_req.clear();
        }
    }

    pub fn append(
        &mut self,
        new_req: &mut Vec<ChatCompletionRequestMessage>,
        msgs: &mut Vec<ChatCompletionRequestMessage>,
        msg: &ChatCompletionRequestMessage,
    ) {
        new_req.push(msg.clone());
        msgs.push(msg.clone());
        self.conversation.messages.push(msg.clone());
    }
}