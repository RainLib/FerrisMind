use axum::response::sse::Event;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio::sync::mpsc;

use crate::graph::kg_search::KgHit;

/// Channel sender for pushing SSE stage events back to the client.
pub type StageSender = mpsc::Sender<Result<Event, Infallible>>;

/// A progress event sent to the frontend between graph tasks.
#[derive(Debug, Clone, Serialize)]
pub struct StageEvent {
    pub stage: String,
    pub message: String,
    pub progress: u8,
}

impl StageEvent {
    pub fn new(stage: impl Into<String>, message: impl Into<String>, progress: u8) -> Self {
        Self {
            stage: stage.into(),
            message: message.into(),
            progress,
        }
    }

    pub fn to_sse(&self) -> Result<Event, Infallible> {
        Ok(Event::default()
            .event("stage")
            .data(serde_json::to_string(self).unwrap_or_default()))
    }
}

/// Helper: send a stage event, ignoring errors (channel may be closed).
pub async fn emit_stage(tx: &StageSender, stage: &str, message: &str, progress: u8) {
    let evt = StageEvent::new(stage, message, progress);
    let _ = tx.send(evt.to_sse()).await;
}

/// Search strategy produced by the ask/entry prompt.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchStrategy {
    pub reasoning: String,
    pub searches: Vec<SearchQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchQuery {
    pub term: String,
    pub instructions: String,
}

/// A single search hit returned from vector / full-text search.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchHit {
    pub chunk_id: String,
    pub document_id: String,
    pub content: String,
    pub score: f64,
}

/// Sub-answer produced by the ask/query_process step for one search query.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubAnswer {
    pub term: String,
    pub answer: String,
    pub source_ids: Vec<String>,
}

/// Shared state flowing through all chat-related graphs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatFlowData {
    // ── Input ──
    pub user_id: String,
    pub notebook_id: String,
    pub session_id: String,
    pub message: String,
    /// Set when the user is chatting about specific source documents.
    pub source_ids: Vec<String>,

    // ── Intent ──
    /// One of: "ask", "chat", "source_chat"
    pub intent: String,

    // ── Context gathered from DB ──
    pub notebook_context: String,
    pub source_context: String,
    pub chat_history: Vec<ChatMessage>,
    /// Whether the notebook has any completed source documents.
    pub has_sources: bool,

    // ── Knowledge Graph search results ──
    pub kg_hits: Vec<KgHit>,
    /// Pre-formatted KG context string ready for LLM prompts.
    pub kg_context: String,

    // ── Ask-specific ──
    pub search_strategy: Option<SearchStrategy>,
    pub search_results: Vec<SearchHit>,
    pub sub_answers: Vec<SubAnswer>,

    // ── Output ──
    pub response: String,
    pub suggested_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Shared state for the source-processing (ingestion) graph.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceFlowData {
    pub notebook_id: String,
    pub document_id: String,
    pub url: String,
    pub user_id: String,

    // ── Progress tracking ──
    pub status: String,
    pub progress_pct: u8,
    pub discovered_urls: Vec<String>,
    pub error: Option<String>,

    // ── Intermediate results ──
    pub raw_content: String,
    pub chunk_count: i64,
}
