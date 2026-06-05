use crate::shared::errors::AppError;
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub session_id: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
}

/// Extractor that reads Claims from request extensions (set by auth middleware).
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<Claims>().cloned().ok_or_else(|| {
            let body = json!({
                "error": {
                    "code": "UNAUTHORIZED",
                    "message": "Authentication required",
                    "status": 401,
                }
            });
            (StatusCode::UNAUTHORIZED, Json(body)).into_response()
        })
    }
}

/// Validates JWT token and extracts Claims.
pub async fn auth_middleware(
    State(state): State<std::sync::Arc<crate::shared::state::AppState>>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let auth_header = match request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
    {
        Some(h) => h,
        None => {
            let body = json!({
                "error": { "code": "UNAUTHORIZED", "message": "Authentication required", "status": 401 }
            });
            return (StatusCode::UNAUTHORIZED, Json(body)).into_response();
        }
    };

    let token = match auth_header.strip_prefix("Bearer ") {
        Some(t) => t,
        None => {
            let body = json!({
                "error": { "code": "INVALID_TOKEN", "message": "Invalid token format", "status": 401 }
            });
            return (StatusCode::UNAUTHORIZED, Json(body)).into_response();
        }
    };

    // Validate JWT
    let claims = match crate::shared::utils::jwt::validate_access_token(token, &state) {
        Ok(c) => c,
        Err(AppError::TokenExpired) => {
            let body = json!({
                "error": { "code": "TOKEN_EXPIRED", "message": "Token expired", "status": 401 }
            });
            return (StatusCode::UNAUTHORIZED, Json(body)).into_response();
        }
        Err(_) => {
            let body = json!({
                "error": { "code": "INVALID_TOKEN", "message": "Invalid token", "status": 401 }
            });
            return (StatusCode::UNAUTHORIZED, Json(body)).into_response();
        }
    };

    // Check session validity in Redis
    let session_key = format!("session:{}", claims.session_id);
    match state.redis.get().await {
        Ok(mut conn) => {
            let exists: Result<bool, _> = conn.exists(&session_key).await;
            match exists {
                Ok(true) => {} // session valid
                _ => {
                    let body = json!({
                        "error": { "code": "INVALID_TOKEN", "message": "Session expired or revoked", "status": 401 }
                    });
                    return (StatusCode::UNAUTHORIZED, Json(body)).into_response();
                }
            }
        }
        Err(_) => {
            // Redis unavailable - allow through but log
            tracing::warn!("Redis unavailable during auth check");
        }
    }

    request.extensions_mut().insert(claims);
    next.run(request).await
}
