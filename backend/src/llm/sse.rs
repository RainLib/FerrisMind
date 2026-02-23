use crate::auth::jwt::Claims;
use crate::db::Db;
use crate::error::AppError;
use crate::llm::manager::LlmManager;
use axum::{
    extract::Extension,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::Stream;
// Rig 0.31.0 streaming types
use rig::agent::MultiTurnStreamItem;
use rig::prelude::*;
use rig::streaming::{StreamedAssistantContent, StreamingPrompt};
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::StreamExt;

#[derive(Deserialize)]
pub struct ChatStreamInput {
    pub session_id: String,
    pub content: String,
}

pub async fn chat_stream_handler(
    Extension(llm): Extension<Arc<LlmManager>>,
    Extension(claims): Extension<Option<Claims>>,
    Extension(db): Extension<Db>,
    Json(input): Json<ChatStreamInput>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    // 1. Verify Authentication
    let claims = claims.ok_or(AppError::Unauthorized)?;

    // 2. Verify Session Access
    let session_id = input.session_id.clone();
    let user_id = claims.sub.clone();

    // Check if session belongs to user
    let session_exists: bool = db
        .query("SELECT * FROM type::thing($session_id) WHERE user = type::thing($user_id)")
        .bind(("session_id", session_id.clone()))
        .bind(("user_id", user_id))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .take::<Vec<serde_json::Value>>(0)
        .map(|v| !v.is_empty())
        .map_err(|e| AppError::Database(e.to_string()))?;

    if !session_exists {
        return Err(AppError::Forbidden(
            "Session not found or access denied".to_string(),
        ));
    }

    // 3. Prepare Rig Agent using Gemini via LlmManager helper
    let agent = llm.agent().build();

    // In rig 0.31.0, stream_prompt returns a Stream of MultiTurnStreamItem
    let stream = agent.stream_prompt(&input.content).await;

    // Map each chunk (MultiTurnStreamItem) to an Event
    let event_stream = stream.map(|chunk| match chunk {
        Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
            Ok(Event::default().data(text.text))
        }
        Ok(MultiTurnStreamItem::FinalResponse(res)) => Ok(Event::default().data(res.response())),
        Ok(_) => Ok(Event::default().data("")),
        Err(e) => Ok(Event::default().event("error").data(e.to_string())),
    });

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}
