use std::io::{self, Write};

pub fn log_user_query(query: &str) {
    println!("user: [Query] {}", query);
}

pub fn log_user_observation(observation: &str) {
    println!("user: [Observation] {}", observation);
}

pub fn log_assistant_thought(thought: &str) {
    println!("assistant: [Thought] {}", thought);
}

pub fn log_assistant_action(action_desc: &str) {
    println!("assistant: [Action] {}", action_desc);
}

pub fn log_assistant_final_answer(answer: &str) {
    println!("assistant: [Final Answer] {}", answer);
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

pub fn log_agent_new() {
    print!("agent: [new]: ");
    io::stdout().flush().unwrap();
}

pub fn log_reason_prompt() {
    print!("agent: [Reason](Could be empty): ");
    io::stdout().flush().unwrap();
}

pub fn log_tool_call_rejected(tool_name: &str, args: &str) {
    println!(
        "agent: [Info] The tool call {}({}) has been rejected by the user.",
        tool_name, args
    );
}

pub fn print_confirmation_continue() {
    print!("agent: [Confirmation] Would you like to continue?(y/n) ");
    io::stdout().flush().unwrap();
}

pub fn print_confirmation_tool_execution(tool_name: &str, args: &str) {
    print!(
        "agent: [Confirmation] The assistant is trying to call {}({}), would you like to execute?(y/n) ",
        tool_name, args
    );
    io::stdout().flush().unwrap();
}

pub fn print_confirmation_retry() {
    print!("agent: [Confirmation] Do you want to retry?(y/n/q) ");
    io::stdout().flush().unwrap();
}

pub fn print_confirmation_ask_reason() {
    print!("agent: [Confirmation] Would you like to ask the reason?(y/n/q) ");
    io::stdout().flush().unwrap();
}