mod errors;
mod structs;
mod operations;
mod fs;

pub use errors::HistoryErrors;
pub use structs::{Conversation, History};
pub use fs::{load_history, save_history};