use bytes::Bytes;

use super::{
    image_placeholder, new_image_id, DocumentParser, ExtractedImage, IngestFile, ParseResult,
};

pub struct HtmlParser;

impl DocumentParser for HtmlParser {
    fn supported_mime_types(&self) -> &[&str] {
        &["text/html", "application/xhtml+xml"]
    }

    fn parse(&self, file: &IngestFile) -> anyhow::Result<ParseResult> {
        let html_str = std::str::from_utf8(&file.data)
            .map_err(|e| anyhow::anyhow!("HTML '{}' is not valid UTF-8: {}", file.filename, e))?;

        // Replace <img> tags with placeholders BEFORE converting to text
        let (replaced_html, images) = replace_img_tags(html_str);

        let plain_text = html2text::from_read(replaced_html.as_bytes(), 120)
            .map_err(|e| anyhow::anyhow!("HTML rendering failed for '{}': {}", file.filename, e))?;

        let sections: Vec<String> = plain_text
            .split("\n\n")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(ParseResult { sections, images })
    }
}

/// Scan HTML for `<img ...>` tags, replace each with `[IMAGE:{uuid}]`,
/// and collect extracted images (data URIs are decoded, URLs are stored as references).
fn replace_img_tags(html: &str) -> (String, Vec<ExtractedImage>) {
    let mut result = String::with_capacity(html.len());
    let mut images = Vec::new();
    let mut remaining = html;

    while let Some(img_start) = remaining.find("<img") {
        result.push_str(&remaining[..img_start]);

        let after_tag = &remaining[img_start..];
        if let Some(tag_end) = after_tag.find('>') {
            let img_tag = &after_tag[..tag_end + 1];
            let src = extract_html_attr(img_tag, "src").unwrap_or_default();

            let id = new_image_id();
            result.push_str(&image_placeholder(&id));

            let (data, mime_type) = decode_image_src(&src);
            images.push(ExtractedImage {
                id,
                data,
                mime_type,
                source_ref: src,
            });

            remaining = &after_tag[tag_end + 1..];
        } else {
            result.push_str(after_tag);
            remaining = "";
        }
    }

    result.push_str(remaining);
    (result, images)
}

/// Extract an attribute value from an HTML tag string.
fn extract_html_attr(tag: &str, attr_name: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let pattern = format!("{}={}", attr_name, quote);
        if let Some(start) = tag.find(&pattern) {
            let value_start = start + pattern.len();
            if let Some(end) = tag[value_start..].find(quote) {
                return Some(tag[value_start..value_start + end].to_string());
            }
        }
    }
    None
}

/// For data URIs, decode the base64 content. For regular URLs, return as-is with no data.
fn decode_image_src(src: &str) -> (Option<Bytes>, String) {
    if src.starts_with("data:") {
        // data:image/png;base64,iVBOR...
        if let Some((header, b64_data)) = src.split_once(",") {
            let mime = header
                .strip_prefix("data:")
                .and_then(|h| h.split(';').next())
                .unwrap_or("image/png")
                .to_string();

            use base64::Engine;
            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(b64_data) {
                return (Some(Bytes::from(decoded)), mime);
            }
        }
        (None, "image/png".to_string())
    } else {
        // Regular URL — store reference only, no download
        let mime = super::mime_from_extension(src);
        (None, mime)
    }
}
