use crate::models::User;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Data structure for updating user profile fields
pub struct UpdateProfileData {
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub full_name: Option<String>,
    pub phone: Option<String>,
    pub timezone: Option<String>,
    pub preferences: Option<serde_json::Value>,
}

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: &User) -> Result<User> {
        let result = sqlx::query_as::<_, User>(
            "INSERT INTO users (id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
             RETURNING id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count"
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.role)
        .bind(user.active)
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(user.last_login)
        .bind(user.last_activity)
        .bind(&user.avatar_url)
        .bind(&user.bio)
        .bind(&user.full_name)
        .bind(&user.phone)
        .bind(&user.timezone)
        .bind(&user.preferences)
        .bind(user.login_count)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn list_users(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<User>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let users = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count
             FROM users
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn update_user(&self, id: Uuid, username: Option<&str>, email: Option<&str>) -> Result<User> {
        let existing_user = self.get_user(id).await?.ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let updated_username = username.unwrap_or(&existing_user.username);
        let updated_email = email.unwrap_or(&existing_user.email);

        let updated_user = sqlx::query_as::<_, User>(
            "UPDATE users SET username = $2, email = $3, updated_at = NOW()
             WHERE id = $1
             RETURNING id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count",
        )
        .bind(id)
        .bind(updated_username)
        .bind(updated_email)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    pub async fn update_user_role(&self, id: Uuid, role: crate::models::UserRole) -> Result<User> {
        let updated_user = sqlx::query_as::<_, User>(
            "UPDATE users SET role = $2, updated_at = NOW()
             WHERE id = $1
             RETURNING id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count",
        )
        .bind(id)
        .bind(&role)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    pub async fn update_user_status(&self, id: Uuid, active: bool) -> Result<User> {
        let updated_user = sqlx::query_as::<_, User>(
            "UPDATE users SET active = $2, updated_at = NOW()
             WHERE id = $1
             RETURNING id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count",
        )
        .bind(id)
        .bind(active)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1").bind(id).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn count_users(&self) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users").fetch_one(&self.pool).await?;
        Ok(count)
    }

    pub async fn get_user_stats(&self) -> Result<UserStats> {
        let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users").fetch_one(&self.pool).await?;

        let active_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE active = true")
            .fetch_one(&self.pool)
            .await?;

        let admin_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE role = 'Admin'")
            .fetch_one(&self.pool)
            .await?;

        let recent_registrations =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE created_at >= NOW() - INTERVAL '30 days'")
                .fetch_one(&self.pool)
                .await?;

        Ok(UserStats { total_users, active_users, admin_users, recent_registrations })
    }

    pub async fn update_last_login(&self, id: Uuid) -> Result<User> {
        let updated_user = sqlx::query_as::<_, User>(
            "UPDATE users SET last_login = NOW(), last_activity = NOW(), login_count = login_count + 1, updated_at = NOW()
             WHERE id = $1
             RETURNING id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    #[allow(dead_code)]
    pub async fn update_last_activity(&self, id: Uuid) -> Result<User> {
        let updated_user = sqlx::query_as::<_, User>(
            "UPDATE users SET last_activity = NOW(), updated_at = NOW()
             WHERE id = $1
             RETURNING id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    pub async fn update_user_password(&self, id: Uuid, password_hash: &str) -> Result<User> {
        let updated_user = sqlx::query_as::<_, User>(
            "UPDATE users SET password_hash = $2, updated_at = NOW()
             WHERE id = $1
             RETURNING id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count",
        )
        .bind(id)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    /// Update user profile fields (avatar, bio, name, etc.)
    /// Uses UpdateProfileData to avoid too many arguments
    pub async fn update_user_profile(&self, id: Uuid, data: UpdateProfileData) -> Result<User> {
        let existing_user = self.get_user(id).await?.ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let updated_avatar = data.avatar_url.or(existing_user.avatar_url);
        let updated_bio = data.bio.or(existing_user.bio);
        let updated_name = data.full_name.or(existing_user.full_name);
        let updated_phone = data.phone.or(existing_user.phone);
        let updated_timezone = data.timezone.or(existing_user.timezone);
        let updated_prefs = data.preferences.or(existing_user.preferences);

        let updated_user = sqlx::query_as::<_, User>(
            "UPDATE users SET avatar_url = $2, bio = $3, full_name = $4, phone = $5, timezone = $6, preferences = $7, updated_at = NOW()
             WHERE id = $1
             RETURNING id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count",
        )
        .bind(id)
        .bind(updated_avatar)
        .bind(updated_bio)
        .bind(updated_name)
        .bind(updated_phone)
        .bind(updated_timezone)
        .bind(&updated_prefs)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    /// Get user activity information
    pub async fn get_user_activity(&self, id: Uuid) -> Result<Option<crate::models::UserActivity>> {
        let user = self.get_user(id).await?;

        Ok(user.map(|u| crate::models::UserActivity {
            last_login: u.last_login,
            login_count: u.login_count,
            last_activity: u.last_activity,
            created_at: u.created_at,
        }))
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserStats {
    pub total_users: i64,
    pub active_users: i64,
    pub admin_users: i64,
    pub recent_registrations: i64,
}
