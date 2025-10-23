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
