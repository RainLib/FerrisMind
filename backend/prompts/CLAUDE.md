# Prompts Module (Rust)

Jinja2 prompt templates for the FerrisMind (Open Notebook) AI workspace, rendered via the Rust `PromptManager`.

## Purpose

Centralized prompt repository using the **Tera** engine to:

1. Separate prompt engineering from Rust application logic.
2. Provide reusable Jinja2 templates with variable injection.
3. Support advanced logic (conditionals, loops) within prompts.
4. Ensure consistency across workflows (chat, search, podcast).

## Architecture Overview

**Template Organization by Workflow**:

- **`ask/`**: Multi-stage search synthesis (context-aware).
- **`chat/`**: Conversational agent with notebook context.
- **`source_chat/`**: Source-focused discussion with source metadata.
- **`podcast/`**: Podcast generation pipeline (outlines, transcripts).

**Rust Integration Pattern**:
Templates are managed by `LlmManager` and its internal `PromptManager`.

```rust
// Example usage in Rust
let mut vars = HashMap::new();
vars.insert("context".to_string(), "Document content...".to_string());
vars.insert("question".to_string(), "What is this about?".to_string());

// Render template: "chat/system.jinja" maps to "chat/system"
let prompt = llm_manager.prompt().render("chat/system", &vars)?;
```

## Prompt Engineering Patterns

### 1. Multi-Stage Chain (Ask Workflow)

Three-template chain for intelligent search:

- `entry.jinja`: Analyzes user question -> search strategy.
- `query_process.jinja`: Search term + results -> sub-answer.
- `final_answer.jinja`: Synthesis of all results with citations.

### 2. Conditional Logic (Podcast & Chat)

Templates use Jinja2/Tera conditional blocks for modular context:

```jinja
{% if notebook %}
# PROJECT INFORMATION
{{ notebook }}
{% endif %}
```

### 3. Standardized Citations

All templates enforce the following citation format to minimize hallucinations:

- Syntax: `[source:id]`, `[note:id]`, `[insight:id]`
- Explicitly forbids manufactured IDs.

### 4. Format Instructions

Templates accept a `{{ format_instructions }}` variable for dynamic output schema injection (e.g., JSON/Schema instructions from the caller).

## Reference Catalog

- **`ask/`**: Search synthesis pipeline templates.
- **`chat/`**: General conversational agent system prompts.
- **`source_chat/`**: Context-specific system prompts for individual sources.
- **`podcast/`**: Multi-step podcast generation (outlines and dialogues).

## Key Dependencies (Backend)

- **Tera**: Pure-Rust Jinja2 template engine implementation.
- **PromptManager**: Internal Rust class for file-based template resolution and rendering.

## How to Add a New Template

1. **Create subdirectory** in `backend/prompts/` (e.g., `prompts/new_feature/`).
2. **Define `.jinja` file(s)** using Jinja2 syntax.
3. **Register in Logic**: Call `render("new_feature/filename", &vars)` from your Rust service.

## Performance & Naming Notes

1. **Template Naming**: Subdirectory prefixes are required (e.g., `"ask/entry"`).
2. **Auto-Extension**: The system looks for the exact name OR the name with a `.jinja` suffix.
3. **Caching**: Templates are loaded and compiled by Tera into memory once when the `PromptManager` is initialized (typically at server startup).
4. **Auto-Escaping**: Disabled for prompts to ensure LLM text remains bit-exact and formatting is preserved.

## Testing Templates

You can run the built-in unit tests in the backend:

```bash
cargo test llm::prompt::tests
```

To test a raw string manually in Rust:

```rust
PromptManager::render_raw("Hello {{ name }}!", &vars);
```
