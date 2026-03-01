use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::auth::{jwt, password};
use crate::config::JwtConfig;
use crate::db::Db;
use crate::error::AppError;
use crate::graphql::guard::{
    check_notebook_access, decode_record_id, get_current_user, require_editor, require_owner,
};
use crate::graphql::types::*;
use surrealdb_types::ToSql;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    // ─── Auth ───

    /// Register a new user.
    async fn register(&self, ctx: &Context<'_>, input: RegisterInput) -> Result<AuthPayload> {
        let db = ctx.data::<Db>()?;
        let jwt_config = ctx.data::<JwtConfig>()?;

        // Validate input
        if input.username.trim().is_empty()
            || input.email.trim().is_empty()
            || input.password.len() < 6
        {
            return Err(AppError::BadRequest(
                "Username and email required, password must be at least 6 characters".to_string(),
            )
            .extend());
        }

        // Check if email already exists
        let existing: Vec<UserRecord> = db
            .query("SELECT * FROM user WHERE email = $email")
            .bind(("email", input.email.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        if !existing.is_empty() {
            return Err(AppError::Conflict("Email already registered".to_string()).extend());
        }

        // Check if username already exists
        let existing: Vec<UserRecord> = db
            .query("SELECT * FROM user WHERE username = $username")
            .bind(("username", input.username.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        if !existing.is_empty() {
            return Err(AppError::Conflict("Username already taken".to_string()).extend());
        }

        // Hash password
        let password_hash = password::hash_password(&input.password).map_err(|e| e.extend())?;

        // Create user
        let records: Vec<UserRecord> = db
            .query(
                "CREATE user SET username = $username, email = $email, password_hash = $password_hash"
            )
            .bind(("username", input.username.clone()))
            .bind(("email", input.email.clone()))
            .bind(("password_hash", password_hash.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let user_record = records
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Failed to create user".to_string()).extend())?;

        let user_id = user_record
            .id
            .as_ref()
            .map(|t| t.to_sql())
            .unwrap_or_default();

        // Generate JWT
        let token = jwt::create_token(jwt_config, &user_id, &input.email, &input.username)
            .map_err(|e| AppError::Internal(e.to_string()).extend())?;

        Ok(AuthPayload {
            token,
            user: User::from(user_record),
        })
    }

    /// Login with email and password.
    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<AuthPayload> {
        let db = ctx.data::<Db>()?;
        let jwt_config = ctx.data::<JwtConfig>()?;

        // Find user by email
        let records: Vec<UserRecord> = db
            .query("SELECT * FROM user WHERE email = $email")
            .bind(("email", input.email.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let user_record = records.into_iter().next().ok_or_else(|| {
            AppError::BadRequest("Invalid email or password".to_string()).extend()
        })?;

        // Verify password
        let valid = password::verify_password(&input.password, &user_record.password_hash)
            .map_err(|e| e.extend())?;

        if !valid {
            return Err(AppError::BadRequest("Invalid email or password".to_string()).extend());
        }

        let user_id = user_record
            .id
            .as_ref()
            .map(|t| t.to_sql())
            .unwrap_or_default();

        // Generate JWT
        let token = jwt::create_token(
            jwt_config,
            &user_id,
            &user_record.email,
            &user_record.username,
        )
        .map_err(|e| AppError::Internal(e.to_string()).extend())?;

        Ok(AuthPayload {
            token,
            user: User::from(user_record),
        })
    }

    // ─── Notebook CRUD ───

    /// Create a new notebook. Caller becomes owner.
    async fn create_notebook(
        &self,
        ctx: &Context<'_>,
        input: CreateNotebookInput,
    ) -> Result<Notebook> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        if input.name.trim().is_empty() {
            return Err(AppError::BadRequest("Notebook name is required".to_string()).extend());
        }

        // Create notebook
        let records: Vec<NotebookRecord> = db
            .query(
                "CREATE notebook SET name = $name, description = $description, owner = type::record($owner_id)"
            )
            .bind(("name", input.name.clone()))
            .bind(("description", input.description.clone()))
            .bind(("owner_id", claims.sub.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let nb = records
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Failed to create notebook".to_string()).extend())?;

        let nb_id = nb.id.as_ref().map(|t| t.to_sql()).unwrap_or_default();

        // Create owner access relation (type::record() not allowed in RELATE path; convert in LET first)
        db.query("LET $uid = type::record($user_id); LET $nid = type::record($notebook_id); RELATE $uid->has_access->$nid SET role = 'owner'")
            .bind(("user_id", claims.sub.clone()))
            .bind(("notebook_id", nb_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(Notebook::from(nb))
    }

    /// Update a notebook (requires editor+ access).
    async fn update_notebook(
        &self,
        ctx: &Context<'_>,
        input: UpdateNotebookInput,
    ) -> Result<Notebook> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        let id = decode_record_id(&input.id);
        require_editor(db, &claims.sub, &id)
            .await
            .map_err(|e| e.extend())?;

        let mut set_clauses = vec!["updated_at = time::now()".to_string()];
        if let Some(ref name) = input.name {
            set_clauses.push(format!("name = '{}'", name.replace('\'', "''")));
        }
        if let Some(ref desc) = input.description {
            set_clauses.push(format!("description = '{}'", desc.replace('\'', "''")));
        }

        let query = format!(
            "UPDATE type::record($id) SET {} RETURN AFTER",
            set_clauses.join(", ")
        );

        let records: Vec<NotebookRecord> = db
            .query(&query)
            .bind(("id", id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        records
            .into_iter()
            .next()
            .map(Notebook::from)
            .ok_or_else(|| AppError::NotFound("Notebook not found".to_string()).extend())
    }

    /// Soft-delete a notebook (requires owner access).
    async fn delete_notebook(&self, ctx: &Context<'_>, id: String) -> Result<bool> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;
        let id = decode_record_id(&id);

        require_owner(db, &claims.sub, &id)
            .await
            .map_err(|e| e.extend())?;

        db.query("UPDATE type::record($id) SET is_deleted = true, updated_at = time::now()")
            .bind(("id", id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(true)
    }

    /// Invite a member to a notebook (requires owner access).
    async fn invite_member(&self, ctx: &Context<'_>, input: InviteMemberInput) -> Result<bool> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;
        let notebook_id = decode_record_id(&input.notebook_id);

        require_owner(db, &claims.sub, &notebook_id)
            .await
            .map_err(|e| e.extend())?;

        // Validate role
        if !["editor", "viewer"].contains(&input.role.as_str()) {
            return Err(
                AppError::BadRequest("Role must be 'editor' or 'viewer'".to_string()).extend(),
            );
        }

        // Find user by email
        let users: Vec<UserRecord> = db
            .query("SELECT * FROM user WHERE email = $email")
            .bind(("email", input.email.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let target_user = users.into_iter().next().ok_or_else(|| {
            AppError::NotFound("User not found with this email".to_string()).extend()
        })?;

        let target_user_id = target_user
            .id
            .as_ref()
            .map(|t| t.to_sql())
            .unwrap_or_default();

        // Check if already has access
        let existing: Vec<AccessRecord> = db
            .query("SELECT * FROM has_access WHERE in = type::record($user_id) AND out = type::record($notebook_id)")
            .bind(("user_id", target_user_id.clone()))
            .bind(("notebook_id", notebook_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        if !existing.is_empty() {
            // Update existing access
            db.query("UPDATE has_access SET role = $role WHERE in = type::record($user_id) AND out = type::record($notebook_id)")
                .bind(("role", input.role.to_string()))
                .bind(("user_id", target_user_id.clone()))
                .bind(("notebook_id", notebook_id.clone()))
                .await
                .map_err(|e| AppError::Database(e.to_string()).extend())?;
        } else {
            // Create new access relation
            db.query("LET $uid = type::record($user_id); LET $nid = type::record($notebook_id); RELATE $uid->has_access->$nid SET role = $role")
                .bind(("user_id", target_user_id.clone()))
                .bind(("notebook_id", notebook_id.clone()))
                .bind(("role", input.role.to_string()))
                .await
                .map_err(|e| AppError::Database(e.to_string()).extend())?;
        }

        Ok(true)
    }

    // ─── Document Management ───

    /// Register a document upload or URL (metadata only; file upload is via REST endpoint).
    async fn create_document(
        &self,
        ctx: &Context<'_>,
        notebook_id: String,
        filename: String,
        file_type: String,
        file_size: i64,
        source_type: String,
        sha256: Option<String>,
        url: Option<String>,
        parsing_rules: Option<String>, // JSON string mapped to Option<object>
    ) -> Result<Document> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;
        let notebook_id = decode_record_id(&notebook_id);

        require_editor(db, &claims.sub, &notebook_id)
            .await
            .map_err(|e| e.extend())?;

        if !["file", "url", "text"].contains(&source_type.as_str()) {
            return Err(AppError::BadRequest(
                "Invalid source_type. Must be file, url, or text.".to_string(),
            )
            .extend());
        }

        let parsed_rules: Option<serde_json::Value> = if let Some(rules) = parsing_rules {
            Some(serde_json::from_str(&rules).map_err(|e| {
                AppError::BadRequest(format!("Invalid parsing rules JSON: {}", e)).extend()
            })?)
        } else {
            None
        };

        let records: Vec<DocumentRecord> = db
            .query(
                "CREATE document SET notebook = type::record($notebook_id), filename = $filename, file_type = $file_type, file_size = $file_size, source_type = $source_type, sha256 = $sha256, url = $url, parsing_rules = $parsing_rules, upload_status = 'pending'"
            )
            .bind(("notebook_id", notebook_id.clone()))
            .bind(("filename", filename.clone()))
            .bind(("file_type", file_type.clone()))
            .bind(("file_size", file_size))
            .bind(("source_type", source_type))
            .bind(("sha256", sha256))
            .bind(("url", url))
            .bind(("parsing_rules", parsed_rules))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let doc = records
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Failed to create document".to_string()).extend())?;

        let doc_id_string = doc.id.as_ref().map(|t| t.to_sql()).unwrap_or_default();
        if !doc_id_string.is_empty() {
            let llm = ctx
                .data::<std::sync::Arc<crate::llm::manager::LlmManager>>()?
                .clone();
            let ingest_config = ctx.data::<crate::config::IngestConfig>()?;
            let ingest_service = std::sync::Arc::new(
                crate::ingest::service::IngestionService::new(db.clone(), llm, ingest_config),
            );

            tokio::spawn(async move {
                ingest_service.process_document(doc_id_string).await;
            });
        }

        Ok(Document::from(doc))
    }

    /// Delete a document (requires editor+ access).
    async fn delete_document(&self, ctx: &Context<'_>, id: String) -> Result<bool> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;
        let id = decode_record_id(&id);

        let doc: Option<DocumentRecord> = db
            .query("SELECT * FROM type::record($id)")
            .bind(("id", id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let doc =
            doc.ok_or_else(|| AppError::NotFound("Document not found".to_string()).extend())?;

        require_editor(db, &claims.sub, &doc.notebook.to_sql())
            .await
            .map_err(|e| e.extend())?;

        // Delete chunks first, then document
        db.query("DELETE chunk WHERE document = type::record($doc_id)")
            .bind(("doc_id", id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        // Soft-delete KG entities for this document (preserve history, just hide from queries)
        db.query("UPDATE kg_entity SET is_active = false WHERE document = type::record($doc_id)")
            .bind(("doc_id", id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        // Soft-delete KG relations that touch any of this document's entities
        // (SurrealDB RELATE edge table uses `in` and `out` for from/to)
        db.query(
            "UPDATE kg_relation SET is_active = false \
             WHERE in IN (SELECT id FROM kg_entity WHERE document = type::record($doc_id) AND is_active = false) \
                OR out IN (SELECT id FROM kg_entity WHERE document = type::record($doc_id) AND is_active = false)"
        )
            .bind(("doc_id", id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        db.query("DELETE type::record($id)")
            .bind(("id", id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(true)
    }

    // ─── Document Summarization ───

    /// Summarize a document's parsed content using the LLM.
    /// Concatenates all chunks, sends to the model with a summarization prompt,
    /// saves the result on the document record, and returns it.
    async fn summarize_document(
        &self,
        ctx: &Context<'_>,
        document_id: String,
    ) -> Result<DocumentSummary> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;
        let document_id = decode_record_id(&document_id);

        let doc: Option<DocumentRecord> = db
            .query("SELECT * FROM type::record($id)")
            .bind(("id", document_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let doc =
            doc.ok_or_else(|| AppError::NotFound("Document not found".to_string()).extend())?;

        require_editor(db, &claims.sub, &doc.notebook.to_sql())
            .await
            .map_err(|e| e.extend())?;

        if doc.upload_status != "completed" {
            return Err(
                AppError::BadRequest("Document is not yet fully processed".to_string()).extend(),
            );
        }

        // Fetch all chunks ordered by index
        let chunks: Vec<ChunkRecord> = db
            .query("SELECT * FROM chunk WHERE document = type::record($doc_id) ORDER BY chunk_index ASC")
            .bind(("doc_id", document_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        if chunks.is_empty() {
            return Err(AppError::BadRequest(
                "Document has no parsed content to summarize".to_string(),
            )
            .extend());
        }

        // Concatenate chunks (truncate if too large for context window)
        let max_chars: usize = 60_000;
        let mut content = String::new();
        for chunk in &chunks {
            if content.len() + chunk.content.len() > max_chars {
                content.push_str(&chunk.content[..max_chars.saturating_sub(content.len())]);
                break;
            }
            content.push_str(&chunk.content);
            content.push('\n');
        }

        // Call LLM to generate summary
        let llm = ctx
            .data::<std::sync::Arc<crate::llm::manager::LlmManager>>()?
            .clone();

        let preamble = format!(
            "You are a document summarizer. The user has uploaded a document titled \"{}\".\n\
             Provide a comprehensive yet concise summary (2-5 paragraphs) of its content.\n\
             Focus on the key points, conclusions, and main arguments.\n\
             Respond in the same language as the document content.",
            doc.filename
        );

        let agent = llm.agent_with_preamble(&preamble);

        let summary_text = agent
            .prompt(&content)
            .await
            .map_err(|e| AppError::Llm(format!("Summarization failed: {}", e)).extend())?;

        // Save summary to document record
        db.query("UPDATE type::record($id) SET summary = $summary")
            .bind(("id", document_id.clone()))
            .bind(("summary", summary_text.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(DocumentSummary {
            document_id,
            summary: summary_text,
        })
    }

    // ─── Session / Chat ───

    /// Create a new chat session in a notebook.
    async fn create_session(
        &self,
        ctx: &Context<'_>,
        input: CreateSessionInput,
    ) -> Result<Session> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;
        let notebook_id = decode_record_id(&input.notebook_id);

        check_notebook_access(db, &claims.sub, &notebook_id)
            .await
            .map_err(|e| e.extend())?;

        let records: Vec<SessionRecord> = db
            .query(
                "CREATE session SET notebook = type::record($notebook_id), user = type::record($user_id), title = $title"
            )
            .bind(("notebook_id", notebook_id.clone()))
            .bind(("user_id", claims.sub.clone()))
            .bind(("title", input.title.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        records
            .into_iter()
            .next()
            .map(Session::from)
            .ok_or_else(|| AppError::Internal("Failed to create session".to_string()).extend())
    }

    /// Send a user message to a session (stores user message, LLM response via SSE endpoint).
    async fn send_message(&self, ctx: &Context<'_>, input: SendMessageInput) -> Result<Message> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;
        let session_id = decode_record_id(&input.session_id);

        let session: Option<SessionRecord> = db
            .query("SELECT * FROM type::record($session_id) WHERE user = type::record($user_id)")
            .bind(("session_id", session_id.clone()))
            .bind(("user_id", claims.sub.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        if session.is_none() {
            return Err(AppError::NotFound("Session not found".to_string()).extend());
        }

        // Store user message
        let records: Vec<MessageRecord> = db
            .query(
                "CREATE message SET session = type::record($session_id), role = 'user', content = $content"
            )
            .bind(("session_id", session_id.clone()))
            .bind(("content", input.content.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        // Update session timestamp
        db.query("UPDATE type::record($session_id) SET updated_at = time::now()")
            .bind(("session_id", session_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        records
            .into_iter()
            .next()
            .map(Message::from)
            .ok_or_else(|| AppError::Internal("Failed to create message".to_string()).extend())
    }
}
