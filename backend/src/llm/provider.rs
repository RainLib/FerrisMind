use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::error::AppError;

/// Represents a single message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system" | "user" | "assistant"
    pub content: String,
}

/// Options for chat completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatOptions {
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            model: None,
            max_tokens: None,
            temperature: Some(0.7),
            top_p: Some(1.0),
        }
    }
}

/// A streamed text chunk from the LLM.
pub type ChatStream = Pin<Box<dyn Stream<Item = Result<String, AppError>> + Send>>;

/// Trait for LLM providers. Implementations handle different APIs (OpenAI, Anthropic, etc.).
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate a complete response.
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        options: ChatOptions,
    ) -> Result<String, AppError>;

    /// Generate a streaming response.
    async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        options: ChatOptions,
    ) -> Result<ChatStream, AppError>;

    /// Get the name of this provider.
    fn name(&self) -> &str;
}
