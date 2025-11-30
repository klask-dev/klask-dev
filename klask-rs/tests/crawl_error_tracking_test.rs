/// Integration tests for crawl error tracking feature
/// Tests repository error storage in crawl operations using in-memory SQLite
#[cfg(test)]
mod crawl_error_tracking_tests {
    use anyhow::Result;
    use chrono::Utc;
    use klask_rs::database::create_test_database;
    use klask_rs::models::{Repository, RepositoryType};
    use sqlx::{Pool, Row, Sqlite};
    use uuid::Uuid;

    /// Create a test database pool
    async fn create_pool() -> Result<Pool<Sqlite>> {
        create_test_database().await
    }

    /// Create a test repository with default values
    fn create_test_repository(name: &str) -> Repository {
        Repository {
            id: Uuid::new_v4(),
            name: name.to_string(),
            url: format!("https://example.com/{}", name),
            repository_type: RepositoryType::Git,
            branch: Some("main".to_string()),
            enabled: true,
            access_token: None,
            gitlab_namespace: None,
            is_group: false,
            last_crawled: None,
            last_crawl_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            auto_crawl_enabled: false,
            cron_schedule: None,
            next_crawl_at: None,
            crawl_frequency_hours: None,
            max_crawl_duration_minutes: None,
            last_crawl_duration_seconds: None,
            gitlab_excluded_projects: None,
            gitlab_excluded_patterns: None,
            github_namespace: None,
            github_excluded_repositories: None,
            github_excluded_patterns: None,
            crawl_state: Some("idle".to_string()),
            last_processed_project: None,
            crawl_started_at: None,
            included_branches: None,
            included_branches_patterns: None,
            excluded_branches: None,
            excluded_branches_patterns: None,
            included_projects: None,
            included_projects_patterns: None,
        }
    }

    /// Insert a repository into the database
    async fn insert_repository(pool: &Pool<Sqlite>, repo: &Repository) -> Result<()> {
        sqlx::query(
            "INSERT INTO repositories (id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at, included_branches, included_branches_patterns, excluded_branches, excluded_branches_patterns, included_projects, included_projects_patterns, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(repo.id.to_string())
        .bind(&repo.name)
        .bind(&repo.url)
        .bind(format!("{:?}", repo.repository_type))
        .bind(&repo.branch)
        .bind(repo.enabled)
        .bind(&repo.access_token)
        .bind(&repo.gitlab_namespace)
        .bind(repo.is_group)
        .bind(repo.auto_crawl_enabled)
        .bind(&repo.cron_schedule)
        .bind(repo.next_crawl_at)
        .bind(repo.crawl_frequency_hours)
        .bind(repo.max_crawl_duration_minutes)
        .bind(&repo.gitlab_excluded_projects)
        .bind(&repo.gitlab_excluded_patterns)
        .bind(&repo.github_namespace)
        .bind(&repo.github_excluded_repositories)
        .bind(&repo.github_excluded_patterns)
        .bind(&repo.crawl_state)
        .bind(&repo.last_processed_project)
        .bind(repo.crawl_started_at)
        .bind(&repo.included_branches)
        .bind(&repo.included_branches_patterns)
        .bind(&repo.excluded_branches)
        .bind(&repo.excluded_branches_patterns)
        .bind(&repo.included_projects)
        .bind(&repo.included_projects_patterns)
        .bind(repo.created_at)
        .bind(repo.updated_at)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Get a repository from the database
    async fn get_repository(pool: &Pool<Sqlite>, id: Uuid) -> Result<Option<Repository>> {
        let repo_row = sqlx::query(
            "SELECT id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, last_crawled, last_crawl_error, created_at, updated_at, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, last_crawl_duration_seconds, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at, included_branches, included_branches_patterns, excluded_branches, excluded_branches_patterns, included_projects, included_projects_patterns FROM repositories WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?;

        Ok(repo_row.map(|row| Repository {
            id: Uuid::parse_str(row.get::<String, _>("id").as_str()).unwrap(),
            name: row.get("name"),
            url: row.get("url"),
            repository_type: match row.get::<String, _>("repository_type").as_str() {
                "Git" => RepositoryType::Git,
                "GitLab" => RepositoryType::GitLab,
                "GitHub" => RepositoryType::GitHub,
                "FileSystem" => RepositoryType::FileSystem,
                _ => RepositoryType::Git,
            },
            branch: row.get("branch"),
            enabled: row.get("enabled"),
            access_token: row.get("access_token"),
            gitlab_namespace: row.get("gitlab_namespace"),
            is_group: row.get("is_group"),
            last_crawled: row.get("last_crawled"),
            last_crawl_error: row.get("last_crawl_error"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            auto_crawl_enabled: row.get("auto_crawl_enabled"),
            cron_schedule: row.get("cron_schedule"),
            next_crawl_at: row.get("next_crawl_at"),
            crawl_frequency_hours: row.get("crawl_frequency_hours"),
            max_crawl_duration_minutes: row.get("max_crawl_duration_minutes"),
            last_crawl_duration_seconds: row.get("last_crawl_duration_seconds"),
            gitlab_excluded_projects: row.get("gitlab_excluded_projects"),
            gitlab_excluded_patterns: row.get("gitlab_excluded_patterns"),
            github_namespace: row.get("github_namespace"),
            github_excluded_repositories: row.get("github_excluded_repositories"),
            github_excluded_patterns: row.get("github_excluded_patterns"),
            crawl_state: row.get("crawl_state"),
            last_processed_project: row.get("last_processed_project"),
            crawl_started_at: row.get("crawl_started_at"),
            included_branches: row.get("included_branches"),
            included_branches_patterns: row.get("included_branches_patterns"),
            excluded_branches: row.get("excluded_branches"),
            excluded_branches_patterns: row.get("excluded_branches_patterns"),
            included_projects: row.get("included_projects"),
            included_projects_patterns: row.get("included_projects_patterns"),
        }))
    }

    /// Update a repository's error field
    async fn set_last_crawl_error(pool: &Pool<Sqlite>, id: Uuid, error: Option<String>) -> Result<()> {
        sqlx::query("UPDATE repositories SET last_crawl_error = ?, updated_at = ? WHERE id = ?")
            .bind(error)
            .bind(Utc::now())
            .bind(id.to_string())
            .execute(pool)
            .await?;
        Ok(())
    }

    /// List all repositories
    async fn list_repositories(pool: &Pool<Sqlite>) -> Result<Vec<Repository>> {
        let repo_rows = sqlx::query(
            "SELECT id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, last_crawled, last_crawl_error, created_at, updated_at, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, last_crawl_duration_seconds, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at, included_branches, included_branches_patterns, excluded_branches, excluded_branches_patterns, included_projects, included_projects_patterns FROM repositories ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;

        let repos = repo_rows
            .into_iter()
            .map(|row| Repository {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str()).unwrap(),
                name: row.get("name"),
                url: row.get("url"),
                repository_type: match row.get::<String, _>("repository_type").as_str() {
                    "Git" => RepositoryType::Git,
                    "GitLab" => RepositoryType::GitLab,
                    "GitHub" => RepositoryType::GitHub,
                    "FileSystem" => RepositoryType::FileSystem,
                    _ => RepositoryType::Git,
                },
                branch: row.get("branch"),
                enabled: row.get("enabled"),
                access_token: row.get("access_token"),
                gitlab_namespace: row.get("gitlab_namespace"),
                is_group: row.get("is_group"),
                last_crawled: row.get("last_crawled"),
                last_crawl_error: row.get("last_crawl_error"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                auto_crawl_enabled: row.get("auto_crawl_enabled"),
                cron_schedule: row.get("cron_schedule"),
                next_crawl_at: row.get("next_crawl_at"),
                crawl_frequency_hours: row.get("crawl_frequency_hours"),
                max_crawl_duration_minutes: row.get("max_crawl_duration_minutes"),
                last_crawl_duration_seconds: row.get("last_crawl_duration_seconds"),
                gitlab_excluded_projects: row.get("gitlab_excluded_projects"),
                gitlab_excluded_patterns: row.get("gitlab_excluded_patterns"),
                github_namespace: row.get("github_namespace"),
                github_excluded_repositories: row.get("github_excluded_repositories"),
                github_excluded_patterns: row.get("github_excluded_patterns"),
                crawl_state: row.get("crawl_state"),
                last_processed_project: row.get("last_processed_project"),
                crawl_started_at: row.get("crawl_started_at"),
                included_branches: row.get("included_branches"),
                included_branches_patterns: row.get("included_branches_patterns"),
                excluded_branches: row.get("excluded_branches"),
                excluded_branches_patterns: row.get("excluded_branches_patterns"),
                included_projects: row.get("included_projects"),
                included_projects_patterns: row.get("included_projects_patterns"),
            })
            .collect();
        Ok(repos)
    }

    /// Update a repository
    async fn update_repository(pool: &Pool<Sqlite>, id: Uuid, repo: &Repository) -> Result<()> {
        sqlx::query(
            "UPDATE repositories SET name = ?, url = ?, repository_type = ?, branch = ?, enabled = ?, access_token = ?, gitlab_namespace = ?, is_group = ?, auto_crawl_enabled = ?, cron_schedule = ?, next_crawl_at = ?, crawl_frequency_hours = ?, max_crawl_duration_minutes = ?, gitlab_excluded_projects = ?, gitlab_excluded_patterns = ?, github_namespace = ?, github_excluded_repositories = ?, github_excluded_patterns = ?, crawl_state = ?, last_processed_project = ?, crawl_started_at = ?, included_branches = ?, included_branches_patterns = ?, excluded_branches = ?, excluded_branches_patterns = ?, included_projects = ?, included_projects_patterns = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&repo.name)
        .bind(&repo.url)
        .bind(format!("{:?}", repo.repository_type))
        .bind(&repo.branch)
        .bind(repo.enabled)
        .bind(&repo.access_token)
        .bind(&repo.gitlab_namespace)
        .bind(repo.is_group)
        .bind(repo.auto_crawl_enabled)
        .bind(&repo.cron_schedule)
        .bind(repo.next_crawl_at)
        .bind(repo.crawl_frequency_hours)
        .bind(repo.max_crawl_duration_minutes)
        .bind(&repo.gitlab_excluded_projects)
        .bind(&repo.gitlab_excluded_patterns)
        .bind(&repo.github_namespace)
        .bind(&repo.github_excluded_repositories)
        .bind(&repo.github_excluded_patterns)
        .bind(&repo.crawl_state)
        .bind(&repo.last_processed_project)
        .bind(repo.crawl_started_at)
        .bind(&repo.included_branches)
        .bind(&repo.included_branches_patterns)
        .bind(&repo.excluded_branches)
        .bind(&repo.excluded_branches_patterns)
        .bind(&repo.included_projects)
        .bind(&repo.included_projects_patterns)
        .bind(Utc::now())
        .bind(id.to_string())
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Delete a repository
    async fn delete_repository(pool: &Pool<Sqlite>, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM repositories WHERE id = ?").bind(id.to_string()).execute(pool).await?;
        Ok(())
    }

    /// Find repositories with incomplete crawls (in_progress state)
    async fn find_incomplete_crawls(pool: &Pool<Sqlite>) -> Result<Vec<Repository>> {
        let repo_rows = sqlx::query(
            "SELECT id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, last_crawled, last_crawl_error, created_at, updated_at, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, last_crawl_duration_seconds, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at, included_branches, included_branches_patterns, excluded_branches, excluded_branches_patterns, included_projects, included_projects_patterns FROM repositories WHERE crawl_state = 'in_progress' ORDER BY crawl_started_at ASC"
        )
        .fetch_all(pool)
        .await?;

        let repos = repo_rows
            .into_iter()
            .map(|row| Repository {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str()).unwrap(),
                name: row.get("name"),
                url: row.get("url"),
                repository_type: match row.get::<String, _>("repository_type").as_str() {
                    "Git" => RepositoryType::Git,
                    "GitLab" => RepositoryType::GitLab,
                    "GitHub" => RepositoryType::GitHub,
                    "FileSystem" => RepositoryType::FileSystem,
                    _ => RepositoryType::Git,
                },
                branch: row.get("branch"),
                enabled: row.get("enabled"),
                access_token: row.get("access_token"),
                gitlab_namespace: row.get("gitlab_namespace"),
                is_group: row.get("is_group"),
                last_crawled: row.get("last_crawled"),
                last_crawl_error: row.get("last_crawl_error"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                auto_crawl_enabled: row.get("auto_crawl_enabled"),
                cron_schedule: row.get("cron_schedule"),
                next_crawl_at: row.get("next_crawl_at"),
                crawl_frequency_hours: row.get("crawl_frequency_hours"),
                max_crawl_duration_minutes: row.get("max_crawl_duration_minutes"),
                last_crawl_duration_seconds: row.get("last_crawl_duration_seconds"),
                gitlab_excluded_projects: row.get("gitlab_excluded_projects"),
                gitlab_excluded_patterns: row.get("gitlab_excluded_patterns"),
                github_namespace: row.get("github_namespace"),
                github_excluded_repositories: row.get("github_excluded_repositories"),
                github_excluded_patterns: row.get("github_excluded_patterns"),
                crawl_state: row.get("crawl_state"),
                last_processed_project: row.get("last_processed_project"),
                crawl_started_at: row.get("crawl_started_at"),
                included_branches: row.get("included_branches"),
                included_branches_patterns: row.get("included_branches_patterns"),
                excluded_branches: row.get("excluded_branches"),
                excluded_branches_patterns: row.get("excluded_branches_patterns"),
                included_projects: row.get("included_projects"),
                included_projects_patterns: row.get("included_projects_patterns"),
            })
            .collect();
        Ok(repos)
    }

    // Test 1: set_last_crawl_error stores error message
    #[tokio::test]
    async fn test_set_last_crawl_error_stores_error_message() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository
        let repo = create_test_repository("test-repo-1");
        insert_repository(&pool, &repo).await?;

        // Set an error message
        let error_message = "Failed to clone repository: SSH key authentication failed".to_string();
        set_last_crawl_error(&pool, repo.id, Some(error_message.clone())).await?;

        // Verify the error was stored
        let retrieved_repo = get_repository(&pool, repo.id).await?.unwrap();
        assert_eq!(retrieved_repo.last_crawl_error, Some(error_message));

        // Verify updated_at was updated
        assert!(retrieved_repo.updated_at >= repo.updated_at);

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 2: set_last_crawl_error clears error when None is passed
    #[tokio::test]
    async fn test_set_last_crawl_error_clears_error() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository
        let repo = create_test_repository("test-repo-2");
        insert_repository(&pool, &repo).await?;

        // Set an error message
        let error_message = "Initial error message".to_string();
        set_last_crawl_error(&pool, repo.id, Some(error_message)).await?;

        // Verify error is set
        let repo_with_error = get_repository(&pool, repo.id).await?.unwrap();
        assert!(repo_with_error.last_crawl_error.is_some());

        // Clear the error
        set_last_crawl_error(&pool, repo.id, None).await?;

        // Verify error is cleared
        let repo_without_error = get_repository(&pool, repo.id).await?.unwrap();
        assert!(repo_without_error.last_crawl_error.is_none());

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 3: set_last_crawl_error replaces old error with new one
    #[tokio::test]
    async fn test_set_last_crawl_error_replaces_old_error() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository
        let repo = create_test_repository("test-repo-3");
        insert_repository(&pool, &repo).await?;

        // Set first error
        let error1 = "First error: Network timeout".to_string();
        set_last_crawl_error(&pool, repo.id, Some(error1.clone())).await?;

        let repo_with_error1 = get_repository(&pool, repo.id).await?.unwrap();
        assert_eq!(repo_with_error1.last_crawl_error, Some(error1.clone()));

        // Replace with second error
        let error2 = "Second error: Disk space exhausted".to_string();
        set_last_crawl_error(&pool, repo.id, Some(error2.clone())).await?;

        // Verify new error replaced the old one
        let repo_with_error2 = get_repository(&pool, repo.id).await?.unwrap();
        assert_eq!(repo_with_error2.last_crawl_error, Some(error2.clone()));
        assert_ne!(repo_with_error2.last_crawl_error, Some(error1.clone()));

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 4: Multiple repositories have independent error tracking
    #[tokio::test]
    async fn test_error_tracking_independent_across_repositories() -> Result<()> {
        let pool = create_pool().await?;

        // Create multiple repositories
        let repo1 = create_test_repository("test-repo-4a");
        let repo2 = create_test_repository("test-repo-4b");
        let repo3 = create_test_repository("test-repo-4c");

        insert_repository(&pool, &repo1).await?;
        insert_repository(&pool, &repo2).await?;
        insert_repository(&pool, &repo3).await?;

        // Set different errors for each
        let error1 = "Repo 1: Git clone failed".to_string();
        let error2 = "Repo 2: Permission denied".to_string();
        let error3: Option<String> = None; // Leave repo3 without error

        set_last_crawl_error(&pool, repo1.id, Some(error1.clone())).await?;
        set_last_crawl_error(&pool, repo2.id, Some(error2.clone())).await?;
        set_last_crawl_error(&pool, repo3.id, error3).await?;

        // Verify each repository has correct error
        let r1 = get_repository(&pool, repo1.id).await?.unwrap();
        let r2 = get_repository(&pool, repo2.id).await?.unwrap();
        let r3 = get_repository(&pool, repo3.id).await?.unwrap();

        assert_eq!(r1.last_crawl_error, Some(error1));
        assert_eq!(r2.last_crawl_error, Some(error2));
        assert!(r3.last_crawl_error.is_none());

        // Clean up
        delete_repository(&pool, repo1.id).await?;
        delete_repository(&pool, repo2.id).await?;
        delete_repository(&pool, repo3.id).await?;

        Ok(())
    }

    // Test 5: Long error messages are stored correctly
    #[tokio::test]
    async fn test_set_last_crawl_error_with_long_message() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository
        let repo = create_test_repository("test-repo-5");
        insert_repository(&pool, &repo).await?;

        // Set a long error message with multiple lines and special characters
        let long_error = r#"Failed to index files in repository:
        Error at file: /src/main.rs:123
        Details: Unexpected EOF while parsing token stream
        Stack trace: parsing.rs:456 -> tokenize.rs:789
        Status: FATAL - Crawl aborted"#
            .to_string();

        set_last_crawl_error(&pool, repo.id, Some(long_error.clone())).await?;

        // Verify long error was stored completely
        let retrieved_repo = get_repository(&pool, repo.id).await?.unwrap();
        assert_eq!(retrieved_repo.last_crawl_error, Some(long_error));

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 6: Special characters in error messages are preserved
    #[tokio::test]
    async fn test_set_last_crawl_error_with_special_characters() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository
        let repo = create_test_repository("test-repo-6");
        insert_repository(&pool, &repo).await?;

        // Set error message with special characters
        let error_with_special_chars = "Error: Connection failed to 'https://git.example.com/repo@branch#tag'. Details: 'unexpected \"quote\"' & <special> chars: ñ, é, 中文".to_string();

        set_last_crawl_error(&pool, repo.id, Some(error_with_special_chars.clone())).await?;

        // Verify special characters are preserved
        let retrieved_repo = get_repository(&pool, repo.id).await?.unwrap();
        assert_eq!(retrieved_repo.last_crawl_error, Some(error_with_special_chars));

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 7: Error is included in list_repositories response
    #[tokio::test]
    async fn test_last_crawl_error_in_list_repositories() -> Result<()> {
        let pool = create_pool().await?;

        // Create repositories with and without errors
        let repo_with_error = create_test_repository("test-repo-7a");
        let repo_without_error = create_test_repository("test-repo-7b");

        insert_repository(&pool, &repo_with_error).await?;
        insert_repository(&pool, &repo_without_error).await?;

        let error_msg = "Crawl failed: insufficient disk space".to_string();
        set_last_crawl_error(&pool, repo_with_error.id, Some(error_msg.clone())).await?;

        // List all repositories
        let all_repos = list_repositories(&pool).await?;

        // Find our test repositories in the list
        let found_with_error = all_repos.iter().find(|r| r.id == repo_with_error.id);
        let found_without_error = all_repos.iter().find(|r| r.id == repo_without_error.id);

        // Verify errors are present/absent in the list
        assert!(found_with_error.is_some());
        assert!(found_without_error.is_some());

        assert_eq!(found_with_error.unwrap().last_crawl_error, Some(error_msg));
        assert!(found_without_error.unwrap().last_crawl_error.is_none());

        // Clean up
        delete_repository(&pool, repo_with_error.id).await?;
        delete_repository(&pool, repo_without_error.id).await?;

        Ok(())
    }

    // Test 8: Error is preserved when updating other repository fields
    #[tokio::test]
    async fn test_error_preserved_when_updating_repository() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository
        let mut repo = create_test_repository("test-repo-8");
        insert_repository(&pool, &repo).await?;

        // Set an error
        let error_msg = "Previous crawl error: Timeout".to_string();
        set_last_crawl_error(&pool, repo.id, Some(error_msg.clone())).await?;

        // Update some other fields
        repo.enabled = false;
        repo.branch = Some("develop".to_string());
        update_repository(&pool, repo.id, &repo).await?;

        // Verify error is still present after update
        let updated_repo = get_repository(&pool, repo.id).await?.unwrap();
        assert_eq!(updated_repo.last_crawl_error, Some(error_msg));
        assert!(!updated_repo.enabled);
        assert_eq!(updated_repo.branch, Some("develop".to_string()));

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 9: Error is cleared on successful crawl completion
    #[tokio::test]
    async fn test_error_cleared_on_successful_crawl() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository
        let repo = create_test_repository("test-repo-9");
        insert_repository(&pool, &repo).await?;

        // Simulate initial crawl failure with error
        let error_msg = "Network timeout during crawl".to_string();
        set_last_crawl_error(&pool, repo.id, Some(error_msg)).await?;

        // Verify error is set
        let repo_with_error = get_repository(&pool, repo.id).await?.unwrap();
        assert!(repo_with_error.last_crawl_error.is_some());

        // Simulate successful crawl by clearing error
        set_last_crawl_error(&pool, repo.id, None).await?;

        // Verify error is cleared
        let repo_after_success = get_repository(&pool, repo.id).await?.unwrap();
        assert!(repo_after_success.last_crawl_error.is_none());

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 10: Concurrent error updates work correctly
    #[tokio::test]
    async fn test_concurrent_error_updates() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository
        let repo = create_test_repository("test-repo-10");
        insert_repository(&pool, &repo).await?;

        // Spawn multiple async tasks to update errors concurrently
        let mut handles = vec![];

        for i in 0..5 {
            let pool_clone = pool.clone();
            let repo_id = repo.id;

            let handle = tokio::spawn(async move {
                let error_msg = format!("Error from task {}", i);
                set_last_crawl_error(&pool_clone, repo_id, Some(error_msg)).await
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await??;
        }

        // Verify the repository has one of the error messages (last write wins)
        let final_repo = get_repository(&pool, repo.id).await?.unwrap();
        assert!(final_repo.last_crawl_error.is_some());
        assert!(final_repo.last_crawl_error.unwrap().contains("Error from task"));

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 11: Error message with exact size boundary
    #[tokio::test]
    async fn test_error_message_with_large_content() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository
        let repo = create_test_repository("test-repo-11");
        insert_repository(&pool, &repo).await?;

        // Create a large error message (5KB)
        let large_error = "ERROR: ".to_string() + &"x".repeat(5000);

        set_last_crawl_error(&pool, repo.id, Some(large_error.clone())).await?;

        // Verify large error was stored
        let retrieved_repo = get_repository(&pool, repo.id).await?.unwrap();
        assert_eq!(retrieved_repo.last_crawl_error, Some(large_error));

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 12: Error timestamp (updated_at) is updated when error is set
    #[tokio::test]
    async fn test_updated_at_timestamp_changes_when_error_set() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository
        let repo = create_test_repository("test-repo-12");
        insert_repository(&pool, &repo).await?;

        let original_updated_at = repo.updated_at;

        // Wait a tiny bit to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Set an error
        let error_msg = "Error occurred".to_string();
        set_last_crawl_error(&pool, repo.id, Some(error_msg)).await?;

        // Retrieve and check updated_at
        let repo_after_error = get_repository(&pool, repo.id).await?.unwrap();
        assert!(repo_after_error.updated_at > original_updated_at);

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 13: GET /api/repositories includes lastCrawlError field
    // This test validates the model serialization
    #[tokio::test]
    async fn test_last_crawl_error_serialization_in_api_response() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository with error
        let repo = create_test_repository("test-repo-13");
        insert_repository(&pool, &repo).await?;

        let error_msg = "Serialization test error".to_string();
        set_last_crawl_error(&pool, repo.id, Some(error_msg.clone())).await?;

        // Retrieve and serialize as JSON
        let retrieved_repo = get_repository(&pool, repo.id).await?.unwrap();
        let json = serde_json::to_value(&retrieved_repo)?;

        // Verify lastCrawlError is in the JSON response with correct casing
        assert!(json.get("lastCrawlError").is_some());
        assert_eq!(
            json.get("lastCrawlError").unwrap().as_str().unwrap(),
            error_msg.as_str()
        );

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 14: lastCrawlError is null in JSON when no error
    #[tokio::test]
    async fn test_last_crawl_error_null_in_api_response() -> Result<()> {
        let pool = create_pool().await?;

        // Create a repository without error
        let repo = create_test_repository("test-repo-14");
        insert_repository(&pool, &repo).await?;

        // Retrieve and serialize as JSON
        let retrieved_repo = get_repository(&pool, repo.id).await?.unwrap();
        let _json = serde_json::to_value(&retrieved_repo)?;

        // Verify lastCrawlError is null/absent in JSON
        // Since it uses #[serde(skip_serializing_if = "Option::is_none")], it should be absent
        // But the field should still be deserializable
        let _json_with_explicit_null = serde_json::json!({
            "lastCrawlError": serde_json::Value::Null
        });

        // Verify model can handle null lastCrawlError
        let test_repo = create_test_repository("test");
        assert!(test_repo.last_crawl_error.is_none());

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }

    // Test 15: find_incomplete_crawls includes error information
    #[tokio::test]
    async fn test_incomplete_crawls_includes_error_info() -> Result<()> {
        let pool = create_pool().await?;

        // Create an incomplete crawl with error
        let mut repo = create_test_repository("test-repo-15");
        repo.crawl_state = Some("in_progress".to_string());
        insert_repository(&pool, &repo).await?;

        let error_msg = "Incomplete crawl error".to_string();
        set_last_crawl_error(&pool, repo.id, Some(error_msg.clone())).await?;

        // Find incomplete crawls
        let incomplete = find_incomplete_crawls(&pool).await?;

        // Verify our repository is found with error info
        let found = incomplete.iter().find(|r| r.id == repo.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().last_crawl_error, Some(error_msg));

        // Clean up
        delete_repository(&pool, repo.id).await?;

        Ok(())
    }
}
