use crate::config::IngestConfig;
use crate::db::Db;
use crate::graph::context::SourceFlowData;
use crate::ingest::service::IngestionService;
use crate::llm::manager::LlmManager;
use async_trait::async_trait;
use graph_flow::{Context, GraphBuilder, GraphError, NextAction, Task, TaskResult};
use graph_flow::{FlowRunner, InMemorySessionStorage, Session, SessionStorage};
use std::sync::Arc;
use tracing::{error, info};

// ─── Task 1: Fetch URL content ───

pub struct FetchUrlTask {
    pub db: Db,
}

#[async_trait]
impl Task for FetchUrlTask {
    fn id(&self) -> &str {
        "FetchUrlTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        let mut data = ctx
            .get::<SourceFlowData>("data")
            .await
            .unwrap_or_default();

        info!("FetchUrlTask: fetching URL {}", data.url);
        data.status = "fetching".to_string();
        data.progress_pct = 10;
        ctx.set("data", data.clone()).await;

        // Update DB status
        let _ = self
            .db
            .query("UPDATE type::record($id) SET upload_status = 'processing'")
            .bind(("id", data.document_id.clone()))
            .await;

        let response = reqwest::get(&data.url).await.map_err(|e| {
            GraphError::TaskExecutionFailed(format!("Failed to fetch URL {}: {}", data.url, e))
        })?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html")
            .to_string();

        let body = response.text().await.map_err(|e| {
            GraphError::TaskExecutionFailed(format!("Failed to read response body: {}", e))
        })?;

        data.raw_content = body;
        data.status = "fetched".to_string();
        data.progress_pct = 30;

        // Store content type for downstream parsing
        ctx.set("content_type", content_type).await;
        ctx.set("data", data).await;

        Ok(TaskResult::new(
            Some("URL fetched successfully".to_string()),
            NextAction::Continue,
        ))
    }
}

// ─── Task 2: Process (parse + chunk + embed) via existing IngestionService ───

pub struct ProcessSourceTask {
    pub db: Db,
    pub llm: Arc<LlmManager>,
    pub ingest_config: IngestConfig,
}

#[async_trait]
impl Task for ProcessSourceTask {
    fn id(&self) -> &str {
        "ProcessSourceTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        let mut data = ctx
            .get::<SourceFlowData>("data")
            .await
            .unwrap_or_default();

        info!(
            "ProcessSourceTask: processing document {}",
            data.document_id
        );
        data.status = "processing".to_string();
        data.progress_pct = 50;
        ctx.set("data", data.clone()).await;

        let service = Arc::new(IngestionService::new(
            self.db.clone(),
            self.llm.clone(),
            &self.ingest_config,
        ));

        // Delegate to the existing ingestion pipeline (runs parse→chunk→embed→store)
        service.process_document(data.document_id.clone()).await;

        // Re-read chunk count from DB
        let count: Option<i64> = self
            .db
            .query("SELECT chunk_count FROM type::record($id)")
            .bind(("id", data.document_id.clone()))
            .await
            .ok()
            .and_then(|mut r| r.take::<Option<i64>>(0).ok().flatten());

        data.chunk_count = count.unwrap_or(0);
        data.status = "processed".to_string();
        data.progress_pct = 90;
        ctx.set("data", data).await;

        Ok(TaskResult::new(
            Some("Source processed".to_string()),
            NextAction::Continue,
        ))
    }
}

// ─── Task 3: Finalize and notify ───

pub struct NotifySourceTask {
    pub db: Db,
}

#[async_trait]
impl Task for NotifySourceTask {
    fn id(&self) -> &str {
        "NotifySourceTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        let mut data = ctx
            .get::<SourceFlowData>("data")
            .await
            .unwrap_or_default();

        info!(
            "NotifySourceTask: finalizing document {}, {} chunks",
            data.document_id, data.chunk_count
        );

        // Update DB with final status
        let status = if data.error.is_some() {
            "failed"
        } else {
            "completed"
        };

        let _ = self
            .db
            .query("UPDATE type::record($id) SET upload_status = $status")
            .bind(("id", data.document_id.clone()))
            .bind(("status", status))
            .await;

        data.status = status.to_string();
        data.progress_pct = 100;
        let doc_id = data.document_id.clone();
        ctx.set("data", data).await;

        Ok(TaskResult::new(
            Some(format!("Source {} — {}", doc_id, status)),
            NextAction::End,
        ))
    }
}

// ─── Graph builder ───

pub struct SourceGraphRunner {
    db: Db,
    llm: Arc<LlmManager>,
    ingest_config: IngestConfig,
}

impl SourceGraphRunner {
    pub fn new(db: Db, llm: Arc<LlmManager>, ingest_config: IngestConfig) -> Self {
        Self {
            db,
            llm,
            ingest_config,
        }
    }

    /// Run the full source-processing pipeline for a URL document.
    /// Returns the final SourceFlowData including status and chunk_count.
    pub async fn run(&self, flow_data: SourceFlowData) -> Result<SourceFlowData, String> {
        let graph = Arc::new(
            GraphBuilder::new("source_graph")
                .add_task(Arc::new(FetchUrlTask {
                    db: self.db.clone(),
                }))
                .add_task(Arc::new(ProcessSourceTask {
                    db: self.db.clone(),
                    llm: self.llm.clone(),
                    ingest_config: self.ingest_config.clone(),
                }))
                .add_task(Arc::new(NotifySourceTask {
                    db: self.db.clone(),
                }))
                .add_edge("FetchUrlTask", "ProcessSourceTask")
                .add_edge("ProcessSourceTask", "NotifySourceTask")
                .set_start_task("FetchUrlTask")
                .build(),
        );

        let storage = Arc::new(InMemorySessionStorage::new());
        let runner = FlowRunner::new(graph, storage.clone());

        let sid = format!("source_{}", &flow_data.document_id);
        let mut session = Session::new_from_task(sid.clone(), "FetchUrlTask");
        session.graph_id = "source_graph".to_string();
        session.context.set("data", flow_data).await;

        storage.save(session).await.map_err(|e| e.to_string())?;
        runner.run(&sid).await.map_err(|e| {
            error!("Source graph failed: {}", e);
            e.to_string()
        })?;

        let final_session = storage
            .get(&sid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Session not found after source graph run")?;

        final_session
            .context
            .get::<SourceFlowData>("data")
            .await
            .ok_or_else(|| "Failed to retrieve SourceFlowData from context".to_string())
    }
}
