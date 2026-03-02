---
name: open-notebook-lm-guidelines
description: Core project guidelines, architecture, and UI styling instructions for the FerrisMind (openNotebookLm) AI workspace project. Use this whenever working on the frontend or backend of this project.
---

# FerrisMind / openNotebookLm Project Guidelines

You are working on the **FerrisMind** project, an open-source AI knowledge workspace heavily inspired by NotebookLM but with a unique, premium design identity and a Rust-driven backend.

Whenever you write or modify code for this project, you **MUST** adhere to the following architecture, styling, and structural guidelines.

## 1. Tech Stack

- **Frontend**: Next.js (App Router), React, TypeScript.
- **Styling**: Tailwind CSS v4 (configured purely via `@theme` in `src/app/globals.css`, there is **no** `tailwind.config.ts`), vanilla CSS.
- **Backend (API)**: Rust, Axum, Async-GraphQL.
- **Database**: SurrealDB (native graph DB — uses `RELATE` for graph edges, `->` / `<-` for graph traversal).
- **AI/LLM**: Multi-model abstraction via `rig-core` (Gemini, OpenAI, Anthropic, DeepSeek). Embedding via same framework.
- **Search**: Hybrid Search — Vector (cosine similarity on chunk embeddings) + Knowledge Graph (entity/relation graph traversal).

## 2. UI / UX Design System (Amber Tech-Chic Linear)

The project utilizes a highly distinct, precision-engineered "Tech-Chic" aesthetic. Do not use generic, default Tailwind components. You must follow these visual rules:

### Colors & Palette

- **Backgrounds**: White (`bg-white` inside workspace), Off-white/Stone-50 (`bg-bg-sources`, `bg-bg-studio` for sidebars).
- **Foreground/Borders**: Softer Black (`#171717`, used as `border-black`, `text-black` or `border-border-bold`).
- **Accents**: Amber (`accent-main` for primary actions like borders/buttons, `accent-secondary` for hovers, `accent-light` for backgrounds/highlights).
- **Secondary Text**: Gray (`text-gray-400`, `text-gray-500` for subtitles and metadata).

### Shadows & Components

- **NEVER use soft blurry shadows!** The design relies entirely on **hard, offset linear shadows**.
- Use the custom CSS variables defined in globals:
  - `shadow-hard` (3px 3px 0px 0px #171717)
  - `shadow-hard-sm` (1px 1px 0px 0px #171717)
  - `shadow-modal` (for floating modals, combination of soft drop and hard offset)
- For hover states, components should shift upwards `hover:-translate-y-0.5` or `hover:-translate-y-1` and increase shadow depth (e.g., `hover:shadow-hard-hover`).
- **Borders**: Elements must use 1px solid borders (`border border-black` or `border border-gray-200/300`). Use dashed borders for drop zones or empty states.
- **Corners**: Keep corners sharp or very slightly rounded (`rounded-sm` or `rounded-none`). No pill shapes (`rounded-full`) except for specific toggles or avatars.

### Typography

- **Primary Font**: `Public Sans` or `Inter`.
- **Headings**: `font-black`, `tracking-tight`, `uppercase`.
- **Labels/Overheads**: Extremely small, spaced-out uppercase tags must be heavily used: `text-[10px] font-bold uppercase tracking-widest text-gray-400/500`.
- **Tooltips**: Use absolute positioned, black background (`bg-black`), white text (`text-white`), tiny font (`text-[10px] font-bold`) tooltips instead of native HTML `title` attributes whenever possible.

### Icons

- Use Google Material Symbols Outlined.
- Structure: `<span className="material-symbols-outlined icon-sm">icon_name</span>`
- Available sizing classes: `text-[16px]`, `icon-sm` (18px), or `icon-lg` (24px).

### Background Patterns

- Utilize the custom background stripes/hatches defined in globals (`bg-background-image-diagonal-hatch`, `bg-background-image-diagonal-pattern`) for empty states, drop zones, or decorative headers.

## 3. Frontend Architecture

- **Editor Layout**: The primary app lies in `src/components/editor/EditorLayout.tsx`. It uses a 3-pane structure: `LeftSidebar` (Sources), `ChatPanel` (Center), and `RightSidebar` (Studio Tools).
- **Responsiveness**: The desktop layout uses `react-resizable-panels` for drag-to-resize columns. The mobile layout (`isMobile`) uses absolute full-screen overlays with swipe/toggle logic (`CollapsedLeftSidebar`, `CollapsedRightSidebar`).
- **State Management**: Sidebar toggle state, width dragging, and selection logic are handled locally using React hooks. When building complex components, encapsulate state into the feature component rather than polluting the global layout.

## 4. Backend Architecture

### Ingest Pipeline (`src/ingest/`)

- **Pipeline**: `IngestPipeline` — parse → sanitize → chunk → embed (via `EmbeddingProvider`).
- **Service**: `IngestionService` in `service.rs` — orchestrates streaming ingestion, stores chunks with embeddings, tracks `chunk_id_map` (chunk_index → DB record id).
- **KG Extraction**: `KgExtractor` in `kg_extractor.rs`:
  - After all chunks are embedded and stored, the **full article** text is assembled and split into token-bounded windows using `tiktoken-rs` (cl100k_base, default 8K tokens/window).
  - Each window → LLM → strict JSON schema response with `entities` and `relations`.
  - Entities stored as `kg_entity` nodes; relations via `RELATE entity_a->kg_relation->entity_b`.
  - KG extraction is async/background (non-blocking to main ingest flow).

### Knowledge Graph Data Model (SurrealDB)

- **`kg_entity`** — Node table: `notebook`, `document`, `chunk_id`, `label`, `entity_type`, `properties`, `is_active`, `created_at`.
- **`kg_relation`** — Edge table (`TYPE RELATION IN kg_entity OUT kg_entity`): `notebook`, `relation_type`, `confidence`, `chunk_id`, `is_active`, `created_at`. SurrealDB manages `in`/`out` automatically.
- **Soft-delete**: Removing a source → `is_active = false` on its KG records (never hard-delete). All KG queries filter by `is_active = true`.
- **Notebook isolation**: All KG records carry a `notebook` field, subgraphs are per-notebook.

### Query Flows (`src/graph/`)

All query flows use `graph-flow` crate (`FlowRunner`/`GraphBuilder`) to compose task DAGs with shared `ChatFlowData` context.

- **Chat flow** (`chat.rs`): `ChatContextTask → ChatKgSearchTask → ChatSearchTask → ChatResponseTask`
- **Ask flow** (`ask.rs`): `AskEntryTask → AskSearchTask → AskKgSearchTask → AskQueryProcessTask → AskFinalAnswerTask`
- **KG Search** (`kg_search.rs`): `KgSearcher` — single SurrealQL query does label fuzzy-match + 1-hop bidirectional graph expansion via `->` and `<-` operators.
- **Context fields**: `ChatFlowData` carries `kg_hits`, `kg_context`, `search_results`, `sub_answers` etc.

### LLM Manager (`src/llm/`)

- `LlmManager` wraps `RigClient` + `PromptManager`.
- `llm.agent()` for no-preamble, `llm.agent_with_preamble(...)` for custom preambles.
- `AnyAgent` supports `.prompt()`, `.prompt_with_retry()`, `.stream_to_sse()`.

### Database Schema (`src/db/schema.rs`)

- Applied at startup via idempotent `DEFINE` statements.
- Tables: `user`, `notebook`, `has_access` (RELATION), `document`, `doc_image`, `chunk`, `session`, `message`, `kg_entity`, `kg_relation` (RELATION).

## 5. Workflows & Rules

1. **Modifying UI**: Refer to _Shadows & Components_ and _Typography_ guidelines. Match the "Tech-Chic" vibe. No generic blue buttons or soft shadows.
2. **Adding Dependencies**: Ask before adding new npm/Cargo packages.
3. **Icons Check**: Use the exact string name for `material-symbols-outlined` icons.
4. **Tailwind Config**: Modify values in `src/app/globals.css` via `@theme`, not `tailwind.config.ts`.
5. **KG Conventions**:
   - Extracted KG data must carry `notebook` + `document` references.
   - Removing sources → soft-delete KG (`is_active = false`), never hard-delete.
   - Query flows must include `kg_context` in prompt template variables.
6. **Graph Flows**: Each query type is a task DAG in `src/graph/`. Adding a stage = create a `Task` struct → add to `GraphBuilder` → wire edges.

Whenever taking an action on the UI, confirm it aligns with the strict visual hierarchy and Amber/Off-black aesthetic.
