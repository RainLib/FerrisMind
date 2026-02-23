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
- **Database**: SurrealDB.
- **AI/LLM**: Multi-model abstraction layer, RAG, Hybrid Search (Vector + Full-text + Knowledge Graph).

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

## 4. Workflows & Rules

1. **Modifying UI**: When asked to add a new button or panel, immediately refer back to the _Shadows & Components_ and _Typography_ guidelines. Ensure it matches the "Tech-Chic" vibe. Do not use generic blue buttons or soft shadows.
2. **Adding Dependencies**: Ask before adding new npm packages. Stick to what's already installed if possible.
3. **Icons Check**: Ensure you use the exact string name for `material-symbols-outlined` icons.
4. **Tailwind Config**: Remember, modifying tailwind values happens in `src/app/globals.css` using the v4 `@theme` directive, not a `tailwind.config.ts` file.

Whenever taking an action on the UI, confirm it aligns with the strict visual hierarchy and Amber/Off-black aesthetic.
