use super::ToolErrors;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry};

/// Resolve Path for auxiliary
pub fn resolve_path(root: &Path, path: &str) -> Result<PathBuf, ToolErrors> {
    let full = root.join(path);
    Ok(full)
}

// Checks if the path is hidden
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .path()
        .file_name()
        .and_then(|file_name| file_name.to_str().map(|s| s.starts_with('.')))
        .unwrap_or(false)
}
