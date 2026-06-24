mod tools_schema;
mod structs;
mod errors;
mod builder;
mod parser;
mod chat;

pub use tools_schema::{
    GIT_SCHEMA,
    GREP_SCHEMA,
    READ_FILE_SCHEMA,
    RM_FILE_SCHEMA,
    WRITE_FILE_SCHEMA,
    LIST_FILES_SCHEMA
};
pub use structs::{LLMClient, LLMResponse};
pub use errors::LLMErrors;
pub use builder::{build_request, tools_available, content_response_format};
pub use parser::parse_response;