use crate::modules::auth::entity::{Session, User};
use crate::shared::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthRepository {
    db: PgPool,
}

impl AuthRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // --- User operations ---

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, role, status,
                   email_verified, last_login_at, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, role, status,
                   email_verified, last_login_at, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, role, status,
                   email_verified, last_login_at, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    pub async fn create(
        &self,
        id: Uuid,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, role, status, email_verified)
            VALUES ($1, $2, $3, $4, 'user', 'active', false)
            RETURNING id, username, email, password_hash, role, status,
                      email_verified, last_login_at, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.db)
        .await?;

        Ok(user)
    }

    pub async fn update_last_login(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users SET last_login_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // --- Session operations ---

    pub async fn create_session(
        &self,
        id: Uuid,
        user_id: Uuid,
        refresh_token_hash: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<Session, AppError> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (id, user_id, refresh_token_hash, ip_address, user_agent,
                                  is_revoked, expires_at)
            VALUES ($1, $2, $3, $4, $5, false, NOW() + INTERVAL '7 days')
            RETURNING id, user_id, refresh_token_hash, device_fingerprint, ip_address,
                      user_agent, is_revoked, issued_at, expires_at, created_at
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(refresh_token_hash)
        .bind(ip_address)
        .bind(user_agent)
        .fetch_one(&self.db)
        .await?;

        Ok(session)
    }

    pub async fn find_session_by_refresh_token(
        &self,
        refresh_token_hash: &str,
    ) -> Result<Option<Session>, AppError> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, user_id, refresh_token_hash, device_fingerprint, ip_address,
                   user_agent, is_revoked, issued_at, expires_at, created_at
            FROM sessions
            WHERE refresh_token_hash = $1 AND is_revoked = false AND expires_at > NOW()
            "#,
        )
        .bind(refresh_token_hash)
        .fetch_optional(&self.db)
        .await?;

        Ok(session)
    }

    pub async fn revoke_session(&self, session_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE sessions SET is_revoked = true
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn revoke_all_user_sessions(&self, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE sessions SET is_revoked = true
            WHERE user_id = $1 AND is_revoked = false
            "#,
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
