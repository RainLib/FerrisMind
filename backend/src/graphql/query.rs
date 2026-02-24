use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::db::Db;
use crate::error::AppError;
use crate::graphql::guard::{check_notebook_access, get_current_user};
use crate::graphql::types::*;
use surrealdb_types::ToSql;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get current authenticated user info.
    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        let record: Option<UserRecord> = db
            .query("SELECT * FROM type::record($id)")
            .bind(("id", claims.sub.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        record
            .map(User::from)
            .ok_or_else(|| AppError::NotFound("User not found".to_string()).extend())
    }

    /// List all notebooks the current user has access to.
    async fn notebooks(&self, ctx: &Context<'_>) -> Result<Vec<Notebook>> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        let records: Vec<NotebookRecord> = db
            .query(
                "SELECT out.id AS id, (out.name ?? '') AS name, out.description AS description, out.owner AS owner, (out.is_deleted ?? false) AS is_deleted, out.created_at AS created_at, out.updated_at AS updated_at FROM has_access WHERE in = type::record($user_id) AND out.is_deleted = false"
            )
            .bind(("user_id", claims.sub.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(records.into_iter().map(Notebook::from).collect())
    }

    /// Get a single notebook by ID (with access check).
    async fn notebook(&self, ctx: &Context<'_>, id: String) -> Result<Notebook> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        // Check access
        check_notebook_access(db, &claims.sub, &id)
            .await
            .map_err(|e| e.extend())?;

        let record: Option<NotebookRecord> = db
            .query("SELECT * FROM type::record($id) WHERE is_deleted = false")
            .bind(("id", id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        record
            .map(Notebook::from)
            .ok_or_else(|| AppError::NotFound("Notebook not found".to_string()).extend())
    }

    /// List documents in a notebook (with access check).
    async fn documents(&self, ctx: &Context<'_>, notebook_id: String) -> Result<Vec<Document>> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        check_notebook_access(db, &claims.sub, &notebook_id)
            .await
            .map_err(|e| e.extend())?;

        let records: Vec<DocumentRecord> = db
            .query("SELECT * FROM document WHERE notebook = type::record($notebook_id) ORDER BY created_at DESC")
            .bind(("notebook_id", notebook_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(records.into_iter().map(Document::from).collect())
    }

    /// List chat sessions in a notebook (with access check).
    async fn sessions(&self, ctx: &Context<'_>, notebook_id: String) -> Result<Vec<Session>> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        check_notebook_access(db, &claims.sub, &notebook_id)
            .await
            .map_err(|e| e.extend())?;

        let records: Vec<SessionRecord> = db
            .query("SELECT * FROM session WHERE notebook = type::record($notebook_id) AND user = type::record($user_id) ORDER BY updated_at DESC")
            .bind(("notebook_id", notebook_id.clone()))
            .bind(("user_id", claims.sub.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(records.into_iter().map(Session::from).collect())
    }

    /// List messages in a session.
    async fn messages(
        &self,
        ctx: &Context<'_>,
        session_id: String,
        #[graphql(default = 50)] limit: i64,
    ) -> Result<Vec<Message>> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        // Verify session belongs to user
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

        let records: Vec<MessageRecord> = db
            .query("SELECT * FROM message WHERE session = type::record($session_id) ORDER BY created_at ASC LIMIT $limit")
            .bind(("session_id", session_id.clone()))
            .bind(("limit", limit))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(records.into_iter().map(Message::from).collect())
    }

    // ─── Document Content & Upload Status ───

    /// Batch-poll upload statuses. Pass a list of document IDs and get back
    /// their current processing state. Ideal for frontend progress bars.
    async fn document_upload_statuses(
        &self,
        ctx: &Context<'_>,
        ids: Vec<String>,
    ) -> Result<Vec<DocumentUploadStatus>> {
        let _claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        if ids.is_empty() {
            return Ok(vec![]);
        }

        let mut results = Vec::with_capacity(ids.len());
        for id in &ids {
            let record: Option<DocumentRecord> = db
                .query("SELECT * FROM type::thing($id)")
                .bind(("id", id.clone()))
                .await
                .map_err(|e| AppError::Database(e.to_string()).extend())?
                .take(0)
                .map_err(|e| AppError::Database(e.to_string()).extend())?;

            if let Some(r) = record {
                results.push(DocumentUploadStatus::from(r));
            }
        }

        Ok(results)
    }

    /// Get the full parsed content (chunks + images) for a document.
    async fn document_content(
        &self,
        ctx: &Context<'_>,
        document_id: String,
    ) -> Result<DocumentContent> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        // Fetch document to get notebook_id for access check
        let doc: Option<DocumentRecord> = db
            .query("SELECT * FROM type::thing($id)")
            .bind(("id", document_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let doc =
            doc.ok_or_else(|| AppError::NotFound("Document not found".to_string()).extend())?;

        check_notebook_access(db, &claims.sub, &doc.notebook.to_sql())
            .await
            .map_err(|e| e.extend())?;

        // Fetch chunks ordered by index
        let chunks: Vec<ChunkRecord> = db
            .query("SELECT * FROM chunk WHERE document = type::thing($doc_id) ORDER BY chunk_index ASC")
            .bind(("doc_id", document_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        // Fetch images
        let images: Vec<DocImageRecord> = db
            .query("SELECT * FROM doc_image WHERE document = type::thing($doc_id)")
            .bind(("doc_id", document_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        Ok(DocumentContent {
            document_id: document_id.clone(),
            filename: doc.filename,
            upload_status: doc.upload_status,
            summary: doc.summary,
            chunks: chunks.into_iter().map(DocumentChunk::from).collect(),
            images: images.into_iter().map(DocumentImage::from).collect(),
        })
    }

    /// List members of a notebook (with access check).
    async fn notebook_members(
        &self,
        ctx: &Context<'_>,
        notebook_id: String,
    ) -> Result<Vec<NotebookMember>> {
        let claims = get_current_user(ctx).map_err(|e| e.extend())?;
        let db = ctx.data::<Db>()?;

        check_notebook_access(db, &claims.sub, &notebook_id)
            .await
            .map_err(|e| e.extend())?;

        // Query all access relations for this notebook, fetching user data
        let records: Vec<AccessRecord> = db
            .query("SELECT * FROM has_access WHERE out = type::record($notebook_id) FETCH in")
            .bind(("notebook_id", notebook_id.clone()))
            .await
            .map_err(|e| AppError::Database(e.to_string()).extend())?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()).extend())?;

        let mut members = Vec::new();
        for record in records {
            let user_record: Option<UserRecord> = db
                .query("SELECT * FROM type::record($user_id)")
                .bind(("user_id", record.r#in.to_sql()))
                .await
                .map_err(|e| AppError::Database(e.to_string()).extend())?
                .take(0)
                .map_err(|e| AppError::Database(e.to_string()).extend())?;

            if let Some(u) = user_record {
                members.push(NotebookMember {
                    user: User::from(u),
                    role: record.role,
                    granted_at: record.granted_at,
                });
            }
        }

        Ok(members)
    }
}
