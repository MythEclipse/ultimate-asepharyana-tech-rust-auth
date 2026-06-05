use crate::modules::auth::repository::AuthRepository;
use crate::modules::auth::schema::{
    AuthResponse, LoginRequest, LogoutRequest, MessageResponse, RefreshTokenRequest,
    RegisterRequest, TokenRefreshResponse,
};
use crate::modules::auth::service::AuthService;
use crate::shared::errors::AppError;
use crate::shared::state::AppState;
use axum::extract::{ConnectInfo, State};
use axum::Json;
use std::net::SocketAddr;
use std::sync::Arc;

/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let repo = AuthRepository::new(state.db.clone());
    let service = AuthService::new(repo);

    let ip_str = addr.ip().to_string();

    let response = service.register(&state, req, Some(&ip_str), None).await?;
    Ok(Json(response))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let repo = AuthRepository::new(state.db.clone());
    let service = AuthService::new(repo);

    let ip_str = addr.ip().to_string();

    let response = service.login(&state, req, Some(&ip_str), None).await?;
    Ok(Json(response))
}

/// POST /api/v1/auth/refresh
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<TokenRefreshResponse>, AppError> {
    let repo = AuthRepository::new(state.db.clone());
    let service = AuthService::new(repo);

    let response = service.refresh(&state, &req.refresh_token).await?;
    Ok(Json(response))
}

/// POST /api/v1/auth/logout
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LogoutRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let repo = AuthRepository::new(state.db.clone());
    let service = AuthService::new(repo);

    let response = service.logout(&state, &req.refresh_token).await?;
    Ok(Json(response))
}
