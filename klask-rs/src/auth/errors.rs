use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingAuthHeader,
    #[error("Invalid authorization header format")]
    InvalidAuthHeader,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User is inactive")]
    UserInactive,
    #[error("Username already exists")]
    UsernameExists,
    #[error("Email already exists")]
    EmailExists,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Registration is currently disabled")]
    RegistrationDisabled,
    #[error("Too many attempts: {0}")]
    TooManyAttempts(String, u32), // message, retry_after_seconds
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message, retry_after) = match &self {
            AuthError::MissingAuthHeader | AuthError::InvalidAuthHeader => (
                StatusCode::UNAUTHORIZED,
                "Missing or invalid authorization header".to_string(),
                None,
            ),
            AuthError::InvalidToken(_) | AuthError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string(), None)
            }
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid username or password".to_string(), None),
            AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "User not found".to_string(), None),
            AuthError::UserInactive => (StatusCode::UNAUTHORIZED, "User account is inactive".to_string(), None),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions".to_string(), None),
            AuthError::UsernameExists => (StatusCode::CONFLICT, "Username already exists".to_string(), None),
            AuthError::EmailExists => (StatusCode::CONFLICT, "Email already exists".to_string(), None),
            AuthError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), None),
            AuthError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone(), None),
            AuthError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone(), None),
            AuthError::RegistrationDisabled => {
                (StatusCode::FORBIDDEN, "Registration is currently disabled".to_string(), None)
            }
            AuthError::TooManyAttempts(msg, retry_after_secs) => {
                (StatusCode::TOO_MANY_REQUESTS, msg.clone(), Some(*retry_after_secs))
            }
        };

        let body = json!({
            "error": error_message,
            "status": status.as_u16()
        });

        let mut response = (status, Json(body)).into_response();

        // Add Retry-After header if rate limited
        if let Some(retry_after_secs) = retry_after {
            if let Ok(header_value) = retry_after_secs.to_string().parse() {
                response.headers_mut().insert("Retry-After", header_value);
            }
        }

        response
    }
}
