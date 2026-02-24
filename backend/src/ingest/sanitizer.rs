use crate::ingest::parser::IMAGE_PLACEHOLDER_PREFIX;

// ─── Trait ───

/// A single text-cleaning rule. Implement this to add custom sanitization logic.
pub trait TextSanitizer: Send + Sync {
    fn name(&self) -> &str;
    fn sanitize(&self, text: &str) -> String;
}

/// Ordered chain of sanitizers. Sections pass through each rule in sequence.
pub struct SanitizerChain {
    sanitizers: Vec<Box<dyn TextSanitizer>>,
}

impl SanitizerChain {
    pub fn new() -> Self {
        Self {
            sanitizers: Vec::new(),
        }
    }

    /// Default chain with all built-in sanitizers in recommended order.
    pub fn with_defaults() -> Self {
        let mut chain = Self::new();
        chain.add(Box::new(HtmlTagSanitizer));
        chain.add(Box::new(ControlCharSanitizer));
        chain.add(Box::new(UnicodeSanitizer));
        chain.add(Box::new(RepeatedSymbolSanitizer));
        chain.add(Box::new(WhitespaceSanitizer));
        chain
    }

    pub fn add(&mut self, sanitizer: Box<dyn TextSanitizer>) {
        self.sanitizers.push(sanitizer);
    }

    /// Run all sanitizers on a single piece of text.
    pub fn sanitize(&self, text: &str) -> String {
        let mut result = text.to_string();
        for s in &self.sanitizers {
            result = s.sanitize(&result);
        }
        result
    }

    /// Sanitize every section in-place, dropping sections that become empty.
    pub fn sanitize_sections(&self, sections: &[String]) -> Vec<String> {
        sections
            .iter()
            .map(|s| self.sanitize(s))
            .filter(|s| !s.trim().is_empty())
            .collect()
    }
}

impl Default for SanitizerChain {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ─── Built-in sanitizers ───

/// Strip residual HTML/XML tags (e.g. `<div>`, `<span class="x">`, `</p>`).
///
/// Preserves `[IMAGE:...]` placeholders.
pub struct HtmlTagSanitizer;

impl TextSanitizer for HtmlTagSanitizer {
    fn name(&self) -> &str {
        "html_tag"
    }

    fn sanitize(&self, text: &str) -> String {
        strip_html_tags(text)
    }
}

/// Remove ASCII/Unicode control characters except common whitespace (`\n`, `\t`).
///
/// Preserves `\x0A` (newline) and `\x09` (tab); removes `\x00`-`\x08`, `\x0B`-`\x0C`,
/// `\x0E`-`\x1F`, `\x7F`, and Unicode categories Cc/Cf that are invisible.
pub struct ControlCharSanitizer;

impl TextSanitizer for ControlCharSanitizer {
    fn name(&self) -> &str {
        "control_char"
    }

    fn sanitize(&self, text: &str) -> String {
        text.chars()
            .filter(|c| {
                if *c == '\n' || *c == '\t' || *c == '\r' {
                    return true;
                }
                // Keep IMAGE placeholder characters
                if c.is_control() {
                    return false;
                }
                true
            })
            .collect()
    }
}

/// Normalize problematic Unicode:
/// - Remove BOM (`\u{FEFF}`)
/// - Remove zero-width characters (`\u{200B}`, `\u{200C}`, `\u{200D}`, `\u{FEFF}`)
/// - Normalize common lookalike punctuation (e.g. fullwidth → ASCII)
pub struct UnicodeSanitizer;

impl TextSanitizer for UnicodeSanitizer {
    fn name(&self) -> &str {
        "unicode"
    }

    fn sanitize(&self, text: &str) -> String {
        text.chars()
            .filter(|c| !matches!(c, '\u{FEFF}' | '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{00AD}'))
            .map(normalize_fullwidth_char)
            .collect()
    }
}

/// Collapse repeated decorative symbols that add no semantic value.
///
/// Patterns like `=====`, `-----`, `*****`, `~~~~~`, `#####` (when not at line-start)
/// are replaced with a single instance. Lines that consist entirely of such
/// decorations are removed.
pub struct RepeatedSymbolSanitizer;

impl TextSanitizer for RepeatedSymbolSanitizer {
    fn name(&self) -> &str {
        "repeated_symbol"
    }

    fn sanitize(&self, text: &str) -> String {
        let threshold = 4;

        text.lines()
            .filter_map(|line| {
                let trimmed = line.trim();

                // Drop lines that are purely decorative separators
                if trimmed.len() >= threshold && is_pure_decoration(trimmed) {
                    return None;
                }

                Some(collapse_repeated_symbols(line, threshold))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Collapse multiple consecutive whitespace/blank-lines into cleaner form:
/// - Multiple spaces → single space
/// - 3+ consecutive newlines → double newline
/// - Trim leading/trailing whitespace per line
pub struct WhitespaceSanitizer;

impl TextSanitizer for WhitespaceSanitizer {
    fn name(&self) -> &str {
        "whitespace"
    }

    fn sanitize(&self, text: &str) -> String {
        // First: collapse spaces within each line
        let lines: Vec<String> = text
            .lines()
            .map(|line| collapse_inline_spaces(line.trim()))
            .collect();

        // Then: collapse excessive blank lines
        collapse_blank_lines(&lines.join("\n"))
    }
}

// ─── Internal helpers ───

/// State-machine HTML tag stripper that preserves `[IMAGE:...]` placeholders.
fn strip_html_tags(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // Don't strip [IMAGE:...] — fast-path check
        if c == '[' && text[i..].starts_with(IMAGE_PLACEHOLDER_PREFIX) {
            if let Some(end) = text[i..].find(']') {
                result.push_str(&text[i..i + end + 1]);
                i += end + 1;
                continue;
            }
        }

        if c == '<' {
            // Only enter tag mode if this looks like an actual tag: <letter or </
            if i + 1 < len && (chars[i + 1].is_ascii_alphabetic() || chars[i + 1] == '/') {
                in_tag = true;
                i += 1;
                continue;
            }
        }

        if in_tag {
            if c == '>' {
                in_tag = false;
                // Replace block-level closing tags with a space so words don't merge
                result.push(' ');
            }
            i += 1;
            continue;
        }

        result.push(c);
        i += 1;
    }

    // Decode common HTML entities
    decode_html_entities(&result)
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

fn normalize_fullwidth_char(c: char) -> char {
    match c {
        '\u{FF01}'..='\u{FF5E}' => {
            // Fullwidth ASCII variants → normal ASCII
            char::from_u32(c as u32 - 0xFEE0).unwrap_or(c)
        }
        '\u{3000}' => ' ', // ideographic space → ASCII space
        _ => c,
    }
}

fn is_pure_decoration(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    let first = line.chars().next().unwrap();
    matches!(first, '=' | '-' | '_' | '*' | '~' | '#' | '+' | '.' | '·')
        && line.chars().all(|c| c == first || c.is_whitespace())
}

fn collapse_repeated_symbols(line: &str, threshold: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::with_capacity(line.len());
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if matches!(c, '=' | '-' | '_' | '*' | '~' | '+' | '·') {
            let mut run_len = 1;
            while i + run_len < chars.len() && chars[i + run_len] == c {
                run_len += 1;
            }
            if run_len >= threshold {
                result.push(c);
            } else {
                for _ in 0..run_len {
                    result.push(c);
                }
            }
            i += run_len;
        } else {
            result.push(c);
            i += 1;
        }
    }

    result
}

fn collapse_inline_spaces(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut prev_space = false;

    for c in line.chars() {
        if c == ' ' || c == '\t' {
            if !prev_space {
                result.push(' ');
            }
            prev_space = true;
        } else {
            result.push(c);
            prev_space = false;
        }
    }

    result
}

fn collapse_blank_lines(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut consecutive_empty = 0;

    for line in text.lines() {
        if line.trim().is_empty() {
            consecutive_empty += 1;
            if consecutive_empty <= 1 {
                result.push('\n');
            }
        } else {
            consecutive_empty = 0;
            result.push_str(line);
            result.push('\n');
        }
    }

    // Trim trailing newlines to at most one
    let trimmed = result.trim_end_matches('\n');
    let mut final_result = trimmed.to_string();
    if !final_result.is_empty() {
        final_result.push('\n');
    }
    final_result
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_tag_strip() {
        let sanitizer = HtmlTagSanitizer;
        let result = sanitizer.sanitize("<p>Hello <b>world</b></p>");
        assert!(result.contains("Hello"));
        assert!(result.contains("world"));
        assert!(!result.contains("<p>"));
        assert!(!result.contains("</b>"));
    }

    #[test]
    fn test_html_preserves_image_placeholder() {
        let sanitizer = HtmlTagSanitizer;
        let input = "before [IMAGE:abc-123] after <div>noise</div>";
        let result = sanitizer.sanitize(input);
        assert!(result.contains("[IMAGE:abc-123]"));
        assert!(!result.contains("<div>"));
    }

    #[test]
    fn test_html_entities() {
        let sanitizer = HtmlTagSanitizer;
        assert_eq!(sanitizer.sanitize("A &amp; B &lt; C"), "A & B < C");
    }

    #[test]
    fn test_control_char_removal() {
        let sanitizer = ControlCharSanitizer;
        let input = "Hello\x00World\x07\nNew line\tTabbed";
        let result = sanitizer.sanitize(input);
        assert_eq!(result, "HelloWorld\nNew line\tTabbed");
    }

    #[test]
    fn test_unicode_zero_width() {
        let sanitizer = UnicodeSanitizer;
        let input = "Hello\u{200B}World\u{FEFF}!";
        assert_eq!(sanitizer.sanitize(input), "HelloWorld!");
    }

    #[test]
    fn test_fullwidth_normalization() {
        let sanitizer = UnicodeSanitizer;
        // \u{FF21} = Ａ (fullwidth A), \u{FF11} = １ (fullwidth 1)
        assert_eq!(sanitizer.sanitize("\u{FF21}\u{FF11}"), "A1");
    }

    #[test]
    fn test_repeated_symbol_separator() {
        let sanitizer = RepeatedSymbolSanitizer;
        let input = "Title\n========\nContent";
        let result = sanitizer.sanitize(input);
        assert!(!result.contains("========"));
        assert!(result.contains("Title"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_repeated_inline_collapse() {
        let sanitizer = RepeatedSymbolSanitizer;
        assert_eq!(sanitizer.sanitize("a---b"), "a---b"); // 3 < threshold(4)
        assert_eq!(sanitizer.sanitize("a----b"), "a-b"); // 4 >= threshold
    }

    #[test]
    fn test_whitespace_collapse() {
        let sanitizer = WhitespaceSanitizer;
        let input = "Hello   world\n\n\n\n\nNew section";
        let result = sanitizer.sanitize(input);
        assert!(!result.contains("   "));
        // Max 2 consecutive newlines preserved
        assert!(!result.contains("\n\n\n"));
    }

    #[test]
    fn test_full_chain() {
        let chain = SanitizerChain::with_defaults();
        let input = "<div>Hello\x00   &amp;   <b>World</b></div>\n==========\n\u{200B}Nice\u{FEFF}";
        let result = chain.sanitize(input);

        assert!(!result.contains("<div>"), "HTML tags should be stripped");
        assert!(!result.contains("\x00"), "control chars should be removed");
        assert!(!result.contains("   "), "extra spaces should be collapsed");
        assert!(!result.contains("=========="), "separator should be removed");
        assert!(!result.contains('\u{200B}'), "zero-width should be removed");
        assert!(result.contains("Hello"), "content should be preserved");
        assert!(result.contains("World"), "content should be preserved");
        assert!(result.contains("Nice"), "content should be preserved");
        assert!(result.contains("&"), "entities should be decoded");
    }

    #[test]
    fn test_chain_preserves_image_placeholders() {
        let chain = SanitizerChain::with_defaults();
        let input = "Text [IMAGE:550e8400-e29b-41d4-a716-446655440000] more text";
        let result = chain.sanitize(input);
        assert!(result.contains("[IMAGE:550e8400-e29b-41d4-a716-446655440000]"));
    }

    #[test]
    fn test_sanitize_sections_drops_empty() {
        let chain = SanitizerChain::with_defaults();
        let sections = vec![
            "Good content".to_string(),
            "==========".to_string(),
            "   \n\n  ".to_string(),
            "Also good".to_string(),
        ];
        let result = chain.sanitize_sections(&sections);
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("Good content"));
        assert!(result[1].contains("Also good"));
    }
}
