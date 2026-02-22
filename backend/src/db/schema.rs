use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;
use tracing::info;

/// Apply all SurrealDB schema definitions.
/// SurrealDB uses DEFINE statements which are idempotent (OVERWRITE).
pub async fn apply_schema(db: &Surreal<Client>) -> anyhow::Result<()> {
    info!("Applying database schema...");

    // ── User table ──
    db.query(
        "
        DEFINE TABLE user SCHEMAFULL;
        DEFINE FIELD username ON user TYPE string;
        DEFINE FIELD email ON user TYPE string;
        DEFINE FIELD password_hash ON user TYPE string;
        DEFINE FIELD avatar ON user TYPE option<string>;
        DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON user TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_user_email ON user FIELDS email UNIQUE;
        DEFINE INDEX idx_user_username ON user FIELDS username UNIQUE;
    ",
    )
    .await?;

    // ── Notebook table ──
    db.query(
        "
        DEFINE TABLE notebook SCHEMAFULL;
        DEFINE FIELD name ON notebook TYPE string;
        DEFINE FIELD description ON notebook TYPE option<string>;
        DEFINE FIELD owner ON notebook TYPE record<user>;
        DEFINE FIELD is_deleted ON notebook TYPE bool DEFAULT false;
        DEFINE FIELD created_at ON notebook TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON notebook TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_notebook_owner ON notebook FIELDS owner;
    ",
    )
    .await?;

    // ── Access control relation (SurrealDB graph relation) ──
    db.query(
        "
        DEFINE TABLE has_access SCHEMAFULL TYPE RELATION FROM user TO notebook;
        DEFINE FIELD role ON has_access TYPE string ASSERT $value IN ['owner', 'editor', 'viewer'];
        DEFINE FIELD granted_at ON has_access TYPE datetime DEFAULT time::now();
    ",
    )
    .await?;

    // ── Document (Source) table ──
    db.query(
        "
        DEFINE TABLE document SCHEMAFULL;
        DEFINE FIELD notebook ON document TYPE record<notebook>;
        DEFINE FIELD filename ON document TYPE string;
        DEFINE FIELD file_type ON document TYPE string;
        DEFINE FIELD file_size ON document TYPE int;
        DEFINE FIELD upload_status ON document TYPE string DEFAULT 'pending';
        DEFINE FIELD chunk_count ON document TYPE int DEFAULT 0;
        DEFINE FIELD created_at ON document TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_doc_notebook ON document FIELDS notebook;
    ",
    )
    .await?;

    // ── Chunk table ──
    db.query(
        "
        DEFINE TABLE chunk SCHEMAFULL;
        DEFINE FIELD document ON chunk TYPE record<document>;
        DEFINE FIELD notebook ON chunk TYPE record<notebook>;
        DEFINE FIELD content ON chunk TYPE string;
        DEFINE FIELD chunk_index ON chunk TYPE int;
        DEFINE FIELD metadata ON chunk TYPE option<object>;
        DEFINE FIELD embedding ON chunk TYPE option<array<float>>;
        DEFINE FIELD created_at ON chunk TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_chunk_notebook ON chunk FIELDS notebook;
        DEFINE INDEX idx_chunk_doc ON chunk FIELDS document;
    ",
    )
    .await?;

    // ── Session table (conversations) ──
    db.query(
        "
        DEFINE TABLE session SCHEMAFULL;
        DEFINE FIELD notebook ON session TYPE record<notebook>;
        DEFINE FIELD user ON session TYPE record<user>;
        DEFINE FIELD title ON session TYPE option<string>;
        DEFINE FIELD created_at ON session TYPE datetime DEFAULT time::now();
        DEFINE FIELD updated_at ON session TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_session_notebook ON session FIELDS notebook;
        DEFINE INDEX idx_session_user ON session FIELDS user;
    ",
    )
    .await?;

    // ── Message table ──
    db.query(
        "
        DEFINE TABLE message SCHEMAFULL;
        DEFINE FIELD session ON message TYPE record<session>;
        DEFINE FIELD role ON message TYPE string ASSERT $value IN ['user', 'assistant', 'system'];
        DEFINE FIELD content ON message TYPE string;
        DEFINE FIELD metadata ON message TYPE option<object>;
        DEFINE FIELD created_at ON message TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_msg_session ON message FIELDS session;
    ",
    )
    .await?;

    info!("Database schema applied successfully.");
    Ok(())
}
