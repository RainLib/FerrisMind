use std::collections::HashMap;
use std::path::Path;
use tera::{Context, Tera};
use tracing::{info, warn};

/// A prompt manager that uses the Tera template engine for Jinja-style templates.
pub struct PromptManager {
    tera: Tera,
}

impl PromptManager {
    /// Create a new PromptManager and load templates from the given directory
    pub fn new<P: AsRef<Path>>(prompts_dir: P) -> Self {
        let path = prompts_dir.as_ref();
        let glob = format!("{}/**/*", path.to_string_lossy());

        // Initialize Tera with the prompts directory
        let mut tera = match Tera::new(&glob) {
            Ok(t) => {
                info!("Loaded prompt templates from {:?}", path);
                t
            }
            Err(e) => {
                warn!(
                    "Failed to initialize Tera with directory {:?}: {}. Falling back to empty.",
                    path, e
                );
                Tera::default()
            }
        };

        // Disable escaping for text prompts (LLM prompts are typically plain text)
        tera.autoescape_on(vec![]);

        Self { tera }
    }

    /// Render a template by name with provided variables
    pub fn render(
        &self,
        template_name: &str,
        variables: &HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let mut context = Context::new();
        for (key, value) in variables {
            context.insert(key, value);
        }

        // Tera templates usually include the extension if the glob matched them
        // We'll try common extensions or the exact name
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

    /// Render a raw string template with provided variables
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
    fn test_conditional_render() {
        let mut vars = HashMap::new();
        vars.insert("show".to_string(), "true".to_string());

        let template = "{% if show == 'true' %}Visible{% else %}Hidden{% endif %}";
        let rendered = PromptManager::render_raw(template, &vars).unwrap();
        assert_eq!(rendered, "Visible");

        vars.insert("show".to_string(), "false".to_string());
        let rendered = PromptManager::render_raw(template, &vars).unwrap();
        assert_eq!(rendered, "Hidden");
    }

    #[test]
    fn test_base_path_resolution() {
        // This is more of a documentation test.
        // If we have "prompts/source_chat/system.jinja", Tera names it "source_chat/system.jinja"
        // so render("source_chat/system") should work.
    }
}
