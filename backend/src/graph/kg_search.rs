use serde::{Deserialize, Serialize};
use surrealdb_types::{SurrealValue, ToSql};
use tracing::info;

use crate::db::Db;

// ─── Output types ───

#[derive(Debug, Clone, Serialize, Deserialize, Default, SurrealValue)]
pub struct NeighborInfo {
    pub label: String,
    pub entity_type: String,
    pub relation_type: String,
    pub chunk_id: Option<surrealdb_types::RecordId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KgHit {
    pub entity_id: String,
    pub label: String,
    pub entity_type: String,
    pub chunk_id: Option<String>,
    pub out_neighbors: Vec<NeighborInfo>,
    pub in_neighbors: Vec<NeighborInfo>,
}

/// Formatted text block ready for insertion into LLM prompts.
pub fn kg_hits_to_context(hits: &[KgHit]) -> String {
    if hits.is_empty() {
        return String::new();
    }
    let mut parts = Vec::new();
    for hit in hits {
        let mut entry = format!("[KG Entity] {} ({})", hit.label, hit.entity_type);
        if !hit.out_neighbors.is_empty() {
            let rels: Vec<String> = hit
                .out_neighbors
                .iter()
                .map(|n| {
                    format!(
                        "  --[{}]--> {} ({})",
                        n.relation_type, n.label, n.entity_type
                    )
                })
                .collect();
            entry.push_str(&format!("\n  Relations:\n{}", rels.join("\n")));
        }
        if !hit.in_neighbors.is_empty() {
            let rels: Vec<String> = hit
                .in_neighbors
                .iter()
                .map(|n| {
                    format!(
                        "  <--[{}]-- {} ({})",
                        n.relation_type, n.label, n.entity_type
                    )
                })
                .collect();
            entry.push_str(&format!("\n  Incoming:\n{}", rels.join("\n")));
        }
        parts.push(entry);
    }
    parts.join("\n\n")
}

// ─── Raw DB row for graph traversal result ───

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
struct KgEntityRow {
    pub id: Option<surrealdb_types::RecordId>,
    pub label: String,
    pub entity_type: String,
    pub chunk_id: Option<surrealdb_types::RecordId>,
    #[serde(default)]
    pub out_neighbors: Vec<NeighborInfo>,
    #[serde(default)]
    pub in_neighbors: Vec<NeighborInfo>,
}

// ─── KgSearcher ───

pub struct KgSearcher {
    pub db: Db,
}

impl KgSearcher {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Search entities by label (case-insensitive substring match) within a notebook,
    /// then expand 1-hop neighbors in both directions using SurrealDB graph traversal.
    pub async fn search_with_expand(&self, notebook_id: &str, terms: &[&str]) -> Vec<KgHit> {
        if terms.is_empty() {
            return vec![];
        }

        // Build WHERE clause for multi-term search
        let term_clauses: Vec<String> = terms
            .iter()
            .enumerate()
            .map(|(i, _)| format!("string::lowercase(label) CONTAINS $term{i}"))
            .collect();
        let where_terms = term_clauses.join(" OR ");

        let query = format!(
            "SELECT id, label, entity_type, chunk_id,
               ->(kg_relation WHERE is_active = true AND notebook = type::record($nb_id))
                  ->(kg_entity WHERE is_active = true).{{
                       label,
                       entity_type,
                       chunk_id,
                       relation_type: ..in.relation_type
                    }} AS out_neighbors,
               <-(kg_relation WHERE is_active = true AND notebook = type::record($nb_id))
                  <-(kg_entity WHERE is_active = true).{{
                       label,
                       entity_type,
                       chunk_id,
                       relation_type: ..out.relation_type
                    }} AS in_neighbors
             FROM kg_entity
             WHERE is_active = true
               AND notebook = type::record($nb_id)
               AND ({where_terms})
             LIMIT 15"
        );

        let mut q = self
            .db
            .query(&query)
            .bind(("nb_id", notebook_id.to_string()));
        for (i, term) in terms.iter().enumerate() {
            q = q.bind((format!("term{i}"), term.to_lowercase()));
        }

        let rows: Vec<KgEntityRow> = match q.await {
            Ok(mut res) => res.take(0).unwrap_or_default(),
            Err(e) => {
                tracing::warn!("KgSearcher query failed: {}", e);
                return vec![];
            }
        };

        let hits: Vec<KgHit> = rows
            .into_iter()
            .map(|r| KgHit {
                entity_id: r.id.as_ref().map(|t| t.to_sql()).unwrap_or_default(),
                label: r.label,
                entity_type: r.entity_type,
                chunk_id: r.chunk_id.map(|c| c.to_sql()),
                out_neighbors: r.out_neighbors,
                in_neighbors: r.in_neighbors,
            })
            .collect();

        info!(
            "KgSearcher: {} hits for terms {:?} in notebook {}",
            hits.len(),
            terms,
            notebook_id
        );
        hits
    }

    /// Extract simple keyword terms from a user message (split on whitespace,
    /// filter stop-words and short tokens).
    pub fn extract_terms(message: &str) -> Vec<String> {
        let stop_words = [
            "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "shall", "should", "may", "might", "must",
            "can", "could", "to", "of", "in", "on", "at", "by", "for", "with", "about", "as",
            "from", "that", "this", "these", "those", "what", "which", "who", "how", "when",
            "where", "why", "and", "or", "but", "not", "it", "its",
        ];

        message
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 3)
            .filter(|w| !stop_words.contains(&w.to_lowercase().as_str()))
            .map(|w| w.to_lowercase())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(8) // cap to avoid too-wide DB queries
            .collect()
    }
}
