use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

/// Injects a correlation ID into every request for distributed tracing.
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
