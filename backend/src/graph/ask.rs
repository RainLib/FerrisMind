use crate::db::Db;
use crate::graph::context::{
    emit_stage, ChatFlowData, SearchHit, SearchStrategy, StageSender, SubAnswer,
};
use crate::llm::manager::LlmManager;
use async_trait::async_trait;
use graph_flow::{
    Context, FlowRunner, GraphBuilder, GraphError, InMemorySessionStorage, NextAction, Session,
    SessionStorage, Task, TaskResult,
};
use std::collections::HashMap;
use std::sync::Arc;
use surrealdb_types::SurrealValue;
use tracing::{info, warn};

// ─── Task 1: Entry — analyze question and produce search strategy ───

pub struct AskEntryTask {
    pub llm: Arc<LlmManager>,
    pub tx: StageSender,
}

#[async_trait]
impl Task for AskEntryTask {
    fn id(&self) -> &str {
        "AskEntryTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        emit_stage(&self.tx, "ask_entry", "Building search strategy...", 25).await;

        let mut data = ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or_default();

        info!("AskEntryTask: analyzing question: {}", data.message);

        let format_instructions = r#"Return a JSON object with keys "reasoning" (string) and "searches" (array of {"term": string, "instructions": string})."#;

        let mut vars = HashMap::new();
        vars.insert("question".to_string(), data.message.clone());
        vars.insert(
            "format_instructions".to_string(),
            format_instructions.to_string(),
        );

        let prompt_text = self
            .llm
            .prompt()
            .render("ask/entry", &vars)
            .map_err(|e| {
                GraphError::TaskExecutionFailed(format!("Failed to render ask/entry prompt: {}", e))
            })?;

        let agent = self.llm.agent();
        let raw = agent
            .prompt_with_retry(&prompt_text, "AskEntry LLM call")
            .await
            .map_err(GraphError::TaskExecutionFailed)?;

        let trimmed = raw
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();

        match serde_json::from_str::<SearchStrategy>(trimmed) {
            Ok(strategy) => {
                info!(
                    "AskEntryTask: {} search queries planned",
                    strategy.searches.len()
                );
                data.search_strategy = Some(strategy);
            }
            Err(e) => {
                warn!("Failed to parse search strategy: {}, raw: {}", e, trimmed);
                data.search_strategy = Some(SearchStrategy {
                    reasoning: "Direct search on user question".to_string(),
                    searches: vec![crate::graph::context::SearchQuery {
                        term: data.message.clone(),
                        instructions: "Answer the user's question directly.".to_string(),
                    }],
                });
            }
        }

        ctx.set("data", data).await;
        Ok(TaskResult::new(
            Some("Search strategy created".to_string()),
            NextAction::ContinueAndExecute,
        ))
    }
}

// ─── Task 2: Search — vector search for each query term ───

pub struct AskSearchTask {
    pub db: Db,
    pub llm: Arc<LlmManager>,
    pub tx: StageSender,
}

#[async_trait]
impl Task for AskSearchTask {
    fn id(&self) -> &str {
        "AskSearchTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        emit_stage(&self.tx, "ask_search", "Searching documents...", 45).await;

        let mut data = ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or_default();

        let strategy = data
            .search_strategy
            .as_ref()
            .cloned()
            .unwrap_or_default();

        info!(
            "AskSearchTask: executing {} searches in notebook {}",
            strategy.searches.len(),
            data.notebook_id
        );

        let mut all_hits: Vec<SearchHit> = Vec::new();

        for query in &strategy.searches {
            let embedding = self.get_embedding(&query.term).await?;
            let hits = self
                .vector_search(&data.notebook_id, &embedding, 5)
                .await?;
            info!("  Search '{}': {} hits", query.term, hits.len());
            all_hits.extend(hits);
        }

        all_hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all_hits.dedup_by(|a, b| a.chunk_id == b.chunk_id);

        data.search_results = all_hits;
        ctx.set("data", data).await;

        Ok(TaskResult::new(
            Some("Search completed".to_string()),
            NextAction::ContinueAndExecute,
        ))
    }
}

impl AskSearchTask {
    async fn get_embedding(&self, text: &str) -> Result<Vec<f64>, GraphError> {
        let model = self.llm.embedding_model();
        let emb = model.embed_text(text).await.map_err(|e| {
            GraphError::TaskExecutionFailed(format!("Embedding failed: {}", e))
        })?;
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

// ─── Task 3: QueryProcess — process each search term's results ───

pub struct AskQueryProcessTask {
    pub llm: Arc<LlmManager>,
    pub tx: StageSender,
}

#[async_trait]
impl Task for AskQueryProcessTask {
    fn id(&self) -> &str {
        "AskQueryProcessTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        emit_stage(&self.tx, "ask_process", "Analyzing search results...", 65).await;

        let mut data = ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or_default();

        let strategy = data
            .search_strategy
            .as_ref()
            .cloned()
            .unwrap_or_default();

        info!(
            "AskQueryProcessTask: processing {} search terms",
            strategy.searches.len()
        );

        let mut sub_answers = Vec::new();

        for query in &strategy.searches {
            let relevant: Vec<&SearchHit> = data.search_results.iter().take(5).collect();

            let results_text = relevant
                .iter()
                .map(|h| format!("[{}] (score: {:.3})\n{}", h.document_id, h.score, h.content))
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");

            let ids_text = relevant
                .iter()
                .map(|h| h.document_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            let mut vars = HashMap::new();
            vars.insert("question".to_string(), data.message.clone());
            vars.insert("term".to_string(), query.term.clone());
            vars.insert("instructions".to_string(), query.instructions.clone());
            vars.insert("results".to_string(), results_text);
            vars.insert("ids".to_string(), ids_text.clone());

            let prompt_text = self
                .llm
                .prompt()
                .render("ask/query_process", &vars)
                .map_err(|e| {
                    GraphError::TaskExecutionFailed(format!(
                        "Failed to render ask/query_process: {}",
                        e
                    ))
                })?;

            let agent = self.llm.agent();
            let answer = agent
                .prompt_with_retry(&prompt_text, "QueryProcess LLM call")
                .await
                .map_err(GraphError::TaskExecutionFailed)?;

            sub_answers.push(SubAnswer {
                term: query.term.clone(),
                answer,
                source_ids: relevant.iter().map(|h| h.document_id.clone()).collect(),
            });
        }

        data.sub_answers = sub_answers;
        ctx.set("data", data).await;

        Ok(TaskResult::new(
            Some("Query processing completed".to_string()),
            NextAction::ContinueAndExecute,
        ))
    }
}

// ─── Task 4: FinalAnswer — synthesize all sub-answers ───

pub struct AskFinalAnswerTask {
    pub llm: Arc<LlmManager>,
    pub tx: StageSender,
}

#[async_trait]
impl Task for AskFinalAnswerTask {
    fn id(&self) -> &str {
        "AskFinalAnswerTask"
    }

    async fn run(&self, ctx: Context) -> Result<TaskResult, GraphError> {
        emit_stage(&self.tx, "ask_answer", "Generating final answer...", 85).await;

        let mut data = ctx
            .get::<ChatFlowData>("data")
            .await
            .unwrap_or_default();

        let strategy = data
            .search_strategy
            .as_ref()
            .cloned()
            .unwrap_or_default();

        info!(
            "AskFinalAnswerTask: synthesizing {} sub-answers",
            data.sub_answers.len()
        );

        let answers_text = data
            .sub_answers
            .iter()
            .enumerate()
            .map(|(i, sa)| {
                format!(
                    "## Search {}: \"{}\"\n\n{}\n\nSources: {}",
                    i + 1,
                    sa.term,
                    sa.answer,
                    sa.source_ids.join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let strategy_text = serde_json::to_string_pretty(&strategy).unwrap_or_default();

        let mut vars = HashMap::new();
        vars.insert("question".to_string(), data.message.clone());
        vars.insert("strategy".to_string(), strategy_text);
        vars.insert("answers".to_string(), answers_text);

        let prompt_text = self
            .llm
            .prompt()
            .render("ask/final_answer", &vars)
            .map_err(|e| {
                GraphError::TaskExecutionFailed(format!(
                    "Failed to render ask/final_answer: {}",
                    e
                ))
            })?;

        let agent = self.llm.agent();
        let response = agent
            .stream_to_sse(&prompt_text, &self.tx, "FinalAnswer LLM call")
            .await
            .map_err(GraphError::TaskExecutionFailed)?;

        data.response = response;
        ctx.set("data", data).await;

        emit_stage(&self.tx, "complete", "Done", 100).await;

        Ok(TaskResult::new(
            Some("Final answer generated".to_string()),
            NextAction::End,
        ))
    }
}

// ─── Graph builder (using graph-flow FlowRunner) ───

pub struct AskGraphRunner {
    pub llm: Arc<LlmManager>,
    pub db: Db,
}

impl AskGraphRunner {
    pub fn new(llm: Arc<LlmManager>, db: Db) -> Self {
        Self { llm, db }
    }

    pub async fn run(
        &self,
        data: ChatFlowData,
        tx: &StageSender,
    ) -> Result<ChatFlowData, String> {
        let graph = Arc::new(
            GraphBuilder::new("ask_graph")
                .add_task(Arc::new(AskEntryTask {
                    llm: self.llm.clone(),
                    tx: tx.clone(),
                }))
                .add_task(Arc::new(AskSearchTask {
                    db: self.db.clone(),
                    llm: self.llm.clone(),
                    tx: tx.clone(),
                }))
                .add_task(Arc::new(AskQueryProcessTask {
                    llm: self.llm.clone(),
                    tx: tx.clone(),
                }))
                .add_task(Arc::new(AskFinalAnswerTask {
                    llm: self.llm.clone(),
                    tx: tx.clone(),
                }))
                .add_edge("AskEntryTask", "AskSearchTask")
                .add_edge("AskSearchTask", "AskQueryProcessTask")
                .add_edge("AskQueryProcessTask", "AskFinalAnswerTask")
                .set_start_task("AskEntryTask")
                .build(),
        );

        let storage = Arc::new(InMemorySessionStorage::new());
        let runner = FlowRunner::new(graph, storage.clone());

        let sid = format!("ask_{}", uuid::Uuid::new_v4());
        let mut session = Session::new_from_task(sid.clone(), "AskEntryTask");
        session.graph_id = "ask_graph".to_string();
        session.context.set("data", data).await;

        storage.save(session).await.map_err(|e| e.to_string())?;
        runner.run(&sid).await.map_err(|e| e.to_string())?;

        let final_session = storage
            .get(&sid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Session not found after ask graph run")?;

        final_session
            .context
            .get::<ChatFlowData>("data")
            .await
            .ok_or_else(|| "Failed to retrieve ChatFlowData".to_string())
    }
}
