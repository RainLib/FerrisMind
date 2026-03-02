use bytes::Bytes;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Cursor;

use super::{
    image_placeholder, mime_from_extension, new_image_id, parse_ooxml_rels, DocumentParser,
    ExtractedImage, IngestFile, ParseResult,
};

pub struct PptxParser;

impl DocumentParser for PptxParser {
    fn supported_mime_types(&self) -> &[&str] {
        &[
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "application/x-pptx",
        ]
    }

    fn parse(&self, file: &IngestFile) -> anyhow::Result<ParseResult> {
        let cursor = Cursor::new(file.data.as_ref());
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
            anyhow::anyhow!("Failed to open PPTX '{}' as zip: {}", file.filename, e)
        })?;

        let mut slide_names: Vec<String> = archive
            .file_names()
            .filter(|name| name.starts_with("ppt/slides/slide") && name.ends_with(".xml"))
            .map(|s| s.to_string())
            .collect();

        slide_names.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));

        let mut sections = Vec::new();
        let mut all_images = Vec::new();

        for slide_name in &slide_names {
            // Parse per-slide relationships
            let rels_path = slide_name.replace("slides/", "slides/_rels/") + ".rels";
            let rels = read_zip_entry_bytes(&mut archive, &rels_path)
                .map(|b| parse_ooxml_rels(&b))
                .unwrap_or_default();

            // Parse slide XML
            let xml_data = read_zip_entry_bytes(&mut archive, slide_name)?;
            let (slide_text, image_refs) = extract_slide_text_and_images(&xml_data)?;

            if !slide_text.is_empty() {
                sections.push(slide_text);
            }

            // Resolve image references: ../media/image1.png relative to ppt/slides/ → ppt/media/image1.png
            for (placeholder_id, r_id) in &image_refs {
                if let Some(target) = rels.get(r_id) {
                    let normalized = normalize_pptx_media_path(slide_name, target);

                    if let Ok(data) = read_zip_entry_bytes(&mut archive, &normalized) {
                        all_images.push(ExtractedImage {
                            id: placeholder_id.clone(),
                            data: Some(Bytes::from(data)),
                            mime_type: mime_from_extension(&normalized),
                            source_ref: normalized,
                        });
                    }
                }
            }
        }

        Ok(ParseResult {
            sections,
            images: all_images,
        })
    }
}

/// Extract text from a slide XML, inserting `[IMAGE:{id}]` for picture elements.
///
/// Returns (slide_text, Vec<(placeholder_id, relationship_id)>).
fn extract_slide_text_and_images(
    xml_bytes: &[u8],
) -> anyhow::Result<(String, Vec<(String, String)>)> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut texts = Vec::new();
    let mut image_refs: Vec<(String, String)> = Vec::new();
    let mut inside_text = false;
    let mut inside_pic = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"t" if !inside_pic => inside_text = true,
                    b"pic" => inside_pic = true,
                    _ => {}
                }

                if inside_pic && local.as_ref() == b"blip" {
                    if let Some(r_id) = extract_embed_attr(&e) {
                        let id = new_image_id();
                        texts.push(image_placeholder(&id));
                        image_refs.push((id, r_id));
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let local = e.local_name();
                if inside_pic && local.as_ref() == b"blip" {
                    if let Some(r_id) = extract_embed_attr(&e) {
                        let id = new_image_id();
                        texts.push(image_placeholder(&id));
                        image_refs.push((id, r_id));
                    }
                }
            }
            Ok(Event::Text(e)) if inside_text => {
                texts.push(e.unescape().unwrap_or_default().to_string());
            }
            Ok(Event::End(e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"t" => inside_text = false,
                    b"pic" => inside_pic = false,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parse error in PPTX slide: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok((texts.join(" "), image_refs))
}

fn extract_embed_attr(e: &quick_xml::events::BytesStart<'_>) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == b"embed" {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

/// Resolve a relative target path like `../media/image1.png` from `ppt/slides/slide1.xml`.
fn normalize_pptx_media_path(slide_path: &str, target: &str) -> String {
    if target.starts_with('/') {
        return target[1..].to_string();
    }

    let base = slide_path
        .rsplit_once('/')
        .map(|(dir, _)| dir)
        .unwrap_or("");

    let mut parts: Vec<&str> = base.split('/').collect();
    for segment in target.split('/') {
        if segment == ".." {
            parts.pop();
        } else if segment != "." {
            parts.push(segment);
        }
    }

    parts.join("/")
}

fn natural_sort_key(s: &str) -> (String, u32) {
    let num_part: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    let num: u32 = num_part.parse().unwrap_or(0);
    let alpha_part: String = s.chars().filter(|c| !c.is_ascii_digit()).collect();
    (alpha_part, num)
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
