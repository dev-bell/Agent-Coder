use std::fs;
use std::path::{Path, PathBuf};
use super::ToolErrors;

/// Recursively search for literal pattern under `path`.
/// case_flag: 0 = case‑sensitive, 1 = case‑insensitive.
/// Excludes hidden files/directories (starting with '.'), skips binary files.
/// Output format: `relative_path:line_number:line_content`
pub fn grep(root: &Path, path: &str, pattern: &str, case_flag: u8) -> Result<String, ToolErrors> {
    let dir = resolve_path(root, path)?;
    if !dir.is_dir() {
        return Err(ToolErrors::InvalidPath(format!("Not a directory: {}", path)));
    }
    let case_insensitive = case_flag == 1;
    let pattern_lower = if case_insensitive { pattern.to_lowercase() } else { pattern.to_string() };

    let mut results = Vec::new();
    walk_dir(&dir, root, &mut |rel_path, content| {
        for (line_num, line) in content.lines().enumerate() {
            let line_to_check = if case_insensitive { line.to_lowercase() } else { line.to_string() };
            if line_to_check.contains(&pattern_lower) {
                results.push(format!("{}:{}:{}", rel_path, line_num + 1, line));
            }
        }
        Ok(())
    })?;
    Ok(results.join("\n"))
}

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

/// Recursively walk directory, skip hidden entries, only regular files, skip binary.
fn walk_dir<F>(dir: &Path, root: &Path, callback: &mut F) -> Result<(), ToolErrors>
where
    F: FnMut(&str, &str) -> Result<(), ToolErrors>,
{
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("warning: cannot read directory {}: {}", dir.display(), e);
            return Ok(());
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("warning: entry error in {}: {}", dir.display(), e);
                continue;
            }
        };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') {
            continue;
        }
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                eprintln!("warning: cannot get file type for {}: {}", entry.path().display(), e);
                continue;
            }
        };
        let path = entry.path();
        if file_type.is_dir() {
            if let Err(e) = walk_dir(&path, root, callback) {
                eprintln!("warning: skipping directory {} due to error: {}", path.display(), e);
                continue;
            }
        } else if file_type.is_file() {
            let data = match fs::read(&path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("warning: cannot read file {}: {}", path.display(), e);
                    continue;
                }
            };
            if data.iter().take(8192).any(|&b| b == 0) {
                continue;
            }
            let content = String::from_utf8_lossy(&data).to_string();
            let rel = match path.strip_prefix(root) {
                Ok(rel) => rel,
                Err(_) => continue,
            };
            let rel_str = rel.to_string_lossy().to_string();
            callback(&rel_str, &content)?;
        }
    }
    Ok(())
}