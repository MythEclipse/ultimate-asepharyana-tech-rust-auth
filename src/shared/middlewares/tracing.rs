use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;
use tower_http::request_id::MakeRequestId;
use uuid::Uuid;

/// Injects a correlation ID into every request for distributed tracing.
#[derive(Clone)]
pub struct CorrelationIdLayer;

impl CorrelationIdLayer {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Clone, Default)]
pub struct CorrelationIdGenerator;

impl MakeRequestId for CorrelationIdGenerator {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<axum::http::HeaderValue> {
        let correlation_id = Uuid::new_v4().to_string();
        axum::http::HeaderValue::from_str(&correlation_id).ok()
    }
}

pub async fn correlation_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let correlation_id = request
        .headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request.extensions_mut().insert(CorrelationId(correlation_id.clone()));

    let mut response = next.run(request).await;
    if let Ok(value) = axum::http::HeaderValue::from_str(&correlation_id) {
        response.headers_mut().insert("x-correlation-id", value);
    }
    response
}

#[derive(Clone, Debug)]
pub struct CorrelationId(pub String);
