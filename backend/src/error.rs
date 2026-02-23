use async_graphql::ErrorExtensions;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum AppError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("LLM error: {0}")]
    Llm(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Conflict: {0}")]
    Conflict(String),
}

impl ErrorExtensions for AppError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_err, e| match self {
            AppError::Unauthorized => e.set("code", "UNAUTHORIZED"),
            AppError::Forbidden(_) => e.set("code", "FORBIDDEN"),
            AppError::NotFound(_) => e.set("code", "NOT_FOUND"),
            AppError::Database(_) => e.set("code", "DATABASE_ERROR"),
            AppError::Llm(_) => e.set("code", "LLM_ERROR"),
            AppError::Internal(_) => e.set("code", "INTERNAL_ERROR"),
            AppError::BadRequest(_) => e.set("code", "BAD_REQUEST"),
            AppError::Conflict(_) => e.set("code", "CONFLICT"),
        })
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(_e: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Llm(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
        };

        (status, message).into_response()
    }
}
