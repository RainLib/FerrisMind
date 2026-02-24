pub mod docx;
pub mod excel;
pub mod html;
pub mod legacy;
pub mod pdf;
pub mod pptx;
pub mod text;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a file to be ingested, regardless of source (upload, S3, MQ, etc.)
#[derive(Debug, Clone)]
pub struct IngestFile {
    pub filename: String,
    pub mime_type: String,
    pub data: Bytes,
}

/// A single chunk of text extracted and split from a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub content: String,
    pub index: usize,
    pub metadata: ChunkMetadata,
}

/// Metadata attached to each chunk for traceability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub source_file: String,
    pub section_index: usize,
    pub char_start: usize,
    pub char_end: usize,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// An image extracted from a document during parsing.
#[derive(Debug, Clone)]
pub struct ExtractedImage {
    /// Unique ID used as placeholder reference: `[IMAGE:{id}]`
    pub id: String,
    /// Raw image bytes. `None` for URL-only references (e.g., remote `<img src>`).
    pub data: Option<Bytes>,
    /// MIME type of the image (image/png, image/jpeg, etc.)
    pub mime_type: String,
    /// Original source reference (file path in zip, URL, page number, etc.)
    pub source_ref: String,
}

/// Result returned by every `DocumentParser::parse()` call.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Text sections with `[IMAGE:{id}]` placeholders where images appeared.
    pub sections: Vec<String>,
    /// All images extracted from the document, keyed by the placeholder ID.
    pub images: Vec<ExtractedImage>,
}

// ─── Placeholder helpers ───

pub const IMAGE_PLACEHOLDER_PREFIX: &str = "[IMAGE:";
pub const IMAGE_PLACEHOLDER_SUFFIX: &str = "]";

pub fn image_placeholder(id: &str) -> String {
    format!("{}{}{}", IMAGE_PLACEHOLDER_PREFIX, id, IMAGE_PLACEHOLDER_SUFFIX)
}

pub fn new_image_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ─── OOXML shared helpers ───

/// Parse an OOXML relationships file (`_rels/*.rels`) and return a map of `rId → target_path`.
pub fn parse_ooxml_rels(xml_bytes: &[u8]) -> HashMap<String, String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut rels = HashMap::new();
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                if e.local_name().as_ref() == b"Relationship" {
                    let mut id = String::new();
                    let mut target = String::new();
                    for attr in e.attributes().flatten() {
                        match attr.key.local_name().as_ref() {
                            b"Id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                            b"Target" => target = String::from_utf8_lossy(&attr.value).to_string(),
                            _ => {}
                        }
                    }
                    if !id.is_empty() && !target.is_empty() {
                        rels.insert(id, target);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    rels
}

/// Guess image MIME type from file extension.
pub fn mime_from_extension(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.ends_with(".png") {
        "image/png".into()
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg".into()
    } else if lower.ends_with(".gif") {
        "image/gif".into()
    } else if lower.ends_with(".bmp") {
        "image/bmp".into()
    } else if lower.ends_with(".svg") {
        "image/svg+xml".into()
    } else if lower.ends_with(".webp") {
        "image/webp".into()
    } else if lower.ends_with(".tiff") || lower.ends_with(".tif") {
        "image/tiff".into()
    } else if lower.ends_with(".emf") {
        "image/emf".into()
    } else if lower.ends_with(".wmf") {
        "image/wmf".into()
    } else {
        "application/octet-stream".into()
    }
}

// ─── DocumentParser trait ───

/// Trait that all document parsers must implement.
///
/// Each parser handles one or more MIME types and returns text sections plus
/// extracted images. Images in the text are replaced with `[IMAGE:{id}]` placeholders.
pub trait DocumentParser: Send + Sync {
    fn supported_mime_types(&self) -> &[&str];

    /// Parse the input file and return text sections + extracted images.
    fn parse(&self, file: &IngestFile) -> anyhow::Result<ParseResult>;
}

// ─── ParserRegistry ───

/// Registry that holds all available parsers and dispatches by MIME type.
pub struct ParserRegistry {
    parsers: Vec<Box<dyn DocumentParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    /// Build a registry pre-loaded with all built-in parsers.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(pdf::PdfParser));
        registry.register(Box::new(docx::DocxParser));
        registry.register(Box::new(pptx::PptxParser));
        registry.register(Box::new(html::HtmlParser));
        registry.register(Box::new(text::TextParser));
        registry.register(Box::new(excel::ExcelParser));
        registry.register(Box::new(legacy::LegacyOfficeParser));
        registry
    }

    pub fn register(&mut self, parser: Box<dyn DocumentParser>) {
        self.parsers.push(parser);
    }

    /// Find a suitable parser for the given MIME type.
    pub fn find(&self, mime_type: &str) -> Option<&dyn DocumentParser> {
        self.parsers
            .iter()
            .find(|p| p.supported_mime_types().iter().any(|m| *m == mime_type))
            .map(|p| p.as_ref())
    }

    /// Resolve MIME type from file extension as a fallback.
    pub fn resolve_mime(filename: &str) -> String {
        mime_guess::from_path(filename)
            .first_or_octet_stream()
            .to_string()
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
