use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Request Schemas ---

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// --- Response Schemas ---

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub status: String,
    pub data: UserProfileData,
}

#[derive(Debug, Serialize)]
pub struct UserProfileData {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub email_verified: bool,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct UsersListResponse {
    pub status: String,
    pub data: Vec<UserProfileData>,
    pub total: usize,
}
