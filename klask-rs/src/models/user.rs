use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub full_name: Option<String>,
    pub phone: Option<String>,
    pub timezone: Option<String>,
    pub preferences: Option<serde_json::Value>,
    pub login_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Type)]
#[sqlx(type_name = "varchar")]
#[sqlx(rename_all = "PascalCase")]
pub enum UserRole {
    Admin,
    User,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Admin => write!(f, "Admin"),
            UserRole::User => write!(f, "User"),
        }
    }
}

impl FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Admin" => Ok(UserRole::Admin),
            "User" => Ok(UserRole::User),
            _ => Err(format!("Unknown user role: {}", s)),
        }
    }
}

/// User preferences stored in JSONB format
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UserPreferences {
    pub theme: Option<String>,    // light, dark, auto
    pub language: Option<String>, // en, fr, es, de, etc.
    pub notifications_email: Option<bool>,
    pub show_activity: Option<bool>,
}

/// Request payload for updating user profile
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub timezone: Option<String>,
    pub preferences: Option<UserPreferences>,
}

/// Request payload for updating sensitive profile fields with password verification
/// Reserved for future use when implementing password-protected profile updates
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileWithPasswordRequest {
    #[serde(flatten)]
    pub profile_data: UpdateProfileRequest,
    /// Password is required for updating sensitive fields (email, phone with sensitive changes)
    pub password: Option<String>,
}

/// Request payload for changing password
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub new_password_confirm: String,
}

/// Request payload for deleting account
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

/// User activity information
#[derive(Debug, Serialize, Deserialize)]
pub struct UserActivity {
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub login_count: i32,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Detailed user profile response
#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub active: bool,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub full_name: Option<String>,
    pub phone: Option<String>,
    pub timezone: Option<String>,
    pub preferences: Option<UserPreferences>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        let preferences = user.preferences.and_then(|prefs| serde_json::from_value(prefs).ok());

        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            active: user.active,
            avatar_url: user.avatar_url,
            bio: user.bio,
            full_name: user.full_name,
            phone: user.phone,
            timezone: user.timezone,
            preferences,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
