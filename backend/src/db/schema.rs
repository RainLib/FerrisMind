use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tracing::info;

/// Apply all SurrealDB schema definitions.
/// SurrealDB uses DEFINE statements which are idempotent (OVERWRITE).
pub async fn apply_schema(db: &Surreal<Any>) -> anyhow::Result<()> {
    info!("Applying database schema...");

    let statements = [
        // ── User table ──
        "DEFINE TABLE user SCHEMAFULL;",
        "DEFINE FIELD username ON user TYPE string;",
        "DEFINE FIELD email ON user TYPE string;",
        "DEFINE FIELD password_hash ON user TYPE string;",
        "DEFINE FIELD avatar ON user TYPE option<string>;",
        "DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();",
        "DEFINE FIELD updated_at ON user TYPE datetime DEFAULT time::now();",
        "DEFINE INDEX idx_user_email ON user FIELDS email UNIQUE;",
        "DEFINE INDEX idx_user_username ON user FIELDS username UNIQUE;",

        // ── Notebook table ──
        "DEFINE TABLE notebook SCHEMAFULL;",
        "DEFINE FIELD name ON notebook TYPE string;",
        "DEFINE FIELD description ON notebook TYPE option<string>;",
        "DEFINE FIELD owner ON notebook TYPE record<user>;",
        "DEFINE FIELD is_deleted ON notebook TYPE bool DEFAULT false;",
        "DEFINE FIELD created_at ON notebook TYPE datetime DEFAULT time::now();",
        "DEFINE FIELD updated_at ON notebook TYPE datetime DEFAULT time::now();",
        "DEFINE INDEX idx_notebook_owner ON notebook FIELDS owner;",

        // ── Access control relation ──
        "DEFINE TABLE has_access SCHEMAFULL TYPE RELATION FROM user TO notebook;",
        "DEFINE FIELD role ON has_access TYPE string ASSERT $value IN ['owner', 'editor', 'viewer'];",
        "DEFINE FIELD granted_at ON has_access TYPE datetime DEFAULT time::now();",

        // ── Document (Source) table ──
        "DEFINE TABLE document SCHEMAFULL;",
        "DEFINE FIELD notebook ON document TYPE record<notebook>;",
        "DEFINE FIELD filename ON document TYPE string;",
        "DEFINE FIELD source_type ON document TYPE string ASSERT $value IN ['file', 'url', 'text'];",
        "DEFINE FIELD sha256 ON document TYPE option<string>;",
        "DEFINE FIELD url ON document TYPE option<string>;",
        "DEFINE FIELD parsing_rules ON document FLEXIBLE TYPE option<object>;",
        "DEFINE FIELD file_type ON document TYPE string;",
        "DEFINE FIELD file_size ON document TYPE int;",
        "DEFINE FIELD upload_status ON document TYPE string ASSERT $value IN ['pending', 'processing', 'completed', 'failed'];",
        "DEFINE FIELD chunk_count ON document TYPE int DEFAULT 0;",
        "DEFINE FIELD summary ON document TYPE option<string>;",
        "DEFINE FIELD created_at ON document TYPE datetime DEFAULT time::now();",
        "DEFINE FIELD updated_at ON document TYPE datetime DEFAULT time::now();",
        "DEFINE INDEX idx_doc_notebook ON document FIELDS notebook;",
        "DEFINE INDEX idx_doc_sha256 ON document FIELDS sha256;",

        // ── Document image table (extracted images from ingested docs) ──
        "DEFINE TABLE doc_image SCHEMAFULL;",
        "DEFINE FIELD image_id ON doc_image TYPE string;",
        "DEFINE FIELD document ON doc_image TYPE record<document>;",
        "DEFINE FIELD notebook ON doc_image TYPE record<notebook>;",
        "DEFINE FIELD mime_type ON doc_image TYPE string;",
        "DEFINE FIELD source_ref ON doc_image TYPE string;",
        "DEFINE FIELD stored_path ON doc_image TYPE string;",
        "DEFINE INDEX idx_doc_image_document ON doc_image FIELDS document;",

        // ── Chunk table ──
        "DEFINE TABLE chunk SCHEMAFULL;",
        "DEFINE FIELD document ON chunk TYPE record<document>;",
        "DEFINE FIELD notebook ON chunk TYPE record<notebook>;",
        "DEFINE FIELD content ON chunk TYPE string;",
        "DEFINE FIELD chunk_index ON chunk TYPE int;",
        "DEFINE FIELD metadata ON chunk TYPE option<string>;",
        "DEFINE FIELD embedding ON chunk TYPE option<array<float>>;",
        "DEFINE FIELD created_at ON chunk TYPE datetime DEFAULT time::now();",
        "DEFINE INDEX idx_chunk_notebook ON chunk FIELDS notebook;",
        "DEFINE INDEX idx_chunk_doc ON chunk FIELDS document;",

        // ── Session table ──
        "DEFINE TABLE session SCHEMAFULL;",
        "DEFINE FIELD notebook ON session TYPE record<notebook>;",
        "DEFINE FIELD user ON session TYPE record<user>;",
        "DEFINE FIELD title ON session TYPE option<string>;",
        "DEFINE FIELD created_at ON session TYPE datetime DEFAULT time::now();",
        "DEFINE FIELD updated_at ON session TYPE datetime DEFAULT time::now();",
        "DEFINE INDEX idx_session_notebook ON session FIELDS notebook;",
        "DEFINE INDEX idx_session_user ON session FIELDS user;",

        // ── Message table ──
        "DEFINE TABLE message SCHEMAFULL;",
        "DEFINE FIELD session ON message TYPE record<session>;",
        "DEFINE FIELD role ON message TYPE string ASSERT $value IN ['user', 'assistant', 'system'];",
        "DEFINE FIELD content ON message TYPE string;",
        "DEFINE FIELD metadata ON message FLEXIBLE TYPE option<object>;",
        "DEFINE FIELD created_at ON message TYPE datetime DEFAULT time::now();",
        "DEFINE INDEX idx_msg_session ON message FIELDS session;",
    ];

    for stmt in statements {
        info!("Executing: {}", stmt);
        match db.query(stmt).await?.check() {
            Ok(_) => (),
            Err(e) if e.to_string().contains("already exists") => {
                info!("Already exists, skipping: {}", stmt);
            }
            Err(e) => return Err(e.into()),
        }
    }

    info!("Database schema applied successfully.");
    Ok(())
}

/// Remove all application tables. Use only in test/dev to reset the database.
/// Order: remove tables that reference others first.
///
/// To reset manually in SurrealDB SQL tab, run:
/// ```sql
/// REMOVE TABLE message;
/// REMOVE TABLE session;
/// REMOVE TABLE chunk;
/// REMOVE TABLE doc_image;
/// REMOVE TABLE document;
/// REMOVE TABLE has_access;
/// REMOVE TABLE notebook;
/// REMOVE TABLE user;
/// ```
pub async fn remove_all_tables(db: &Surreal<Any>) -> anyhow::Result<()> {
    info!("Removing all database tables...");

    let tables = [
        "message",
        "session",
        "chunk",
        "doc_image",
        "document",
        "has_access",
        "notebook",
        "user",
    ];

    for table in tables {
        let stmt = format!("REMOVE TABLE {table};");
        info!("Executing: {}", stmt);
        if let Err(e) = db.query(&stmt).await?.check() {
            let msg = e.to_string();
            if msg.contains("does not exist") || msg.contains("Unknown table") {
                info!("Table {} does not exist, skipping", table);
            } else {
                return Err(e.into());
            }
        }
    }

    info!("All tables removed.");
    Ok(())
}
