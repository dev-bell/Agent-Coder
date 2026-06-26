use std::io::{self, Write};
use serde_json;

pub fn log_user_query(query: &str) {
    println!("user: [Query] {}", query);
}

pub fn log_user_observation(observation: &str) {
    println!("user: [Observation]\n{}", observation);
}

pub fn log_assistant_field(field_name: &str, field_content: &str) {
    println!("assistant: [{}] {}", field_name, field_content);
}

pub fn log_agent_error(error: &str) {
    println!("agent: [Error] {}", error);
}

pub fn log_agent_info(info: &str) {
    println!("agent: [Info] {}", info);
}

pub fn log_agent_new_with_type_quit() {
    print!("agent [New](Enter \"/quit\" to quit): ");
    io::stdout().flush().unwrap();
}

pub fn log_agent_new_with_enter_as_quit() {
    print!("agent [New](Enter to quit): ");
    io::stdout().flush().unwrap();
}

pub fn log_reason_prompt() {
    print!("agent: [Reason](Enter to skip): ");
    io::stdout().flush().unwrap();
}

pub fn log_tool_call_rejected(tool_name: &str, args: &str) {
    let display_args = if tool_name == "write_file" {
        match serde_json::from_str::<serde_json::Value>(args) {
            Ok(mut json) => {
                if let Some(obj) = json.as_object_mut() {
                    if obj.contains_key("content") {
                        obj.insert("content".to_string(), serde_json::Value::String("<complex content>".to_string()));
                        serde_json::to_string(&json).unwrap_or_else(|_| args.to_string())
                    } else {
                        args.to_string()
                    }
                } else {
                    args.to_string()
                }
            }
            Err(_) => args.to_string(),
        }
    } else {
        args.to_string()
    };

    println!(
        "agent: [Info] The tool call {}({}) has been rejected by the user.",
        tool_name, display_args
    );
}

pub fn print_confirmation_tool_execution(tool_name: &str, args: &str) {
    let display_args = if tool_name == "write_file" {
        match serde_json::from_str::<serde_json::Value>(args) {
            Ok(mut json) => {
                if let Some(obj) = json.as_object_mut() {
                    if obj.contains_key("content") {
                        obj.insert("content".to_string(), serde_json::Value::String("<complex content>".to_string()));
                        serde_json::to_string(&json).unwrap_or_else(|_| args.to_string())
                    } else {
                        args.to_string()
                    }
                } else {
                    args.to_string()
                }
            }
            Err(_) => args.to_string(),
        }
    } else {
        args.to_string()
    };

    print!(
        "agent: [Confirmation] The assistant is trying to call {}({}), would you like to execute?(y/n)",
        tool_name, display_args
    );
    io::stdout().flush().unwrap();
}

pub fn print_confirmation_retry() {
    print!("agent: [Confirmation] Would you like to retry?(y/n)");
    io::stdout().flush().unwrap();
}