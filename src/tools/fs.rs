use std::fs;
use std::path::{Path};
use walkdir::{WalkDir};
use super::ToolErrors;
use super::{resolve_path, is_hidden};

/// List files.
/// mode: 1 = recursive, 2 = non‑recursive
pub fn list_files(root: &Path, path: &str, mode: u8) -> Result<String, ToolErrors> {
    let dir = resolve_path(root, path)?;
    if !dir.exists() {
        return Err(ToolErrors::InvalidPath(format!("Path does not exist: {}", path)));
    }
    if !dir.is_dir() {
        return Err(ToolErrors::InvalidPath(format!("Not a directory: {}", path)));
    }

    let max_depth = if mode == 1 { usize::MAX } else { 1 };
    let mut results = Vec::new();
    let walker = WalkDir::new(&dir).max_depth(max_depth).into_iter();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("warning: cannot read entry: {}", e);
                continue;
            }
        };
        let path = entry.path();
        let rel = match path.strip_prefix(&root) {
            Ok(rel) => rel,
            Err(_) => continue,
        };
        let rel_str = rel.to_str().unwrap_or("").replace('\\', "/");
        let output = format!("{}", rel_str);
        results.push(output);
    }

    Ok(results.join("\n"))
}

pub fn read_file(root: &Path, path: &str) -> Result<String, ToolErrors> {
    let full = resolve_path(root, path)?;
    if !full.exists() {
        return Err(ToolErrors::InvalidPath(format!("Path does not exist: {}", path)));
    }
    if !full.is_file() {
        return Err(ToolErrors::InvalidPath(format!("Not a file: {}", path)));
    }
    fs::read_to_string(&full).map_err(|e| ToolErrors::Io(e))
}

pub fn write_file(root: &Path, path: &str, content: &str) -> Result<String, ToolErrors> {
    let full = resolve_path(root, path)?;
    if full.exists() && full.is_dir() {
        return Err(ToolErrors::InvalidPath(format!("Not a file: {}", path)));
    }
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&full, content)
        .map_err(|e| ToolErrors::Io(e))?;
    Ok("Written".to_string())
}

pub fn rm_file(root: &Path, path: &str) -> Result<String, ToolErrors> {
    let full = resolve_path(root, path)?;
    if !full.exists() {
        return Err(ToolErrors::InvalidPath(format!("Path does not exist: {}", path)));
    }
    if full.is_dir() {
        fs::remove_dir_all(&full)
            .map_err(|e| ToolErrors::Io(e))?;
    } else {
        fs::remove_file(&full)
            .map_err(|e| ToolErrors::Io(e))?;
    }
    Ok("Deleted".to_string())
}