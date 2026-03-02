use crate::llm::manager::LlmManager;
use async_trait::async_trait;
use graph_flow::{
    Context, FlowRunner, GraphBuilder, GraphError, InMemorySessionStorage, NextAction, Session,
    SessionStorage, Task, TaskResult,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FlowData {
    pub query: String,
    pub research_notes: Vec<String>,
    pub final_summary: String,
}

pub struct ResearchTask {
    pub llm: Arc<LlmManager>,
}

#[async_trait]
impl Task for ResearchTask {
    fn id(&self) -> &str {
        "ResearchTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        let mut data = ctx.get::<FlowData>("data").await.unwrap_or_default();

        info!("Executing ResearchTask for query: {}", data.query);

        // Real LLM call using rig agent
        let agent = self.llm.agent_with_preamble(
            "You are a research assistant. Extract 3 key facts or research points about the user's query.",
        );

        let response = agent
            .prompt(&data.query)
            .await
            .map_err(|e| GraphError::TaskExecutionFailed(format!("LLM prompt failed: {}", e)))?;

        data.research_notes.push(response);

        ctx.set("data", data).await;

        Ok(TaskResult::new(
            Some("Research completed".to_string()),
            NextAction::Continue,
        ))
    }
}

pub struct SummarizeTask {
    pub llm: Arc<LlmManager>,
}

#[async_trait]
impl Task for SummarizeTask {
    fn id(&self) -> &str {
        "SummarizeTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        let mut data = ctx.get::<FlowData>("data").await.unwrap_or_default();

        info!(
            "Executing SummarizeTask with {} research notes",
            data.research_notes.len()
        );

        let notes = data.research_notes.join("\n\n---\n\n");

        // Real LLM call for summarization
        let agent = self.llm.agent_with_preamble(
            "You are a summarization assistant. Create a concise summary of the research notes provided.",
        );

        let prompt = format!("Query: {}\n\nResearch Notes:\n{}", data.query, notes);
        let response = agent
            .prompt(&prompt)
            .await
            .map_err(|e| GraphError::TaskExecutionFailed(format!("LLM prompt failed: {}", e)))?;

        data.final_summary = response;

        ctx.set("data", data).await;

        Ok(TaskResult::new(
            Some("Summarization completed".to_string()),
            NextAction::End,
        ))
    }
}

pub struct WorkflowManager {
    pub llm: Arc<LlmManager>,
}

impl WorkflowManager {
    pub fn new(llm: Arc<LlmManager>) -> Self {
        Self { llm }
    }

    pub async fn run_research_flow(&self, query: String) -> Result<FlowData, String> {
        let data = FlowData {
            query,
            ..Default::default()
        };

        let graph = Arc::new(
            GraphBuilder::new("research_flow")
                .add_task(Arc::new(ResearchTask {
                    llm: self.llm.clone(),
                }))
                .add_task(Arc::new(SummarizeTask {
                    llm: self.llm.clone(),
                }))
                .add_edge("ResearchTask", "SummarizeTask")
                .set_start_task("ResearchTask")
                .build(),
        );

        let storage = Arc::new(InMemorySessionStorage::new());
        let runner = FlowRunner::new(graph.clone(), storage.clone());

        let sid = "temp_session".to_string();
        let mut session = Session::new_from_task(sid.clone(), "ResearchTask");
        session.graph_id = "research_flow".to_string();
        session.context.set("data", data).await;

        storage.save(session).await.map_err(|e| e.to_string())?;

        runner.run(&sid).await.map_err(|e| e.to_string())?;

        let final_session = storage
            .get(&sid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Session not found after run".to_string())?;

        let final_data = final_session
            .context
            .get::<FlowData>("data")
            .await
            .ok_or_else(|| "Failed to retrieve final data from context".to_string())?;

        Ok(final_data)
    }
}
