use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, SurrealValue, ToSql};

// ─── Database Record Types (deserialized from SurrealDB) ───

/// Generic SurrealDB Thing reference
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ThingRef {
    pub tb: String,
    pub id: String,
}

/// User record from SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct UserRecord {
    pub id: Option<RecordId>,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub avatar: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Notebook record from SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct NotebookRecord {
    pub id: Option<RecordId>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner: RecordId,
    pub is_deleted: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Access relation record.
/// `in`/`out` are Option because SurrealDB may return null for relation fields
/// in some response shapes (e.g. protocol or FETCH behavior).
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct AccessRecord {
    pub id: Option<RecordId>,
    pub r#in: Option<RecordId>,  // user
    pub out: Option<RecordId>,   // notebook
    pub role: String,
    pub granted_at: Option<DateTime<Utc>>,
}

/// Document record from SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct DocumentRecord {
    pub id: Option<RecordId>,
    pub notebook: RecordId,
    pub filename: String,
    pub source_type: String,
    pub sha256: Option<String>,
    pub url: Option<String>,
    pub parsing_rules: Option<serde_json::Value>,
    pub file_type: String,
    pub file_size: i64,
    pub upload_status: String,
    pub chunk_count: i64,
    pub summary: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Chunk record from SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ChunkRecord {
    pub id: Option<RecordId>,
    pub document: RecordId,
    pub notebook: RecordId,
    pub content: String,
    pub chunk_index: i64,
    pub metadata: Option<String>,
}

/// Image record from SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct DocImageRecord {
    pub id: Option<RecordId>,
    pub image_id: String,
    pub document: RecordId,
    pub notebook: RecordId,
    pub mime_type: String,
    pub source_ref: String,
    pub stored_path: String,
}

/// Session record from SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct SessionRecord {
    pub id: Option<RecordId>,
    pub notebook: RecordId,
    pub user: RecordId,
    pub title: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Message record from SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct MessageRecord {
    pub id: Option<RecordId>,
    pub session: RecordId,
    pub role: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

// ─── GraphQL Output Types ───

fn thing_to_string(thing: &Option<RecordId>) -> String {
    thing.as_ref().map(|t| t.to_sql()).unwrap_or_default()
}

#[derive(SimpleObject, Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub avatar: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<UserRecord> for User {
    fn from(r: UserRecord) -> Self {
        Self {
            id: thing_to_string(&r.id),
            username: r.username,
            email: r.email,
            avatar: r.avatar,
            created_at: r.created_at,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub is_deleted: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<NotebookRecord> for Notebook {
    fn from(r: NotebookRecord) -> Self {
        Self {
            id: thing_to_string(&r.id),
            name: r.name.unwrap_or_default(),
            description: r.description,
            owner_id: r.owner.to_sql(),
            is_deleted: r.is_deleted,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct Document {
    pub id: String,
    pub notebook_id: String,
    pub filename: String,
    pub source_type: String,
    pub sha256: Option<String>,
    pub url: Option<String>,
    pub parsing_rules: Option<String>,
    pub file_type: String,
    pub file_size: i64,
    pub upload_status: String,
    pub chunk_count: i64,
    pub summary: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<DocumentRecord> for Document {
    fn from(r: DocumentRecord) -> Self {
        Self {
            id: thing_to_string(&r.id),
            notebook_id: r.notebook.to_sql(),
            filename: r.filename,
            source_type: r.source_type,
            sha256: r.sha256,
            url: r.url,
            parsing_rules: r.parsing_rules.map(|m| m.to_string()),
            file_type: r.file_type,
            file_size: r.file_size,
            upload_status: r.upload_status,
            chunk_count: r.chunk_count,
            summary: r.summary,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

// ─── Ingestion-specific output types ───

/// Lightweight status object for batch polling during upload.
#[derive(SimpleObject, Debug, Clone)]
pub struct DocumentUploadStatus {
    pub id: String,
    pub filename: String,
    pub upload_status: String,
    pub chunk_count: i64,
    pub sha256: Option<String>,
}

impl From<DocumentRecord> for DocumentUploadStatus {
    fn from(r: DocumentRecord) -> Self {
        Self {
            id: thing_to_string(&r.id),
            filename: r.filename,
            upload_status: r.upload_status,
            chunk_count: r.chunk_count,
            sha256: r.sha256,
        }
    }
}

/// A single chunk returned to the frontend.
#[derive(SimpleObject, Debug, Clone)]
pub struct DocumentChunk {
    pub index: i64,
    pub content: String,
    pub metadata: Option<String>,
}

impl From<ChunkRecord> for DocumentChunk {
    fn from(r: ChunkRecord) -> Self {
        Self {
            index: r.chunk_index,
            content: r.content,
            metadata: r.metadata,
        }
    }
}

/// An image reference extracted from a document.
#[derive(SimpleObject, Debug, Clone)]
pub struct DocumentImage {
    pub image_id: String,
    pub mime_type: String,
    pub source_ref: String,
    pub stored_path: String,
}

impl From<DocImageRecord> for DocumentImage {
    fn from(r: DocImageRecord) -> Self {
        Self {
            image_id: r.image_id,
            mime_type: r.mime_type,
            source_ref: r.source_ref,
            stored_path: r.stored_path,
        }
    }
}

/// Full parsed content for a document (chunks + images).
#[derive(SimpleObject, Debug, Clone)]
pub struct DocumentContent {
    pub document_id: String,
    pub filename: String,
    pub upload_status: String,
    pub summary: Option<String>,
    pub chunks: Vec<DocumentChunk>,
    pub images: Vec<DocumentImage>,
}

/// Summary result returned after LLM summarization.
#[derive(SimpleObject, Debug, Clone)]
pub struct DocumentSummary {
    pub document_id: String,
    pub summary: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct Session {
    pub id: String,
    pub notebook_id: String,
    pub user_id: String,
    pub title: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<SessionRecord> for Session {
    fn from(r: SessionRecord) -> Self {
        Self {
            id: thing_to_string(&r.id),
            notebook_id: r.notebook.to_sql(),
            user_id: r.user.to_sql(),
            title: r.title,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub metadata: Option<String>, // JSON string
    pub created_at: Option<DateTime<Utc>>,
}

impl From<MessageRecord> for Message {
    fn from(r: MessageRecord) -> Self {
        Self {
            id: thing_to_string(&r.id),
            session_id: r.session.to_sql(),
            role: r.role,
            content: r.content,
            metadata: r.metadata.map(|m| m.to_string()),
            created_at: r.created_at,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct AuthPayload {
    pub token: String,
    pub user: User,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct NotebookMember {
    pub user: User,
    pub role: String,
    pub granted_at: Option<DateTime<Utc>>,
}

// ─── GraphQL Input Types ───

#[derive(InputObject)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(InputObject)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(InputObject)]
pub struct CreateNotebookInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateNotebookInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct InviteMemberInput {
    pub notebook_id: String,
    pub email: String,
    pub role: String,
}

#[derive(InputObject)]
pub struct CreateSessionInput {
    pub notebook_id: String,
    pub title: Option<String>,
}

#[derive(InputObject)]
pub struct SendMessageInput {
    pub session_id: String,
    pub content: String,
}
