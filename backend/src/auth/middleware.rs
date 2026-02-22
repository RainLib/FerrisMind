use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

use crate::auth::jwt::{Claims, verify_token};
use crate::config::JwtConfig;

/// Extract authenticated user from request headers.
/// Returns None if no valid token is present (allows unauthenticated GraphQL queries).
pub fn extract_auth_user(config: &JwtConfig, req: &Request) -> Option<Claims> {
    let header = req.headers().get("authorization")?;
    let header_str = header.to_str().ok()?;
    let token = header_str.strip_prefix("Bearer ")?;
    verify_token(config, token).ok()
}

/// Axum middleware that optionally extracts auth user and stores in request extensions.
/// Does NOT reject unauthenticated requests — that's handled by GraphQL guards.
pub async fn auth_middleware(mut req: Request, next: Next) -> Response {
    // Try to extract JWT config from request extensions
    if let Some(config) = req.extensions().get::<JwtConfig>().cloned() {
        if let Some(claims) = extract_auth_user(&config, &req) {
            req.extensions_mut().insert(claims);
        }
    }

    next.run(req).await
}
