use crate::modules::auth::entity::User;
use crate::shared::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserRepository {
    db: PgPool,
}

impl UserRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
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

    pub async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, role, status,
                   email_verified, last_login_at, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    pub async fn count_all(&self) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) as count FROM users"#,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(count.0)
    }

    pub async fn update_username(&self, id: Uuid, username: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users SET username = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(username)
        .bind(id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users SET password_hash = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(password_hash)
        .bind(id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}
