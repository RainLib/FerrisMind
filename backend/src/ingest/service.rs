use std::sync::Arc;
use tracing::{error, info};

use crate::db::Db;
use crate::error::AppError;
use crate::graphql::types::DocumentRecord;
use crate::llm::manager::LlmManager;

pub struct IngestionService {
    db: Db,
    llm: Arc<LlmManager>,
}

impl IngestionService {
    pub fn new(db: Db, llm: Arc<LlmManager>) -> Self {
        Self { db, llm }
    }

    /// Process a document asynchronously.
    /// This handles global deduplication (via sha256), text extraction,
    /// chunking, and LLM embedding generation.
    pub async fn process_document(self: Arc<Self>, document_id: String) {
        info!("Starting ingestion process for document: {}", document_id);

        let doc: Option<DocumentRecord> = match self
            .db
            .query("SELECT * FROM type::thing($id)")
            .bind(("id", document_id.clone()))
            .await
        {
            Ok(mut res) => res.take(0).unwrap_or(None),
            Err(e) => {
                error!("Failed to fetch document {}: {}", document_id, e);
                return;
            }
        };

        if let Some(doc) = doc {
            // Check for global deduplication
            if let Some(ref sha256) = doc.sha256 {
                let existing_docs: Vec<DocumentRecord> = match self
                    .db
                    .query("SELECT * FROM document WHERE sha256 = $sha256 AND upload_status = 'completed' AND id != type::thing($id) LIMIT 1")
                    .bind(("sha256", sha256.clone()))
                    .bind(("id", document_id.clone()))
                    .await
                {
                    Ok(mut res) => res.take(0).unwrap_or_default(),
                    Err(_) => vec![],
                };

                if let Some(existing) = existing_docs.first() {
                    info!(
                        "Global deduplication hit! Reusing parsed content from document: {:?}",
                        existing.id
                    );
                    // Fast path: Reuse chunks from the existing document
                    if let Err(e) = self.duplicate_chunks(&doc, existing).await {
                        error!("Failed to reuse chunks: {}", e);
                        self.mark_status(&document_id, "failed").await;
                    } else {
                        // Mark current document as completed
                        self.mark_status(&document_id, "completed").await;
                    }
                    return;
                }
            }

            // Normal Path: Update status to processing
            self.mark_status(&document_id, "processing").await;

            // TODO: Extract text based on source_type (file/url/text)
            let extracted_text = "Simulated extracted content. This will later come from pdf-extract, reqwest HTML parsers, or direct text uploads.".to_string();

            // TODO: Chunking
            let chunks = crate::ingest::chunker::Chunker::split_text(&extracted_text, 1000, 100);

            // TODO: Embedding & Storage
            info!(
                "Generated {} chunks for document {}",
                chunks.len(),
                document_id
            );

            // Mark complete
            self.mark_status(&document_id, "completed").await;
        } else {
            error!("Document {} not found for ingestion.", document_id);
        }
    }

    async fn duplicate_chunks(
        &self,
        new_doc: &DocumentRecord,
        source_doc: &DocumentRecord,
    ) -> Result<(), AppError> {
        // Here we would copy Chunk record metadata but keep pointing to the same document/embeddings,
        // or just link the chunks to the new notebook.
        // For simplicity in MVP, we just query existing chunks and insert clones for the new document's context.
        info!(
            "Duplicating {} chunks for new notebook context.",
            source_doc.chunk_count
        );

        let notebook_id = new_doc.notebook.to_string();
        let target_doc_id = new_doc
            .id
            .as_ref()
            .map(|t| t.to_string())
            .unwrap_or_default();
        let source_doc_id = source_doc
            .id
            .as_ref()
            .map(|t| t.to_string())
            .unwrap_or_default();

        let query = format!(
            "
            BEGIN TRANSACTION;

            LET $source_chunks = (SELECT content, chunk_index, metadata, embedding FROM chunk WHERE document = type::thing('{}'));

            FOR $chunk IN $source_chunks {{
                CREATE chunk SET
                    document = type::thing('{}'),
                    notebook = type::thing('{}'),
                    content = $chunk.content,
                    chunk_index = $chunk.chunk_index,
                    metadata = $chunk.metadata,
                    embedding = $chunk.embedding;
            }};

            UPDATE type::thing('{}') SET chunk_count = type::number((SELECT count() FROM chunk WHERE document = type::thing('{}'))[0].count);

            COMMIT TRANSACTION;
            ",
            source_doc_id, target_doc_id, notebook_id, target_doc_id, target_doc_id
        );

        self.db
            .query(&query)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    async fn mark_status(&self, id: &str, status: &str) {
        if let Err(e) = self
            .db
            .query("UPDATE type::thing($id) SET upload_status = $status")
            .bind(("id", id.to_string()))
            .bind(("status", status.to_string()))
            .await
        {
            error!("Failed to update status for {}: {}", id, e);
        }
    }
}
