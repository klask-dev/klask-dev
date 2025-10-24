use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::auth::{extractors::AppState, AuthError, AuthenticatedUser};
use crate::models::user::{
    ChangePasswordRequest, DeleteAccountRequest, UpdateProfileRequest, User, UserActivity, UserProfile, UserRole,
};
use crate::repositories::user_repository::{UpdateProfileData, UserRepository};

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
        .route("/profile", get(get_profile).put(update_profile))
        .route("/password", put(change_password))
        .route("/avatar", post(upload_avatar))
        .route("/activity", get(get_user_activity))
        .route("/account", delete(delete_account))
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
        avatar_url: None,
        bio: None,
        full_name: None,
        phone: None,
        timezone: Some("UTC".to_string()),
        preferences: None,
        login_count: 0,
    };

    let user = user_repo.create_user(&new_user).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Generate JWT token
    let token = app_state
        .jwt_service
        .create_token_for_user(user.id, user.username.clone(), user.role.to_string())
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    Ok(Json(AuthResponse { token, user: UserInfo::from(user) }))
}

async fn get_profile(auth_user: AuthenticatedUser) -> Result<Json<UserProfile>, AuthError> {
    Ok(Json(UserProfile::from(auth_user.user)))
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
        avatar_url: None,
        bio: None,
        full_name: None,
        phone: None,
        timezone: Some("UTC".to_string()),
        preferences: None,
        login_count: 0,
    };

    let user = user_repo.create_user(&admin_user).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Generate JWT token
    let token = app_state
        .jwt_service
        .create_token_for_user(user.id, user.username.clone(), user.role.to_string())
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    Ok(Json(AuthResponse { token, user: UserInfo::from(user) }))
}

/// Update user profile with new information
async fn update_profile(
    auth_user: AuthenticatedUser,
    State(app_state): State<AppState>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, AuthError> {
    // Validate inputs
    if let Some(ref name) = payload.full_name {
        if name.is_empty() || name.len() > 255 {
            return Err(AuthError::InvalidInput(
                "Full name must be 1-255 characters".to_string(),
            ));
        }
    }

    if let Some(ref bio) = payload.bio {
        if bio.len() > 2000 {
            return Err(AuthError::InvalidInput(
                "Bio must be 2000 characters or less".to_string(),
            ));
        }
    }

    if let Some(ref avatar_url) = payload.avatar_url {
        // Allow large base64 data URIs (typical avatar ~100KB base64 = ~133KB string)
        if avatar_url.len() > 1_000_000 {
            return Err(AuthError::InvalidInput("Avatar URL must be 1MB or less".to_string()));
        }
    }

    if let Some(ref phone) = payload.phone {
        if phone.len() > 20 {
            return Err(AuthError::InvalidInput(
                "Phone must be 20 characters or less".to_string(),
            ));
        }
    }

    if let Some(ref tz) = payload.timezone {
        if !validate_timezone(tz) {
            return Err(AuthError::InvalidInput("Invalid timezone".to_string()));
        }
    }

    let user_repo = UserRepository::new(app_state.database.pool().clone());

    // Convert preferences to JSON
    let preferences_json =
        payload.preferences.map(|prefs| serde_json::to_value(prefs).unwrap_or(serde_json::json!({})));

    let profile_data = UpdateProfileData {
        avatar_url: payload.avatar_url,
        bio: payload.bio,
        full_name: payload.full_name,
        phone: payload.phone,
        timezone: payload.timezone,
        preferences: preferences_json,
    };

    let updated_user = user_repo
        .update_user_profile(auth_user.user.id, profile_data)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(Json(UserProfile::from(updated_user)))
}

/// Change user password
async fn change_password(
    auth_user: AuthenticatedUser,
    State(app_state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Verify passwords match
    if payload.new_password != payload.new_password_confirm {
        return Err(AuthError::InvalidInput("Passwords do not match".to_string()));
    }

    // Validate password strength
    validate_password_strength(&payload.new_password)?;

    // Verify current password
    let is_valid = verify_password(&payload.current_password, &auth_user.user.password_hash)
        .map_err(|_| AuthError::InvalidCredentials)?;

    if !is_valid {
        return Err(AuthError::InvalidCredentials);
    }

    // Hash new password
    let new_password_hash = hash_password(&payload.new_password)
        .map_err(|_| AuthError::InvalidInput("Password hashing failed".to_string()))?;

    let user_repo = UserRepository::new(app_state.database.pool().clone());

    user_repo
        .update_user_password(auth_user.user.id, &new_password_hash)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Password changed successfully"
    })))
}

/// Upload avatar image for user
#[derive(Debug, Serialize)]
pub struct AvatarUploadResponse {
    pub avatar_url: String,
}

async fn upload_avatar(_auth_user: AuthenticatedUser) -> Result<Json<AvatarUploadResponse>, AuthError> {
    // Avatar is processed in the frontend and sent via PUT /api/auth/profile
    // This endpoint exists to acknowledge the upload endpoint exists
    // The frontend handles base64 conversion and storage
    Ok(Json(AvatarUploadResponse {
        avatar_url: "avatar_upload_acknowledged".to_string(),
    }))
}

/// Get user activity information
async fn get_user_activity(
    auth_user: AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<Json<UserActivity>, AuthError> {
    let user_repo = UserRepository::new(app_state.database.pool().clone());

    let activity = user_repo
        .get_user_activity(auth_user.user.id)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::Forbidden("User not found".to_string()))?;

    Ok(Json(activity))
}

/// Delete user account with rate limiting for security
async fn delete_account(
    auth_user: AuthenticatedUser,
    State(app_state): State<AppState>,
    Json(payload): Json<DeleteAccountRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    const MAX_ATTEMPTS: u32 = 5;
    const RATE_LIMIT_WINDOW_SECS: u64 = 300; // 5 minutes

    let user_id = auth_user.user.id;
    let now = std::time::SystemTime::now();

    // Check rate limiting
    {
        let mut limiter = app_state.delete_account_rate_limiter.write().await;

        if let Some((attempts, last_reset)) = limiter.get_mut(&user_id) {
            // Check if we need to reset the counter
            if let Ok(elapsed) = now.duration_since(*last_reset) {
                if elapsed.as_secs() > RATE_LIMIT_WINDOW_SECS {
                    // Reset the counter
                    *attempts = 0;
                    *last_reset = now;
                } else if *attempts >= MAX_ATTEMPTS {
                    // Too many attempts
                    let remaining_time = RATE_LIMIT_WINDOW_SECS - elapsed.as_secs();
                    return Err(AuthError::InvalidInput(format!(
                        "Too many delete attempts. Please try again in {} seconds",
                        remaining_time
                    )));
                }
            }
            // Increment attempt counter
            *attempts += 1;
        } else {
            // First attempt for this user
            limiter.insert(user_id, (1, now));
        }
    }

    // Verify password
    let is_valid =
        verify_password(&payload.password, &auth_user.user.password_hash).map_err(|_| AuthError::InvalidCredentials)?;

    if !is_valid {
        return Err(AuthError::InvalidCredentials);
    }

    let user_repo = UserRepository::new(app_state.database.pool().clone());

    // Delete the user
    user_repo.delete_user(auth_user.user.id).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Clear rate limiter entry on successful deletion
    app_state.delete_account_rate_limiter.write().await.remove(&user_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Account deleted successfully"
    })))
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

/// Validate password meets minimum security requirements
fn validate_password_strength(password: &str) -> Result<(), AuthError> {
    if password.len() < 8 {
        return Err(AuthError::InvalidInput(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(AuthError::InvalidInput(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(AuthError::InvalidInput(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err(AuthError::InvalidInput(
            "Password must contain at least one number".to_string(),
        ));
    }

    Ok(())
}

/// Validate timezone string
fn validate_timezone(tz: &str) -> bool {
    // Common valid timezones
    let valid_timezones = vec![
        "UTC",
        "GMT",
        "Europe/London",
        "Europe/Paris",
        "Europe/Berlin",
        "Europe/Amsterdam",
        "Europe/Brussels",
        "Europe/Vienna",
        "Europe/Prague",
        "Europe/Warsaw",
        "Europe/Moscow",
        "Europe/Istanbul",
        "Asia/Tokyo",
        "Asia/Shanghai",
        "Asia/Hong_Kong",
        "Asia/Singapore",
        "Asia/Bangkok",
        "Asia/Dubai",
        "Asia/Kolkata",
        "Asia/Jakarta",
        "Asia/Manila",
        "Asia/Seoul",
        "America/New_York",
        "America/Chicago",
        "America/Denver",
        "America/Los_Angeles",
        "America/Anchorage",
        "Pacific/Honolulu",
        "America/Toronto",
        "America/Mexico_City",
        "America/Buenos_Aires",
        "America/Sao_Paulo",
        "America/Los_Angeles",
        "Australia/Sydney",
        "Australia/Melbourne",
        "Australia/Brisbane",
        "Australia/Perth",
        "Pacific/Auckland",
        "Pacific/Fiji",
        "Africa/Cairo",
        "Africa/Johannesburg",
        "Africa/Lagos",
        "Africa/Nairobi",
    ];

    valid_timezones.contains(&tz) || tz == "UTC"
}
