use async_openai::types::chat::ChatCompletionRequestMessage;
use chrono::{DateTime, Utc};
use super::structs::{Conversation, History};
use super::HistoryErrors;

impl Conversation {
    pub fn delete_message(&mut self, index: usize) -> Result<(), HistoryErrors> {
        if index < self.messages.len() {
            self.messages.remove(index);
            Ok(())
        } else {
            Err(HistoryErrors::MessageIndexOutOfRange)
        }
    }
}

impl History {
    pub fn add_conversation(&mut self, conv: Conversation) {
        self.conversations.push(conv);
    }

    pub fn delete_message(&mut self, conv_id: &str, index: usize) -> Result<(), HistoryErrors> {
        for conv in &mut self.conversations {
            if conv.id == conv_id {
                return conv.delete_message(index);
            }
        }
        Err(HistoryErrors::ConversationNotFound(conv_id.to_string()))
    }

    pub fn delete_conversation(&mut self, conv_id: &str) -> Result<(), HistoryErrors> {
        let initial_len = self.conversations.len();
        self.conversations.retain(|c| c.id != conv_id);
        if self.conversations.len() < initial_len {
            Ok(())
        } else {
            Err(HistoryErrors::ConversationNotFound(conv_id.to_string()))
        }
    }

    pub fn get_conversation(&self, conv_id: &str) -> Option<&Conversation> {
        self.conversations.iter().find(|c| c.id == conv_id)
    }

    pub fn list_conversations(&self) -> Vec<(String, DateTime<Utc>, String)> {
        self.conversations
            .iter()
            .map(|c| (c.id.clone(), c.start_time.clone(), c.query.clone()))
            .collect()
    }

    pub fn prepare_for_llm(&self, selected: &[String]) -> Vec<ChatCompletionRequestMessage> {
        let mut messages = Vec::new();
        for conv in &self.conversations {
            if selected.contains(&conv.id) {
                messages.extend(conv.messages.clone());
            }
        }
        messages
    }

}