use bytes::Bytes;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Cursor;

use super::{
    image_placeholder, mime_from_extension, new_image_id, parse_ooxml_rels, DocumentParser,
    ExtractedImage, IngestFile, ParseResult,
};

pub struct DocxParser;

impl DocumentParser for DocxParser {
    fn supported_mime_types(&self) -> &[&str] {
        &[
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "application/x-docx",
        ]
    }

    fn parse(&self, file: &IngestFile) -> anyhow::Result<ParseResult> {
        let cursor = Cursor::new(file.data.as_ref());
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
            anyhow::anyhow!("Failed to open DOCX '{}' as zip: {}", file.filename, e)
        })?;

        // 1. Parse relationships to map rId → media file path
        let rels = read_zip_entry_bytes(&mut archive, "word/_rels/document.xml.rels")
            .map(|b| parse_ooxml_rels(&b))
            .unwrap_or_default();

        // 2. Parse document.xml — extract text + detect image references
        let xml_data = read_zip_entry_bytes(&mut archive, "word/document.xml").map_err(|e| {
            anyhow::anyhow!("DOCX '{}' missing word/document.xml: {}", file.filename, e)
        })?;

        let (sections, image_refs) = extract_docx_text_and_images(&xml_data)?;

        // 3. Read actual image bytes from the zip
        let mut images = Vec::new();
        for (placeholder_id, r_id) in &image_refs {
            if let Some(target) = rels.get(r_id) {
                let media_path = if target.starts_with('/') {
                    target[1..].to_string()
                } else {
                    format!("word/{}", target)
                };

                if let Ok(data) = read_zip_entry_bytes(&mut archive, &media_path) {
                    images.push(ExtractedImage {
                        id: placeholder_id.clone(),
                        data: Some(Bytes::from(data)),
                        mime_type: mime_from_extension(&media_path),
                        source_ref: media_path,
                    });
                }
            }
        }

        Ok(ParseResult { sections, images })
    }
}

/// Extract text grouped by `<w:p>` paragraphs, inserting `[IMAGE:{id}]` when `<a:blip>` is found.
///
/// Returns (sections, Vec<(placeholder_id, relationship_id)>).
fn extract_docx_text_and_images(
    xml_bytes: &[u8],
) -> anyhow::Result<(Vec<String>, Vec<(String, String)>)> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut sections = Vec::new();
    let mut image_refs: Vec<(String, String)> = Vec::new();
    let mut current_paragraph = String::new();
    let mut inside_text = false;
    let mut inside_drawing = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"t" => inside_text = true,
                    b"drawing" | b"pict" => inside_drawing = true,
                    _ => {}
                }
                if inside_drawing && local.as_ref() == b"blip" {
                    if let Some(r_id) = extract_embed_attr(e) {
                        let id = new_image_id();
                        current_paragraph.push_str(&image_placeholder(&id));
                        image_refs.push((id, r_id));
                    }
                }
            }
            Ok(Event::Empty(ref e)) if inside_drawing => {
                if e.local_name().as_ref() == b"blip" {
                    if let Some(r_id) = extract_embed_attr(e) {
                        let id = new_image_id();
                        current_paragraph.push_str(&image_placeholder(&id));
                        image_refs.push((id, r_id));
                    }
                }
            }
            Ok(Event::Text(e)) if inside_text => {
                current_paragraph.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"t" => inside_text = false,
                    b"drawing" | b"pict" => inside_drawing = false,
                    b"p" => {
                        let trimmed = current_paragraph.trim().to_string();
                        if !trimmed.is_empty() {
                            sections.push(trimmed);
                        }
                        current_paragraph.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parse error in DOCX: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok((sections, image_refs))
}

/// Extract the `r:embed` attribute from an `<a:blip>` element.
fn extract_embed_attr(e: &quick_xml::events::BytesStart<'_>) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == b"embed" {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

fn read_zip_entry_bytes(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> anyhow::Result<Vec<u8>> {
    let mut entry = archive.by_name(name)?;
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut buf)?;
    Ok(buf)
}
