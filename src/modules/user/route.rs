use crate::modules::user::controller;
use axum::routing::{delete, get, put};
use axum::Router;
use std::sync::Arc;

/// Build user routes with auth middleware protection.
pub fn routes() -> Router<Arc<crate::shared::state::AppState>> {
    Router::new()
        .route("/api/v1/users/me", get(controller::get_me))
        .route("/api/v1/users/me", put(controller::update_profile))
        .route("/api/v1/users/me", delete(controller::delete_account))
        .route("/api/v1/users/change-password", put(controller::change_password))
        .route("/api/v1/users", get(controller::list_users))
        .route("/api/v1/users/{id}", get(controller::get_user_by_id))
}
