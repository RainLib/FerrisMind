use crate::config::LlmConfig;
use crate::graph::context::StageSender;
use crate::llm::prompt::PromptManager;
use crate::llm::rig_client::RigClient;
use axum::response::sse::Event;
use rig::client::{CompletionClient, EmbeddingsClient};
use rig::completion::Prompt;
use rig::embeddings::EmbeddingModel;
use std::sync::Arc;
use tracing::warn;

const LLM_MAX_RETRIES: u32 = 3;

pub type GeminiAgent = rig::agent::Agent<rig::providers::gemini::completion::CompletionModel>;

pub struct LlmManager {
    rig: Arc<RigClient>,
    prompt: Arc<PromptManager>,
}

impl LlmManager {
    pub fn new(config: &LlmConfig) -> Self {
        let rig = Arc::new(RigClient::new(config));
        let prompt = Arc::new(PromptManager::new("prompts"));

        Self { rig, prompt }
    }

    pub fn rig(&self) -> Arc<RigClient> {
        self.rig.clone()
    }

    pub fn prompt(&self) -> Arc<PromptManager> {
        self.prompt.clone()
    }

    /// Helper to get a completion agent with the default model
    pub fn agent(
        &self,
    ) -> rig::agent::AgentBuilder<rig::providers::gemini::completion::CompletionModel> {
        let model = self.rig.default_model();
        self.rig.client().agent(model)
    }

    /// Helper to get the default embedding model
    pub fn embedding_model(&self) -> impl EmbeddingModel {
        let model = self.rig.embedding_model();
        self.rig.client().embedding_model(model)
    }
}

/// Call an LLM prompt with automatic retry on failure.
/// Retries up to `LLM_MAX_RETRIES` times with linear backoff (1s, 2s, 3s).
pub async fn prompt_with_retry(
    agent: &impl Prompt,
    prompt_text: &str,
    operation: &str,
) -> Result<String, String> {
    let mut last_err = String::new();
    for attempt in 1..=LLM_MAX_RETRIES + 1 {
        match agent.prompt(prompt_text).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                last_err = e.to_string();
                if attempt <= LLM_MAX_RETRIES {
                    warn!(
                        "{} failed (attempt {}/{}): {}",
                        operation,
                        attempt,
                        LLM_MAX_RETRIES + 1,
                        last_err
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(attempt as u64)).await;
                }
            }
        }
    }
    Err(format!(
        "{} failed after {} attempts: {}",
        operation,
        LLM_MAX_RETRIES + 1,
        last_err
    ))
}

/// Stream an LLM response token-by-token to the SSE channel.
/// Retries if connection fails before any content is produced.
/// Returns the full accumulated response text for DB persistence.
pub async fn stream_prompt_to_sse(
    agent: &GeminiAgent,
    prompt_text: &str,
    tx: &StageSender,
    operation: &str,
) -> Result<String, String> {
    use futures::StreamExt;
    use rig::agent::MultiTurnStreamItem;
    use rig::streaming::{StreamedAssistantContent, StreamingPrompt};

    let mut last_err = String::new();
    for attempt in 1..=LLM_MAX_RETRIES + 1 {
        let mut stream = agent.stream_prompt(prompt_text.to_string()).await;
        let mut full_response = String::new();
        let mut has_content = false;
        let mut stream_error = None;

        while let Some(item) = stream.next().await {
            match item {
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::Text(text),
                )) => {
                    has_content = true;
                    full_response.push_str(&text.text);
                    let _ = tx
                        .send(Ok(Event::default().event("answer").data(&text.text)))
                        .await;
                }
                Ok(MultiTurnStreamItem::FinalResponse(_)) => break,
                Err(e) => {
                    stream_error = Some(e.to_string());
                    break;
                }
                _ => {}
            }
        }

        match stream_error {
            Some(err) if has_content => {
                return Err(format!(
                    "{} streaming error (partial): {}",
                    operation, err
                ));
            }
            Some(err) => {
                last_err = err;
                if attempt <= LLM_MAX_RETRIES {
                    warn!(
                        "{} failed (attempt {}/{}): {}",
                        operation,
                        attempt,
                        LLM_MAX_RETRIES + 1,
                        last_err
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(attempt as u64)).await;
                    continue;
                }
            }
            None => return Ok(full_response),
        }
    }
    Err(format!(
        "{} failed after {} attempts: {}",
        operation,
        LLM_MAX_RETRIES + 1,
        last_err
    ))
}
