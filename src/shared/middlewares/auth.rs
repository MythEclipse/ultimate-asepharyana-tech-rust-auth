use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use crate::shared::errors::AppError;
use crate::shared::state::AppState;
use std::sync::Arc;

/// Validates JWT tokens and extracts user identity.
/// Uses Redis session cache for fast revocation checks.
pub async fn auth_middleware(
    state: Arc<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::InvalidToken)?;

    let claims = crate::shared::utils::jwt::validate_access_token(token, &state)?;

    // Check session validity in Redis
    let session_key = format!("session:{}", claims.session_id);
    let mut conn = state.redis.get().await.map_err(|_| AppError::InternalError("Redis unavailable".into()))?;
    let exists: bool = redis::cmd("EXISTS")
        .arg(&session_key)
        .query_async(&mut *conn)
        .await
        .map_err(|_| AppError::InternalError("Cache error".into()))?;

    if !exists {
        return Err(AppError::InvalidToken);
    }

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
