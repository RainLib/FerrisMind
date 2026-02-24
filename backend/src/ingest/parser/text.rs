use super::{
    image_placeholder, new_image_id, DocumentParser, ExtractedImage, IngestFile, ParseResult,
};

pub struct TextParser;

impl DocumentParser for TextParser {
    fn supported_mime_types(&self) -> &[&str] {
        &["text/plain", "text/markdown", "text/x-markdown"]
    }

    fn parse(&self, file: &IngestFile) -> anyhow::Result<ParseResult> {
        let text = std::str::from_utf8(&file.data)
            .map_err(|e| anyhow::anyhow!("File '{}' is not valid UTF-8: {}", file.filename, e))?;

        let is_markdown = file.mime_type == "text/markdown"
            || file.mime_type == "text/x-markdown"
            || file.filename.ends_with(".md");

        let (processed_text, images) = if is_markdown {
            replace_md_images(text)
        } else {
            (text.to_string(), Vec::new())
        };

        let sections: Vec<String> = if is_markdown {
            split_markdown_sections(&processed_text)
        } else {
            processed_text
                .split("\n\n")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        Ok(ParseResult { sections, images })
    }
}

/// Split markdown by top-level headings, keeping each heading with its content.
fn split_markdown_sections(text: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        if line.starts_with('#') && !current.trim().is_empty() {
            sections.push(current.trim().to_string());
            current.clear();
        }
        current.push_str(line);
        current.push('\n');
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sections.push(trimmed);
    }

    sections
}

/// Replace `![alt](url)` image references with `[IMAGE:{uuid}]` placeholders.
fn replace_md_images(text: &str) -> (String, Vec<ExtractedImage>) {
    let mut result = String::with_capacity(text.len());
    let mut images = Vec::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("![") {
        result.push_str(&remaining[..start]);

        let after_bang = &remaining[start..];

        // Find the closing bracket ](
        if let Some(bracket_close) = after_bang.find("](") {
            let paren_start = bracket_close + 2;
            if let Some(paren_close) = after_bang[paren_start..].find(')') {
                let url = &after_bang[paren_start..paren_start + paren_close];

                let id = new_image_id();
                result.push_str(&image_placeholder(&id));

                images.push(ExtractedImage {
                    id,
                    data: None,
                    mime_type: super::mime_from_extension(url),
                    source_ref: url.to_string(),
                });

                remaining = &after_bang[paren_start + paren_close + 1..];
                continue;
            }
        }

        // Couldn't parse — copy as-is
        result.push_str("![");
        remaining = &after_bang[2..];
    }

    result.push_str(remaining);
    (result, images)
}
