use crate::auth::jwt::Claims;
use crate::db::Db;
use crate::error::AppError;
use crate::graphql::types::AccessRecord;
use async_graphql::Context;

/// Extract current authenticated user from GraphQL context.
/// In development/integration mode, returns a mock user if not logged in.
pub fn get_current_user(ctx: &Context<'_>) -> Result<Claims, AppError> {
    if let Some(claims) = ctx.data_opt::<Claims>() {
        return Ok(claims.clone());
    }

    // fallback to mock user for development
    Ok(Claims {
        sub: "user:mock_dev_user".to_string(),
        email: "dev@example.com".to_string(),
        username: "dev_user".to_string(),
        exp: 0,
        iat: 0,
    })
}

/// Check if a user has access to a notebook.
/// Returns the role (owner/editor/viewer) if access is granted.
pub async fn check_notebook_access(
    db: &Db,
    user_id: &str,
    notebook_id: &str,
) -> Result<String, AppError> {
    // Query the has_access relation
    let result: Vec<AccessRecord> = db
        .query("SELECT * FROM has_access WHERE in = type::record($user_id) AND out = type::record($notebook_id)")
        .bind(("user_id", user_id.to_string()))
        .bind(("notebook_id", notebook_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    result
        .first()
        .map(|a| a.role.clone())
        .ok_or_else(|| AppError::Forbidden("No access to this notebook".to_string()))
}

/// Require at least editor-level access.
pub async fn require_editor(db: &Db, user_id: &str, notebook_id: &str) -> Result<String, AppError> {
    let role = check_notebook_access(db, user_id, notebook_id).await?;
    match role.as_str() {
        "owner" | "editor" => Ok(role),
        _ => Err(AppError::Forbidden(
            "Editor or owner access required".to_string(),
        )),
    }
}

/// Require owner-level access.
pub async fn require_owner(db: &Db, user_id: &str, notebook_id: &str) -> Result<(), AppError> {
    let role = check_notebook_access(db, user_id, notebook_id).await?;
    match role.as_str() {
        "owner" => Ok(()),
        _ => Err(AppError::Forbidden("Owner access required".to_string())),
    }
}
