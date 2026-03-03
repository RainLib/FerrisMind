use std::sync::Arc;

use axum::{
    extract::{Extension, Multipart},
    http::StatusCode,
    Json,
};
use percent_encoding::percent_decode_str;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tracing::{error, info};

use crate::config::IngestConfig;
use crate::db::Db;
use crate::graphql::types::DocumentRecord;
use crate::ingest::service::IngestionService;
use crate::llm::manager::LlmManager;
use surrealdb_types::ToSql;

#[derive(Serialize)]
pub struct UploadResponse {
    pub documents: Vec<UploadedDoc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadedDoc {
    pub id: String,
    pub filename: String,
    pub file_size: i64,
    pub sha256: String,
    pub upload_status: String,
}

#[derive(Serialize)]
pub struct UploadError {
    pub error: String,
}

/// `POST /api/upload`
///
/// Accepts multipart form data:
/// - `notebook_id` (text field, required)
/// - one or more file fields named `files`
///
/// For each file: saves to disk, creates a document record, triggers async ingestion.
pub async fn upload_handler(
    Extension(db): Extension<Db>,
    Extension(llm): Extension<Arc<LlmManager>>,
    Extension(ingest_config): Extension<IngestConfig>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<UploadError>)> {
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
    tokio::fs::create_dir_all(&upload_dir).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UploadError {
                error: format!("Cannot create upload dir: {}", e),
            }),
        )
    })?;

    let mut notebook_id: Option<String> = None;
    let mut uploaded: Vec<UploadedDoc> = Vec::new();
    let mut pending_files: Vec<(String, String, Vec<u8>, String)> = Vec::new(); // (filename, mime, data, sha256)

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(UploadError {
                error: format!("Multipart error: {}", e),
            }),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "notebook_id" {
            let text = field.text().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(UploadError {
                        error: format!("Cannot read notebook_id: {}", e),
                    }),
                )
            })?;
            let decoded = percent_decode_str(&text).decode_utf8_lossy().into_owned();
            notebook_id = Some(decoded);
            continue;
        }

        if name == "files" || name == "file" {
            let filename = field.file_name().unwrap_or("unnamed").to_string();
            let content_type = mime_guess::from_path(&filename)
                .first_or_octet_stream()
                .to_string();
            let data = field.bytes().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(UploadError {
                        error: format!("Cannot read file '{}': {}", filename, e),
                    }),
                )
            })?;

            let sha256 = format!("{:x}", Sha256::digest(&data));
            pending_files.push((filename, content_type, data.to_vec(), sha256));
        }
    }

    let notebook_id = notebook_id.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(UploadError {
                error: "Missing required field: notebook_id".to_string(),
            }),
        )
    })?;

    if pending_files.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UploadError {
                error: "No files provided".to_string(),
            }),
        ));
    }

    for (filename, mime, data, sha256) in pending_files {
        let file_size = data.len() as i64;

        // Create document record
        let records: Vec<DocumentRecord> = db
            .query(
                "CREATE document SET \
                    notebook = type::record($nb_id), \
                    filename = $filename, \
                    file_type = $file_type, \
                    file_size = $file_size, \
                    source_type = 'file', \
                    sha256 = $sha256, \
                    upload_status = 'pending', \
                    chunk_count = 0",
            )
            .bind(("nb_id", notebook_id.clone()))
            .bind(("filename", filename.clone()))
            .bind(("file_type", mime.clone()))
            .bind(("file_size", file_size))
            .bind(("sha256", sha256.clone()))
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UploadError {
                        error: format!("DB error: {}", e),
                    }),
                )
            })?
            .take(0)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UploadError {
                        error: format!("DB error: {}", e),
                    }),
                )
            })?;

        let doc = records.into_iter().next().ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UploadError {
                    error: "Failed to create document record".to_string(),
                }),
            )
        })?;

        let doc_id = doc.id.as_ref().map(|t| t.to_sql()).unwrap_or_default();
        let sanitized_id = doc_id.replace(':', "_").replace('/', "_");
        let file_path =
            std::path::Path::new(&upload_dir).join(format!("{}_{}", sanitized_id, filename));

        // Write file to disk
        tokio::fs::write(&file_path, &data).await.map_err(|e| {
            error!("Failed to write file {}: {}", file_path.display(), e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UploadError {
                    error: format!("Cannot save file: {}", e),
                }),
            )
        })?;

        info!(
            "Uploaded '{}' ({} bytes, sha256: {}) → {}",
            filename,
            file_size,
            &sha256[..12],
            doc_id
        );

        uploaded.push(UploadedDoc {
            id: doc_id.clone(),
            filename: filename.clone(),
            file_size,
            sha256: sha256.clone(),
            upload_status: "pending".to_string(),
        });

        // Spawn async ingestion
        let ingest_service = Arc::new(IngestionService::new(
            db.clone(),
            llm.clone(),
            &ingest_config,
        ));
        tokio::spawn(async move {
            ingest_service.process_document(doc_id).await;
        });
    }

    Ok(Json(UploadResponse {
        documents: uploaded,
    }))
}
