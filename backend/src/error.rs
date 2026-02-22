use async_graphql::ErrorExtensions;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Authentication required")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("LLM error: {0}")]
    Llm(String),
}

impl ErrorExtensions for AppError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(self.to_string()).extend_with(|_, e| {
            let code = match self {
                AppError::Unauthorized => "UNAUTHORIZED",
                AppError::Forbidden(_) => "FORBIDDEN",
                AppError::NotFound(_) => "NOT_FOUND",
                AppError::BadRequest(_) => "BAD_REQUEST",
                AppError::Conflict(_) => "CONFLICT",
                AppError::Internal(_) => "INTERNAL_ERROR",
                AppError::Database(_) => "DATABASE_ERROR",
                AppError::Llm(_) => "LLM_ERROR",
            };
            e.set("code", code);
        })
    }
}

impl From<surrealdb::Error> for AppError {
    fn from(e: surrealdb::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized
    }
}
