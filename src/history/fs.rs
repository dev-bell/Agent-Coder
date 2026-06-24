use std::fs;
use std::path::{Path};
use super::structs::{History, Conversation};
use super::HistoryErrors;

pub fn load_history(path: &Path) -> Result<History, HistoryErrors> {
    if !path.exists() {
        return Err(HistoryErrors::FileNotFound(path.to_path_buf()));
    }
    let content = fs::read_to_string(path)?;
    let conversations: Vec<Conversation> = serde_json::from_str(&content)?;
    Ok(History {
        conversations,
        path: path.to_path_buf(),
    })
}

pub fn save_history(history: &History) -> Result<(), HistoryErrors> {
    let content = serde_json::to_string_pretty(&history.conversations)?;
    fs::write(&history.path, content)?;
    Ok(())
}