use crate::shared::config::AppConfig;
use crate::shared::errors::AppError;
use crate::shared::state::AppState;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,          // user ID
    pub email: String,
    pub session_id: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
}

/// Creates an access token (short-lived, typically 15 min).
pub fn create_access_token(
    user_id: &Uuid,
    email: &str,
    session_id: &Uuid,
    role: &str,
) -> Result<String, AppError> {
    let config = AppConfig::global();
    let now = chrono::Utc::now();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        session_id: session_id.to_string(),
        role: role.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::try_seconds(config.jwt.access_expiration).unwrap())
            .timestamp() as usize,
        iss: config.jwt.issuer.clone(),
        aud: config.jwt.audience.clone(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("Token creation failed: {}", e)))
}

/// Creates a refresh token (long-lived, typically 7 days).
pub fn create_refresh_token(user_id: &Uuid, session_id: &Uuid) -> Result<String, AppError> {
    let config = AppConfig::global();
    let now = chrono::Utc::now();

    let claims = Claims {
        sub: user_id.to_string(),
        email: String::new(),
        session_id: session_id.to_string(),
        role: String::new(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::try_seconds(config.jwt.refresh_expiration).unwrap())
            .timestamp() as usize,
        iss: config.jwt.issuer.clone(),
        aud: config.jwt.audience.clone(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("Refresh token creation failed: {}", e)))
}

/// Validates an access token and returns the claims.
pub fn validate_access_token(token: &str, _state: &AppState) -> Result<Claims, AppError> {
    let config = AppConfig::global();

    let mut validation = Validation::default();
    validation.set_issuer(&[config.jwt.issuer.clone()]);
    validation.set_audience(&[config.jwt.audience.clone()]);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt.secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
        _ => AppError::InvalidToken,
    })?;

    Ok(token_data.claims)
}

/// Validates a refresh token, returns claims.
pub fn validate_refresh_token(token: &str) -> Result<Claims, AppError> {
    let config = AppConfig::global();

    let mut validation = Validation::default();
    validation.set_issuer(&[config.jwt.issuer.clone()]);
    validation.set_audience(&[config.jwt.audience.clone()]);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt.secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::InvalidRefreshToken,
        _ => AppError::InvalidRefreshToken,
    })?;

    Ok(token_data.claims)
}
