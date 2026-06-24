use async_openai::types::chat::ChatCompletionRequestMessage;
use crate::llm::{LLMErrors};
use crate::console;
use super::structs::Agent;
use super::messages::{
    load_system_message,
    build_user_message,
    build_tool_message,
    build_assistant_message,
};
use super::executor::{parse_tool, execute_tool};

impl Agent {
    pub async fn run(&mut self) {
        console::log_user_query(&self.query);

        let mut new_request_messages: Vec<ChatCompletionRequestMessage> = Vec::new();
        let mut messages_to_be_passed: Vec<ChatCompletionRequestMessage> = Vec::new();
        messages_to_be_passed.push(load_system_message());
        messages_to_be_passed.extend_from_slice(&self.selected_history);
        let query_msg = build_user_message(self.query.clone());
        self.append(&mut new_request_messages, &mut messages_to_be_passed, &query_msg);

        loop {
            let new_response_message = self.llm.chat(&messages_to_be_passed).await;

            match new_response_message {
                Err(e) => {
                    console::log_agent_error(&e.to_string());

                    match e {
                        LLMErrors::SafetyRefusal(_) => {
                            self.rollback(&mut new_request_messages, &mut messages_to_be_passed);
                            console::log_agent_new_with_enter_as_quit();
                            let new_query = console::read_line();
                            if new_query.trim().is_empty() {
                                console::log_agent_info("The query ends.");
                                return;
                            } else {
                                let msg = build_user_message(new_query);
                                self.append(&mut new_request_messages, &mut messages_to_be_passed, &msg);
                                continue;
                            }
                        }

                        LLMErrors::OpenAIError(_) => {
                            loop {
                                console::print_confirmation_retry();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    break;
                                } else if instruction == "n" {
                                    self.rollback(&mut new_request_messages, &mut messages_to_be_passed);
                                    console::log_agent_new_with_enter_as_quit();
                                    let new_query = console::read_line();
                                    if new_query.trim().is_empty() {
                                        console::log_agent_info("The query ends.");
                                        return;
                                    } else {
                                        let msg = build_user_message(new_query);
                                        self.append(&mut new_request_messages, &mut messages_to_be_passed, &msg);
                                        break;
                                    }
                                } else {
                                    console::log_agent_error("The input must be y or n.");
                                }
                            }
                            continue;
                        }
                    }
                }

                Ok(response) => {
                    new_request_messages.clear();

                    let assistant_msg = build_assistant_message(&response);
                    messages_to_be_passed.push(assistant_msg.clone());
                    self.conversation.messages.push(assistant_msg);

                    let raw:&str = response.content.as_str().trim();
                    if !raw.is_empty() {
                        console::log_assistant_field("Reply", raw);
                    }

                    if let Some(tool_calls) = response.tool_calls {
                        for tc in tool_calls {
                            let tools_for_execute = parse_tool(&tc);
                            loop {
                                console::print_confirmation_tool_execution(
                                    &tools_for_execute.name,
                                    &tools_for_execute.arguments
                                );
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    let output = match execute_tool(&tools_for_execute, &self.project_root) {
                                        Ok(out) => out,
                                        Err(e) => e.to_string(),
                                    };
                                    console::log_user_observation(&output);
                                    let tool_msg = build_tool_message(output, tools_for_execute.id);
                                    self.append(&mut new_request_messages, &mut messages_to_be_passed, &tool_msg);
                                    break;
                                } else if instruction == "n" {
                                    console::log_reason_prompt();
                                    let reason_read = console::read_line();
                                    let reason:&str = reason_read.trim();
                                    console::log_tool_call_rejected(&tools_for_execute.name, &tools_for_execute.arguments);
                                    let tool_msg_content = if reason.is_empty() {
                                        format!(
                                            "The tool call {}({}) has been rejected by the user.",
                                            tools_for_execute.name, tools_for_execute.arguments
                                        )
                                    } else {
                                        format!(
                                            "The tool call {}({}) has been rejected by the user. And the reason is that {}.",
                                            tools_for_execute.name, tools_for_execute.arguments, reason
                                        )
                                    };
                                    let tool_msg = build_tool_message(tool_msg_content, tools_for_execute.id);
                                    self.append(&mut new_request_messages, &mut messages_to_be_passed, &tool_msg);
                                    break;
                                } else {
                                    console::log_agent_error("The input must be y or n.");
                                }
                            }
                        }
                    }

                    console::log_agent_new_with_type_quit();
                    let new_query = console::read_line();
                    if new_query.trim().is_empty() {
                    } else if new_query == "/quit" {
                        console::log_agent_info("The query ends.");
                        return;
                    } else {
                        let msg = build_user_message(new_query);
                        self.append(&mut new_request_messages, &mut messages_to_be_passed, &msg);
                    }
                    continue;
                }
            }
        }
    }
}