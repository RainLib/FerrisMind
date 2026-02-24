use bytes::Bytes;
use std::io::Write;
use tracing::warn;

use super::{
    image_placeholder, new_image_id, DocumentParser, ExtractedImage, IngestFile, ParseResult,
};

pub struct PdfParser;

impl DocumentParser for PdfParser {
    fn supported_mime_types(&self) -> &[&str] {
        &["application/pdf"]
    }

    fn parse(&self, file: &IngestFile) -> anyhow::Result<ParseResult> {
        // 1. Extract text via pdf-extract (requires temp file)
        let mut tmp = tempfile::NamedTempFile::new()?;
        tmp.write_all(&file.data)?;
        tmp.flush()?;

        let text = pdf_extract::extract_text(tmp.path()).map_err(|e| {
            anyhow::anyhow!("PDF extraction failed for '{}': {}", file.filename, e)
        })?;

        let mut sections: Vec<String> = if text.contains('\x0C') {
            text.split('\x0C')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            text.split("\n\n")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        if sections.is_empty() {
            anyhow::bail!("PDF '{}' produced no extractable text", file.filename);
        }

        // 2. Extract images via lopdf
        let images = extract_pdf_images(&file.data, &file.filename);

        // 3. Distribute image placeholders across sections (best-effort per-page mapping)
        if !images.is_empty() {
            let section_count = sections.len();
            for (i, img) in images.iter().enumerate() {
                let target_section = (i * section_count / images.len()).min(section_count - 1);
                sections[target_section].push('\n');
                sections[target_section].push_str(&image_placeholder(&img.id));
            }
        }

        Ok(ParseResult { sections, images })
    }
}

/// Scan all PDF objects for image streams and extract their data.
fn extract_pdf_images(data: &[u8], filename: &str) -> Vec<ExtractedImage> {
    let doc = match lopdf::Document::load_mem(data) {
        Ok(d) => d,
        Err(e) => {
            warn!("lopdf failed to load '{}' for image extraction: {}", filename, e);
            return vec![];
        }
    };

    let mut images = Vec::new();

    for (&obj_id, object) in doc.objects.iter() {
        let stream = match object.as_stream() {
            Ok(s) => s,
            Err(_) => continue,
        };

        let subtype = stream
            .dict
            .get(b"Subtype")
            .ok()
            .and_then(|v| v.as_name_str().ok());

        if subtype != Some("Image") {
            continue;
        }

        let filter = stream
            .dict
            .get(b"Filter")
            .ok()
            .and_then(|v| v.as_name_str().ok())
            .unwrap_or("");

        let mime_type = match filter {
            "DCTDecode" => "image/jpeg",
            "JPXDecode" => "image/jp2",
            "CCITTFaxDecode" => "image/tiff",
            _ => "image/png",
        };

        // For DCTDecode the raw stream IS the JPEG; for others try decompression
        let image_bytes = if filter == "DCTDecode" || filter == "JPXDecode" {
            stream.content.clone()
        } else {
            stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone())
        };

        if image_bytes.is_empty() {
            continue;
        }

        let id = new_image_id();
        images.push(ExtractedImage {
            id,
            data: Some(Bytes::from(image_bytes)),
            mime_type: mime_type.to_string(),
            source_ref: format!("pdf_object_{}", obj_id.0),
        });
    }

    images
}
