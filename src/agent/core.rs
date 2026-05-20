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
    build_refusal_message,
    build_messages_to_be_passed,
};
use super::executor::{parse_tool, execute_tool};

impl Agent {
    pub async fn run(&mut self) {
        console::log_user_query(&self.query);

        let mut new_request_messages: Vec<ChatCompletionRequestMessage> = Vec::new();
        let mut messages_to_be_passed: Vec<ChatCompletionRequestMessage> = Vec::new();

        let query_msg = string_to_message(self.query.clone());
        push_element(&mut new_request_messages, &mut messages_to_be_passed, query_msg);

        messages_to_be_passed = build_messages_to_be_passed(
            &self.selected_history,
            &self.conversation,
            &new_request_messages,
        );

        loop {
            let new_response_message = self.llm.chat(&messages_to_be_passed).await;

            match new_response_message {
                Ok(response) => {
                    match response.status.as_str() {
                        "Well-formatted" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            empty_elements(&mut new_request_messages, &mut messages_to_be_passed);

                            if let Some(obj) = response.content.as_object() {
                                if let Some(thought) = obj.get("Thought").and_then(|v| v.as_str()) {
                                    console::log_assistant_thought(thought);
                                }
                                if let Some(action) = obj.get("Action").and_then(|v| v.as_str()) {
                                    console::log_assistant_action(action);
                                }
                                if let Some(final_ans) = obj.get("Final Answer").and_then(|v| v.as_str()) {
                                    console::log_assistant_final_answer(final_ans);
                                }
                            }

                            let has_final_answer = response.content.as_object()
                                .and_then(|obj| obj.get("Final Answer"))
                                .and_then(|v| v.as_str())
                                .is_some();

                            if has_final_answer {
                                console::log_agent_info("The query ends.");
                                return;
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
                                            console::log_agent_error("The input must be y/n.");
                                        }
                                    }
                                }
                            }
                            continue;
                        }
                        "Extra Field" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                            console::log_agent_error("The message sent by the assistant has extra field.");
                            if let Some(obj) = response.content.as_object() {
                                for (key, value) in obj {
                                    let content_str = if value.is_string() {
                                        value.as_str().unwrap_or("").to_string()
                                    } else {
                                        value.to_string()
                                    };
                                    console::log_assistant_field(key, &content_str);
                                }
                            }
                            let extra = response.extra_fields.unwrap_or_default().join(", ");
                            let msg = string_to_message(format!("Your content is not well-formatted with the extra fields: {}.", extra));
                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                            loop {
                                console::print_confirmation_continue();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    console::log_agent_info("The query ends.");
                                    return;
                                } else if instruction == "n" {
                                    break;
                                } else {
                                    console::log_agent_error("The input must be y/n.");
                                }
                            }
                            continue;
                        }
                        "Both Action and Final Answer are detected" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                            console::log_agent_error("The message sent by the assistant has both Action and Final Answer fields.");
                            if let Some(obj) = response.content.as_object() {
                                for (key, value) in obj {
                                    let content_str = if value.is_string() {
                                        value.as_str().unwrap_or("").to_string()
                                    } else {
                                        value.to_string()
                                    };
                                    console::log_assistant_field(key, &content_str);
                                }
                            }
                            let msg = string_to_message("Your content is not well-formatted with both Action and Final Answer fields detected.".to_string());
                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                            loop {
                                console::print_confirmation_continue();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    console::log_agent_info("The query ends.");
                                    return;
                                } else if instruction == "n" {
                                    break;
                                } else {
                                    console::log_agent_error("The input must be y/n.");
                                }
                            }
                            continue;
                        }
                        "No Fields" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                            console::log_agent_error("The message sent by the assistant has no fields detected.");
                            let msg = string_to_message("Your content is not well-formatted with no fields detected.".to_string());
                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                            loop {
                                console::print_confirmation_continue();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    console::log_agent_info("The query ends.");
                                    return;
                                } else if instruction == "n" {
                                    break;
                                } else {
                                    console::log_agent_error("The input must be y/n.");
                                }
                            }
                            continue;
                        }
                        "Missing Thought" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                            console::log_agent_error("The message sent by the assistant is missing the Thought field.");
                            if let Some(obj) = response.content.as_object() {
                                for (key, value) in obj {
                                    let content_str = if value.is_string() {
                                        value.as_str().unwrap_or("").to_string()
                                    } else {
                                        value.to_string()
                                    };
                                    console::log_assistant_field(key, &content_str);
                                }
                            }
                            let msg = string_to_message("Your content is not well-formatted because the Thought field is missing.".to_string());
                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                            loop {
                                console::print_confirmation_continue();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    console::log_agent_info("The query ends.");
                                    return;
                                } else if instruction == "n" {
                                    break;
                                } else {
                                    console::log_agent_error("The input must be y/n.");
                                }
                            }
                            continue;
                        }
                        "Missing Action or Final Answer" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                            console::log_agent_error("The message sent by the assistant is missing the Action or Final Answer field.");
                            if let Some(obj) = response.content.as_object() {
                                for (key, value) in obj {
                                    let content_str = if value.is_string() {
                                        value.as_str().unwrap_or("").to_string()
                                    } else {
                                        value.to_string()
                                    };
                                    console::log_assistant_field(key, &content_str);
                                }
                            }
                            let msg = string_to_message("Your content is not well-formatted because one of the Action or Final Answer field is missing.".to_string());
                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                            loop {
                                console::print_confirmation_continue();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    console::log_agent_info("The query ends.");
                                    return;
                                } else if instruction == "n" {
                                    break;
                                } else {
                                    console::log_agent_error("The input must be y/n.");
                                }
                            }
                            continue;
                        }
                        "Unformatted Fields" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                            console::log_agent_error("The message sent by the assistant has unformatted fields.");
                            if let Some(obj) = response.content.as_object() {
                                for (key, value) in obj {
                                    let content_str = if value.is_string() {
                                        value.as_str().unwrap_or("").to_string()
                                    } else {
                                        value.to_string()
                                    };
                                    console::log_assistant_field(key, &content_str);
                                }
                            }
                            let unformatted = response.unformatted_fields.unwrap_or_default().join(", ");
                            let msg = string_to_message(format!("Your content is not well-formatted with unformatted fields: {}.", unformatted));
                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);

                            let has_final_answer = response.content.as_object()
                                .and_then(|obj| obj.get("Final Answer"))
                                .and_then(|v| v.as_str())
                                .is_some();
                            if has_final_answer {
                                console::log_agent_info("The query ends.");
                                return;
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
                                            console::log_agent_error("The input must be y/n.");
                                        }
                                    }
                                }
                            }
                            continue;
                        }
                        "Not JSON" => {
                            conversation_update(&mut self.conversation, &new_request_messages, &response);
                            empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                            console::log_agent_error("The message sent by the assistant is not in JSON format.");
                            let raw = response.content.as_str().unwrap_or("").to_string();
                            console::log_assistant_field("Raw data", &raw);
                            let msg = string_to_message("Your content is not JSON format.".to_string());
                            push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                            loop {
                                console::print_confirmation_continue();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    console::log_agent_info("The query ends.");
                                    return;
                                } else if instruction == "n" {
                                    break;
                                } else {
                                    console::log_agent_error("The input must be y/n.");
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
                Err(e) => {
                    console::log_agent_error(&e.to_string());

                    match e {
                        LLMErrors::OpenAIError(_) => {
                            loop {
                                console::print_confirmation_retry();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    break;
                                } else if instruction == "n" {
                                    console::log_agent_new();
                                    let mut new_line = console::read_line();
                                    while new_line.trim().is_empty() {
                                        console::log_agent_error("The input must not be empty.");
                                        console::log_agent_new();
                                        new_line = console::read_line();
                                    }
                                    empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                                    let msg = string_to_message(new_line);
                                    push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                                    break;
                                } else if instruction == "q" {
                                    console::log_agent_info("The query ends.");
                                    empty_elements(&mut new_request_messages, &mut messages_to_be_passed);
                                    return;
                                } else {
                                    console::log_agent_error("The input must be y/n/q.");
                                }
                            }
                            continue;
                        }
                        LLMErrors::SafetyRefusal(refusal) => {
                            for msg in &new_request_messages {
                                self.conversation.add_message(msg.clone());
                            }
                            let refusal_msg = build_refusal_message(&refusal);
                            self.conversation.add_message(refusal_msg);
                            empty_elements(&mut new_request_messages, &mut messages_to_be_passed);

                            loop {
                                console::print_confirmation_ask_reason();
                                let instruction = console::read_line().to_lowercase();
                                if instruction == "y" {
                                    let ask_msg = string_to_message("What is the reason that you refused to answer?".to_string());
                                    push_element(&mut new_request_messages, &mut messages_to_be_passed, ask_msg);
                                    break;
                                } else if instruction == "n" {
                                    console::log_agent_new();
                                    let mut new_line = console::read_line();
                                    while new_line.trim().is_empty() {
                                        console::log_agent_error("The input must not be empty.");
                                        console::log_agent_new();
                                        new_line = console::read_line();
                                    }
                                    let msg = string_to_message(new_line);
                                    push_element(&mut new_request_messages, &mut messages_to_be_passed, msg);
                                    break;
                                } else if instruction == "q" {
                                    console::log_agent_info("The query ends.");
                                    return;
                                } else {
                                    console::log_agent_error("The input must be y/n/q.");
                                }
                            }
                            continue;
                        }
                    }
                }
            }
        }
    }
}