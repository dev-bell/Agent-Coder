use std::fs;
use std::path::{Path, PathBuf};
use serde_json::json;
use super::ToolErrors;

fn resolve_path(root: &Path, path: &str) -> Result<PathBuf, ToolErrors> {
    if !path.starts_with("./") {
        return Err(ToolErrors::InvalidPath(
            format!("Path must start with './', got: {}", path)
        ));
    }
    let stripped = path.trim_start_matches("./");
    let full = root.join(stripped);
    full.canonicalize()
        .map_err(|_| ToolErrors::InvalidPath(format!("Path does not exist: {}", path)))
}

/// List files.
/// mode: 1 = recursive (tree JSON), 2 = non‑recursive (flat list JSON)
pub fn list_files(root: &Path, path: &str, mode: u8) -> Result<String, ToolErrors> {
    let dir = resolve_path(root, path)?;
    if !dir.is_dir() {
        return Err(ToolErrors::InvalidPath(format!("Not a directory: {}", path)));
    }

    fn walk(dir: &Path, root: &Path, recursive: bool) -> Result<Vec<serde_json::Value>, ToolErrors> {
        let mut items = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = entry.file_type()?;
            let is_dir = file_type.is_dir();

            if recursive && is_dir {
                let children = walk(&entry.path(), root, true)?;
                items.push(json!({
                    "name": name,
                    "type": "directory",
                    "children": children
                }));
            } else {
                items.push(json!({
                    "name": name,
                    "type": if is_dir { "directory" } else { "file" }
                }));
            }
        }
        items.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        Ok(items)
    }

    let recursive = mode == 1;
    let items = walk(&dir, root, recursive)?;
    Ok(serde_json::to_string_pretty(&items)?)
}

pub fn read_file(root: &Path, path: &str) -> Result<String, ToolErrors> {
    let full = resolve_path(root, path)?;
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
    if full.is_dir() {
        fs::remove_dir_all(&full)
            .map_err(|e| ToolErrors::Io(e))?;
    } else {
        fs::remove_file(&full)
            .map_err(|e| ToolErrors::Io(e))?;
    }
    Ok("Deleted".to_string())
}