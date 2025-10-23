use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
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
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AuthError::MissingAuthHeader | AuthError::InvalidAuthHeader => (
                StatusCode::UNAUTHORIZED,
                "Missing or invalid authorization header".to_string(),
            ),
            AuthError::InvalidToken(_) | AuthError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string())
            }
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid username or password".to_string()),
            AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "User not found".to_string()),
            AuthError::UserInactive => (StatusCode::UNAUTHORIZED, "User account is inactive".to_string()),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions".to_string()),
            AuthError::UsernameExists => (StatusCode::CONFLICT, "Username already exists".to_string()),
            AuthError::EmailExists => (StatusCode::CONFLICT, "Email already exists".to_string()),
            AuthError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            AuthError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
