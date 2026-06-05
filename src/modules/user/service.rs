use crate::modules::user::repository::UserRepository;
use crate::modules::user::schema::{UserProfileData, UserProfileResponse, UsersListResponse};
use crate::shared::errors::AppError;
use crate::shared::utils::password;
use crate::shared::utils::validation;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserService {
    repo: UserRepository,
}

impl UserService {
    pub fn new(repo: UserRepository) -> Self {
        Self { repo }
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfileResponse, AppError> {
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(UserProfileResponse {
            status: "success".to_string(),
            data: UserProfileData {
                id: user.id,
                username: user.username,
                email: user.email,
                role: user.role,
                status: user.status,
                email_verified: user.email_verified,
                last_login_at: user.last_login_at.map(|t| t.to_rfc3339()),
                created_at: user.created_at.to_rfc3339(),
                updated_at: user.updated_at.to_rfc3339(),
            },
        })
    }

    pub async fn list_users(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<UsersListResponse, AppError> {
        let offset = (page - 1) * per_page;
        let users = self.repo.list_all(per_page, offset).await?;
        let total = self.repo.count_all().await?;

        Ok(UsersListResponse {
            status: "success".to_string(),
            data: users
                .into_iter()
                .map(|u| UserProfileData {
                    id: u.id,
                    username: u.username,
                    email: u.email,
                    role: u.role,
                    status: u.status,
                    email_verified: u.email_verified,
                    last_login_at: u.last_login_at.map(|t| t.to_rfc3339()),
                    created_at: u.created_at.to_rfc3339(),
                    updated_at: u.updated_at.to_rfc3339(),
                })
                .collect(),
            total: total as usize,
        })
    }

    pub async fn update_username(
        &self,
        user_id: Uuid,
        new_username: &str,
    ) -> Result<UserProfileResponse, AppError> {
        validation::validate_username(new_username)?;

        self.repo.update_username(user_id, new_username).await?;
        self.get_profile(user_id).await
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        // Validate new password
        validation::validate_password(new_password)?;

        // Verify current password
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let valid = password::verify_password(current_password, &user.password_hash)
            .map_err(|_| AppError::InternalError("Password verification failed".into()))?;

        if !valid {
            return Err(AppError::InvalidCredentials);
        }

        // Hash and update
        let new_hash = password::hash_password(new_password)
            .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?;

        self.repo.update_password(user_id, &new_hash).await?;

        Ok(())
    }

    pub async fn delete_account(&self, user_id: Uuid) -> Result<(), AppError> {
        self.repo.delete(user_id).await?;
        Ok(())
    }
}
