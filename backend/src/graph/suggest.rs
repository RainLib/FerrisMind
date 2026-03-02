use crate::graph::context::{emit_stage, ChatFlowData, StageSender};
use crate::llm::manager::LlmManager;
use async_trait::async_trait;
use graph_flow::{Context, GraphError, NextAction, Task, TaskResult};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

/// Generate 3 suggested follow-up questions from the user's question only.
/// Designed to run in parallel with the main answer flow (no dependency on answer).
pub async fn generate_suggestions_parallel(
    llm: Arc<LlmManager>,
    question: String,
) -> Vec<String> {
    if question.trim().is_empty() {
        return Vec::new();
    }
    let mut vars = HashMap::new();
    vars.insert("question".to_string(), question);
    let prompt_text = match llm.prompt().render("suggest/from_question", &vars) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to render suggest/from_question: {}", e);
            return Vec::new();
        }
    };
    let agent = llm.agent();
    match agent
        .prompt_with_retry(&prompt_text, "SuggestQuestions parallel")
        .await
    {
        Ok(raw) => {
            let trimmed = raw
                .trim()
                .trim_start_matches("```json")
                .trim_end_matches("```")
                .trim();
            match serde_json::from_str::<Vec<String>>(trimmed) {
                Ok(questions) => {
                    let q: Vec<String> = questions.into_iter().take(3).collect();
                    info!("generate_suggestions_parallel: {} suggestions", q.len());
                    q
                }
                Err(e) => {
                    warn!("Parse suggestion JSON failed: {}, raw: {}", e, &trimmed[..trimmed.len().min(150)]);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            warn!("SuggestQuestions parallel LLM failed: {}", e);
            Vec::new()
        }
    }
}

pub struct SuggestQuestionsTask {
    pub llm: Arc<LlmManager>,
    pub tx: StageSender,
}

#[async_trait]
impl Task for SuggestQuestionsTask {
    fn id(&self) -> &str {
        "SuggestQuestionsTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        emit_stage(
            &self.tx,
            "suggest",
            "Generating follow-up suggestions...",
            90,
        )
        .await;

        let mut data = ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or_default();

        if data.response.is_empty() {
            info!("SuggestQuestionsTask: no response to base suggestions on, skipping");
            ctx.set("data", data).await;
            emit_stage(&self.tx, "complete", "Done", 100).await;
            return Ok(TaskResult::new(
                Some("Skipped suggestions".to_string()),
                NextAction::End,
            ));
        }

        let response_preview = if data.response.len() > 1500 {
            format!("{}...", &data.response[..1500])
        } else {
            data.response.clone()
        };

        let mut vars = HashMap::new();
        vars.insert("question".to_string(), data.message.clone());
        vars.insert("response".to_string(), response_preview);
        vars.insert(
            "notebook_context".to_string(),
            if data.notebook_context.len() > 500 {
                data.notebook_context[..500].to_string()
            } else {
                data.notebook_context.clone()
            },
        );

        let prompt_text = match self.llm.prompt().render("suggest/system", &vars) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to render suggest prompt: {}", e);
                emit_stage(&self.tx, "complete", "Done", 100).await;
                ctx.set("data", data).await;
                return Ok(TaskResult::new(
                    Some("Suggestions skipped (template error)".to_string()),
                    NextAction::End,
                ));
            }
        };

        let agent = self.llm.agent();
        match agent
            .prompt_with_retry(&prompt_text, "SuggestQuestions LLM call")
            .await
        {
            Ok(raw) => {
                let trimmed = raw
                    .trim()
                    .trim_start_matches("```json")
                    .trim_end_matches("```")
                    .trim();

                match serde_json::from_str::<Vec<String>>(trimmed) {
                    Ok(questions) => {
                        let questions: Vec<String> =
                            questions.into_iter().take(3).collect();
                        info!("SuggestQuestionsTask: generated {} suggestions", questions.len());
                        data.suggested_questions = questions;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to parse suggestion JSON: {}, raw: {}",
                            e,
                            &trimmed[..trimmed.len().min(200)]
                        );
                    }
                }
            }
            Err(e) => {
                warn!("SuggestQuestions LLM call failed: {}", e);
            }
        }

        ctx.set("data", data).await;
        emit_stage(&self.tx, "complete", "Done", 100).await;

        Ok(TaskResult::new(
            Some("Suggestions generated".to_string()),
            NextAction::End,
        ))
    }
}
