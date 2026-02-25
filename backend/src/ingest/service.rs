use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;
use tracing::{error, info, warn};

use crate::config::IngestConfig;
use crate::db::Db;
use crate::error::AppError;
use crate::graphql::types::DocumentRecord;
use crate::ingest::chunker::ChunkConfig;
use crate::ingest::parser::{ExtractedImage, IngestFile, ParserRegistry};
use crate::ingest::pipeline::{EmbeddedChunk, EmbeddingProvider, IngestPipeline, IngestStreamItem};
use crate::llm::manager::LlmManager;
use surrealdb_types::ToSql;

/// Rig/Gemini-based embedding provider.
pub struct RigEmbeddingProvider {
    llm: Arc<LlmManager>,
}

impl RigEmbeddingProvider {
    pub fn new(llm: Arc<LlmManager>) -> Self {
        Self { llm }
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for RigEmbeddingProvider {
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f64>> {
        let model = self.llm.embedding_model();
        let embeddings = model
            .embed_text(text)
            .await
            .map_err(|e| anyhow::anyhow!("Embedding failed: {}", e))?;
        Ok(embeddings.vec.into_iter().map(|v| v as f64).collect())
    }
}

pub struct IngestionService {
    db: Db,
    pipeline: Arc<IngestPipeline>,
}

impl IngestionService {
    pub fn new(db: Db, llm: Arc<LlmManager>, config: &IngestConfig) -> Self {
        let chunk_config = ChunkConfig::new(config.chunk_size, config.overlap_ratio);
        let embedder: Arc<dyn EmbeddingProvider> = Arc::new(RigEmbeddingProvider::new(llm));

        let pipeline = Arc::new(
            IngestPipeline::new(ParserRegistry::with_defaults(), chunk_config, embedder)
                .with_batch_size(config.embed_batch_size),
        );

        Self { db, pipeline }
    }

    /// Main entry point: read file → compute sha256 → dedup → parse → chunk → embed → store.
    pub async fn process_document(self: Arc<Self>, document_id: String) {
        info!("Starting ingestion for document: {}", document_id);

        let doc: Option<DocumentRecord> = match self
            .db
            .query("SELECT * FROM type::record($id)")
            .bind(("id", document_id.clone()))
            .await
        {
            Ok(mut res) => res.take(0).unwrap_or(None),
            Err(e) => {
                error!("Failed to fetch document {}: {}", document_id, e);
                return;
            }
        };

        let Some(doc) = doc else {
            error!("Document {} not found for ingestion.", document_id);
            return;
        };

        // Load file bytes
        let file_data = match self.load_file_bytes(&doc).await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to load file for document {}: {}", document_id, e);
                self.mark_status(&document_id, "failed").await;
                return;
            }
        };

        // ──── SHA256 computation & deduplication ────
        let sha256 = IngestPipeline::compute_sha256(&file_data);
        info!("Document '{}' sha256: {}", doc.filename, &sha256[..12]);

        // Persist the sha256 on the document record
        if let Err(e) = self
            .db
            .query("UPDATE type::record($id) SET sha256 = $sha256")
            .bind(("id", document_id.clone()))
            .bind(("sha256", sha256.clone()))
            .await
        {
            warn!("Failed to store sha256 for {}: {}", document_id, e);
        }

        // Check for existing processed document with the same hash
        if let Some(existing) = self.find_duplicate(&sha256, &document_id).await {
            info!(
                "Dedup hit — reusing chunks from document {:?}",
                existing.id
            );
            if let Err(e) = self.duplicate_chunks(&doc, &existing).await {
                error!("Failed to reuse chunks: {}", e);
                self.mark_status(&document_id, "failed").await;
            } else {
                self.mark_status(&document_id, "completed").await;
            }
            return;
        }

        self.mark_status(&document_id, "processing").await;

        // ──── Streaming ingestion: parse → chunk → embed → store ────
        let ingest_file = IngestFile {
            filename: doc.filename.clone(),
            mime_type: doc.file_type.clone(),
            data: file_data,
        };

        let notebook_id = doc.notebook.to_sql();
        let doc_id_sql = doc.id.as_ref().map(|t| t.to_sql()).unwrap_or_default();
        let mut chunk_count: i64 = 0;

        let mut stream = self.pipeline.ingest_stream(&ingest_file);
        while let Some(result) = stream.next().await {
            match result {
                Ok(IngestStreamItem::Meta { images, .. }) => {
                    // Store extracted images
                    for img in &images {
                        if let Err(e) = self.store_image(img, &doc_id_sql, &notebook_id).await {
                            warn!("Failed to store image {}: {}", img.id, e);
                        }
                    }
                    info!(
                        "Stored {} images for document '{}'",
                        images.len(),
                        doc.filename
                    );
                }
                Ok(IngestStreamItem::Chunk(embedded)) => {
                    if let Err(e) = self
                        .store_chunk(&embedded, &doc_id_sql, &notebook_id)
                        .await
                    {
                        warn!("Failed to store chunk {}: {}", embedded.chunk.index, e);
                    } else {
                        chunk_count += 1;
                    }
                }
                Err(e) => {
                    error!("Streaming ingestion error for {}: {}", document_id, e);
                    self.mark_status(&document_id, "failed").await;
                    return;
                }
            }
        }

        // Finalize
        if let Err(e) = self
            .db
            .query(
                "UPDATE type::record($id) SET chunk_count = $count, upload_status = 'completed'",
            )
            .bind(("id", document_id.clone()))
            .bind(("count", chunk_count))
            .await
        {
            error!("Failed to finalize document {}: {}", document_id, e);
            return;
        }

        info!(
            "Ingestion complete for '{}': {} chunks stored",
            doc.filename, chunk_count
        );
    }

    // ──── Storage helpers ────

    async fn load_file_bytes(&self, doc: &DocumentRecord) -> anyhow::Result<Bytes> {
        match doc.source_type.as_str() {
            "file" => {
                let upload_dir =
                    std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
                let doc_id = doc.id.as_ref().map(|t| t.to_sql()).unwrap_or_default();
                let sanitized_id = doc_id.replace(':', "_").replace('/', "_");
                let path = std::path::Path::new(&upload_dir)
                    .join(format!("{}_{}", sanitized_id, doc.filename));

                let path = if path.exists() {
                    path
                } else {
                    std::path::Path::new(&upload_dir).join(&doc.filename)
                };

                let data = tokio::fs::read(&path).await.map_err(|e| {
                    anyhow::anyhow!("Cannot read file '{}': {}", path.display(), e)
                })?;
                Ok(Bytes::from(data))
            }
            "text" => Ok(Bytes::from(doc.filename.clone().into_bytes())),
            "url" => {
                let url = doc
                    .url
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("URL source but no URL provided"))?;
                let resp = reqwest::get(url).await?;
                let data = resp.bytes().await?;
                Ok(data)
            }
            other => anyhow::bail!("Unknown source_type: {}", other),
        }
    }

    /// Store a single embedded chunk into the database.
    async fn store_chunk(
        &self,
        embedded: &EmbeddedChunk,
        document_id: &str,
        notebook_id: &str,
    ) -> Result<(), AppError> {
        let metadata_json =
            serde_json::to_string(&embedded.chunk.metadata).unwrap_or_else(|_| "{}".to_string());

        self.db
            .query(
                "CREATE chunk SET \
                    document = type::record($doc_id), \
                    notebook = type::record($nb_id), \
                    content = $content, \
                    chunk_index = $idx, \
                    metadata = $meta, \
                    embedding = $embedding",
            )
            .bind(("doc_id", document_id.to_string()))
            .bind(("nb_id", notebook_id.to_string()))
            .bind(("content", embedded.chunk.content.clone()))
            .bind(("idx", embedded.chunk.index as i64))
            .bind(("meta", metadata_json))
            .bind(("embedding", embedded.embedding.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Store an extracted image: write bytes to disk + create a DB record.
    async fn store_image(
        &self,
        image: &ExtractedImage,
        document_id: &str,
        notebook_id: &str,
    ) -> Result<(), AppError> {
        // Write image bytes to disk (if available)
        let mut stored_path = String::new();
        if let Some(ref data) = image.data {
            let upload_dir =
                std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
            let images_dir = std::path::Path::new(&upload_dir).join("images");
            tokio::fs::create_dir_all(&images_dir)
                .await
                .map_err(|e| AppError::Internal(format!("Cannot create images dir: {}", e)))?;

            let ext = match image.mime_type.as_str() {
                "image/png" => "png",
                "image/jpeg" => "jpg",
                "image/gif" => "gif",
                "image/webp" => "webp",
                "image/bmp" => "bmp",
                "image/tiff" => "tiff",
                "image/svg+xml" => "svg",
                _ => "bin",
            };
            let filename = format!("{}.{}", image.id, ext);
            let file_path = images_dir.join(&filename);

            tokio::fs::write(&file_path, data)
                .await
                .map_err(|e| AppError::Internal(format!("Cannot write image: {}", e)))?;

            stored_path = file_path.to_string_lossy().to_string();
        }

        // Create DB record so the image ID can be resolved later
        self.db
            .query(
                "CREATE doc_image SET \
                    image_id = $image_id, \
                    document = type::record($doc_id), \
                    notebook = type::record($nb_id), \
                    mime_type = $mime, \
                    source_ref = $source_ref, \
                    stored_path = $stored_path",
            )
            .bind(("image_id", image.id.clone()))
            .bind(("doc_id", document_id.to_string()))
            .bind(("nb_id", notebook_id.to_string()))
            .bind(("mime", image.mime_type.clone()))
            .bind(("source_ref", image.source_ref.clone()))
            .bind(("stored_path", stored_path))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    // ──── Dedup helpers ────

    async fn find_duplicate(&self, sha256: &str, exclude_id: &str) -> Option<DocumentRecord> {
        let result: Vec<DocumentRecord> = match self
            .db
            .query("SELECT * FROM document WHERE sha256 = $sha256 AND upload_status = 'completed' AND id != type::record($id) LIMIT 1")
            .bind(("sha256", sha256.to_string()))
            .bind(("id", exclude_id.to_string()))
            .await
        {
            Ok(mut res) => res.take(0).unwrap_or_default(),
            Err(_) => vec![],
        };
        result.into_iter().next()
    }

    async fn duplicate_chunks(
        &self,
        new_doc: &DocumentRecord,
        source_doc: &DocumentRecord,
    ) -> Result<(), AppError> {
        info!(
            "Duplicating {} chunks for new notebook context.",
            source_doc.chunk_count
        );

        let notebook_id = new_doc.notebook.to_sql();
        let target_doc_id = new_doc.id.as_ref().map(|t| t.to_sql()).unwrap_or_default();
        let source_doc_id = source_doc
            .id
            .as_ref()
            .map(|t| t.to_sql())
            .unwrap_or_default();

        let query = format!(
            "
            BEGIN TRANSACTION;

            LET $source_chunks = (SELECT content, chunk_index, metadata, embedding FROM chunk WHERE document = type::record('{}'));

            FOR $chunk IN $source_chunks {{
                CREATE chunk SET
                    document = type::record('{}'),
                    notebook = type::record('{}'),
                    content = $chunk.content,
                    chunk_index = $chunk.chunk_index,
                    metadata = $chunk.metadata,
                    embedding = $chunk.embedding;
            }};

            LET $source_images = (SELECT image_id, mime_type, source_ref, stored_path FROM doc_image WHERE document = type::record('{}'));

            FOR $img IN $source_images {{
                CREATE doc_image SET
                    image_id = $img.image_id,
                    document = type::record('{}'),
                    notebook = type::record('{}'),
                    mime_type = $img.mime_type,
                    source_ref = $img.source_ref,
                    stored_path = $img.stored_path;
            }};

            UPDATE type::record('{}') SET chunk_count = type::number((SELECT count() FROM chunk WHERE document = type::record('{}'))[0].count);

            COMMIT TRANSACTION;
            ",
            source_doc_id,
            target_doc_id,
            notebook_id,
            source_doc_id,
            target_doc_id,
            notebook_id,
            target_doc_id,
            target_doc_id
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
            .query("UPDATE type::record($id) SET upload_status = $status")
            .bind(("id", id.to_string()))
            .bind(("status", status.to_string()))
            .await
        {
            error!("Failed to update status for {}: {}", id, e);
        }
    }
}
