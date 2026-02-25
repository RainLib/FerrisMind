use crate::db::Db;
use crate::graph::context::{emit_stage, ChatFlowData, ChatMessage, StageSender};
use crate::llm::manager::LlmManager;
use async_trait::async_trait;
use graph_flow::{
    Context, FlowRunner, GraphBuilder, GraphError, InMemorySessionStorage, NextAction, Session,
    SessionStorage, Task, TaskResult,
};
use rig::completion::Prompt;
use std::collections::HashMap;
use std::sync::Arc;
use surrealdb_types::SurrealValue;
use tracing::info;

// ─── Task 1: Load source document context ───

pub struct SourceContextTask {
    pub db: Db,
    pub tx: StageSender,
}

#[async_trait]
impl Task for SourceContextTask {
    fn id(&self) -> &str {
        "SourceContextTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        emit_stage(&self.tx, "source_context", "Loading source document...", 30).await;

        let mut data = ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or_default();

        let source_id = data
            .source_id
            .as_deref()
            .ok_or_else(|| GraphError::TaskExecutionFailed("No source_id provided".to_string()))?
            .to_string();

        info!("SourceContextTask: loading source {}", source_id);

        let doc: Option<SourceDocRow> = self
            .db
            .query("SELECT id, filename, summary FROM type::record($id)")
            .bind(("id", source_id.clone()))
            .await
            .ok()
            .and_then(|mut r| r.take(0).ok().flatten());

        let chunks: Vec<SourceChunkRow> = self
            .db
            .query("SELECT content, chunk_index FROM chunk WHERE document = type::record($doc_id) ORDER BY chunk_index ASC LIMIT 30")
            .bind(("doc_id", source_id.clone()))
            .await
            .ok()
            .and_then(|mut r| r.take(0).ok())
            .unwrap_or_default();

        let mut context_parts = Vec::new();

        if let Some(ref d) = doc {
            context_parts.push(format!("Source: {} ({})", d.filename, source_id));
            if let Some(ref s) = d.summary {
                context_parts.push(format!("Summary: {}", s));
            }
        }

        let max_chars: usize = 40_000;
        let mut content = String::new();
        for chunk in &chunks {
            if content.len() + chunk.content.len() > max_chars {
                break;
            }
            content.push_str(&chunk.content);
            content.push('\n');
        }
        context_parts.push(format!("Content:\n{}", content));

        data.source_context = context_parts.join("\n\n");

        if !data.session_id.is_empty() {
            let messages: Vec<MsgRow> = self
                .db
                .query("SELECT role, content, created_at FROM message WHERE session = type::record($sid) ORDER BY created_at DESC LIMIT 20")
                .bind(("sid", data.session_id.clone()))
                .await
                .ok()
                .and_then(|mut r| r.take(0).ok())
                .unwrap_or_default();

            data.chat_history = messages
                .into_iter()
                .rev()
                .map(|m| ChatMessage {
                    role: m.role,
                    content: m.content,
                })
                .collect();
        }

        ctx.set("data", data).await;

        Ok(TaskResult::new(
            Some("Source context loaded".to_string()),
            NextAction::Continue,
        ))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, surrealdb_types::SurrealValue)]
struct SourceDocRow {
    id: Option<surrealdb_types::RecordId>,
    filename: String,
    summary: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, surrealdb_types::SurrealValue)]
struct SourceChunkRow {
    content: String,
    chunk_index: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, surrealdb_types::SurrealValue)]
struct MsgRow {
    role: String,
    content: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ─── Task 2: Generate source-specific response ───

pub struct SourceResponseTask {
    pub llm: Arc<LlmManager>,
    pub tx: StageSender,
}

#[async_trait]
impl Task for SourceResponseTask {
    fn id(&self) -> &str {
        "SourceResponseTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        emit_stage(&self.tx, "source_response", "Generating response...", 60).await;

        let mut data = ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or_default();

        info!("SourceResponseTask: generating source-specific response");

        let source_id = data.source_id.clone().unwrap_or_default();

        let mut vars = HashMap::new();

        let source_json = format!(
            r#"{{"id": "{}", "title": "Source document", "topics": []}}"#,
            source_id
        );
        vars.insert("source".to_string(), source_json);
        vars.insert("context".to_string(), data.source_context.clone());

        let system_prompt = self
            .llm
            .prompt()
            .render("source_chat/system", &vars)
            .map_err(|e| {
                GraphError::TaskExecutionFailed(format!(
                    "Failed to render source_chat/system: {}",
                    e
                ))
            })?;

        let agent = self.llm.agent().preamble(&system_prompt).build();

        let response = agent.prompt(&data.message).await.map_err(|e| {
            GraphError::TaskExecutionFailed(format!("Source chat LLM call failed: {}", e))
        })?;

        data.response = response;
        ctx.set("data", data).await;

        emit_stage(&self.tx, "complete", "Done", 100).await;

        Ok(TaskResult::new(
            Some("Source chat response generated".to_string()),
            NextAction::End,
        ))
    }
}

// ─── Graph builder (using graph-flow FlowRunner) ───

pub struct SourceChatGraphRunner {
    pub llm: Arc<LlmManager>,
    pub db: Db,
}

impl SourceChatGraphRunner {
    pub fn new(llm: Arc<LlmManager>, db: Db) -> Self {
        Self { llm, db }
    }

    pub async fn run(
        &self,
        data: ChatFlowData,
        tx: &StageSender,
    ) -> Result<ChatFlowData, String> {
        let graph = Arc::new(
            GraphBuilder::new("source_chat_graph")
                .add_task(Arc::new(SourceContextTask {
                    db: self.db.clone(),
                    tx: tx.clone(),
                }))
                .add_task(Arc::new(SourceResponseTask {
                    llm: self.llm.clone(),
                    tx: tx.clone(),
                }))
                .add_edge("SourceContextTask", "SourceResponseTask")
                .set_start_task("SourceContextTask")
                .build(),
        );

        let storage = Arc::new(InMemorySessionStorage::new());
        let runner = FlowRunner::new(graph, storage.clone());

        let sid = format!("source_chat_{}", uuid::Uuid::new_v4());
        let mut session = Session::new_from_task(sid.clone(), "SourceContextTask");
        session.graph_id = "source_chat_graph".to_string();
        session.context.set("data", data).await;

        storage.save(session).await.map_err(|e| e.to_string())?;
        runner.run(&sid).await.map_err(|e| e.to_string())?;

        let final_session = storage
            .get(&sid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Session not found after source_chat graph run")?;

        final_session
            .context
            .get::<ChatFlowData>("data")
            .await
            .ok_or_else(|| "Failed to retrieve ChatFlowData".to_string())
    }
}
