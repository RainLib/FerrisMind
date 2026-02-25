use crate::graph::context::ChatFlowData;
use crate::llm::manager::LlmManager;
use async_trait::async_trait;
use graph_flow::{Context, GraphError, NextAction, Task, TaskResult};
use rig::completion::Prompt;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Deserialize)]
struct IntentResponse {
    intent: String,
    #[allow(dead_code)]
    confidence: f64,
    #[allow(dead_code)]
    reasoning: String,
}

/// Classifies user intent into ask / chat / source_chat.
pub struct IntentTask {
    pub llm: Arc<LlmManager>,
}

#[async_trait]
impl Task for IntentTask {
    fn id(&self) -> &str {
        "IntentTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        let mut data = ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or_default();

        info!("IntentTask: classifying message: {}", &data.message);

        let has_sources = !data.notebook_context.is_empty();
        let has_active_source = data.source_id.is_some();

        let mut vars = HashMap::new();
        vars.insert("message".to_string(), data.message.clone());
        vars.insert("has_sources".to_string(), has_sources.to_string());
        vars.insert("source_count".to_string(), "unknown".to_string());
        vars.insert(
            "has_active_source".to_string(),
            has_active_source.to_string(),
        );
        if let Some(ref sid) = data.source_id {
            vars.insert("active_source_id".to_string(), sid.clone());
        }

        let history_str = data
            .chat_history
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        vars.insert("chat_history".to_string(), history_str);

        let prompt_text = match self
            .llm
            .prompt()
            .render("intent/classify", &vars)
        {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to render intent prompt, defaulting to 'chat': {}", e);
                data.intent = "chat".to_string();
                ctx.set("data", data).await;
                return Ok(TaskResult::new(
                    Some("Intent: chat (fallback)".to_string()),
                    NextAction::Continue,
                ));
            }
        };

        let agent = self.llm.agent().build();
        let raw = agent.prompt(&prompt_text).await.map_err(|e| {
            GraphError::TaskExecutionFailed(format!("Intent classification LLM call failed: {}", e))
        })?;

        let trimmed = raw.trim().trim_start_matches("```json").trim_end_matches("```").trim();

        match serde_json::from_str::<IntentResponse>(trimmed) {
            Ok(parsed) => {
                let intent = match parsed.intent.as_str() {
                    "ask" | "chat" | "source_chat" => parsed.intent,
                    _ => "chat".to_string(),
                };
                info!("IntentTask result: {} (conf={:.2})", intent, parsed.confidence);
                data.intent = intent;
            }
            Err(e) => {
                warn!("Failed to parse intent response, defaulting to 'chat': {}", e);
                data.intent = "chat".to_string();
            }
        }

        // If intent is source_chat but no active source, downgrade to ask
        if data.intent == "source_chat" && data.source_id.is_none() {
            info!("IntentTask: source_chat but no active source, downgrading to ask");
            data.intent = "ask".to_string();
        }

        ctx.set("data", data).await;

        Ok(TaskResult::new(
            Some("Intent classified".to_string()),
            NextAction::Continue,
        ))
    }
}
