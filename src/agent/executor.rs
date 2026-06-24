use std::path::Path;
use serde_json::Value;
use crate::tools::{
    list_files, read_file, write_file, rm_file, grep, git,
    ToolErrors,
};
use super::structs::ToolsForExecute;

pub fn parse_tool(
    tc: &async_openai::types::chat::ChatCompletionMessageToolCall,
) -> ToolsForExecute {
    ToolsForExecute {
        id: tc.id.clone(),
        name: tc.function.name.clone(),
        arguments: tc.function.arguments.clone(),
    }
}

pub fn execute_tool(
    tfe: &ToolsForExecute,
    root: &Path,
) -> Result<String, ToolErrors> {
    let args: Value = serde_json::from_str(&tfe.arguments)?;

    match tfe.name.as_str() {
        "list_files" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolErrors::InvalidArgument("path".to_string()))?;
            let mode = args
                .get("mode")
                .and_then(|v| v.as_u64())
                .map(|v| v as u8)
                .ok_or_else(|| ToolErrors::InvalidArgument("mode".to_string()))?;
            list_files(root, path, mode)
        }
        "read_file" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolErrors::InvalidArgument("path".to_string()))?;
            read_file(root, path)
        }
        "write_file" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolErrors::InvalidArgument("path".to_string()))?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolErrors::InvalidArgument("content".to_string()))?;
            write_file(root, path, content)
        }
        "rm_file" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolErrors::InvalidArgument("path".to_string()))?;
            rm_file(root, path)
        }
        "grep" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolErrors::InvalidArgument("path".to_string()))?;
            let pattern = args
                .get("pattern")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolErrors::InvalidArgument("pattern".to_string()))?;
            let case_flag = args
                .get("case_flag")
                .and_then(|v| v.as_u64())
                .map(|v| v as u8)
                .ok_or_else(|| ToolErrors::InvalidArgument("case_flag".to_string()))?;
            grep(root, path, pattern, case_flag)
        }
        "git" => {
            let command = args
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolErrors::InvalidArgument("command".to_string()))?;
            git(root, command)
        }
        _ => Err(ToolErrors::InvalidOperation(format!(
            "Unknown tool: {}",
            tfe.name
        ))),
    }
}