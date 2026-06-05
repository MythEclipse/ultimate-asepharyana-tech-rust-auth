use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// --- Request Schemas ---

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 32), regex = r"^[a-zA-Z0-9_]+$")]
    pub username: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub email: String,

    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

// --- Response Schemas ---

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub status: String,
    pub data: AuthData,
}

#[derive(Debug, Serialize)]
pub struct AuthData {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub email_verified: bool,
}

#[derive(Debug, Serialize)]
pub struct TokenRefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub status: String,
    pub message: String,
}
