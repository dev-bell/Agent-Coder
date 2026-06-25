mod llm;
mod console;
mod history;
mod tools;
mod agent;

use std::path::{PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use std::io::Write;
use dotenv::dotenv;

use async_openai::types::chat::{
    ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestToolMessageContent,
    ChatCompletionMessageToolCalls
};

use llm::LLMClient;
use history::{History, Conversation, load_history, save_history};
use agent::Agent;

// ================================
// AppState
// ================================
struct AppState {
    project_path: Option<PathBuf>,
    history: Option<History>,
}

// ================================
// main
// ================================
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("Welcome to Agent Coder - ReAct agent for local projects.");
    println!("Type /help for available commands.\n");

    let state = Arc::new(Mutex::new(AppState {
        project_path: None,
        history: None,
    }));

    loop {
        print!("> ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.starts_with('/') {
            let parts: Vec<&str> = input.split_whitespace().collect();
            let cmd = parts[0];
            match cmd {
                "/help" => {
                    println!("Available commands:");
                    println!("  /help            Show this help");
                    println!("  /history         Enter history manage mode");
                    println!("  /query           Start a new query session");
                    println!("  /shell           Enter interactive shell mode (type exit to return)");
                    println!("  /load            Load project directory (interactive prompt)");
                    println!("  /quit            Exit the program");
                }
                "/history" => {
                    history_manage_mode(state.clone()).await?;
                }
                "/query" => {
                    handle_query(state.clone()).await?;
                }
                "/shell" => {
                    shell_mode(state.clone()).await?;
                }
                "/load" => {
                    load_project_path(state.clone()).await?;
                }
                "/quit" => {
                    println!("Goodbye!");
                    std::process::exit(0);
                }
                _ => {
                    println!("Unknown command. Type /help for available commands.");
                }
            }
        } else {
            println!("Unknown input. Commands start with '/'. Type /help.");
        }
    }
}

// ================================
// Query
// ================================
async fn handle_query(state: Arc<Mutex<AppState>>) -> Result<()> {
    let (project_root, mut history) = {
        let mut guard = state.lock().await;
        let project_root = match guard.project_path.clone() {
            Some(p) => p,
            None => {
                println!("No project path loaded. Use /load first.");
                return Ok(());
            }
        };
        let history = match guard.history.take() {
            Some(h) => h,
            None => {
                println!("No history loaded. Use /history then /load or /create.");
                return Ok(());
            }
        };
        (project_root, history)
    };

    println!("Enter your question for the agent:");
    let mut question = String::new();
    std::io::stdin().read_line(&mut question)?;
    let question = question.trim().to_string();
    if question.is_empty() {
        println!("The input may not be empty.");
        let mut guard = state.lock().await;
        guard.history = Some(history);
        return Ok(());
    }

    let conversation = Conversation::new(&question);

    let selected_history = loop {
        print!("Would you like choose the history to be sent?(y/n)(Enter for n) ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() || input == "n" {
            let all_uuids: Vec<String> = history
            .list_conversations()
            .into_iter()
            .map(|(id, _, _)| id)
            .collect();

            break history.prepare_for_llm(&all_uuids);
        } else if input == "y" {
            break select_history(&mut history).await?;
        } else {
            println!("The input must be y/n or Enter.");
        }
    };

    let llm = LLMClient::new();

    let mut agent = Agent {
        llm,
        selected_history,
        query: question,
        conversation,
        project_root,
    };
    agent.run().await;

    let updated_conversation = agent.conversation;

    if !updated_conversation.messages.is_empty() {
        history.add_conversation(updated_conversation);
        if let Err(e) = save_history(&history) {
            eprintln!("Failed to save history: {}", e);
        }
    }

    let mut guard = state.lock().await;
    guard.history = Some(history);

    Ok(())
}

// ================================
// History Selection
// ================================
async fn select_history(history: &mut History) -> Result<Vec<ChatCompletionRequestMessage>> {
    let convs = history.list_conversations();
    if convs.is_empty() {
        println!("No conversations in history. Sending empty history.");
        return Ok(Vec::new());
    }

    println!("Available conversations:");
    let mut id_map = Vec::new();
    for (idx, (id, time, query)) in convs.iter().enumerate() {
        let msg_count = history.get_conversation(id).map(|c| c.messages.len()).unwrap_or(0);
        let formatted_time = time.format("%Y-%m-%d %H:%M:%S %z");
        println!("Conversation {}: | uuid:{} | {} | {} ({} messages count)", idx, id, formatted_time, query, msg_count);
        id_map.push((idx, id.clone()));
    }

    let mode = loop {
        print!("Type the number (0, 1) for mode, 0 stands for selection, 1 stands for inverse selection: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        match input.trim() {
            "0" => break 0,
            "1" => break 1,
            _ => println!("The input must be 0 or 1."),
        }
    };

    let prompt = if mode == 0 {
        "Choose the conversations to be sent(separated by commas): "
    } else {
        "Choose the conversations not to be sent(separated by commas): "
    };

    let selected_indices = loop {
        print!("{}", prompt);
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.is_empty() {
            println!("Input cannot be empty. Please enter numbers separated by commas.");
            continue;
        }
        let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();
        let mut indices = Vec::new();
        let mut invalid = false;
        for part in parts {
            if let Ok(num) = part.parse::<usize>() {
                if num < id_map.len() {
                    indices.push(num);
                } else {
                    println!("Invalid number: {}", num);
                    invalid = true;
                    break;
                }
            } else {
                println!("Invalid number: {}", part);
                invalid = true;
                break;
            }
        }
        if !invalid && !indices.is_empty() {
            break indices;
        } else if invalid {
            continue;
        } else {
            println!("No valid numbers entered. Please try again.");
        }
    };

    let selected_uuids: Vec<String> = if mode == 0 {
        selected_indices
            .iter()
            .filter_map(|&idx| id_map.iter().find(|(i, _)| *i == idx).map(|(_, id)| id.clone()))
            .collect()
    } else {
        let all_uuids: Vec<String> = id_map.iter().map(|(_, id)| id.clone()).collect();
        let selected_uuids_set: std::collections::HashSet<_> = selected_indices
            .iter()
            .filter_map(|&idx| id_map.iter().find(|(i, _)| *i == idx).map(|(_, id)| id.clone()))
            .collect();
        all_uuids
            .into_iter()
            .filter(|id| !selected_uuids_set.contains(id))
            .collect()
    };

    let messages = history.prepare_for_llm(&selected_uuids);
    Ok(messages)
}

// ================================
// History Manage
// ================================
async fn history_manage_mode(state: Arc<Mutex<AppState>>) -> Result<()> {
    loop {
        let has_history = {
            let guard = state.lock().await;
            guard.history.is_some()
        };

        if !has_history {
            print!("history> ");
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            match input {
                "/quit" => break,
                "/load" => {
                    let path = prompt_for_path("Enter history file path to load:")?;
                    match load_history(&path) {
                        Ok(h) => {
                            let mut guard = state.lock().await;
                            guard.history = Some(h);
                            println!("History loaded from {}", path.display());
                        }
                        Err(e) => println!("Error loading history: {}. Try again or type /quit.", e),
                    }
                }
                "/create" => {
                    let path = prompt_for_path("Enter new history file path to create:")?;
                    let new_history = History::new(path.clone());
                    match save_history(&new_history) {
                        Ok(()) => {
                            let mut guard = state.lock().await;
                            guard.history = Some(new_history);
                            println!("History created and loaded at {}", path.display());
                        }
                        Err(e) => println!("Error creating history: {}. Try again or type /quit.", e),
                    }
                }
                "/help" => {
                    println!("History mode commands (no history loaded):");
                    println!("  /load    - load an existing history file");
                    println!("  /create  - create a new history file");
                    println!("  /quit    - exit history mode");
                }
                _ => println!("Unknown command. Type /help."),
            }
        } else {
            print!("history (loaded)> ");
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            let mut guard = state.lock().await;
            let history = match guard.history.as_mut() {
                Some(h) => h,
                None => {
                    println!("No history loaded (unexpected).");
                    continue;
                }
            };

            match input {
                "/help" => {
                    println!("History mode commands (history loaded):");
                    println!("  /display            show all conversations (with indices)");
                    println!("  /display <idx>      show messages of a specific conversation");
                    println!("  /delete <idx>       delete an entire conversation");
                    println!("  /delete <idx> <msg> delete a message from a conversation");
                    println!("  /len                show total number of conversations");
                    println!("  /len <idx>          show message count of a conversation");
                    println!("  /load               load a different history file");
                    println!("  /create             create a new history file");
                    println!("  /quit               exit history mode");
                }
                "/display" => {
                    let convs = history.list_conversations();
                    if convs.is_empty() {
                        println!("No conversations.");
                    } else {
                        for (idx, (id, time, query)) in convs.iter().enumerate() {
                            let msg_count = history.get_conversation(id).map(|c| c.messages.len()).unwrap_or(0);
                            let formatted_time = time.format("%Y-%m-%d %H:%M:%S %z");
                            println!("Conversation {}: | uuid:{} | {} | {} ({} messages count)", idx, id, formatted_time, query, msg_count);
                        }
                    }
                }
                cmd if cmd.starts_with("/display ") => {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() == 2 {
                        if let Ok(idx) = parts[1].parse::<usize>() {
                            let convs = history.list_conversations();
                            if let Some((_, (id, _, _))) = convs.iter().enumerate().find(|(i, _)| *i == idx) {
                                if let Some(conv) = history.get_conversation(id) {
                                    for (idx, msg) in conv.messages.iter().enumerate() {
                                        println!("Message {}:", idx);
                                        match msg {
                                            ChatCompletionRequestMessage::User(u) => {
                                                let content = match &u.content {
                                                    ChatCompletionRequestUserMessageContent::Text(t) => t.as_str(),
                                                    _ => "[complex content]",
                                                };
                                                println!("User [Message]:\n{}\n", content);
                                            }
                                            ChatCompletionRequestMessage::Assistant(a) => {
                                                if let Some(content) = &a.content {
                                                    let text = match content {
                                                        ChatCompletionRequestAssistantMessageContent::Text(t) => t.as_str(),
                                                        _ => "[complex]",
                                                    };
                                                    if !text.trim().is_empty() {
                                                        println!("Assistant [Message]:\n{}\n", text);
                                                    }
                                                }
                                                if let Some(tool_calls) = &a.tool_calls {
                                                    for tool_call in tool_calls {
                                                        if let ChatCompletionMessageToolCalls::Function(f) = tool_call {
                                                            println!(
                                                                "Assistant [Tool Call]:\n{}({})\n",
                                                                f.function.name, f.function.arguments
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                            ChatCompletionRequestMessage::Tool(t) => {
                                                let content = match &t.content {
                                                    ChatCompletionRequestToolMessageContent::Text(s) => s.as_str(),
                                                    _ => "[complex]",
                                                };
                                                println!("User [Observation]:\n{}\n", content);
                                            }
                                            _ => {}
                                        }
                                    }
                                } else {
                                    println!("Conversation not found.");
                                }
                            } else {
                                println!("Invalid index.");
                            }
                        } else {
                            println!("Invalid index.");
                        }
                    } else {
                        println!("Usage: /display <idx>");
                    }
                }
                "/len" => {
                    let total = history.list_conversations().len();
                    println!("Total conversations: {}", total);
                }
                cmd if cmd.starts_with("/len ") => {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() == 2 {
                        if let Ok(idx) = parts[1].parse::<usize>() {
                            let convs = history.list_conversations();
                            if let Some((_, (id, _, _))) = convs.iter().enumerate().find(|(i, _)| *i == idx) {
                                if let Some(conv) = history.get_conversation(id) {
                                    println!("Messages in conversation {}: {}", idx, conv.messages.len());
                                } else {
                                    println!("Conversation not found.");
                                }
                            } else {
                                println!("Invalid index.");
                            }
                        } else {
                            println!("Invalid index.");
                        }
                    } else {
                        println!("Usage: /len <idx>");
                    }
                }
                cmd if cmd.starts_with("/delete ") => {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() == 2 {
                        if let Ok(idx) = parts[1].parse::<usize>() {
                            let convs = history.list_conversations();
                            if let Some((_, (id, _, _))) = convs.iter().enumerate().find(|(i, _)| *i == idx) {
                                match history.delete_conversation(id) {
                                    Ok(()) => {
                                        println!("Conversation {} deleted.", idx);
                                        if let Err(e) = save_history(history) {
                                            eprintln!("Failed to save history: {}", e);
                                        }
                                    }
                                    Err(e) => println!("Delete failed: {}", e),
                                }
                            } else {
                                println!("Invalid index.");
                            }
                        } else {
                            println!("Invalid index.");
                        }
                    } else if parts.len() == 3 {
                        if let Ok(idx) = parts[1].parse::<usize>() {
                            if let Ok(msg_idx) = parts[2].parse::<usize>() {
                                let convs = history.list_conversations();
                                if let Some((_, (id, _, _))) = convs.iter().enumerate().find(|(i, _)| *i == idx) {
                                    match history.delete_message(id, msg_idx) {
                                        Ok(()) => {
                                            println!("Message {} from conversation {} deleted.", msg_idx, idx);
                                            if let Err(e) = save_history(history) {
                                                eprintln!("Failed to save history: {}", e);
                                            }
                                        }
                                        Err(e) => println!("Delete failed: {}", e),
                                    }
                                } else {
                                    println!("Invalid conversation index.");
                                }
                            } else {
                                println!("Invalid message index.");
                            }
                        } else {
                            println!("Invalid conversation index.");
                        }
                    } else {
                        println!("Usage: /delete <idx> or /delete <idx> <msg_idx>");
                    }
                }
                "/load" => {
                    drop(guard);
                    let path = prompt_for_path("Enter history file path to load:")?;
                    match load_history(&path) {
                        Ok(new_history) => {
                            let mut guard = state.lock().await;
                            *guard.history.as_mut().unwrap() = new_history;
                            println!("History loaded from {}", path.display());
                        }
                        Err(e) => println!("Error loading history: {}", e),
                    }
                }
                "/create" => {
                    drop(guard);
                    let path = prompt_for_path("Enter new history file path to create:")?;
                    let new_history = History {
                        conversations: Vec::new(),
                        path: path.clone(),
                    };
                    match save_history(&new_history) {
                        Ok(()) => {
                            let mut guard = state.lock().await;
                            *guard.history.as_mut().unwrap() = new_history;
                            println!("History created and loaded at {}", path.display());
                        }
                        Err(e) => println!("Error creating history: {}", e),
                    }
                }
                "/quit" => break,
                _ => println!("Unknown command. Type /help."),
            }
        }
    }
    Ok(())
}

// ================================
// Auxiliary
// ================================
async fn load_project_path(state: Arc<Mutex<AppState>>) -> Result<()> {
    println!("Enter project directory path:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let path = PathBuf::from(input.trim());
    if path.exists() && path.is_dir() {
        match std::fs::read_dir(&path) {
            Ok(_) => {
                let mut guard = state.lock().await;
                guard.project_path = Some(path.clone());
                println!("Project path set to {}", path.display());
            }
            Err(e) => println!("Cannot read directory: {}. Path not set.", e),
        }
    } else {
        println!("Invalid path or not a directory. Project path not set.");
    }
    Ok(())
}

async fn shell_mode(state: Arc<Mutex<AppState>>) -> Result<()> {
    let start_dir = {
        let guard = state.lock().await;
        if let Some(ref proj) = guard.project_path {
            proj.clone()
        } else {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        }
    };

    println!("Starting interactive shell. Current directory will be: {}", start_dir.display());
    println!("Type 'exit' or press Ctrl+D to return to agent.");

    #[cfg(windows)]
    let shell_cmd = "powershell";
    #[cfg(unix)]
    let shell_cmd = "sh";

    let status = std::process::Command::new(shell_cmd)
        .current_dir(&start_dir)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to start shell: {}", e))?;

    if !status.success() {
        println!("Shell exited with non-zero status: {}", status);
    }
    println!("Exited shell mode. Returning to command state.");
    Ok(())
}

fn prompt_for_path(prompt: &str) -> Result<PathBuf> {
    println!("{}", prompt);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(PathBuf::from(input.trim()))
}