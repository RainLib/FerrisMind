use crate::config::IngestConfig;
use crate::db::Db;
use crate::graph::ask::AskGraphRunner;
use crate::graph::chat::{ChatContextTask, ChatGraphRunner};
use crate::graph::context::{emit_stage, ChatFlowData, SourceFlowData, StageSender};
use crate::graph::intent::IntentTask;
use crate::graph::source::SourceGraphRunner;
use crate::graph::source_chat::SourceChatGraphRunner;
use crate::llm::manager::LlmManager;
use graph_flow::{Context, Task};
use std::sync::Arc;
use tracing::info;

pub struct ChatRouter {
    db: Db,
    llm: Arc<LlmManager>,
    ingest_config: IngestConfig,
}

impl ChatRouter {
    pub fn new(db: Db, llm: Arc<LlmManager>, ingest_config: IngestConfig) -> Self {
        Self {
            db,
            llm,
            ingest_config,
        }
    }

    /// Main entry point: context → intent → dispatch → sub-graph (via FlowRunner).
    /// Sends stage events via `tx` so the frontend can show progress.
    pub async fn handle_message(
        &self,
        mut data: ChatFlowData,
        tx: &StageSender,
    ) -> Result<ChatFlowData, String> {
        // ── Step 1: Gather minimal context for intent classification ──
        emit_stage(tx, "context", "Loading notebook context...", 5).await;

        let context_task = ChatContextTask {
            db: self.db.clone(),
            tx: tx.clone(),
        };
        let tmp_ctx = Context::new();
        tmp_ctx.set("data", data.clone()).await;
        let _ = context_task
            .run(tmp_ctx.clone())
            .await
            .map_err(|e| e.to_string())?;
        data = tmp_ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or(data);

        // ── Step 2: Intent classification ──
        emit_stage(tx, "intent", "Analyzing intent...", 15).await;

        let intent_task = IntentTask {
            llm: self.llm.clone(),
        };
        let intent_ctx = Context::new();
        intent_ctx.set("data", data.clone()).await;
        let _ = intent_task
            .run(intent_ctx.clone())
            .await
            .map_err(|e| e.to_string())?;
        data = intent_ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or(data);

        info!("ChatRouter: intent = {}", data.intent);

        emit_stage(
            tx,
            "dispatch",
            &format!("Intent: {} — routing...", data.intent),
            20,
        )
        .await;

        // ── Step 3: Dispatch to sub-graph (each uses graph-flow FlowRunner) ──
        match data.intent.as_str() {
            "ask" => {
                let runner = AskGraphRunner::new(self.llm.clone(), self.db.clone());
                runner.run(data, tx).await
            }
            "source_chat" => {
                let runner = SourceChatGraphRunner::new(self.llm.clone(), self.db.clone());
                runner.run(data, tx).await
            }
            _ => {
                let runner = ChatGraphRunner::new(self.llm.clone(), self.db.clone());
                runner.run(data, tx).await
            }
        }
    }

    pub async fn process_source(&self, data: SourceFlowData) -> Result<SourceFlowData, String> {
        let runner = SourceGraphRunner::new(
            self.db.clone(),
            self.llm.clone(),
            self.ingest_config.clone(),
        );
        runner.run(data).await
    }
}
