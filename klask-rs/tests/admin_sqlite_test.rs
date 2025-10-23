use anyhow::Result;
use chrono::Utc;
use klask_rs::database::create_test_database;
use klask_rs::models::{User, UserRole};
use klask_rs::repositories::test_user_repository::TestUserRepository;
use uuid::Uuid;

#[tokio::test]
async fn test_sqlite_user_repository() -> Result<()> {
    // Create isolated in-memory database
    let pool = create_test_database().await?;
    let user_repo = TestUserRepository::new(pool);

    // Create test user
    let user = User {
        id: Uuid::new_v4(),
        username: "test_user".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        role: UserRole::User,
        active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
        last_activity: None,
    };

    // Test create
    user_repo.create_user(&user).await?;

    // Test retrieve
    let retrieved_user = user_repo.get_user(user.id).await?;
    assert!(retrieved_user.is_some());
    let retrieved_user = retrieved_user.unwrap();
    assert_eq!(retrieved_user.username, user.username);
    assert_eq!(retrieved_user.email, user.email);

    // Test stats
    let stats = user_repo.get_user_stats().await?;
    assert_eq!(stats.total_users, 1);
    assert_eq!(stats.active_users, 1);
    assert_eq!(stats.admin_users, 0);

    Ok(())
}

#[tokio::test]
async fn test_sqlite_isolation() -> Result<()> {
    // Create two separate databases
    let pool1 = create_test_database().await?;
    let pool2 = create_test_database().await?;

    let user_repo1 = TestUserRepository::new(pool1);
    let user_repo2 = TestUserRepository::new(pool2);

    // Create user in first database
    let user1 = User {
        id: Uuid::new_v4(),
        username: "user1".to_string(),
        email: "user1@example.com".to_string(),
        password_hash: "hash1".to_string(),
        role: UserRole::User,
        active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
        last_activity: None,
    };
    user_repo1.create_user(&user1).await?;

    // Create different user in second database
    let user2 = User {
        id: Uuid::new_v4(),
        username: "user2".to_string(),
        email: "user2@example.com".to_string(),
        password_hash: "hash2".to_string(),
        role: UserRole::Admin,
        active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
        last_activity: None,
    };
    user_repo2.create_user(&user2).await?;

    // Verify isolation
    let stats1 = user_repo1.get_user_stats().await?;
    let stats2 = user_repo2.get_user_stats().await?;

    assert_eq!(stats1.total_users, 1);
    assert_eq!(stats1.admin_users, 0); // user1 is not admin

    assert_eq!(stats2.total_users, 1);
    assert_eq!(stats2.admin_users, 1); // user2 is admin

    // Verify users are not visible across databases
    assert!(user_repo1.get_user(user2.id).await?.is_none());
    assert!(user_repo2.get_user(user1.id).await?.is_none());

    Ok(())
}

#[tokio::test]
async fn test_sqlite_concurrent_access() -> Result<()> {
    // Test that multiple tests can run concurrently without interfering
    let tasks = (0..5).map(|i| {
        tokio::spawn(async move {
            let pool = create_test_database().await?;
            let user_repo = TestUserRepository::new(pool);

            let user = User {
                id: Uuid::new_v4(),
                username: format!("user_{}", i),
                email: format!("user_{}@example.com", i),
                password_hash: format!("hash_{}", i),
                role: if i % 2 == 0 { UserRole::Admin } else { UserRole::User },
                active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login: None,
                last_activity: None,
            };

            user_repo.create_user(&user).await?;
            let stats = user_repo.get_user_stats().await?;

            // Each database should have exactly 1 user
            assert_eq!(stats.total_users, 1);

            anyhow::Ok(())
        })
    });

    // Wait for all tasks to complete
    for task in tasks {
        task.await??;
    }

    Ok(())
}
