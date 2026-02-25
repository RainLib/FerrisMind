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

// ─── Task 1: Gather notebook context + chat history ───

pub struct ChatContextTask {
    pub db: Db,
    pub tx: StageSender,
}

#[async_trait]
impl Task for ChatContextTask {
    fn id(&self) -> &str {
        "ChatContextTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        emit_stage(&self.tx, "chat_context", "Preparing context...", 30).await;

        let mut data = ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or_default();

        info!(
            "ChatContextTask: gathering context for notebook {}",
            data.notebook_id
        );

        let nb_info: Option<NotebookInfo> = self
            .db
            .query("SELECT name, description FROM type::record($id)")
            .bind(("id", data.notebook_id.clone()))
            .await
            .ok()
            .and_then(|mut r| r.take(0).ok().flatten());

        let notebook_text = match nb_info {
            Some(nb) => format!(
                "Notebook: {}\nDescription: {}",
                nb.name.unwrap_or_default(),
                nb.description.unwrap_or_default()
            ),
            None => String::new(),
        };

        let docs: Vec<DocSummaryRow> = self
            .db
            .query("SELECT id, filename, summary FROM document WHERE notebook = type::record($nb_id) AND upload_status = 'completed'")
            .bind(("nb_id", data.notebook_id.clone()))
            .await
            .ok()
            .and_then(|mut r| r.take(0).ok())
            .unwrap_or_default();

        let docs_text = docs
            .iter()
            .filter_map(|d| {
                let id = d.id.as_ref().map(|t| {
                    use surrealdb_types::ToSql;
                    t.to_sql()
                })?;
                let summary = d.summary.as_deref().unwrap_or("No summary");
                Some(format!("[{}] {}: {}", id, d.filename, summary))
            })
            .collect::<Vec<_>>()
            .join("\n");

        data.notebook_context = format!("{}\n\nDocuments:\n{}", notebook_text, docs_text);

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
            Some("Context gathered".to_string()),
            NextAction::Continue,
        ))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, surrealdb_types::SurrealValue)]
struct NotebookInfo {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, surrealdb_types::SurrealValue)]
struct DocSummaryRow {
    id: Option<surrealdb_types::RecordId>,
    filename: String,
    summary: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, surrealdb_types::SurrealValue)]
struct MsgRow {
    role: String,
    content: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ─── Task 2: Generate response ───

pub struct ChatResponseTask {
    pub llm: Arc<LlmManager>,
    pub tx: StageSender,
}

#[async_trait]
impl Task for ChatResponseTask {
    fn id(&self) -> &str {
        "ChatResponseTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        emit_stage(&self.tx, "chat_response", "Generating response...", 60).await;

        let mut data = ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or_default();

        info!("ChatResponseTask: generating response");

        let history_str = data
            .chat_history
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let mut vars = HashMap::new();
        vars.insert("notebook".to_string(), data.notebook_context.clone());
        vars.insert("context".to_string(), history_str);

        let system_prompt = self
            .llm
            .prompt()
            .render("chat/system", &vars)
            .map_err(|e| {
                GraphError::TaskExecutionFailed(format!("Failed to render chat/system: {}", e))
            })?;

        let agent = self.llm.agent().preamble(&system_prompt).build();

        let response = agent.prompt(&data.message).await.map_err(|e| {
            GraphError::TaskExecutionFailed(format!("Chat LLM call failed: {}", e))
        })?;

        data.response = response;
        ctx.set("data", data).await;

        emit_stage(&self.tx, "complete", "Done", 100).await;

        Ok(TaskResult::new(
            Some("Chat response generated".to_string()),
            NextAction::End,
        ))
    }
}

// ─── Graph builder (using graph-flow FlowRunner) ───

pub struct ChatGraphRunner {
    pub llm: Arc<LlmManager>,
    pub db: Db,
}

impl ChatGraphRunner {
    pub fn new(llm: Arc<LlmManager>, db: Db) -> Self {
        Self { llm, db }
    }

    pub async fn run(
        &self,
        data: ChatFlowData,
        tx: &StageSender,
    ) -> Result<ChatFlowData, String> {
        let graph = Arc::new(
            GraphBuilder::new("chat_graph")
                .add_task(Arc::new(ChatContextTask {
                    db: self.db.clone(),
                    tx: tx.clone(),
                }))
                .add_task(Arc::new(ChatResponseTask {
                    llm: self.llm.clone(),
                    tx: tx.clone(),
                }))
                .add_edge("ChatContextTask", "ChatResponseTask")
                .set_start_task("ChatContextTask")
                .build(),
        );

        let storage = Arc::new(InMemorySessionStorage::new());
        let runner = FlowRunner::new(graph, storage.clone());

        let sid = format!("chat_{}", uuid::Uuid::new_v4());
        let mut session = Session::new_from_task(sid.clone(), "ChatContextTask");
        session.graph_id = "chat_graph".to_string();
        session.context.set("data", data).await;

        storage.save(session).await.map_err(|e| e.to_string())?;
        runner.run(&sid).await.map_err(|e| e.to_string())?;

        let final_session = storage
            .get(&sid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Session not found after chat graph run")?;

        final_session
            .context
            .get::<ChatFlowData>("data")
            .await
            .ok_or_else(|| "Failed to retrieve ChatFlowData".to_string())
    }
}
