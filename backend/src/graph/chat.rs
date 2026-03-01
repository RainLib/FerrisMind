use crate::db::Db;
use crate::graph::context::{emit_stage, ChatFlowData, ChatMessage, SearchHit, StageSender};
use crate::graph::kg_search::{kg_hits_to_context, KgSearcher};
use crate::llm::manager::LlmManager;
use async_trait::async_trait;
use graph_flow::{
    Context, FlowRunner, GraphBuilder, GraphError, InMemorySessionStorage, NextAction, Session,
    SessionStorage, Task, TaskResult,
};
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
        emit_stage(&self.tx, "chat_context", "Preparing context...", 25).await;

        let mut data = ctx.get::<ChatFlowData>("data").await.unwrap_or_default();

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

        data.has_sources = !docs.is_empty();

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
            NextAction::ContinueAndExecute,
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

// ─── Task 1b: KG search ───

pub struct ChatKgSearchTask {
    pub db: Db,
    pub tx: StageSender,
}

#[async_trait]
impl Task for ChatKgSearchTask {
    fn id(&self) -> &str {
        "ChatKgSearchTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        let mut data = ctx.get::<ChatFlowData>("data").await.unwrap_or_default();

        if !data.has_sources {
            ctx.set("data", data).await;
            return Ok(TaskResult::new(
                Some("No sources — skipped KG search".to_string()),
                NextAction::ContinueAndExecute,
            ));
        }

        emit_stage(
            &self.tx,
            "chat_kg_search",
            "Searching knowledge graph...",
            35,
        )
        .await;

        let terms_owned = KgSearcher::extract_terms(&data.message);
        let terms: Vec<&str> = terms_owned.iter().map(|s| s.as_str()).collect();

        if !terms.is_empty() {
            let searcher = KgSearcher::new(self.db.clone());
            let hits = searcher.search_with_expand(&data.notebook_id, &terms).await;
            info!("ChatKgSearchTask: {} KG hits", hits.len());
            data.kg_context = kg_hits_to_context(&hits);
            data.kg_hits = hits;
        }

        ctx.set("data", data).await;
        Ok(TaskResult::new(
            Some("KG search completed".to_string()),
            NextAction::ContinueAndExecute,
        ))
    }
}

// ─── Task 2: Lightweight vector search using user's message ───

pub struct ChatSearchTask {
    pub db: Db,
    pub llm: Arc<LlmManager>,
    pub tx: StageSender,
}

#[async_trait]
impl Task for ChatSearchTask {
    fn id(&self) -> &str {
        "ChatSearchTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        let mut data = ctx.get::<ChatFlowData>("data").await.unwrap_or_default();

        if !data.has_sources {
            info!("ChatSearchTask: no sources in notebook, skipping search");
            ctx.set("data", data).await;
            return Ok(TaskResult::new(
                Some("No sources — skipped search".to_string()),
                NextAction::ContinueAndExecute,
            ));
        }

        emit_stage(&self.tx, "chat_search", "Searching relevant content...", 45).await;

        info!(
            "ChatSearchTask: vector search for '{}' in notebook {}",
            data.message, data.notebook_id
        );

        let embedding = self.get_embedding(&data.message).await?;
        let hits = self.vector_search(&data.notebook_id, &embedding, 5).await?;

        info!("ChatSearchTask: {} hits found", hits.len());
        data.search_results = hits;
        ctx.set("data", data).await;

        Ok(TaskResult::new(
            Some("Chat search completed".to_string()),
            NextAction::ContinueAndExecute,
        ))
    }
}

impl ChatSearchTask {
    async fn get_embedding(&self, text: &str) -> Result<Vec<f64>, GraphError> {
        let model = self.llm.embedding_model();
        let emb = model
            .embed_text(text)
            .await
            .map_err(|e| GraphError::TaskExecutionFailed(format!("Embedding failed: {}", e)))?;
        Ok(emb.vec.into_iter().map(|v| v as f64).collect())
    }

    async fn vector_search(
        &self,
        notebook_id: &str,
        embedding: &[f64],
        top_k: usize,
    ) -> Result<Vec<SearchHit>, GraphError> {
        use surrealdb_types::ToSql;

        let query = format!(
            "SELECT id, document, content, vector::similarity::cosine(embedding, $vec) AS score \
             FROM chunk \
             WHERE notebook = type::record($nb_id) AND embedding != NONE \
             ORDER BY score DESC \
             LIMIT {}",
            top_k
        );

        let result: Vec<ChunkSearchRow> = self
            .db
            .query(&query)
            .bind(("nb_id", notebook_id.to_string()))
            .bind(("vec", embedding.to_vec()))
            .await
            .map_err(|e| {
                GraphError::TaskExecutionFailed(format!("Vector search query failed: {}", e))
            })?
            .take(0)
            .map_err(|e| {
                GraphError::TaskExecutionFailed(format!("Vector search deserialize failed: {}", e))
            })?;

        Ok(result
            .into_iter()
            .map(|r| SearchHit {
                chunk_id: r.id.as_ref().map(|t| t.to_sql()).unwrap_or_default(),
                document_id: r.document.to_sql(),
                content: r.content,
                score: r.score,
            })
            .collect())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, surrealdb_types::SurrealValue)]
struct ChunkSearchRow {
    pub id: Option<surrealdb_types::RecordId>,
    pub document: surrealdb_types::RecordId,
    pub content: String,
    pub score: f64,
}

// ─── Task 3: Generate response ───

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
        emit_stage(&self.tx, "chat_response", "Generating response...", 70).await;

        let mut data = ctx.get::<ChatFlowData>("data").await.unwrap_or_default();

        info!("ChatResponseTask: generating response");

        let history_str = data
            .chat_history
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        // Build search results context from vector search
        let search_context = if data.search_results.is_empty() {
            String::new()
        } else {
            data.search_results
                .iter()
                .map(|h| format!("[{}] (score: {:.3})\n{}", h.document_id, h.score, h.content))
                .collect::<Vec<_>>()
                .join("\n\n---\n\n")
        };

        let mut vars = HashMap::new();
        vars.insert("notebook".to_string(), data.notebook_context.clone());
        vars.insert("context".to_string(), history_str);
        vars.insert("has_sources".to_string(), data.has_sources.to_string());
        vars.insert("search_results".to_string(), search_context);
        vars.insert("kg_context".to_string(), data.kg_context.clone());

        let system_prompt = self
            .llm
            .prompt()
            .render("chat/system", &vars)
            .map_err(|e| {
                GraphError::TaskExecutionFailed(format!("Failed to render chat/system: {}", e))
            })?;

        let agent = self.llm.agent_with_preamble(&system_prompt);

        let response = agent
            .stream_to_sse(&data.message, &self.tx, "Chat LLM call")
            .await
            .map_err(GraphError::TaskExecutionFailed)?;

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
// ChatContextTask → ChatSearchTask → ChatResponseTask

pub struct ChatGraphRunner {
    pub llm: Arc<LlmManager>,
    pub db: Db,
}

impl ChatGraphRunner {
    pub fn new(llm: Arc<LlmManager>, db: Db) -> Self {
        Self { llm, db }
    }

    pub async fn run(&self, data: ChatFlowData, tx: &StageSender) -> Result<ChatFlowData, String> {
        let graph = Arc::new(
            GraphBuilder::new("chat_graph")
                .add_task(Arc::new(ChatContextTask {
                    db: self.db.clone(),
                    tx: tx.clone(),
                }))
                .add_task(Arc::new(ChatKgSearchTask {
                    db: self.db.clone(),
                    tx: tx.clone(),
                }))
                .add_task(Arc::new(ChatSearchTask {
                    db: self.db.clone(),
                    llm: self.llm.clone(),
                    tx: tx.clone(),
                }))
                .add_task(Arc::new(ChatResponseTask {
                    llm: self.llm.clone(),
                    tx: tx.clone(),
                }))
                .add_edge("ChatContextTask", "ChatKgSearchTask")
                .add_edge("ChatKgSearchTask", "ChatSearchTask")
                .add_edge("ChatSearchTask", "ChatResponseTask")
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
