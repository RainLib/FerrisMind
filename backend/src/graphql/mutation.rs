use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::auth::{jwt, password};
use crate::config::JwtConfig;
use crate::db::Db;
use crate::error::AppError;
use crate::graphql::guard::{
    check_notebook_access, get_current_user, require_editor, require_owner,
};
use crate::graphql::types::*;

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
            .bind(("email", &input.email))
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
            .bind(("username", &input.username))
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
            .bind(("username", &input.username))
            .bind(("email", &input.email))
            .bind(("password_hash", &password_hash))
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
            .map(|t| t.to_string())
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
            .bind(("email", &input.email))
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
            .map(|t| t.to_string())
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
                "CREATE notebook SET name = $name, description = $description, owner = type::thing($owner_id)"
            )
            .bind(("name", &input.name))
            .bind(("description", &input.description))
            .bind(("owner_id", &claims.sub))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let nb = records
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Failed to create notebook".to_string()).extend())?;

        let nb_id = nb.id.as_ref().map(|t| t.to_string()).unwrap_or_default();

        // Create owner access relation
        db.query(
            "RELATE type::thing($user_id) -> has_access -> type::thing($notebook_id) SET role = 'owner'"
        )
        .bind(("user_id", &claims.sub))
        .bind(("notebook_id", &nb_id))
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

        require_editor(db, &claims.sub, &input.id)
            .await
            .map_err(|e| e.extend())?;

        // Build dynamic update query
        let mut set_clauses = vec!["updated_at = time::now()".to_string()];
        if let Some(ref name) = input.name {
            set_clauses.push(format!("name = '{}'", name.replace('\'', "''")));
        }
        if let Some(ref desc) = input.description {
            set_clauses.push(format!("description = '{}'", desc.replace('\'', "''")));
        }

        let query = format!(
            "UPDATE type::thing($id) SET {} RETURN AFTER",
            set_clauses.join(", ")
        );

        let records: Vec<NotebookRecord> = db
            .query(&query)
            .bind(("id", &input.id))
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

        require_owner(db, &claims.sub, &id)
            .await
            .map_err(|e| e.extend())?;

        db.query("UPDATE type::thing($id) SET is_deleted = true, updated_at = time::now()")
            .bind(("id", &id))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(true)
    }

    /// Invite a member to a notebook (requires owner access).
    async fn invite_member(&self, ctx: &Context<'_>, input: InviteMemberInput) -> Result<bool> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        require_owner(db, &claims.sub, &input.notebook_id)
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
            .bind(("email", &input.email))
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
            .map(|t| t.to_string())
            .unwrap_or_default();

        // Check if already has access
        let existing: Vec<AccessRecord> = db
            .query("SELECT * FROM has_access WHERE in = type::thing($user_id) AND out = type::thing($notebook_id)")
            .bind(("user_id", &target_user_id))
            .bind(("notebook_id", &input.notebook_id))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        if !existing.is_empty() {
            // Update existing access
            db.query("UPDATE has_access SET role = $role WHERE in = type::thing($user_id) AND out = type::thing($notebook_id)")
                .bind(("role", &input.role))
                .bind(("user_id", &target_user_id))
                .bind(("notebook_id", &input.notebook_id))
                .await
                .map_err(|e| AppError::Database(e.to_string()).extend())?;
        } else {
            // Create new access relation
            db.query("RELATE type::thing($user_id) -> has_access -> type::thing($notebook_id) SET role = $role")
                .bind(("user_id", &target_user_id))
                .bind(("notebook_id", &input.notebook_id))
                .bind(("role", &input.role))
                .await
                .map_err(|e| AppError::Database(e.to_string()).extend())?;
        }

        Ok(true)
    }

    // ─── Document Management ───

    /// Register a document upload (metadata only; file upload is via REST endpoint).
    async fn create_document(
        &self,
        ctx: &Context<'_>,
        notebook_id: String,
        filename: String,
        file_type: String,
        file_size: i64,
    ) -> Result<Document> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        require_editor(db, &claims.sub, &notebook_id)
            .await
            .map_err(|e| e.extend())?;

        let records: Vec<DocumentRecord> = db
            .query(
                "CREATE document SET notebook = type::thing($notebook_id), filename = $filename, file_type = $file_type, file_size = $file_size"
            )
            .bind(("notebook_id", &notebook_id))
            .bind(("filename", &filename))
            .bind(("file_type", &file_type))
            .bind(("file_size", file_size))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        records
            .into_iter()
            .next()
            .map(Document::from)
            .ok_or_else(|| AppError::Internal("Failed to create document".to_string()).extend())
    }

    /// Delete a document (requires editor+ access).
    async fn delete_document(&self, ctx: &Context<'_>, id: String) -> Result<bool> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        // Get document to find notebook_id
        let doc: Option<DocumentRecord> = db
            .query("SELECT * FROM type::thing($id)")
            .bind(("id", &id))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let doc =
            doc.ok_or_else(|| AppError::NotFound("Document not found".to_string()).extend())?;

        require_editor(db, &claims.sub, &doc.notebook.to_string())
            .await
            .map_err(|e| e.extend())?;

        // Delete chunks first, then document
        db.query("DELETE chunk WHERE document = type::thing($doc_id)")
            .bind(("doc_id", &id))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        db.query("DELETE type::thing($id)")
            .bind(("id", &id))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(true)
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

        check_notebook_access(db, &claims.sub, &input.notebook_id)
            .await
            .map_err(|e| e.extend())?;

        let records: Vec<SessionRecord> = db
            .query(
                "CREATE session SET notebook = type::thing($notebook_id), user = type::thing($user_id), title = $title"
            )
            .bind(("notebook_id", &input.notebook_id))
            .bind(("user_id", &claims.sub))
            .bind(("title", &input.title))
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

        // Verify session belongs to user
        let session: Option<SessionRecord> = db
            .query("SELECT * FROM type::thing($session_id) WHERE user = type::thing($user_id)")
            .bind(("session_id", &input.session_id))
            .bind(("user_id", &claims.sub))
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
                "CREATE message SET session = type::thing($session_id), role = 'user', content = $content"
            )
            .bind(("session_id", &input.session_id))
            .bind(("content", &input.content))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        // Update session timestamp
        db.query("UPDATE type::thing($session_id) SET updated_at = time::now()")
            .bind(("session_id", &input.session_id))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        records
            .into_iter()
            .next()
            .map(Message::from)
            .ok_or_else(|| AppError::Internal("Failed to create message".to_string()).extend())
    }
}
