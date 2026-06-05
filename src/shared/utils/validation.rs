use crate::shared::errors::AppError;
use regex::Regex;

/// Validates email format.
pub fn validate_email(email: &str) -> Result<(), AppError> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|_| AppError::InternalError("Regex compilation failed".into()))?;

    if !email_regex.is_match(email) {
        return Err(AppError::ValidationError(
            "Invalid email format".to_string(),
        ));
    }

    if email.len() > 254 {
        return Err(AppError::ValidationError(
            "Email too long (max 254 characters)".to_string(),
        ));
    }

    Ok(())
}

/// Validates password strength.
/// Rules: min 8 chars, at least 1 uppercase, 1 lowercase, 1 digit, 1 special char.
pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::ValidationError(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    if password.len() > 128 {
        return Err(AppError::ValidationError(
            "Password too long (max 128 characters)".to_string(),
        ));
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        return Err(AppError::ValidationError(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }
    if !has_lowercase {
        return Err(AppError::ValidationError(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }
    if !has_digit {
        return Err(AppError::ValidationError(
            "Password must contain at least one digit".to_string(),
        ));
    }
    if !has_special {
        return Err(AppError::ValidationError(
            "Password must contain at least one special character".to_string(),
        ));
    }

    Ok(())
}

/// Validates username: alphanumeric + underscore, 3-32 chars.
pub fn validate_username(username: &str) -> Result<(), AppError> {
    if username.len() < 3 {
        return Err(AppError::ValidationError(
            "Username must be at least 3 characters long".to_string(),
        ));
    }

    if username.len() > 32 {
        return Err(AppError::ValidationError(
            "Username too long (max 32 characters)".to_string(),
        ));
    }

    let username_regex = Regex::new(r"^[a-zA-Z0-9_]+$")
        .map_err(|_| AppError::InternalError("Regex compilation failed".into()))?;

    if !username_regex.is_match(username) {
        return Err(AppError::ValidationError(
            "Username can only contain letters, numbers, and underscores".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name+tag@domain.co.uk").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@domain.com").is_err());
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("Abcdef1!").is_ok());
        assert!(validate_password("Pass1234!Strong").is_ok());
        assert!(validate_password("short1A!").is_err()); // < 8
        assert!(validate_password("nouppercase1!").is_err());
        assert!(validate_password("NOLOWERCASE1!").is_err());
        assert!(validate_password("NoDigits!@").is_err());
        assert!(validate_password("NoSpecialChar1").is_err());
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("john_doe").is_ok());
        assert!(validate_username("abc").is_ok());
        assert!(validate_username("ab").is_err()); // too short
        assert!(validate_username("user name").is_err()); // space
        assert!(validate_username("user-name").is_err()); // hyphen
    }
}
