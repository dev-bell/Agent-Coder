use async_openai::types::chat::ChatCompletionRequestMessage;
use crate::llm::{LLMErrors};
use crate::console;
use super::structs::Agent;
use super::messages::{
    string_to_message,
    push_element,
    empty_elements,
    conversation_update,
    build_tool_message,
    load_system_message,
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
        let query_msg = string_to_message(self.query.clone());
        push_element(&mut new_request_messages, &mut messages_to_be_passed, query_msg);

        loop {
            let new_response_message = self.llm.chat(&messages_to_be_passed).await;

            match new_response_message {
                Err(e) => {
                    console::log_agent_error(&e.to_string());

                    match e {
                        LLMErrors::SafetyRefusal(_) => {
                            empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                            console::log_agent_new_with_quit(); // "agent: [New](Enter for quit) "
                            let new_query = console::read_line();
                            if new_query.trim().is_empty() {
                                console::log_agent_info("The query ends.");
                                return;
                            } else {
                                let msg = string_to_message(new_query);
                                push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                                continue;
                            }
                        }

                        LLMErrors::OpenAIError(_) => {
                            loop {
                                console::print_confirmation_retry(); // "agent: [Confirmation] Would you like to retry?(y/n) "
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    break;
                                } else if instruction == "n" {
                                    empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                                    console::log_agent_new_with_quit();
                                    let new_query = console::read_line();
                                    if new_query.trim().is_empty() {
                                        console::log_agent_info("The query ends.");
                                        return;
                                    } else {
                                        let msg = string_to_message(new_query);
                                        push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
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
                    match response.status.as_str() {
                        "Final Answer Not Detected" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            new_request_messages.clear();

                            let assistant_msg = build_assistant_message(&response);
                            messages_to_be_passed.push(assistant_msg);

                            if let Some(obj) = response.content.as_object() {
                                for (key, value) in obj {
                                    if !value.is_null() {
                                        let content_str = if value.is_string() {
                                            value.as_str().unwrap_or("").to_string()
                                        } else {
                                            value.to_string()
                                        };
                                        console::log_assistant_field(key, &content_str);
                                    }
                                }
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
                                            push_element(&mut new_request_messages, &mut messages_to_be_passed, tool_msg);
                                            break;
                                        } else if instruction == "n" {
                                            console::log_reason_prompt();
                                            let reason = console::read_line();
                                            console::log_tool_call_rejected(&tools_for_execute.name, &tools_for_execute.arguments);
                                            let msg = if reason.trim().is_empty() {
                                                string_to_message(format!(
                                                    "The tool call {}({}) has been rejected by the user.",
                                                    tools_for_execute.name, tools_for_execute.arguments
                                                ))
                                            } else {
                                                string_to_message(format!(
                                                    "The tool call {}({}) has been rejected by the user. And the reason is that {}.",
                                                    tools_for_execute.name, tools_for_execute.arguments, reason
                                                ))
                                            };
                                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                                            break;
                                        } else {
                                            console::log_agent_error("The input must be y or n.");
                                        }
                                    }
                                }
                            }

                            loop {
                                console::print_confirmation_append();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    loop {
                                        console::log_agent_new();
                                        let new_query = console::read_line();
                                        if new_query.trim().is_empty() {
                                            console::log_agent_error("The input cannot be whitespaces.");
                                            continue;
                                        } else {
                                            let msg = string_to_message(new_query);
                                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                                            break;
                                        }
                                    }
                                    break;
                                } else if instruction == "n" {
                                    break;
                                } else if instruction == "q" {
                                    console::log_agent_info("The query ends.");
                                    return;
                                } else {
                                    console::log_agent_error("The input must be y, n or q.");
                                }
                            }
                            continue;
                        }

                        "Final Answer Detected" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            new_request_messages.clear();

                            let assistant_msg = build_assistant_message(&response);
                            messages_to_be_passed.push(assistant_msg);

                            if let Some(obj) = response.content.as_object() {
                                for (key, value) in obj {
                                    if !value.is_null() {
                                        let content_str = if value.is_string() {
                                            value.as_str().unwrap_or("").to_string()
                                        } else {
                                            value.to_string()
                                        };
                                        console::log_assistant_field(key, &content_str);
                                    }
                                }
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
                                            push_element(&mut new_request_messages, &mut messages_to_be_passed, tool_msg);
                                            break;
                                        } else if instruction == "n" {
                                            console::log_reason_prompt();
                                            let reason = console::read_line();
                                            console::log_tool_call_rejected(&tools_for_execute.name, &tools_for_execute.arguments);
                                            let msg = if reason.trim().is_empty() {
                                                string_to_message(format!(
                                                    "The tool call {}({}) has been rejected by the user.",
                                                    tools_for_execute.name, tools_for_execute.arguments
                                                ))
                                            } else {
                                                string_to_message(format!(
                                                    "The tool call {}({}) has been rejected by the user. And the reason is that {}.",
                                                    tools_for_execute.name, tools_for_execute.arguments, reason
                                                ))
                                            };
                                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                                            break;
                                        } else {
                                            console::log_agent_error("The input must be y or n.");
                                        }
                                    }
                                }
                            }

                            for msg in &new_request_messages {
                                self.conversation.add_message(msg.clone());
                            }
                            new_request_messages.clear();

                            console::log_agent_info("The query ends.");
                            return;
                        }

                        "Not JSON" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            new_request_messages.clear();

                            let assistant_msg = build_assistant_message(&response);
                            messages_to_be_passed.push(assistant_msg);

                            let raw = response.content.as_str().unwrap_or("").to_string();
                            console::log_assistant_field("Raw data", &raw);

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
                                            push_element(&mut new_request_messages, &mut messages_to_be_passed, tool_msg);
                                            break;
                                        } else if instruction == "n" {
                                            console::log_reason_prompt();
                                            let reason = console::read_line();
                                            console::log_tool_call_rejected(&tools_for_execute.name, &tools_for_execute.arguments);
                                            let msg = if reason.trim().is_empty() {
                                                string_to_message(format!(
                                                    "The tool call {}({}) has been rejected by the user.",
                                                    tools_for_execute.name, tools_for_execute.arguments
                                                ))
                                            } else {
                                                string_to_message(format!(
                                                    "The tool call {}({}) has been rejected by the user. And the reason is that {}.",
                                                    tools_for_execute.name, tools_for_execute.arguments, reason
                                                ))
                                            };
                                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                                            break;
                                        } else {
                                            console::log_agent_error("The input must be y or n.");
                                        }
                                    }
                                }
                            }

                            loop {
                                console::print_confirmation_append();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    loop {
                                        console::log_agent_new();
                                        let new_query = console::read_line();
                                        if new_query.trim().is_empty() {
                                            console::log_agent_error("The input cannot be whitespaces.");
                                            continue;
                                        } else {
                                            let msg = string_to_message(new_query);
                                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                                            break;
                                        }
                                    }
                                    break;
                                } else if instruction == "n" {
                                    break;
                                } else if instruction == "q" {
                                    console::log_agent_info("The query ends.");
                                    return;
                                } else {
                                    console::log_agent_error("The input must be y, n or q.");
                                }
                            }
                            continue;
                        }

                        _ => {
                            console::log_agent_error(&format!("Unexpected status: {}", response.status));
                            continue;
                        }
                    }
                }
            }
        }
    }
}