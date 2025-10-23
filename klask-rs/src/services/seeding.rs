use anyhow::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::models::{Repository, RepositoryType, User, UserRole};
use crate::repositories::{RepositoryRepository, UserRepository};

#[derive(Debug, Serialize, Deserialize)]
pub struct SeedingStats {
    pub users_created: i64,
    pub repositories_created: i64,
}

pub struct SeedingService {
    pool: PgPool,
}

impl SeedingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn seed_all(&self) -> Result<()> {
        info!("Starting database seeding...");

        self.seed_users().await?;
        self.seed_repositories().await?;

        info!("Database seeding completed successfully!");
        Ok(())
    }

    pub async fn clear_all(&self) -> Result<()> {
        info!("Clearing all seed data...");

        // Clear in reverse order due to foreign key constraints
        sqlx::query("DELETE FROM repositories").execute(&self.pool).await?;
        sqlx::query("DELETE FROM users").execute(&self.pool).await?;

        info!("All seed data cleared!");
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<SeedingStats> {
        // Count only the seeded users (exclude integration test admin users)
        let users_created = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username IN ('admin', 'demo', 'viewer', 'inactive', 'tester')",
        )
        .fetch_one(&self.pool)
        .await?;

        let repositories_created =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM repositories").fetch_one(&self.pool).await?;

        Ok(SeedingStats { users_created, repositories_created })
    }

    async fn seed_users(&self) -> Result<()> {
        info!("Seeding users...");
        let user_repo = UserRepository::new(self.pool.clone());

        let users = vec![
            User {
                id: Uuid::new_v4(),
                username: "admin".to_string(),
                email: "admin@klask.dev".to_string(),
                password_hash: "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewDR8F4Ap5xV/2zS".to_string(), // "admin123"
                role: UserRole::Admin,
                active: true,
                created_at: Utc::now() - Duration::days(30),
                updated_at: Utc::now() - Duration::days(1),
                last_login: Some(Utc::now() - Duration::days(1)),
                last_activity: Some(Utc::now() - Duration::hours(12)),
            },
            User {
                id: Uuid::new_v4(),
                username: "demo".to_string(),
                email: "demo@klask.dev".to_string(),
                password_hash: "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewDR8F4Ap5xV/2zS".to_string(), // "demo123"
                role: UserRole::User,
                active: true,
                created_at: Utc::now() - Duration::days(15),
                updated_at: Utc::now() - Duration::hours(6),
                last_login: Some(Utc::now() - Duration::hours(6)),
                last_activity: Some(Utc::now() - Duration::hours(3)),
            },
            User {
                id: Uuid::new_v4(),
                username: "viewer".to_string(),
                email: "viewer@klask.dev".to_string(),
                password_hash: "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewDR8F4Ap5xV/2zS".to_string(), // "viewer123"
                role: UserRole::User,
                active: true,
                created_at: Utc::now() - Duration::days(7),
                updated_at: Utc::now() - Duration::hours(2),
                last_login: Some(Utc::now() - Duration::hours(2)),
                last_activity: Some(Utc::now() - Duration::hours(1)),
            },
            User {
                id: Uuid::new_v4(),
                username: "inactive".to_string(),
                email: "inactive@klask.dev".to_string(),
                password_hash: "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewDR8F4Ap5xV/2zS".to_string(), // "inactive123"
                role: UserRole::User,
                active: false, // Inactive user
                created_at: Utc::now() - Duration::days(60),
                updated_at: Utc::now() - Duration::days(30),
                last_login: Some(Utc::now() - Duration::days(30)),
                last_activity: Some(Utc::now() - Duration::days(30)),
            },
            User {
                id: Uuid::new_v4(),
                username: "tester".to_string(),
                email: "tester@klask.dev".to_string(),
                password_hash: "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewDR8F4Ap5xV/2zS".to_string(), // "tester123"
                role: UserRole::User,
                active: true,
                created_at: Utc::now() - Duration::days(5),
                updated_at: Utc::now() - Duration::hours(1),
                last_login: Some(Utc::now() - Duration::hours(1)),
                last_activity: Some(Utc::now() - Duration::minutes(30)),
            },
        ];

        for user in users {
            match user_repo.create_user(&user).await {
                Ok(_) => {
                    debug!("Created user: {}", user.username);
                }
                Err(e) => {
                    if e.to_string().contains("duplicate key value") {
                        debug!("User {} already exists, skipping", user.username);
                    } else {
                        error!("Failed to create user {}: {:?}", user.username, e);
                        return Err(e);
                    }
                }
            }
        }

        info!("Users seeding completed");
        Ok(())
    }

    async fn seed_repositories(&self) -> Result<()> {
        info!("Seeding repositories...");
        let repo_repo = RepositoryRepository::new(self.pool.clone());

        let repositories = vec![
            Repository {
                id: Uuid::new_v4(),
                name: "klask-react".to_string(),
                url: "https://github.com/klask-dev/klask-react".to_string(),
                repository_type: RepositoryType::Git,
                branch: Some("main".to_string()),
                enabled: true,
                access_token: None,
                gitlab_namespace: None,
                is_group: false,
                last_crawled: Some(Utc::now() - Duration::hours(6)),
                created_at: Utc::now() - Duration::days(20),
                updated_at: Utc::now() - Duration::hours(6),
                auto_crawl_enabled: true,
                cron_schedule: Some("0 2 * * *".to_string()), // Daily at 2 AM
                next_crawl_at: Some(Utc::now() + Duration::hours(18)),
                crawl_frequency_hours: Some(24),
                max_crawl_duration_minutes: Some(30),
                last_crawl_duration_seconds: None,
                gitlab_excluded_projects: None,
                gitlab_excluded_patterns: None,
                github_namespace: None,
                github_excluded_repositories: None,
                github_excluded_patterns: None,
                crawl_state: None,
                last_processed_project: None,
                crawl_started_at: None,
            },
            Repository {
                id: Uuid::new_v4(),
                name: "klask-rs".to_string(),
                url: "https://github.com/klask-dev/klask-rs".to_string(),
                repository_type: RepositoryType::Git,
                branch: Some("main".to_string()),
                enabled: true,
                access_token: None,
                gitlab_namespace: None,
                is_group: false,
                last_crawled: Some(Utc::now() - Duration::hours(2)),
                created_at: Utc::now() - Duration::days(25),
                updated_at: Utc::now() - Duration::hours(2),
                auto_crawl_enabled: true,
                cron_schedule: Some("0 */6 * * *".to_string()), // Every 6 hours
                next_crawl_at: Some(Utc::now() + Duration::hours(4)),
                crawl_frequency_hours: Some(6),
                max_crawl_duration_minutes: Some(45),
                last_crawl_duration_seconds: None,
                gitlab_excluded_projects: None,
                gitlab_excluded_patterns: None,
                github_namespace: None,
                github_excluded_repositories: None,
                github_excluded_patterns: None,
                crawl_state: None,
                last_processed_project: None,
                crawl_started_at: None,
            },
            Repository {
                id: Uuid::new_v4(),
                name: "docs".to_string(),
                url: "/var/www/docs".to_string(),
                repository_type: RepositoryType::FileSystem,
                branch: None,
                enabled: true,
                access_token: None,
                gitlab_namespace: None,
                is_group: false,
                last_crawled: Some(Utc::now() - Duration::hours(12)),
                created_at: Utc::now() - Duration::days(10),
                updated_at: Utc::now() - Duration::hours(12),
                auto_crawl_enabled: false,
                cron_schedule: None,
                next_crawl_at: None,
                crawl_frequency_hours: None,
                max_crawl_duration_minutes: Some(20),
                last_crawl_duration_seconds: None,
                gitlab_excluded_projects: None,
                gitlab_excluded_patterns: None,
                github_namespace: None,
                github_excluded_repositories: None,
                github_excluded_patterns: None,
                crawl_state: None,
                last_processed_project: None,
                crawl_started_at: None,
            },
            Repository {
                id: Uuid::new_v4(),
                name: "example-api".to_string(),
                url: "https://gitlab.example.com/api/example-api".to_string(),
                repository_type: RepositoryType::GitLab,
                branch: Some("develop".to_string()),
                enabled: true,
                access_token: None,
                gitlab_namespace: Some("api".to_string()),
                is_group: false,
                last_crawled: None, // Never crawled
                created_at: Utc::now() - Duration::days(5),
                updated_at: Utc::now() - Duration::days(5),
                auto_crawl_enabled: false,
                cron_schedule: None,
                next_crawl_at: None,
                crawl_frequency_hours: None,
                max_crawl_duration_minutes: Some(15),
                last_crawl_duration_seconds: None,
                gitlab_excluded_projects: None,
                gitlab_excluded_patterns: None,
                github_namespace: None,
                github_excluded_repositories: None,
                github_excluded_patterns: None,
                crawl_state: None,
                last_processed_project: None,
                crawl_started_at: None,
            },
            Repository {
                id: Uuid::new_v4(),
                name: "legacy-system".to_string(),
                url: "https://github.com/company/legacy-system".to_string(),
                repository_type: RepositoryType::Git,
                branch: Some("master".to_string()),
                enabled: false, // Disabled repository
                access_token: None,
                gitlab_namespace: None,
                is_group: false,
                last_crawled: Some(Utc::now() - Duration::days(30)),
                created_at: Utc::now() - Duration::days(45),
                updated_at: Utc::now() - Duration::days(30),
                auto_crawl_enabled: false,
                cron_schedule: None,
                next_crawl_at: None,
                crawl_frequency_hours: None,
                max_crawl_duration_minutes: Some(60),
                last_crawl_duration_seconds: None,
                gitlab_excluded_projects: None,
                gitlab_excluded_patterns: None,
                github_namespace: None,
                github_excluded_repositories: None,
                github_excluded_patterns: None,
                crawl_state: None,
                last_processed_project: None,
                crawl_started_at: None,
            },
        ];

        for repo in repositories {
            match repo_repo.create_repository(&repo).await {
                Ok(_) => {
                    debug!("Created repository: {}", repo.name);
                }
                Err(e) => {
                    if e.to_string().contains("duplicate key value") {
                        debug!("Repository {} already exists, skipping", repo.name);
                    } else {
                        error!("Failed to create repository {}: {:?}", repo.name, e);
                        return Err(e);
                    }
                }
            }
        }

        info!("Repositories seeding completed");
        Ok(())
    }
}
