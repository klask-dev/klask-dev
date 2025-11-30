/// Integration tests for UserRepository
/// Tests repository operations with in-memory SQLite
#[cfg(test)]
mod user_repository_tests {
    use anyhow::Result;
    use chrono::Utc;
    use klask_rs::models::{User, UserRole};
    use klask_rs::testing::TestDatabase;
    use sqlx::Row;
    use uuid::Uuid;

    /// Create a test user with default values
    fn create_test_user(username: &str) -> User {
        User {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: format!("{}@example.com", username),
            password_hash: "test_hash".to_string(),
            role: UserRole::User,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
            last_activity: None,
            avatar_url: None,
            bio: None,
            full_name: Some(format!("Test {}", username)),
            phone: None,
            timezone: None,
            preferences: None,
            login_count: 0,
        }
    }

    /// Insert a user into the database
    async fn insert_user(db: &TestDatabase, user: &User) -> Result<()> {
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user.id.to_string())
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.role.to_string())
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
        .execute(db.pool())
        .await?;

        Ok(())
    }

    /// Get a user from the database
    async fn get_user(db: &TestDatabase, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count FROM users WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(db.pool())
        .await?;

        Ok(row.map(|row| User {
            id: Uuid::parse_str(row.get::<String, _>("id").as_str()).unwrap(),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            role: row.get::<String, _>("role").parse::<UserRole>().unwrap_or(UserRole::User),
            active: row.get("active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login: row.get("last_login"),
            last_activity: row.get("last_activity"),
            avatar_url: row.get("avatar_url"),
            bio: row.get("bio"),
            full_name: row.get("full_name"),
            phone: row.get("phone"),
            timezone: row.get("timezone"),
            preferences: row.get("preferences"),
            login_count: row.get("login_count"),
        }))
    }

    /// Find user by username
    async fn find_by_username(db: &TestDatabase, username: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, role, active, created_at, updated_at, last_login, last_activity, avatar_url, bio, full_name, phone, timezone, preferences, login_count FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(db.pool())
        .await?;

        Ok(row.map(|row| User {
            id: Uuid::parse_str(row.get::<String, _>("id").as_str()).unwrap(),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            role: row.get::<String, _>("role").parse::<UserRole>().unwrap_or(UserRole::User),
            active: row.get("active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login: row.get("last_login"),
            last_activity: row.get("last_activity"),
            avatar_url: row.get("avatar_url"),
            bio: row.get("bio"),
            full_name: row.get("full_name"),
            phone: row.get("phone"),
            timezone: row.get("timezone"),
            preferences: row.get("preferences"),
            login_count: row.get("login_count"),
        }))
    }

    /// Delete a user
    async fn delete_user(db: &TestDatabase, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?").bind(id.to_string()).execute(db.pool()).await?;
        Ok(())
    }

    /// Count total users
    async fn count_users(db: &TestDatabase) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(db.pool()).await?;
        Ok(count)
    }

    /// Count active users
    async fn count_active_users(db: &TestDatabase) -> Result<i64> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE active = true").fetch_one(db.pool()).await?;
        Ok(count)
    }

    // Tests

    #[tokio::test]
    async fn test_create_user() -> Result<()> {
        let db = TestDatabase::new().await?;

        let user = create_test_user("john_doe");
        insert_user(&db, &user).await?;

        let retrieved = get_user(&db, user.id).await?;
        assert!(retrieved.is_some());

        let retrieved_user = retrieved.unwrap();
        assert_eq!(retrieved_user.username, "john_doe");
        assert_eq!(retrieved_user.email, "john_doe@example.com");
        assert_eq!(retrieved_user.role, UserRole::User);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_user_by_username() -> Result<()> {
        let db = TestDatabase::new().await?;

        let user = create_test_user("alice");
        insert_user(&db, &user).await?;

        let found = find_by_username(&db, "alice").await?;
        assert!(found.is_some());

        let found_user = found.unwrap();
        assert_eq!(found_user.id, user.id);
        assert_eq!(found_user.username, "alice");

        Ok(())
    }

    #[tokio::test]
    async fn test_find_nonexistent_user() -> Result<()> {
        let db = TestDatabase::new().await?;

        let found = find_by_username(&db, "nonexistent").await?;
        assert!(found.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_count_users() -> Result<()> {
        let db = TestDatabase::new().await?;

        let initial_count = count_users(&db).await?;
        assert_eq!(initial_count, 0);

        let user1 = create_test_user("user1");
        let user2 = create_test_user("user2");

        insert_user(&db, &user1).await?;
        insert_user(&db, &user2).await?;

        let final_count = count_users(&db).await?;
        assert_eq!(final_count, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_active_users_filter() -> Result<()> {
        let db = TestDatabase::new().await?;

        let active_user = create_test_user("active");
        let mut inactive_user = create_test_user("inactive");
        inactive_user.active = false;

        insert_user(&db, &active_user).await?;
        insert_user(&db, &inactive_user).await?;

        let active_count = count_active_users(&db).await?;
        assert_eq!(active_count, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_user() -> Result<()> {
        let db = TestDatabase::new().await?;

        let user = create_test_user("to_delete");
        insert_user(&db, &user).await?;

        let count_before = count_users(&db).await?;
        assert_eq!(count_before, 1);

        delete_user(&db, user.id).await?;

        let count_after = count_users(&db).await?;
        assert_eq!(count_after, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_user_roles() -> Result<()> {
        let db = TestDatabase::new().await?;

        let mut admin_user = create_test_user("admin");
        admin_user.role = UserRole::Admin;

        insert_user(&db, &admin_user).await?;

        let retrieved = get_user(&db, admin_user.id).await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().role, UserRole::Admin);

        Ok(())
    }

    #[tokio::test]
    async fn test_user_profile_fields() -> Result<()> {
        let db = TestDatabase::new().await?;

        let mut user = create_test_user("profile_user");
        user.avatar_url = Some("https://example.com/avatar.jpg".to_string());
        user.bio = Some("A passionate developer".to_string());
        user.phone = Some("+1-555-1234".to_string());
        user.timezone = Some("America/New_York".to_string());

        insert_user(&db, &user).await?;

        let retrieved = get_user(&db, user.id).await?;
        let retrieved_user = retrieved.unwrap();

        assert_eq!(
            retrieved_user.avatar_url,
            Some("https://example.com/avatar.jpg".to_string())
        );
        assert_eq!(retrieved_user.bio, Some("A passionate developer".to_string()));
        assert_eq!(retrieved_user.phone, Some("+1-555-1234".to_string()));
        assert_eq!(retrieved_user.timezone, Some("America/New_York".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_users_independent() -> Result<()> {
        let db = TestDatabase::new().await?;

        let user1 = create_test_user("user1");
        let user2 = create_test_user("user2");
        let user3 = create_test_user("user3");

        insert_user(&db, &user1).await?;
        insert_user(&db, &user2).await?;
        insert_user(&db, &user3).await?;

        // Verify each user can be retrieved independently
        let retrieved1 = get_user(&db, user1.id).await?;
        let retrieved2 = get_user(&db, user2.id).await?;
        let retrieved3 = get_user(&db, user3.id).await?;

        assert_eq!(retrieved1.unwrap().username, "user1");
        assert_eq!(retrieved2.unwrap().username, "user2");
        assert_eq!(retrieved3.unwrap().username, "user3");

        Ok(())
    }

    #[tokio::test]
    async fn test_user_uniqueness() -> Result<()> {
        let db = TestDatabase::new().await?;

        let user1 = create_test_user("unique_user");
        insert_user(&db, &user1).await?;

        // Try to insert another user with same username
        let mut user2 = create_test_user("unique_user");
        user2.id = Uuid::new_v4();

        // SQLite should reject this due to unique constraint
        let result = insert_user(&db, &user2).await;
        assert!(result.is_err(), "Should not allow duplicate username");

        Ok(())
    }
}
