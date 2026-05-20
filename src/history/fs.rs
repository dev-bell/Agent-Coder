use std::fs;
use std::path::{Path};
use super::structs::History;
use super::HistoryErrors;

pub fn load_history(path: &Path) -> Result<History, HistoryErrors> {
    if !path.exists() {
        return Err(HistoryErrors::FileNotFound(path.to_path_buf()));
    }
    let content = fs::read_to_string(path)?;
    let history: History = serde_json::from_str(&content)?;
    Ok(history)
}

pub fn save_history(history: &History) -> Result<(), HistoryErrors> {
    let content = serde_json::to_string_pretty(history)?;
    fs::write(&history.path, content)?;
    Ok(())
}