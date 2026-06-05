use crate::modules::auth::entity::User;
use crate::modules::auth::repository::AuthRepository;
use crate::modules::auth::schema::{
    AuthData, AuthResponse, LoginRequest, MessageResponse, RegisterRequest, TokenRefreshResponse,
    UserResponse,
};
use crate::shared::errors::AppError;
use crate::shared::state::AppState;
use crate::shared::utils::jwt;
use crate::shared::utils::password;
use crate::shared::utils::validation;
use deadpool_redis::redis::AsyncCommands;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthService {
    repo: AuthRepository,
}

impl AuthService {
    pub fn new(repo: AuthRepository) -> Self {
        Self { repo }
    }

    /// Register a new user account.
    pub async fn register(
        &self,
        state: &AppState,
        req: RegisterRequest,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AuthResponse, AppError> {
        // Validate input
        validation::validate_email(&req.email)?;
        validation::validate_password(&req.password)?;
        validation::validate_username(&req.username)?;

        // Check for existing user
        if (self.repo.find_by_email(&req.email).await?).is_some() {
            return Err(AppError::AccountAlreadyExists);
        }
        if (self.repo.find_by_username(&req.username).await?).is_some() {
            return Err(AppError::ValidationError("Username already taken".to_string()));
        }

        // Hash password
        let password_hash = password::hash_password(&req.password)
            .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?;

        // Create user
        let user_id = Uuid::new_v4();
        let user = self.repo.create(user_id, &req.username, &req.email, &password_hash).await?;

        // Create session
        let (session, access_token, refresh_token) = self
            .create_session_with_tokens(&user, ip_address, user_agent, state)
            .await?;

        // Store session in Redis
        self.cache_session(state, &session, &access_token).await?;

        Ok(AuthResponse {
            status: "success".to_string(),
            data: AuthData {
                user: Self::map_user(&user),
                access_token,
                refresh_token,
                expires_in: state.config.jwt.access_expiration,
            },
        })
    }

    /// Authenticate a user with email + password.
    pub async fn login(
        &self,
        state: &AppState,
        req: LoginRequest,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AuthResponse, AppError> {
        let user = self
            .repo
            .find_by_email(&req.email)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        // Verify password
        let valid = password::verify_password(&req.password, &user.password_hash)
            .map_err(|_| AppError::InternalError("Password verification failed".into()))?;

        if !valid {
            return Err(AppError::InvalidCredentials);
        }

        // Check account status
        if user.status == "locked" {
            return Err(AppError::AccountLocked);
        }

        // Update last login
        self.repo.update_last_login(user.id).await?;

        // Create session
        let (session, access_token, refresh_token) = self
            .create_session_with_tokens(&user, ip_address, user_agent, state)
            .await?;

        // Store in Redis
        self.cache_session(state, &session, &access_token).await?;

        Ok(AuthResponse {
            status: "success".to_string(),
            data: AuthData {
                user: Self::map_user(&user),
                access_token,
                refresh_token,
                expires_in: state.config.jwt.access_expiration,
            },
        })
    }

    /// Refresh both access and refresh tokens.
    pub async fn refresh(
        &self,
        state: &AppState,
        refresh_token: &str,
    ) -> Result<TokenRefreshResponse, AppError> {
        // Validate refresh token
        let claims = jwt::validate_refresh_token(refresh_token)?;
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::InvalidRefreshToken)?;

        // Hash incoming refresh token to find session
        let token_hash = hash_token(refresh_token);
        let session = self
            .repo
            .find_session_by_refresh_token(&token_hash)
            .await?
            .ok_or(AppError::InvalidRefreshToken)?;

        // Get user
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::InvalidRefreshToken)?;

        // Revoke old session
        self.repo.revoke_session(session.id).await?;

        // Remove old session from Redis
        let old_key = format!("session:{}", session.id);
        if let Ok(mut conn) = state.redis.get().await {
            let _: Result<(), _> = conn.del::<_, ()>(&old_key).await;
        }

        // Create new session
        let (new_session, new_access_token, new_refresh_token) = self
            .create_session_with_tokens(&user, None, None, state)
            .await?;

        // Cache new session
        self.cache_session(state, &new_session, &new_access_token).await?;

        Ok(TokenRefreshResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            expires_in: state.config.jwt.access_expiration,
        })
    }

    /// Logout by revoking the session.
    pub async fn logout(&self, state: &AppState, refresh_token: &str) -> Result<MessageResponse, AppError> {
        let token_hash = hash_token(refresh_token);
        let session = self
            .repo
            .find_session_by_refresh_token(&token_hash)
            .await?;

        if let Some(session) = session {
            self.repo.revoke_session(session.id).await?;

        // Remove from Redis
            let session_key = format!("session:{}", session.id);
            if let Ok(mut conn) = state.redis.get().await {
                let _: Result<(), _> = conn.del::<_, ()>(&session_key).await;
            }
        }

        Ok(MessageResponse {
            status: "success".to_string(),
            message: "Logged out successfully".to_string(),
        })
    }

    // --- Private helpers ---

    async fn create_session_with_tokens(
        &self,
        user: &User,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        _state: &AppState,
    ) -> Result<(crate::modules::auth::entity::Session, String, String), AppError> {
        let session_id = Uuid::new_v4();
        let access_token = jwt::create_access_token(&user.id, &user.email, &session_id, &user.role)?;
        let refresh_token = jwt::create_refresh_token(&user.id, &session_id)?;
        let refresh_token_hash = hash_token(&refresh_token);

        let session = self
            .repo
            .create_session(session_id, user.id, &refresh_token_hash, ip_address, user_agent)
            .await?;

        Ok((session, access_token, refresh_token))
    }

    async fn cache_session(
        &self,
        state: &AppState,
        session: &crate::modules::auth::entity::Session,
        access_token: &str,
    ) -> Result<(), AppError> {
        let mut conn = state.redis.get().await
            .map_err(|_| AppError::InternalError("Redis connection failed".into()))?;

        let session_key = format!("session:{}", session.id);
        let ttl = state.config.jwt.access_expiration as u64;

        conn.set_ex::<_, _, ()>(&session_key, access_token, ttl)
            .await
            .map_err(|_| AppError::InternalError("Redis cache set failed".into()))?;

        Ok(())
    }

    fn map_user(user: &User) -> UserResponse {
        UserResponse {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            email_verified: user.email_verified,
        }
    }
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
