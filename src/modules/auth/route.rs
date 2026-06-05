use crate::modules::auth::controller;
use axum::routing::post;
use axum::Router;
use std::sync::Arc;

pub fn routes() -> Router<Arc<crate::shared::state::AppState>> {
    Router::new()
        .route("/api/v1/auth/register", post(controller::register))
        .route("/api/v1/auth/login", post(controller::login))
        .route("/api/v1/auth/refresh", post(controller::refresh))
        .route("/api/v1/auth/logout", post(controller::logout))
}
