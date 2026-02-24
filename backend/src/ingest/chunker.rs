use crate::ingest::parser::{ChunkMetadata, TextChunk};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for text chunking behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkConfig {
    /// Maximum number of characters per chunk.
    pub chunk_size: usize,
    /// Overlap ratio between consecutive chunks (0.0 ~ 1.0). Default: 0.1 (10%).
    pub overlap_ratio: f64,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            overlap_ratio: 0.1,
        }
    }
}

impl ChunkConfig {
    pub fn new(chunk_size: usize, overlap_ratio: f64) -> Self {
        let overlap_ratio = overlap_ratio.clamp(0.0, 0.5);
        Self {
            chunk_size: chunk_size.max(100),
            overlap_ratio,
        }
    }

    /// Effective overlap in characters.
    pub fn overlap_chars(&self) -> usize {
        (self.chunk_size as f64 * self.overlap_ratio) as usize
    }

    /// Step size = chunk_size - overlap.
    pub fn step_size(&self) -> usize {
        self.chunk_size - self.overlap_chars()
    }
}

/// Stateless text chunker that splits text into overlapping chunks.
pub struct TextChunker {
    config: ChunkConfig,
}

impl TextChunker {
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &ChunkConfig {
        &self.config
    }

    /// Split a list of text sections into a flat list of overlapping chunks.
    ///
    /// Each section (e.g., a PDF page or a slide) is chunked independently.
    /// Chunks from all sections are combined with a global index.
    pub fn chunk_sections(&self, sections: &[String], source_file: &str) -> Vec<TextChunk> {
        let mut all_chunks = Vec::new();
        let mut global_index = 0;

        for (section_idx, section) in sections.iter().enumerate() {
            let section_chunks = self.chunk_text(section, source_file, section_idx, &mut global_index);
            all_chunks.extend(section_chunks);
        }

        all_chunks
    }

    /// Chunk a single text block using a sliding window with overlap.
    fn chunk_text(
        &self,
        text: &str,
        source_file: &str,
        section_index: usize,
        global_index: &mut usize,
    ) -> Vec<TextChunk> {
        let text = text.trim();
        if text.is_empty() {
            return Vec::new();
        }

        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();

        if total_chars <= self.config.chunk_size {
            let chunk = TextChunk {
                content: text.to_string(),
                index: *global_index,
                metadata: ChunkMetadata {
                    source_file: source_file.to_string(),
                    section_index,
                    char_start: 0,
                    char_end: total_chars,
                    extra: HashMap::new(),
                },
            };
            *global_index += 1;
            return vec![chunk];
        }

        let step = self.config.step_size();
        let mut chunks = Vec::new();
        let mut start = 0;

        while start < total_chars {
            let end = (start + self.config.chunk_size).min(total_chars);
            let chunk_str: String = chars[start..end].iter().collect();

            // Try to break at a word boundary (look backwards for whitespace)
            let (final_str, final_end) = if end < total_chars {
                snap_to_word_boundary(&chunk_str, start, end)
            } else {
                (chunk_str, end)
            };

            if !final_str.trim().is_empty() {
                chunks.push(TextChunk {
                    content: final_str,
                    index: *global_index,
                    metadata: ChunkMetadata {
                        source_file: source_file.to_string(),
                        section_index,
                        char_start: start,
                        char_end: final_end,
                        extra: HashMap::new(),
                    },
                });
                *global_index += 1;
            }

            start += step;
        }

        chunks
    }

    /// Returns an iterator over chunks (for streaming pipeline usage).
    pub fn chunk_sections_iter<'a>(
        &'a self,
        sections: &'a [String],
        source_file: &'a str,
    ) -> ChunkIterator<'a> {
        ChunkIterator {
            chunker: self,
            sections,
            source_file,
            section_idx: 0,
            char_offset: 0,
            global_index: 0,
        }
    }
}

/// Iterator that lazily yields chunks one at a time across sections.
pub struct ChunkIterator<'a> {
    chunker: &'a TextChunker,
    sections: &'a [String],
    source_file: &'a str,
    section_idx: usize,
    char_offset: usize,
    global_index: usize,
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = TextChunk;

    fn next(&mut self) -> Option<Self::Item> {
        let config = self.chunker.config();

        while self.section_idx < self.sections.len() {
            let section = self.sections[self.section_idx].trim();
            let chars: Vec<char> = section.chars().collect();
            let total = chars.len();

            if total == 0 || self.char_offset >= total {
                self.section_idx += 1;
                self.char_offset = 0;
                continue;
            }

            let start = self.char_offset;
            let end = (start + config.chunk_size).min(total);
            let chunk_str: String = chars[start..end].iter().collect();

            let (final_str, final_end) = if end < total {
                snap_to_word_boundary(&chunk_str, start, end)
            } else {
                (chunk_str, end)
            };

            if final_str.trim().is_empty() {
                self.char_offset += config.step_size();
                continue;
            }

            let chunk = TextChunk {
                content: final_str,
                index: self.global_index,
                metadata: ChunkMetadata {
                    source_file: self.source_file.to_string(),
                    section_index: self.section_idx,
                    char_start: start,
                    char_end: final_end,
                    extra: HashMap::new(),
                },
            };

            self.global_index += 1;
            self.char_offset += config.step_size();

            return Some(chunk);
        }

        None
    }
}

/// If the chunk doesn't end at a word boundary, backtrack to the last whitespace.
fn snap_to_word_boundary(chunk: &str, start: usize, original_end: usize) -> (String, usize) {
    if let Some(pos) = chunk.rfind(|c: char| c.is_whitespace()) {
        if pos > chunk.len() / 2 {
            let snapped = &chunk[..pos];
            return (snapped.to_string(), start + pos);
        }
    }
    (chunk.to_string(), original_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = ChunkConfig::default();
        assert_eq!(cfg.chunk_size, 1000);
        assert!((cfg.overlap_ratio - 0.1).abs() < f64::EPSILON);
        assert_eq!(cfg.overlap_chars(), 100);
        assert_eq!(cfg.step_size(), 900);
    }

    #[test]
    fn test_chunker_small_text() {
        let chunker = TextChunker::new(ChunkConfig::new(100, 0.1));
        let sections = vec!["Hello world".to_string()];
        let chunks = chunker.chunk_sections(&sections, "test.txt");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Hello world");
        assert_eq!(chunks[0].index, 0);
    }

    #[test]
    fn test_chunker_overlap() {
        let chunker = TextChunker::new(ChunkConfig::new(100, 0.2));
        let text = "a]".repeat(120);
        let sections = vec![text];
        let chunks = chunker.chunk_sections(&sections, "test.txt");
        assert!(chunks.len() > 1, "expected >1 chunks, got {}", chunks.len());
        assert_eq!(chunks[0].metadata.char_start, 0);
        assert_eq!(chunker.config().overlap_chars(), 20);
        assert_eq!(chunker.config().step_size(), 80);
    }

    #[test]
    fn test_chunk_iterator_matches_batch() {
        let chunker = TextChunker::new(ChunkConfig::new(50, 0.1));
        let sections = vec![
            "This is the first section with enough text to span multiple chunks hopefully.".to_string(),
            "Second section also has some text.".to_string(),
        ];
        let batch = chunker.chunk_sections(&sections, "test.txt");
        let iter_chunks: Vec<_> = chunker.chunk_sections_iter(&sections, "test.txt").collect();
        assert_eq!(batch.len(), iter_chunks.len());
    }
}
