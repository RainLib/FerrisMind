use std::collections::HashMap;
use tera::{Context, Tera};
use tracing::info;

/// A prompt manager that embeds all templates at compile time via `include_str!`
/// and uses Tera for Jinja-style rendering.
pub struct PromptManager {
    tera: Tera,
}

impl PromptManager {
    /// Create a new PromptManager with all templates compiled into the binary.
    pub fn new<P: AsRef<std::path::Path>>(_prompts_dir: P) -> Self {
        let mut tera = Tera::default();
        tera.autoescape_on(vec![]);

        // All templates are embedded at compile time — no runtime file I/O.
        let templates: &[(&str, &str)] = &[
            (
                "chat/system.jinja",
                include_str!("../../prompts/chat/system.jinja"),
            ),
            (
                "ask/entry.jinja",
                include_str!("../../prompts/ask/entry.jinja"),
            ),
            (
                "ask/query_process.jinja",
                include_str!("../../prompts/ask/query_process.jinja"),
            ),
            (
                "ask/final_answer.jinja",
                include_str!("../../prompts/ask/final_answer.jinja"),
            ),
            (
                "intent/classify.jinja",
                include_str!("../../prompts/intent/classify.jinja"),
            ),
            (
                "source_chat/system.jinja",
                include_str!("../../prompts/source_chat/system.jinja"),
            ),
            (
                "suggest/system.jinja",
                include_str!("../../prompts/suggest/system.jinja"),
            ),
            (
                "suggest/from_question.jinja",
                include_str!("../../prompts/suggest/from_question.jinja"),
            ),
        ];

        for (name, content) in templates {
            if let Err(e) = tera.add_raw_template(name, content) {
                panic!("Failed to register inline template '{}': {}", name, e);
            }
        }

        let names: Vec<&str> = tera.get_template_names().collect();
        info!(
            "Loaded {} inline prompt templates: {:?}",
            names.len(),
            names
        );

        Self { tera }
    }

    /// Render a template by name with provided variables.
    pub fn render(
        &self,
        template_name: &str,
        variables: &HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let mut context = Context::new();
        for (key, value) in variables {
            context.insert(key, value);
        }

        // Try exact name first, then with .jinja suffix
        let possible_names = [
            template_name.to_string(),
            format!("{}.jinja", template_name),
        ];

        for name in &possible_names {
            if self.tera.get_template_names().any(|n| n == name) {
                return self
                    .tera
                    .render(name, &context)
                    .map_err(|e| anyhow::anyhow!(e));
            }
        }

        Err(anyhow::anyhow!(
            "Template '{}' not found in loaded prompts",
            template_name
        ))
    }

    /// Render a raw string template with provided variables.
    pub fn render_raw(
        template: &str,
        variables: &HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let mut context = Context::new();
        for (key, value) in variables {
            context.insert(key, value);
        }
        Tera::one_off(template, &context, false).map_err(|e| anyhow::anyhow!(e))
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new("prompts")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_raw() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let rendered = PromptManager::render_raw("Hello {{ name }}!", &vars).unwrap();
        assert_eq!(rendered, "Hello World!");
    }

    #[test]
    fn test_inline_templates_loaded() {
        let pm = PromptManager::new("prompts");
        let mut vars = HashMap::new();
        vars.insert("notebook".to_string(), "Test Notebook".to_string());
        vars.insert("has_sources".to_string(), "true".to_string());
        vars.insert("search_results".to_string(), String::new());
        vars.insert("context".to_string(), String::new());

        // Should not panic or error
        let result = pm.render("chat/system", &vars);
        assert!(
            result.is_ok(),
            "chat/system render failed: {:?}",
            result.err()
        );
    }
}
