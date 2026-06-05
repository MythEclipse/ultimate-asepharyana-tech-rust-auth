use crate::shared::state::AppState;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use deadpool_redis::redis::AsyncCommands;
use serde_json::json;
use std::sync::Arc;

/// Token bucket rate limiter using Redis.
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let key = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let should_allow = rate_limit_check(&key, state.clone()).await;

    if !should_allow {
        let body = json!({
            "error": {
                "code": "RATE_LIMIT_EXCEEDED",
                "message": "Too many requests. Please slow down.",
                "status": 429,
            }
        });
        return (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();
    }

    next.run(request).await
}

async fn rate_limit_check(key: &str, state: Arc<AppState>) -> bool {
    match state.redis.get().await {
        Ok(mut conn) => {
            let redis_key = format!("ratelimit:{}", key);
            let window: u64 = 60; // 60 second window
            let max_requests: u64 = 100;

            let count: Result<u64, _> = conn.incr(&redis_key, 1).await;

            match count {
                Ok(1) => {
                    let _: Result<(), _> = conn.expire(&redis_key, window as i64).await;
                    true
                }
                Ok(count) if count <= max_requests => true,
                Ok(_) => false,
                Err(_) => true, // Redis failure - allow through
            }
        }
        Err(_) => true, // Pool failure - allow through
    }
}
