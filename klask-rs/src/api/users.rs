use crate::auth::extractors::{AdminUser, AppState};
use crate::models::{User, UserRole};
use crate::repositories::{user_repository::UserStats, UserRepository};
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<UserRole>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<UserRole>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            active: user.active,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login: user.last_login,
            last_activity: user.last_activity,
        }
    }
}

pub async fn create_router() -> Result<Router<AppState>> {
    let router = Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/{id}/role", put(update_user_role))
        .route("/{id}/status", put(update_user_status))
        .route("/stats", get(get_user_stats));

    Ok(router)
}

async fn list_users(
    State(app_state): State<AppState>,
    Query(query): Query<UserListQuery>,
    _admin_user: AdminUser, // Require admin authentication
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    let user_repository = UserRepository::new(app_state.database.pool().clone());

    match user_repository.list_users(query.limit, query.offset).await {
        Ok(users) => {
            let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
            Ok(Json(user_responses))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_user(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    _admin_user: AdminUser, // Require admin authentication
) -> Result<Json<UserResponse>, StatusCode> {
    let user_repository = UserRepository::new(app_state.database.pool().clone());

    match user_repository.get_user(id).await {
        Ok(Some(user)) => Ok(Json(UserResponse::from(user))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_user(
    State(app_state): State<AppState>,
    _admin_user: AdminUser, // Require admin authentication
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user_repository = UserRepository::new(app_state.database.pool().clone());

    // Check if username or email already exists
    if let Ok(Some(_)) = user_repository.find_by_username(&payload.username).await {
        return Err(StatusCode::CONFLICT);
    }

    if let Ok(Some(_)) = user_repository.find_by_email(&payload.email).await {
        return Err(StatusCode::CONFLICT);
    }

    // Hash password using argon2
    let password_hash = match hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let new_user = User {
        id: Uuid::new_v4(),
        username: payload.username,
        email: payload.email,
        password_hash,
        role: payload.role.unwrap_or(UserRole::User),
        active: payload.active.unwrap_or(true),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login: None,
        last_activity: None,
    };

    match user_repository.create_user(&new_user).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_user(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    _admin_user: AdminUser, // Require admin authentication
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user_repository = UserRepository::new(app_state.database.pool().clone());

    // Check if user exists
    match user_repository.get_user(id).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    // Check for username/email conflicts if they're being updated
    if let Some(ref username) = payload.username {
        if let Ok(Some(existing_user)) = user_repository.find_by_username(username).await {
            if existing_user.id != id {
                return Err(StatusCode::CONFLICT);
            }
        }
    }

    if let Some(ref email) = payload.email {
        if let Ok(Some(existing_user)) = user_repository.find_by_email(email).await {
            if existing_user.id != id {
                return Err(StatusCode::CONFLICT);
            }
        }
    }

    // Update basic user info if provided
    let mut updated_user = if payload.username.is_some() || payload.email.is_some() {
        match user_repository.update_user(id, payload.username.as_deref(), payload.email.as_deref()).await {
            Ok(user) => user,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        match user_repository.get_user(id).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(StatusCode::NOT_FOUND),
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    };

    // Update password if provided
    if let Some(password) = payload.password {
        let password_hash = match hash_password(&password) {
            Ok(hash) => hash,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
        updated_user = match user_repository.update_user_password(id, &password_hash).await {
            Ok(user) => user,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
    }

    // Update role if provided
    if let Some(role) = payload.role {
        updated_user = match user_repository.update_user_role(id, role).await {
            Ok(user) => user,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
    }

    // Update status if provided
    if let Some(active) = payload.active {
        updated_user = match user_repository.update_user_status(id, active).await {
            Ok(user) => user,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
    }

    Ok(Json(UserResponse::from(updated_user)))
}

async fn update_user_role(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    _admin_user: AdminUser, // Require admin authentication
    Json(payload): Json<UserRole>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user_repository = UserRepository::new(app_state.database.pool().clone());

    match user_repository.update_user_role(id, payload).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_user_status(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    _admin_user: AdminUser, // Require admin authentication
    Json(active): Json<bool>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user_repository = UserRepository::new(app_state.database.pool().clone());

    match user_repository.update_user_status(id, active).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_user(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    admin_user: AdminUser, // Require admin authentication
) -> Result<StatusCode, StatusCode> {
    let user_repository = UserRepository::new(app_state.database.pool().clone());

    // Prevent self-deletion
    if admin_user.0.user.id == id {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if user exists
    match user_repository.get_user(id).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    match user_repository.delete_user(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_user_stats(
    State(app_state): State<AppState>,
    _admin_user: AdminUser, // Require admin authentication
) -> Result<Json<UserStats>, StatusCode> {
    let user_repository = UserRepository::new(app_state.database.pool().clone());

    match user_repository.get_user_stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
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
