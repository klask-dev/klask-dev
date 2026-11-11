use anyhow::Result;
use axum::{
    Router,
    extract::State,
    http::header::SET_COOKIE,
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;
use validator::Validate;

use crate::auth::rate_limit::{PeerAddr, client_key};
use crate::auth::{AuthError, AuthenticatedUser, extractors::AppState};
use crate::models::user::{
    ChangePasswordRequest, DeleteAccountRequest, UpdateProfileRequest, User, UserActivity, UserProfile, UserRole,
};
use crate::repositories::ApiTokenRepository;
use crate::repositories::user_repository::{UpdateProfileData, UserRepository};
use crate::utils::password::{get_dummy_hash, hash_password_async, verify_password_async};

/// Build the Set-Cookie header value for the auth_token HttpOnly cookie.
/// `max_age_secs` matches the JWT lifetime; `secure` adds the `Secure` flag
/// (see `COOKIE_SECURE`).
fn build_auth_cookie(token: &str, max_age_secs: i64, secure: bool) -> String {
    let secure_attr = if secure { "; Secure" } else { "" };
    format!(
        "auth_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}{}",
        token, max_age_secs, secure_attr
    )
}

/// Build the Set-Cookie header value that clears the auth cookie.
fn build_logout_cookie(secure: bool) -> String {
    build_auth_cookie("", 0, secure)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    // Length check only, so accounts created under an older policy can still log in.
    // The strength policy (`validate_password_strength`) applies when a password is set.
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SetupRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub password: String,
    /// Required when the instance sets `SETUP_TOKEN`.
    #[serde(default)]
    pub setup_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupCheckResponse {
    pub needs_setup: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationStatusResponse {
    pub registration_allowed: bool,
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
        .route("/logout", post(logout))
        .route("/register", post(register))
        .route("/registration/status", get(get_registration_status))
        .route("/profile", get(get_profile).put(update_profile))
        .route("/password", put(change_password))
        .route("/avatar", post(upload_avatar))
        .route("/activity", get(get_user_activity))
        .route("/account", delete(delete_account))
        .route("/setup/check", get(check_setup))
        .route("/setup", post(initial_setup));

    Ok(router)
}

/// Attach an HttpOnly auth cookie to an AuthResponse and return it as a full response.
fn auth_response_with_cookie(app_state: &AppState, auth_response: AuthResponse) -> Response {
    let cookie_value = build_auth_cookie(
        &auth_response.token,
        app_state.jwt_service.expires_in_secs(),
        app_state.config.auth.cookie_secure,
    );
    let mut response = Json(auth_response).into_response();
    if let Ok(hv) = HeaderValue::from_str(&cookie_value) {
        response.headers_mut().insert(SET_COOKIE, hv);
    }
    response
}

/// Error returned when a client has exhausted an auth rate-limit budget.
fn too_many_attempts(retry_after_secs: u64) -> AuthError {
    AuthError::TooManyAttempts(
        format!("Too many attempts. Please try again in {} seconds", retry_after_secs),
        retry_after_secs.min(u32::MAX as u64) as u32,
    )
}

/// Client address used for rate limiting (see `TRUSTED_PROXY_HOPS`).
fn client_address(app_state: &AppState, headers: &HeaderMap, peer: PeerAddr) -> String {
    client_key(headers, peer.0, app_state.config.auth.trusted_proxy_hops)
}

/// Delay requests from a client that exceeded its shared budget. Never refuses.
async fn apply_penalty(limiter: &crate::auth::rate_limit::FixedWindowLimiter, client: &str) {
    let delay = limiter.penalty(client);
    if !delay.is_zero() {
        warn!("Slowing down auth requests from {} by {:?}", client, delay);
        tokio::time::sleep(delay).await;
    }
}

async fn login(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    peer: PeerAddr,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate request
    req.validate().map_err(|_| AuthError::InvalidCredentials)?;

    let client = client_address(&app_state, &headers, peer);
    let username = req.username.clone();
    // Per-(client, user) budget: a victim's account cannot be locked from elsewhere.
    let client_user_key = format!("{}|{}", client, username);
    let limits = &app_state.auth_rate_limits;

    if let Err(retry_after) = limits.login_per_client_and_user.check(&client_user_key) {
        warn!("Login rate limit exceeded for user {} from {}", username, client);
        return Err(too_many_attempts(retry_after));
    }
    apply_penalty(&limits.login_per_client, &client).await;

    let user_repo = UserRepository::new(app_state.database.pool().clone());

    // Find user by username
    let user = match user_repo.find_by_username(&username).await.map_err(|e| AuthError::DatabaseError(e.to_string()))? {
        Some(user) => user,
        None => {
            // Perform a dummy password verification to mitigate timing attacks
            // This equalizes the response time between "user not found" and "wrong password"
            let _ = verify_password_async(req.password.clone(), get_dummy_hash().to_string()).await;
            limits.login_per_client_and_user.record(&client_user_key);
            limits.login_per_client.record(&client);
            return Err(AuthError::InvalidCredentials);
        }
    };

    // Verify user is active
    if !user.active {
        return Err(AuthError::UserInactive);
    }

    // Verify password (Argon2 runs on the blocking pool)
    let is_valid = verify_password_async(req.password.clone(), user.password_hash.clone())
        .await
        .map_err(|_| AuthError::InvalidCredentials)?;

    if !is_valid {
        limits.login_per_client_and_user.record(&client_user_key);
        limits.login_per_client.record(&client);
        return Err(AuthError::InvalidCredentials);
    }

    // Clear the per-user counter on successful login
    limits.login_per_client_and_user.reset(&client_user_key);

    // Re-hash password if it uses old parameters (m=65536,t=3,p=2) for automatic migration
    let mut user = user;
    if user.password_hash.contains("m=65536,t=3,p=2")
        && let Ok(new_hash) = hash_password_async(req.password.clone()).await
        && let Ok(updated_user) = user_repo.update_user_password(user.id, &new_hash).await
    {
        user = updated_user;
    }

    // Update last_login and last_activity timestamps
    let user = user_repo.update_last_login(user.id).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Generate JWT token
    let token = app_state
        .jwt_service
        .create_token_for_user(user.id, user.username.clone(), user.role.to_string())
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    Ok(auth_response_with_cookie(
        &app_state,
        AuthResponse { token, user: UserInfo::from(user) },
    ))
}

/// Logout: clear the auth_token cookie by setting Max-Age=0.
async fn logout(State(app_state): State<AppState>) -> impl IntoResponse {
    let cookie = build_logout_cookie(app_state.config.auth.cookie_secure);
    (
        [(SET_COOKIE, cookie)],
        Json(serde_json::json!({"message": "Logged out successfully"})),
    )
}

async fn register(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    peer: PeerAddr,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Check if registration is allowed
    if !app_state.config.auth.allow_registration {
        return Err(AuthError::RegistrationDisabled);
    }

    let client = client_address(&app_state, &headers, peer);
    let limits = &app_state.auth_rate_limits;
    apply_penalty(&limits.registration_per_client, &client).await;
    limits.registration_per_client.record(&client);

    // Validate request
    req.validate().map_err(|_| AuthError::InvalidCredentials)?;
    validate_password_strength(&req.password)?;

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
    let password_hash = hash_password_async(req.password).await.map_err(|_| AuthError::InvalidCredentials)?;

    // Create new user
    let now = chrono::Utc::now();
    let new_user = User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        email: req.email,
        password_hash,
        role: UserRole::User, // Default role
        active: true,
        created_at: now,
        updated_at: now,
        last_login: None,
        last_activity: None,
        avatar_url: None,
        bio: None,
        full_name: None,
        phone: None,
        timezone: Some("UTC".to_string()),
        preferences: None,
        login_count: 0,
        password_changed_at: now,
    };

    let user = user_repo.create_user(&new_user).await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Generate JWT token
    let token = app_state
        .jwt_service
        .create_token_for_user(user.id, user.username.clone(), user.role.to_string())
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    Ok(auth_response_with_cookie(
        &app_state,
        AuthResponse { token, user: UserInfo::from(user) },
    ))
}

async fn get_profile(auth_user: AuthenticatedUser) -> Result<Json<UserProfile>, AuthError> {
    Ok(Json(UserProfile::from(auth_user.user)))
}

async fn check_setup(State(app_state): State<AppState>) -> Result<Json<SetupCheckResponse>, AuthError> {
    let user_repo = UserRepository::new(app_state.database.pool().clone());

    let user_count = user_repo.count_users().await.map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(Json(SetupCheckResponse { needs_setup: user_count == 0 }))
}

async fn get_registration_status(
    State(app_state): State<AppState>,
) -> Result<Json<RegistrationStatusResponse>, AuthError> {
    Ok(Json(RegistrationStatusResponse {
        registration_allowed: app_state.config.auth.allow_registration,
    }))
}

/// Compare two secrets without short-circuiting on the first differing byte.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn initial_setup(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    peer: PeerAddr,
    Json(req): Json<SetupRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let client = client_address(&app_state, &headers, peer);
    let limits = &app_state.auth_rate_limits;
    apply_penalty(&limits.registration_per_client, &client).await;
    limits.registration_per_client.record(&client);

    // Optional setup secret (SETUP_TOKEN): without it, whoever reaches the
    // instance first before any account exists becomes the administrator.
    if let Some(expected) = app_state.config.auth.setup_token.as_deref() {
        let provided = req.setup_token.as_deref().unwrap_or("");
        if !constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
            warn!("Initial setup rejected: invalid setup token from {}", client);
            return Err(AuthError::Forbidden("Invalid setup token".to_string()));
        }
    }

    // Validate request
    req.validate().map_err(|_| AuthError::InvalidCredentials)?;
    validate_password_strength(&req.password)?;

    let user_repo = UserRepository::new(app_state.database.pool().clone());

    // Hash password
    let password_hash = hash_password_async(req.password).await.map_err(|_| AuthError::InvalidCredentials)?;

    // Create the first admin user atomically
    let now = chrono::Utc::now();
    let admin_user = User {
        id: Uuid::new_v4(),
        username: req.username.clone(),
        email: req.email,
        password_hash,
        role: UserRole::Admin, // First user is always admin
        active: true,
        created_at: now,
        updated_at: now,
        last_login: None,
        last_activity: None,
        avatar_url: None,
        bio: None,
        full_name: None,
        phone: None,
        timezone: Some("UTC".to_string()),
        preferences: None,
        login_count: 0,
        password_changed_at: now,
    };

    // Use atomic insert: only succeeds if no users exist
    let user = user_repo
        .create_first_admin_if_not_exists(&admin_user)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AuthError::Forbidden("Setup already completed".to_string()))?;

    // Generate JWT token
    let token = app_state
        .jwt_service
        .create_token_for_user(user.id, user.username.clone(), user.role.to_string())
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    Ok(auth_response_with_cookie(
        &app_state,
        AuthResponse { token, user: UserInfo::from(user) },
    ))
}

/// Update user profile with new information
async fn update_profile(
    auth_user: AuthenticatedUser,
    State(app_state): State<AppState>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, AuthError> {
    // Validate inputs
    if let Some(ref name) = payload.full_name
        && (name.is_empty() || name.len() > 255)
    {
        return Err(AuthError::InvalidInput(
            "Full name must be 1-255 characters".to_string(),
        ));
    }

    if let Some(ref bio) = payload.bio
        && bio.len() > 2000
    {
        return Err(AuthError::InvalidInput(
            "Bio must be 2000 characters or less".to_string(),
        ));
    }

    if let Some(ref avatar_url) = payload.avatar_url {
        if avatar_url.len() > 1_000_000 {
            return Err(AuthError::InvalidInput("Avatar URL must be 1MB or less".to_string()));
        }
        validate_avatar_url(avatar_url)?;
    }

    if let Some(ref phone) = payload.phone
        && phone.len() > 20
    {
        return Err(AuthError::InvalidInput(
            "Phone must be 20 characters or less".to_string(),
        ));
    }

    if let Some(ref tz) = payload.timezone
        && !validate_timezone(tz)
    {
        return Err(AuthError::InvalidInput("Invalid timezone".to_string()));
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
    let is_valid = verify_password_async(payload.current_password.clone(), auth_user.user.password_hash.clone())
        .await
        .map_err(|_| AuthError::InvalidCredentials)?;

    if !is_valid {
        return Err(AuthError::InvalidCredentials);
    }

    // Hash new password
    let new_password_hash = hash_password_async(payload.new_password.clone())
        .await
        .map_err(|_| AuthError::InvalidInput("Password hashing failed".to_string()))?;

    let user_repo = UserRepository::new(app_state.database.pool().clone());

    user_repo
        .update_user_password(auth_user.user.id, &new_password_hash)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // A password change usually follows a suspected compromise: revoke every
    // personal API token of the account. Session JWTs are invalidated already
    // through `password_changed_at`.
    ApiTokenRepository::new(app_state.database.pool().clone())
        .revoke_all_for_user(auth_user.user.id)
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
    let is_valid = verify_password_async(payload.password.clone(), auth_user.user.password_hash.clone())
        .await
        .map_err(|_| AuthError::InvalidCredentials)?;

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

/// Validate avatar_url is a safe https:// URL or an allowed data URI (jpeg/png/gif/webp only).
/// Rejects SVG and any other MIME type to prevent stored XSS via malicious SVG payloads.
fn validate_avatar_url(url: &str) -> Result<(), AuthError> {
    // Allow plain https:// external image URLs
    if url.starts_with("https://") {
        return Ok(());
    }

    // Allow only safe raster image data URIs
    const ALLOWED_PREFIXES: &[&str] = &[
        "data:image/jpeg;base64,",
        "data:image/jpg;base64,",
        "data:image/png;base64,",
        "data:image/gif;base64,",
        "data:image/webp;base64,",
    ];
    if ALLOWED_PREFIXES.iter().any(|p| url.starts_with(p)) {
        return Ok(());
    }

    Err(AuthError::InvalidInput(
        "avatar_url must be an https:// URL or a data URI with MIME type image/jpeg, image/png, image/gif, or image/webp".to_string(),
    ))
}

/// Validate password meets minimum security requirements
pub fn validate_password_strength(password: &str) -> Result<(), AuthError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_cookie_secure_flag_is_configurable() {
        let plain = build_auth_cookie("tok", 3600, false);
        assert!(plain.contains("HttpOnly"));
        assert!(plain.contains("Max-Age=3600"));
        assert!(!plain.contains("Secure"));

        let secure = build_auth_cookie("tok", 3600, true);
        assert!(secure.ends_with("; Secure"));
    }

    #[test]
    fn test_logout_cookie_clears_and_keeps_secure_flag() {
        let cookie = build_logout_cookie(true);
        assert!(cookie.starts_with("auth_token=;"));
        assert!(cookie.contains("Max-Age=0"));
        assert!(cookie.ends_with("; Secure"));
        assert!(!build_logout_cookie(false).contains("Secure"));
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(!constant_time_eq(b"secret", b"secreT"));
        assert!(!constant_time_eq(b"secret", b"secre"));
        assert!(!constant_time_eq(b"", b"x"));
    }

    #[test]
    fn test_password_policy_is_single_rule() {
        assert!(validate_password_strength("Short1A").is_err());
        assert!(validate_password_strength("lowercase1").is_err());
        assert!(validate_password_strength("NoDigitsHere").is_err());
        assert!(validate_password_strength("GoodPass1").is_ok());
    }
}
