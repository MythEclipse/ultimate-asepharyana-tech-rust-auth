use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // --- Authentication ---
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Refresh token invalid or expired")]
    InvalidRefreshToken,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Account already exists")]
    AccountAlreadyExists,

    #[error("Account locked")]
    AccountLocked,

    #[error("Account not verified")]
    AccountNotVerified,

    // --- Authorization ---
    #[error("Forbidden: insufficient permissions")]
    Forbidden,

    #[error("Unauthorized")]
    Unauthorized,

    // --- Validation ---
    #[error("Validation error: {0}")]
    ValidationError(String),

    // --- Database ---
    #[error("Database error: {0}")]
    DatabaseError(String),

    // --- Configuration ---
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    // --- Internal ---
    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS", self.to_string())
            }
            AppError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "INVALID_TOKEN", self.to_string())
            }
            AppError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "TOKEN_EXPIRED", self.to_string())
            }
            AppError::InvalidRefreshToken => {
                (StatusCode::UNAUTHORIZED, "INVALID_REFRESH_TOKEN", self.to_string())
            }
            AppError::AccountNotFound => {
                (StatusCode::NOT_FOUND, "ACCOUNT_NOT_FOUND", self.to_string())
            }
            AppError::AccountAlreadyExists => {
                (StatusCode::CONFLICT, "ACCOUNT_ALREADY_EXISTS", self.to_string())
            }
            AppError::AccountLocked => {
                (StatusCode::FORBIDDEN, "ACCOUNT_LOCKED", self.to_string())
            }
            AppError::AccountNotVerified => {
                (StatusCode::FORBIDDEN, "ACCOUNT_NOT_VERIFIED", self.to_string())
            }
            AppError::Forbidden => {
                (StatusCode::FORBIDDEN, "FORBIDDEN", self.to_string())
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.to_string())
            }
            AppError::ValidationError(_) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", self.to_string())
            }
            AppError::NotFound(_) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string())
            }
            AppError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", self.to_string())
            }
            AppError::ConfigurationError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "CONFIGURATION_ERROR", self.to_string())
            }
            AppError::InternalError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", self.to_string())
            }
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
                "status": status.as_u16(),
            }
        });

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(?err, "Database error");
        match &err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            sqlx::Error::Protocol(e) => AppError::DatabaseError(e.to_string()),
            sqlx::Error::PoolClosed => AppError::InternalError("Connection pool closed".to_string()),
            sqlx::Error::PoolTimedOut => AppError::InternalError("Connection pool timeout".to_string()),
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::ValidationError(err.to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        tracing::error!(?err, "Redis error");
        AppError::InternalError(format!("Cache error: {}", err))
    }
}
