use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};

use crate::auth::jwt::{verify_token, Claims};
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

/// Extractor that reads optional Claims from request extensions (set by auth_middleware).
/// Does not consume the request body, so it can be used together with GraphQLRequest.
#[derive(Clone)]
pub struct OptionalAuth(pub Option<Claims>);

impl<S> FromRequestParts<S> for OptionalAuth
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts.extensions.get::<Claims>().cloned();
        Ok(OptionalAuth(claims))
    }
}
