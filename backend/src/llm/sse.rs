use crate::auth::middleware::OptionalAuth;
use crate::config::IngestConfig;
use crate::db::Db;
use crate::error::AppError;
use crate::graph::context::ChatFlowData;
use crate::graph::router::ChatRouter;
use crate::graph::suggest;
use crate::graphql::types::SessionRecord;
use crate::llm::manager::LlmManager;
use std::time::Duration;
use axum::{
    extract::Extension,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::Stream;
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use surrealdb_types::ToSql;
use tracing::info;

#[derive(Deserialize)]
pub struct ChatStreamInput {
    pub notebook_id: String,
    pub content: String,
    /// Optional: resume a specific session. If absent, the most recent
    /// session for this user+notebook is used (or a new one is created).
    pub session_id: Option<String>,
    /// Selected source document IDs. Empty = use all sources in the notebook.
    #[serde(default)]
    pub source_ids: Vec<String>,
}

pub async fn chat_stream_handler(
    Extension(llm): Extension<Arc<LlmManager>>,
    auth: OptionalAuth,
    Extension(db): Extension<Db>,
    Extension(ingest_config): Extension<IngestConfig>,
    Json(input): Json<ChatStreamInput>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let user_id = auth
        .0
        .map(|c| c.sub)
        .unwrap_or_else(|| "user:mock_dev_user".to_string());

    let notebook_id = input.notebook_id.clone();

    // Resolve session: explicit > most-recent > auto-create
    let session_id = resolve_session(&db, &user_id, &notebook_id, input.session_id.as_deref())
        .await?;

    info!(
        "chat_stream: user={}, notebook={}, session={}",
        user_id, notebook_id, session_id
    );

    // Persist user message
    match db
        .query(
            "CREATE message SET \
             session = type::record($session_id), \
             role = 'user', \
             content = $content, \
             metadata = $metadata, \
             created_at = time::now()",
        )
        .bind(("session_id", session_id.clone()))
        .bind(("content", input.content.clone()))
        .bind((
            "metadata",
            serde_json::json!({
                "notebook_id": notebook_id,
                "source_ids": input.source_ids,
            })
            .to_string(),
        ))
        .await
    {
        Ok(response) => {
            if let Err(e) = response.check() {
                tracing::error!("Failed to persist user message (query error): {}", e);
            } else {
                tracing::info!("User message persisted for session {}", session_id);
            }
        }
        Err(e) => {
            tracing::error!("Failed to persist user message (transport error): {}", e);
        }
    }

    // Update session timestamp
    match db
        .query("UPDATE type::record($sid) SET updated_at = time::now()")
        .bind(("sid", session_id.clone()))
        .await
    {
        Ok(response) => {
            if let Err(e) = response.check() {
                tracing::error!("Failed to update session timestamp (query error): {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to update session timestamp (transport error): {}", e);
        }
    }

    let flow_data = ChatFlowData {
        user_id,
        notebook_id: notebook_id.clone(),
        session_id: session_id.clone(),
        message: input.content.clone(),
        source_ids: input.source_ids.clone(),
        ..Default::default()
    };

    // Start suggestion generation in parallel (only needs user question; no wait for answer)
    let (suggestion_tx, suggestion_rx) = tokio::sync::oneshot::channel();
    let llm_sug = llm.clone();
    let question_sug = input.content.clone();
    tokio::spawn(async move {
        let q = suggest::generate_suggestions_parallel(llm_sug, question_sug).await;
        let _ = suggestion_tx.send(q);
    });

    let router = ChatRouter::new(db.clone(), llm, ingest_config);
    let db_for_save = db.clone();
    let save_session_id = session_id.clone();

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(64);

    // Send session_id back so the frontend knows which session is being used
    let session_event = Event::default()
        .event("session")
        .data(serde_json::json!({ "session_id": session_id }).to_string());
    let _ = tx.send(Ok(session_event)).await;

    let nb_id_for_save = notebook_id.clone();

    tokio::spawn(async move {
        match router.handle_message(flow_data, &tx).await {
            Ok(result) => {
                // Use parallel suggestion result (wait up to 8s if not ready yet)
                let suggested = tokio::time::timeout(Duration::from_secs(8), suggestion_rx)
                    .await
                    .unwrap_or(Ok(Vec::new()))
                    .unwrap_or_default();

                let response_text = result.response.clone();
                tracing::info!(
                    "Graph completed: intent={}, response_len={}, suggestions={}",
                    result.intent,
                    response_text.len(),
                    suggested.len()
                );

                if response_text.is_empty() {
                    tracing::warn!("Assistant response is empty — skipping DB persist");
                } else {
                    let metadata_obj = serde_json::json!({
                        "intent": result.intent,
                        "notebook_id": result.notebook_id,
                        "source_ids": result.source_ids,
                        "search_query_count": result.search_strategy.as_ref().map(|s| s.searches.len()).unwrap_or(0),
                        "search_hit_count": result.search_results.len(),
                        "suggested_questions": suggested,
                    });
                    let metadata_str = metadata_obj.to_string();

                    match db_for_save
                        .query(
                            "CREATE message SET \
                             session = type::record($session_id), \
                             role = 'assistant', \
                             content = $content, \
                             metadata = $metadata, \
                             created_at = time::now()",
                        )
                        .bind(("session_id", save_session_id.clone()))
                        .bind(("content", response_text))
                        .bind(("metadata", metadata_str))
                        .await
                    {
                        Ok(response) => {
                            if let Err(e) = response.check() {
                                tracing::error!(
                                    "Failed to persist assistant message (query error): {}",
                                    e
                                );
                            } else {
                                tracing::info!(
                                    "Assistant message persisted for session {}",
                                    save_session_id
                                );
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to persist assistant message (transport error): {}",
                                e
                            );
                        }
                    }
                }

                // Update session timestamp after assistant message
                if let Err(e) = db_for_save
                    .query("UPDATE type::record($sid) SET updated_at = time::now()")
                    .bind(("sid", save_session_id.clone()))
                    .await
                {
                    tracing::error!("Failed to update session timestamp after reply: {}", e);
                }

                // Send suggested follow-up questions (from parallel task)
                if !suggested.is_empty() {
                    let _ = tx
                        .send(Ok(Event::default()
                            .event("suggestions")
                            .data(
                                serde_json::to_string(&suggested)
                                    .unwrap_or_default(),
                            )))
                        .await;
                }

                let _ = tx
                    .send(Ok(Event::default()
                        .event("metadata")
                        .data(
                            serde_json::json!({
                                "intent": result.intent,
                                "notebook_id": nb_id_for_save,
                            })
                            .to_string(),
                        )))
                    .await;

                let _ = tx
                    .send(Ok(Event::default().event("done").data("[DONE]")))
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(Ok(Event::default()
                        .event("error")
                        .data(format!("Graph execution failed: {}", e))))
                    .await;
            }
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Find or create a session for user+notebook.
///
/// Priority:
/// 1. If `explicit_session_id` is provided and belongs to the user, use it.
/// 2. Otherwise find the most recently updated session for user+notebook.
/// 3. If none exists, auto-create one.
async fn resolve_session(
    db: &Db,
    user_id: &str,
    notebook_id: &str,
    explicit_session_id: Option<&str>,
) -> Result<String, AppError> {
    // 1. Explicit session — verify ownership via typed struct
    if let Some(sid) = explicit_session_id {
        let found: Vec<SessionRecord> = db
            .query(
                "SELECT * FROM type::record($sid) \
                 WHERE user = type::record($uid) \
                   AND notebook = type::record($nid)",
            )
            .bind(("sid", sid.to_string()))
            .bind(("uid", user_id.to_string()))
            .bind(("nid", notebook_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()))?;

        if !found.is_empty() {
            return Ok(sid.to_string());
        }
    }

    // 2. Most recent session for this user+notebook
    let recent: Vec<SessionRecord> = db
        .query(
            "SELECT * FROM session \
             WHERE user = type::record($uid) \
               AND notebook = type::record($nid) \
             ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(("uid", user_id.to_string()))
        .bind(("nid", notebook_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    if let Some(row) = recent.first() {
        if let Some(ref id) = row.id {
            return Ok(id.to_sql());
        }
    }

    // 3. Auto-create
    let created: Vec<SessionRecord> = db
        .query(
            "CREATE session SET \
             notebook = type::record($nid), \
             user = type::record($uid), \
             title = 'Chat', \
             created_at = time::now(), \
             updated_at = time::now()",
        )
        .bind(("nid", notebook_id.to_string()))
        .bind(("uid", user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let row = created
        .first()
        .ok_or_else(|| AppError::Internal("Failed to create session".to_string()))?;

    let session_id = row
        .id
        .as_ref()
        .map(|t| t.to_sql())
        .ok_or_else(|| AppError::Internal("Created session has no id".to_string()))?;

    info!(
        "Auto-created session {} for notebook {}",
        session_id, notebook_id
    );
    Ok(session_id)
}

