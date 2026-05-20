pub const READ_FILE_SCHEMA: &str = r#"
{
  "type": "function",
  "function": {
    "name": "read_file",
    "description": "Read the content of a file.",
    "strict": true,
    "parameters": {
      "type": "object",
      "properties": {
        "path": {
          "type": "string",
          "description": "Relative path starting with './', e.g., './src/main.rs'."
        }
      },
      "required": ["path"],
      "additionalProperties": false
    }
  }
}
"#;

pub const LIST_FILES_SCHEMA: &str = r#"
{
  "type": "function",
  "function": {
    "name": "list_files",
    "description": "List files in a directory.",
    "strict": true,
    "parameters": {
      "type": "object",
      "properties": {
        "path": {
          "type": "string",
          "description": "Directory path starting with './', e.g., './src'."
        },
        "mode": {
          "type": "integer",
          "description": "Mode 1 returns recursive tree (all subdirectories), Mode 2 returns flat list of immediate children only.",
          "enum": [1, 2]
        }
      },
      "required": ["path", "mode"],
      "additionalProperties": false
    }
  }
}
"#;

pub const WRITE_FILE_SCHEMA: &str = r#"
{
  "type": "function",
  "function": {
    "name": "write_file",
    "description": "Write content to a file.",
    "strict": true,
    "parameters": {
      "type": "object",
      "properties": {
        "path": {
          "type": "string",
          "description": "Relative path starting with './', e.g., './src/main.rs'."
        },
        "content": {
          "type": "string",
          "description": "The content to write to the file."
        }
      },
      "required": ["path", "content"],
      "additionalProperties": false
    }
  }
}
"#;

pub const RM_FILE_SCHEMA: &str = r#"
{
  "type": "function",
  "function": {
    "name": "rm_file",
    "description": "Delete a file or an entire directory (including all contents).",
    "strict": true,
    "parameters": {
      "type": "object",
      "properties": {
        "path": {
          "type": "string",
          "description": "Relative path starting with './', e.g., './src/main.rs'."
        }
      },
      "required": ["path"],
      "additionalProperties": false
    }
  }
}
"#;

pub const GIT_SCHEMA: &str = r#"
{
  "type": "function",
  "function": {
    "name": "git",
    "description": "Execute a git command in the project root directory.",
    "strict": true,
    "parameters": {
      "type": "object",
      "properties": {
        "command": {
          "type": "string",
          "description": "Git command starting with 'git', e.g., 'git status'."
        }
      },
      "required": ["command"],
      "additionalProperties": false
    }
  }
}
"#;

pub const GREP_SCHEMA: &str = r#"
{
  "type": "function",
  "function": {
    "name": "grep",
    "description": "Recursively search for a literal pattern in text files under a directory.",
    "strict": true,
    "parameters": {
      "type": "object",
      "properties": {
        "path": {
          "type": "string",
          "description": "Directory path starting with './', e.g., './src'."
        },
        "pattern": {
          "type": "string",
          "description": "Literal substring to search for in each line."
        },
        "case_flag": {
          "type": "integer",
          "description": "0 = case-sensitive, 1 = case-insensitive.",
          "enum": [0, 1]
        }
      },
      "required": ["path", "pattern", "case_flag"],
      "additionalProperties": false
    }
  }
}
"#;