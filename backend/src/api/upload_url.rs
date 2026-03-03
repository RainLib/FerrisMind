use std::sync::Arc;

use axum::{
    extract::{Extension, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::config::IngestConfig;
use crate::db::Db;
use crate::graphql::guard::decode_record_id;
use crate::graphql::types::DocumentRecord;
use crate::ingest::service::IngestionService;
use crate::llm::manager::LlmManager;
use surrealdb_types::ToSql;

#[derive(Deserialize)]
pub struct UploadUrlRequest {
    pub notebook_id: String,
    pub urls: Vec<String>,
}

#[derive(Serialize)]
pub struct UploadUrlResponse {
    pub documents: Vec<UploadedUrlDoc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadedUrlDoc {
    pub id: String,
    pub url: String,
    pub upload_status: String,
}

#[derive(Serialize)]
pub struct UploadError {
    pub error: String,
}

/// `POST /api/upload/url`
///
/// Accepts JSON:
/// {
///   "notebook_id": "...",
///   "urls": ["https://example.com"]
/// }
pub async fn upload_url_handler(
    Extension(db): Extension<Db>,
    Extension(llm): Extension<Arc<LlmManager>>,
    Extension(ingest_config): Extension<IngestConfig>,
    Json(payload): Json<UploadUrlRequest>,
) -> Result<Json<UploadUrlResponse>, (StatusCode, Json<UploadError>)> {
    if payload.urls.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(UploadError {
                error: "No URLs provided".to_string(),
            }),
        ));
    }

    let notebook_id = decode_record_id(&payload.notebook_id);
    let mut uploaded: Vec<UploadedUrlDoc> = Vec::new();

    for url in payload.urls {
        // Create document record
        let records: Vec<DocumentRecord> = db
            .query(
                "CREATE document SET \
                    notebook = type::record($nb_id), \
                    filename = $url, \
                    url = $url, \
                    file_type = 'text/html', \
                    file_size = 0, \
                    source_type = 'url', \
                    upload_status = 'pending', \
                    chunk_count = 0",
            )
            .bind(("nb_id", notebook_id.clone()))
            .bind(("url", url.clone()))
            .await
            .map_err(|e| {
                error!("Database error creating URL document: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UploadError {
                        error: format!("DB error: {}", e),
                    }),
                )
            })?
            .take(0)
            .map_err(|e| {
                error!("Database error taking URL document: {}", e);
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

        info!("Created URL document '{}' → {}", url, doc_id);

        uploaded.push(UploadedUrlDoc {
            id: doc_id.clone(),
            url: url.clone(),
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

    Ok(Json(UploadUrlResponse {
        documents: uploaded,
    }))
}
