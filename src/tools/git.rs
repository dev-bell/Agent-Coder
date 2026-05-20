use std::process::Command;
use std::path::Path;
use super::ToolErrors;

pub fn git(root: &Path, command: &str) -> Result<String, ToolErrors> {
    let trimmed = command.trim_start();
    if !trimmed.starts_with("git") {
        return Err(ToolErrors::Git("Command must start with 'git'".to_string()));
    }
    let rest = trimmed.strip_prefix("git").unwrap_or(trimmed).trim_start();
    let args: Vec<&str> = rest.split_whitespace().collect();
    let output = Command::new("git")
        .current_dir(root)
        .args(&args)
        .output()
        .map_err(|e| ToolErrors::Git(format!("Failed to execute git: {}", e)))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ToolErrors::Git(format!("Git error: {}", stderr)))
    }
}