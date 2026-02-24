use std::pin::Pin;
use std::sync::Arc;

use async_stream::stream;
use futures::Stream;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::ingest::chunker::{ChunkConfig, TextChunker};
use crate::ingest::parser::{ExtractedImage, IngestFile, ParseResult, ParserRegistry, TextChunk};
use crate::ingest::sanitizer::SanitizerChain;

/// The result of embedding a single chunk.
#[derive(Debug, Clone)]
pub struct EmbeddedChunk {
    pub chunk: TextChunk,
    pub embedding: Vec<f64>,
}

/// Full output of the ingestion pipeline for a single file.
#[derive(Debug)]
pub struct IngestOutput {
    /// SHA256 hex digest of the raw file bytes.
    pub sha256: String,
    /// All embedded chunks.
    pub embedded_chunks: Vec<EmbeddedChunk>,
    /// All images extracted during parsing.
    pub images: Vec<ExtractedImage>,
}

/// Trait for embedding providers — allows swapping Gemini, OpenAI, local models, etc.
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f64>>;

    async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f64>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }
}

/// Streaming ingestion pipeline: parse → sanitize → chunk → embed.
pub struct IngestPipeline {
    registry: ParserRegistry,
    sanitizer: SanitizerChain,
    chunker: TextChunker,
    embedder: Arc<dyn EmbeddingProvider>,
    batch_size: usize,
}

/// Yielded items from the streaming ingestion.
pub enum IngestStreamItem {
    Chunk(EmbeddedChunk),
    /// Emitted once at the start, before any chunks.
    Meta {
        sha256: String,
        images: Vec<ExtractedImage>,
    },
}

impl IngestPipeline {
    pub fn new(
        registry: ParserRegistry,
        chunk_config: ChunkConfig,
        embedder: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            registry,
            sanitizer: SanitizerChain::with_defaults(),
            chunker: TextChunker::new(chunk_config),
            embedder,
            batch_size: 16,
        }
    }

    /// Replace the default sanitizer chain with a custom one.
    pub fn with_sanitizer(mut self, sanitizer: SanitizerChain) -> Self {
        self.sanitizer = sanitizer;
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }

    pub fn chunker(&self) -> &TextChunker {
        &self.chunker
    }

    /// Compute SHA256 of raw file bytes.
    pub fn compute_sha256(data: &[u8]) -> String {
        let hash = Sha256::digest(data);
        format!("{:x}", hash)
    }

    /// Full (non-streaming) ingestion: parse → chunk → embed.
    pub async fn ingest(&self, file: &IngestFile) -> anyhow::Result<IngestOutput> {
        let sha256 = Self::compute_sha256(&file.data);

        let (chunks, images) = self.parse_and_chunk(file)?;
        info!(
            "Parsed '{}' into {} chunks + {} images, starting embedding...",
            file.filename,
            chunks.len(),
            images.len()
        );

        let mut embedded = Vec::with_capacity(chunks.len());
        for batch in chunks.chunks(self.batch_size) {
            let texts: Vec<String> = batch.iter().map(|c| c.content.clone()).collect();
            let embeddings = self.embedder.embed_batch(&texts).await?;

            for (chunk, embedding) in batch.iter().zip(embeddings.into_iter()) {
                embedded.push(EmbeddedChunk {
                    chunk: chunk.clone(),
                    embedding,
                });
            }
            debug!("Embedded batch of {} chunks", batch.len());
        }

        info!(
            "Completed embedding for '{}': {} chunks",
            file.filename,
            embedded.len()
        );

        Ok(IngestOutput {
            sha256,
            embedded_chunks: embedded,
            images,
        })
    }

    /// Streaming ingestion: yields `IngestStreamItem` one at a time.
    ///
    /// The first item is always `Meta { sha256, images }`, followed by embedded chunks.
    pub fn ingest_stream<'a>(
        &'a self,
        file: &'a IngestFile,
    ) -> Pin<Box<dyn Stream<Item = anyhow::Result<IngestStreamItem>> + Send + 'a>> {
        Box::pin(stream! {
            let sha256 = Self::compute_sha256(&file.data);

            let (chunks, images) = match self.parse_and_chunk(file) {
                Ok(r) => r,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };

            info!(
                "Streaming {} chunks + {} images from '{}' (sha256: {})",
                chunks.len(),
                images.len(),
                file.filename,
                &sha256[..12]
            );

            // Yield metadata first so the caller can check dedup / store images
            yield Ok(IngestStreamItem::Meta { sha256, images });

            for batch_chunks in chunks.chunks(self.batch_size) {
                let texts: Vec<String> = batch_chunks.iter().map(|c| c.content.clone()).collect();

                match self.embedder.embed_batch(&texts).await {
                    Ok(embeddings) => {
                        for (chunk, embedding) in batch_chunks.iter().zip(embeddings.into_iter()) {
                            yield Ok(IngestStreamItem::Chunk(EmbeddedChunk {
                                chunk: chunk.clone(),
                                embedding,
                            }));
                        }
                    }
                    Err(e) => {
                        warn!("Embedding batch failed: {}", e);
                        yield Err(e);
                        return;
                    }
                }
            }

            info!("Streaming ingestion complete for '{}'", file.filename);
        })
    }

    /// Parse → sanitize → chunk. Returns (chunks, images).
    fn parse_and_chunk(
        &self,
        file: &IngestFile,
    ) -> anyhow::Result<(Vec<TextChunk>, Vec<ExtractedImage>)> {
        let mime = if file.mime_type.is_empty() || file.mime_type == "application/octet-stream" {
            ParserRegistry::resolve_mime(&file.filename)
        } else {
            file.mime_type.clone()
        };

        let parser = self.registry.find(&mime).ok_or_else(|| {
            anyhow::anyhow!(
                "No parser found for MIME type '{}' (file: '{}')",
                mime,
                file.filename
            )
        })?;

        let ParseResult { sections, images } = parser.parse(file)?;
        if sections.is_empty() {
            anyhow::bail!("Parser returned no text sections for '{}'", file.filename);
        }

        // Sanitize: strip HTML tags, control chars, normalize whitespace, etc.
        let clean_sections = self.sanitizer.sanitize_sections(&sections);
        if clean_sections.is_empty() {
            anyhow::bail!(
                "All sections became empty after sanitization for '{}'",
                file.filename
            );
        }

        debug!(
            "Sanitized '{}': {} → {} sections",
            file.filename,
            sections.len(),
            clean_sections.len()
        );

        let chunks = self.chunker.chunk_sections(&clean_sections, &file.filename);
        if chunks.is_empty() {
            anyhow::bail!("Chunker produced no chunks for '{}'", file.filename);
        }

        Ok((chunks, images))
    }
}
