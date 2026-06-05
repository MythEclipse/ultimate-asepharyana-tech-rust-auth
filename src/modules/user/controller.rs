use crate::modules::user::repository::UserRepository;
use crate::modules::user::schema::{
    ChangePasswordRequest, UpdateProfileRequest, UserProfileResponse, UsersListResponse,
};
use crate::modules::user::service::UserService;
use crate::shared::errors::AppError;
use crate::shared::middlewares::tracing::CorrelationId;
use crate::shared::state::AppState;
use crate::shared::utils::jwt::Claims;
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    page: Option<i64>,
    per_page: Option<i64>,
}

/// GET /api/v1/users/me - Get current user's profile
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<UserProfileResponse>, AppError> {
    let repo = UserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InternalError("Invalid user ID in token".into()))?;

    let response = service.get_profile(user_id).await?;
    Ok(Json(response))
}

/// GET /api/v1/users - List all users (admin)
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<UsersListResponse>, AppError> {
    // Only admins can list all users
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let repo = UserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = pagination.per_page.unwrap_or(20).clamp(1, 100);

    let response = service.list_users(page, per_page).await?;
    Ok(Json(response))
}

/// GET /api/v1/users/:id - Get user by ID (admin)
pub async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserProfileResponse>, AppError> {
    if claims.role != "admin" && claims.sub != user_id.to_string() {
        return Err(AppError::Forbidden);
    }

    let repo = UserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    let response = service.get_profile(user_id).await?;
    Ok(Json(response))
}

/// PUT /api/v1/users/me - Update current user's profile
pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, AppError> {
    let repo = UserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InternalError("Invalid user ID in token".into()))?;

    if let Some(username) = &req.username {
        let response = service.update_username(user_id, username).await?;
        Ok(Json(response))
    } else {
        // No fields to update, just return profile
        service.get_profile(user_id).await.map(Json)
    }
}

/// POST /api/v1/users/change-password
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = UserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InternalError("Invalid user ID in token".into()))?;

    service
        .change_password(user_id, &req.current_password, &req.new_password)
        .await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Password changed successfully"
    })))
}

/// DELETE /api/v1/users/me - Delete current user's account
pub async fn delete_account(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = UserRepository::new(state.db.clone());
    let service = UserService::new(repo);

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InternalError("Invalid user ID in token".into()))?;

    service.delete_account(user_id).await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Account deleted successfully"
    })))
}
