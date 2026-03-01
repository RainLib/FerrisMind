use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use surrealdb_types::ToSql;
use tracing::{debug, info, warn};

use crate::db::Db;
use crate::llm::manager::LlmManager;

// ─── LLM extraction DTOs (JSON schema that the LLM must produce) ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub label: String,
    pub entity_type: String,
    #[serde(default)]
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelation {
    pub from_label: String,
    pub to_label: String,
    pub relation_type: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_confidence() -> f64 {
    0.8
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgExtractionResult {
    #[serde(default)]
    pub entities: Vec<ExtractedEntity>,
    #[serde(default)]
    pub relations: Vec<ExtractedRelation>,
}

// ─── KgExtractor ───

pub struct KgExtractor {
    db: Db,
    llm: Arc<LlmManager>,
    /// Maximum tokens per article window sent to the LLM for extraction.
    max_tokens_per_window: usize,
}

impl KgExtractor {
    pub fn new(db: Db, llm: Arc<LlmManager>) -> Self {
        Self {
            db,
            llm,
            max_tokens_per_window: 8_000,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens_per_window = max_tokens;
        self
    }

    // ─── Token-aware article splitting ───

    /// Splits `text` into windows each containing at most `max_tokens` tokens.
    /// Uses tiktoken cl100k_base encoding. Splits on paragraph boundaries
    /// (double newline) to avoid cutting mid-sentence.
    pub fn split_by_token_budget(text: &str, max_tokens: usize) -> Vec<String> {
        use tiktoken_rs::cl100k_base;

        let bpe = match cl100k_base() {
            Ok(b) => b,
            Err(e) => {
                warn!("tiktoken init failed ({}), falling back to char split", e);
                return Self::split_by_char_budget(text, max_tokens * 4);
            }
        };

        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut windows: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut current_tokens = 0usize;

        for para in &paragraphs {
            let para_tokens = bpe.encode_with_special_tokens(para).len();

            if current_tokens + para_tokens > max_tokens && !current.is_empty() {
                windows.push(current.trim().to_string());
                current = String::new();
                current_tokens = 0;
            }

            // If a single paragraph exceeds the budget, hard-split it by chars
            if para_tokens > max_tokens {
                let char_chunks = Self::split_by_char_budget(para, max_tokens * 4);
                for chunk in char_chunks {
                    windows.push(chunk);
                }
                continue;
            }

            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(para);
            current_tokens += para_tokens;
        }

        if !current.trim().is_empty() {
            windows.push(current.trim().to_string());
        }

        if windows.is_empty() && !text.trim().is_empty() {
            windows.push(text.trim().to_string());
        }

        windows
    }

    fn split_by_char_budget(text: &str, max_chars: usize) -> Vec<String> {
        text.chars()
            .collect::<Vec<_>>()
            .chunks(max_chars)
            .map(|c| c.iter().collect())
            .collect()
    }

    // ─── LLM extraction for a single window ───

    async fn extract_from_window(&self, window: &str) -> anyhow::Result<KgExtractionResult> {
        let system_prompt = r#"You are a knowledge graph extraction assistant.
Extract all named entities and their relationships from the provided text.

Return ONLY a valid JSON object with this exact schema:
{
  "entities": [
    {"label": "entity name", "entity_type": "PERSON|ORG|CONCEPT|LOCATION|EVENT|PRODUCT|OTHER", "properties": {}}
  ],
  "relations": [
    {"from_label": "entity A", "to_label": "entity B", "relation_type": "VERB_PHRASE", "confidence": 0.9}
  ]
}

Rules:
- entity_type must be one of: PERSON, ORG, CONCEPT, LOCATION, EVENT, PRODUCT, OTHER
- relation_type should be a concise verb phrase (e.g. "works_at", "is_part_of", "created_by")
- confidence is a float 0.0-1.0 reflecting how certain you are
- Return empty arrays if nothing relevant is found
- Do NOT include markdown fences, only raw JSON"#;

        let agent = self.llm.agent_with_preamble(system_prompt);
        let prompt = format!(
            "Extract entities and relations from this text:\n\n{}",
            window
        );

        let raw = agent
            .prompt_with_retry(&prompt, "KG extraction")
            .await
            .map_err(|e| anyhow::anyhow!("KG LLM call failed: {}", e))?;

        let trimmed = raw
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str::<KgExtractionResult>(trimmed).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse KG JSON: {}. Raw: {}",
                e,
                &trimmed[..trimmed.len().min(200)]
            )
        })
    }

    // ─── Main entry point: extract + store ───

    /// Extracts KG from the full article text and stores entities/relations in SurrealDB.
    ///
    /// - `doc_id`       : e.g. "document:abc123"
    /// - `notebook_id`  : e.g. "notebook:xyz"
    /// - `full_text`    : concatenated plain text of the entire document
    /// - `chunk_id_map` : chunk_index → SurrealDB chunk record id (for linking entities to source chunks)
    pub async fn extract_and_store(
        &self,
        doc_id: &str,
        notebook_id: &str,
        full_text: &str,
        chunk_id_map: &HashMap<usize, String>,
    ) -> anyhow::Result<()> {
        if full_text.trim().is_empty() {
            info!("KgExtractor: empty text for doc {}, skipping", doc_id);
            return Ok(());
        }

        let windows = Self::split_by_token_budget(full_text, self.max_tokens_per_window);
        info!(
            "KgExtractor: {} windows for doc '{}' (total {} chars)",
            windows.len(),
            doc_id,
            full_text.len()
        );

        // Pick the most relevant chunk id (first window → first chunk)
        let first_chunk_id = chunk_id_map.get(&0).cloned();

        // label → SurrealDB entity record id (built up per-window, then shared)
        let mut entity_id_map: HashMap<String, String> = HashMap::new();

        for (win_idx, window) in windows.iter().enumerate() {
            debug!(
                "KgExtractor: processing window {}/{}",
                win_idx + 1,
                windows.len()
            );

            let result = match self.extract_from_window(window).await {
                Ok(r) => r,
                Err(e) => {
                    warn!("KgExtractor window {} failed: {}", win_idx, e);
                    continue;
                }
            };

            // Approximate: map window index to the nearest chunk
            let approx_chunk_idx = (win_idx * self.max_tokens_per_window / 1000)
                .min(chunk_id_map.len().saturating_sub(1));
            let chunk_ref = chunk_id_map
                .get(&approx_chunk_idx)
                .or(first_chunk_id.as_ref())
                .cloned()
                .unwrap_or_default();

            // Upsert entities
            for entity in &result.entities {
                let label_key = entity.label.to_lowercase();
                if entity_id_map.contains_key(&label_key) {
                    continue; // already stored in a prior window
                }

                let props_json =
                    serde_json::to_string(&entity.properties).unwrap_or_else(|_| "{}".to_string());
                let chunk_opt = if chunk_ref.is_empty() {
                    "NONE".to_string()
                } else {
                    format!("type::record('{}')", chunk_ref)
                };

                let created: Vec<surrealdb_types::RecordId> = self
                    .db
                    .query(format!(
                        "CREATE kg_entity SET \
                            notebook = type::record($nb_id), \
                            document = type::record($doc_id), \
                            chunk_id = {chunk_opt}, \
                            label = $label, \
                            entity_type = $entity_type, \
                            properties = $props, \
                            is_active = true"
                    ))
                    .bind(("nb_id", notebook_id.to_string()))
                    .bind(("doc_id", doc_id.to_string()))
                    .bind(("label", entity.label.clone()))
                    .bind(("entity_type", entity.entity_type.clone()))
                    .bind(("props", props_json))
                    .await
                    .map_err(|e| anyhow::anyhow!("Store kg_entity failed: {}", e))?
                    .take("id")
                    .unwrap_or_default();

                if let Some(rid) = created.into_iter().next() {
                    entity_id_map.insert(label_key, rid.to_sql());
                }
            }

            // Upsert relations via RELATE
            for rel in &result.relations {
                let from_key = rel.from_label.to_lowercase();
                let to_key = rel.to_label.to_lowercase();

                let (Some(from_id), Some(to_id)) =
                    (entity_id_map.get(&from_key), entity_id_map.get(&to_key))
                else {
                    debug!(
                        "KgExtractor: skipping relation '{}' -> '{}' (entity not found yet)",
                        rel.from_label, rel.to_label
                    );
                    continue;
                };

                let chunk_opt = if chunk_ref.is_empty() {
                    "NONE".to_string()
                } else {
                    format!("type::record('{}')", chunk_ref)
                };

                if let Err(e) = self
                    .db
                    .query(format!(
                        "RELATE type::record($from_id)->kg_relation->type::record($to_id) \
                         SET notebook = type::record($nb_id), \
                             relation_type = $rel_type, \
                             confidence = $confidence, \
                             chunk_id = {chunk_opt}, \
                             is_active = true"
                    ))
                    .bind(("from_id", from_id.clone()))
                    .bind(("to_id", to_id.clone()))
                    .bind(("nb_id", notebook_id.to_string()))
                    .bind(("rel_type", rel.relation_type.clone()))
                    .bind(("confidence", rel.confidence))
                    .await
                {
                    warn!("KgExtractor: RELATE failed: {}", e);
                }
            }

            info!(
                "KgExtractor: window {} → {} entities, {} relations",
                win_idx + 1,
                result.entities.len(),
                result.relations.len()
            );
        }

        info!(
            "KgExtractor: completed for doc '{}' — {} total entities stored",
            doc_id,
            entity_id_map.len()
        );
        Ok(())
    }
}
