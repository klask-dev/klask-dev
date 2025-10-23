use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::auth::{extractors::AppState, AuthError, AuthenticatedUser};
use crate::models::user::{User, UserRole};
use crate::repositories::user_repository::UserRepository;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SetupRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupCheckResponse {
    pub needs_setup: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub active: bool,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self { id: user.id, username: user.username, email: user.email, role: user.role, active: user.active }
    }
}

pub async fn create_router() -> Result<Router<AppState>> {
    let router = Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/profile", get(get_profile))
        .route("/setup/check", get(check_setup))
        .route("/setup", post(initial_setup));

    Ok(router)
}

async fn login(
    State(app_state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // Validate request
    req.validate().map_err(|_| AuthError::InvalidCredentials)?;

    let user_repo = UserRepository::new(app_state.database.pool().clone());

    // Find user by username
    let user = user_repo
        .find_by_username(&req.username)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

    // Verify user is active
    if !user.active {
        return Err(AuthError::UserInactive);
    }

    // Verify password
    let is_valid = verify_password(&req.password, &user.password_hash).map_err(|_| AuthError::InvalidCredentials)?;

    if !is_valid {
        return Err(AuthError::InvalidCredentials);
    }

    // Update last_login and last_activity timestamps
    let user = user_repo.update_last_login(user.id).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Generate JWT token
    let token = app_state
        .jwt_service
        .create_token_for_user(user.id, user.username.clone(), user.role.to_string())
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    Ok(Json(AuthResponse { token, user: UserInfo::from(user) }))
}

async fn register(
    State(app_state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // Validate request
    req.validate().map_err(|_| AuthError::InvalidCredentials)?;

    let user_repo = UserRepository::new(app_state.database.pool().clone());

    // Check if username already exists
    if user_repo.find_by_username(&req.username).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?.is_some() {
        return Err(AuthError::UsernameExists);
    }

    // Check if email already exists
    if user_repo.find_by_email(&req.email).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?.is_some() {
        return Err(AuthError::EmailExists);
    }

    // Hash password
    let password_hash = hash_password(&req.password).map_err(|_| AuthError::InvalidCredentials)?;

    // Create new user
    let new_user = User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        email: req.email,
        password_hash,
        role: UserRole::User, // Default role
        active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login: None,
        last_activity: None,
    };

    let user = user_repo.create_user(&new_user).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Generate JWT token
    let token = app_state
        .jwt_service
        .create_token_for_user(user.id, user.username.clone(), user.role.to_string())
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    Ok(Json(AuthResponse { token, user: UserInfo::from(user) }))
}

async fn get_profile(auth_user: AuthenticatedUser) -> Result<Json<UserInfo>, AuthError> {
    Ok(Json(UserInfo::from(auth_user.user)))
}

async fn check_setup(State(app_state): State<AppState>) -> Result<Json<SetupCheckResponse>, AuthError> {
    let user_repo = UserRepository::new(app_state.database.pool().clone());

    let user_count = user_repo.count_users().await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(Json(SetupCheckResponse { needs_setup: user_count == 0 }))
}

async fn initial_setup(
    State(app_state): State<AppState>,
    Json(req): Json<SetupRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    // Validate request
    req.validate().map_err(|_| AuthError::InvalidCredentials)?;

    let user_repo = UserRepository::new(app_state.database.pool().clone());

    // Check if any users exist
    let user_count = user_repo.count_users().await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    if user_count > 0 {
        return Err(AuthError::Forbidden("Setup already completed".to_string()));
    }

    // Hash password
    let password_hash = hash_password(&req.password).map_err(|_| AuthError::InvalidCredentials)?;

    // Create the first admin user
    let admin_user = User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        email: req.email,
        password_hash,
        role: UserRole::Admin, // First user is always admin
        active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login: None,
        last_activity: None,
    };

    let user = user_repo.create_user(&admin_user).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Generate JWT token
    let token = app_state
        .jwt_service
        .create_token_for_user(user.id, user.username.clone(), user.role.to_string())
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    Ok(Json(AuthResponse { token, user: UserInfo::from(user) }))
}

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Password hash parsing failed: {}", e))?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}
