use std::path::{Path};
use walkdir::{WalkDir};
use grep_regex::RegexMatcher;
use grep_searcher::{Searcher, BinaryDetection, sinks::UTF8};
use super::ToolErrors;
use super::{resolve_path, is_hidden};

pub fn grep(root: &Path, path: &str, pattern: &str, case_flag: u8) -> Result<String, ToolErrors> {
    let dir = resolve_path(root, path)?;
    if !dir.is_dir() {
        return Err(ToolErrors::InvalidPath(format!("Not a directory: {}", path)));
    }

    let case_sensitive = case_flag == 1;
    let regex_str = if case_sensitive {
        pattern.to_string()
    } else {
        format!("(?i){}", pattern)
    };
    let matcher = RegexMatcher::new(&regex_str)
        .map_err(|e| ToolErrors::Pattern(e.to_string()))?;

    let mut results = Vec::new();
    let walker = WalkDir::new(&dir).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("warning: cannot read entry: {}", e);
                continue;
            }
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let rel_path = match path.strip_prefix(&root) {
            Ok(p) => {
                let rel_str = p.display().to_string();
                format!("./{}", rel_str.replace('\\', "/"))
            }
            Err(_) => path.display().to_string(),
        };

        let mut searcher = Searcher::new();
        searcher.set_binary_detection(BinaryDetection::quit(0));

        searcher.search_path(
            &matcher,
            path,
            UTF8(|line_num, line| {
                results.push(format!("{}:{}:{}", rel_path, line_num, line));
                Ok(true)
            }),
        ).map_err(|e| ToolErrors::Grep(e.to_string()))?;
    }

    Ok(results.join("\n"))
}
