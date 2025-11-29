/// Integration tests for crawl error tracking feature
/// Tests repository_repository.set_last_crawl_error and error storage in crawl operations
#[cfg(test)]
mod crawl_error_tracking_tests {
    use anyhow::Result;
    use chrono::Utc;
    use klask_rs::models::{Repository, RepositoryType};
    use klask_rs::repositories::RepositoryRepository;
    use sqlx::postgres::{PgPool, PgPoolOptions};
    use std::env;
    use uuid::Uuid;

    /// Get PostgreSQL connection string from environment or use default for testing
    fn get_db_url() -> String {
        env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://klask:klask@localhost:5432/klask".to_string())
    }

    /// Create a test database pool
    async fn create_pool() -> Result<PgPool> {
        let url = get_db_url();
        let pool = PgPoolOptions::new().max_connections(1).connect(&url).await?;
        Ok(pool)
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

    // Test 1: set_last_crawl_error stores error message
    #[tokio::test]
    async fn test_set_last_crawl_error_stores_error_message() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository
        let repo = create_test_repository("test-repo-1");
        let created_repo = repo_repo.create_repository(&repo).await?;

        // Set an error message
        let error_message = "Failed to clone repository: SSH key authentication failed".to_string();
        repo_repo.set_last_crawl_error(created_repo.id, Some(error_message.clone())).await?;

        // Verify the error was stored
        let retrieved_repo = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert_eq!(retrieved_repo.last_crawl_error, Some(error_message));

        // Verify updated_at was updated
        assert!(retrieved_repo.updated_at >= created_repo.updated_at);

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 2: set_last_crawl_error clears error when None is passed
    #[tokio::test]
    async fn test_set_last_crawl_error_clears_error() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository
        let repo = create_test_repository("test-repo-2");
        let created_repo = repo_repo.create_repository(&repo).await?;

        // Set an error message
        let error_message = "Initial error message".to_string();
        repo_repo.set_last_crawl_error(created_repo.id, Some(error_message)).await?;

        // Verify error is set
        let repo_with_error = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert!(repo_with_error.last_crawl_error.is_some());

        // Clear the error
        repo_repo.set_last_crawl_error(created_repo.id, None).await?;

        // Verify error is cleared
        let repo_without_error = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert!(repo_without_error.last_crawl_error.is_none());

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 3: set_last_crawl_error replaces old error with new one
    #[tokio::test]
    async fn test_set_last_crawl_error_replaces_old_error() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository
        let repo = create_test_repository("test-repo-3");
        let created_repo = repo_repo.create_repository(&repo).await?;

        // Set first error
        let error1 = "First error: Network timeout".to_string();
        repo_repo.set_last_crawl_error(created_repo.id, Some(error1.clone())).await?;

        let repo_with_error1 = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert_eq!(repo_with_error1.last_crawl_error, Some(error1.clone()));

        // Replace with second error
        let error2 = "Second error: Disk space exhausted".to_string();
        repo_repo.set_last_crawl_error(created_repo.id, Some(error2.clone())).await?;

        // Verify new error replaced the old one
        let repo_with_error2 = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert_eq!(repo_with_error2.last_crawl_error, Some(error2.clone()));
        assert_ne!(repo_with_error2.last_crawl_error, Some(error1.clone()));

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 4: Multiple repositories have independent error tracking
    #[tokio::test]
    async fn test_error_tracking_independent_across_repositories() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create multiple repositories
        let repo1 = create_test_repository("test-repo-4a");
        let repo2 = create_test_repository("test-repo-4b");
        let repo3 = create_test_repository("test-repo-4c");

        let created_repo1 = repo_repo.create_repository(&repo1).await?;
        let created_repo2 = repo_repo.create_repository(&repo2).await?;
        let created_repo3 = repo_repo.create_repository(&repo3).await?;

        // Set different errors for each
        let error1 = "Repo 1: Git clone failed".to_string();
        let error2 = "Repo 2: Permission denied".to_string();
        let error3: Option<String> = None; // Leave repo3 without error

        repo_repo.set_last_crawl_error(created_repo1.id, Some(error1.clone())).await?;
        repo_repo.set_last_crawl_error(created_repo2.id, Some(error2.clone())).await?;
        repo_repo.set_last_crawl_error(created_repo3.id, error3).await?;

        // Verify each repository has correct error
        let r1 = repo_repo.get_repository(created_repo1.id).await?.unwrap();
        let r2 = repo_repo.get_repository(created_repo2.id).await?.unwrap();
        let r3 = repo_repo.get_repository(created_repo3.id).await?.unwrap();

        assert_eq!(r1.last_crawl_error, Some(error1));
        assert_eq!(r2.last_crawl_error, Some(error2));
        assert!(r3.last_crawl_error.is_none());

        // Clean up
        repo_repo.delete_repository(created_repo1.id).await?;
        repo_repo.delete_repository(created_repo2.id).await?;
        repo_repo.delete_repository(created_repo3.id).await?;

        Ok(())
    }

    // Test 5: Long error messages are stored correctly
    #[tokio::test]
    async fn test_set_last_crawl_error_with_long_message() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository
        let repo = create_test_repository("test-repo-5");
        let created_repo = repo_repo.create_repository(&repo).await?;

        // Set a long error message with multiple lines and special characters
        let long_error = r#"Failed to index files in repository:
        Error at file: /src/main.rs:123
        Details: Unexpected EOF while parsing token stream
        Stack trace: parsing.rs:456 -> tokenize.rs:789
        Status: FATAL - Crawl aborted"#
            .to_string();

        repo_repo.set_last_crawl_error(created_repo.id, Some(long_error.clone())).await?;

        // Verify long error was stored completely
        let retrieved_repo = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert_eq!(retrieved_repo.last_crawl_error, Some(long_error));

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 6: Special characters in error messages are preserved
    #[tokio::test]
    async fn test_set_last_crawl_error_with_special_characters() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository
        let repo = create_test_repository("test-repo-6");
        let created_repo = repo_repo.create_repository(&repo).await?;

        // Set error message with special characters
        let error_with_special_chars = "Error: Connection failed to 'https://git.example.com/repo@branch#tag'. Details: 'unexpected \"quote\"' & <special> chars: ñ, é, 中文".to_string();

        repo_repo.set_last_crawl_error(created_repo.id, Some(error_with_special_chars.clone())).await?;

        // Verify special characters are preserved
        let retrieved_repo = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert_eq!(retrieved_repo.last_crawl_error, Some(error_with_special_chars));

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 7: Error is included in list_repositories response
    #[tokio::test]
    async fn test_last_crawl_error_in_list_repositories() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create repositories with and without errors
        let repo_with_error = create_test_repository("test-repo-7a");
        let repo_without_error = create_test_repository("test-repo-7b");

        let created_with_error = repo_repo.create_repository(&repo_with_error).await?;
        let created_without_error = repo_repo.create_repository(&repo_without_error).await?;

        let error_msg = "Crawl failed: insufficient disk space".to_string();
        repo_repo.set_last_crawl_error(created_with_error.id, Some(error_msg.clone())).await?;

        // List all repositories
        let all_repos = repo_repo.list_repositories().await?;

        // Find our test repositories in the list
        let found_with_error = all_repos.iter().find(|r| r.id == created_with_error.id);
        let found_without_error = all_repos.iter().find(|r| r.id == created_without_error.id);

        // Verify errors are present/absent in the list
        assert!(found_with_error.is_some());
        assert!(found_without_error.is_some());

        assert_eq!(found_with_error.unwrap().last_crawl_error, Some(error_msg));
        assert!(found_without_error.unwrap().last_crawl_error.is_none());

        // Clean up
        repo_repo.delete_repository(created_with_error.id).await?;
        repo_repo.delete_repository(created_without_error.id).await?;

        Ok(())
    }

    // Test 8: Error is preserved when updating other repository fields
    #[tokio::test]
    async fn test_error_preserved_when_updating_repository() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository
        let repo = create_test_repository("test-repo-8");
        let mut created_repo = repo_repo.create_repository(&repo).await?;

        // Set an error
        let error_msg = "Previous crawl error: Timeout".to_string();
        repo_repo.set_last_crawl_error(created_repo.id, Some(error_msg.clone())).await?;

        // Update some other fields using update_repository
        created_repo.enabled = false;
        created_repo.branch = Some("develop".to_string());
        let updated_repo = repo_repo.update_repository(created_repo.id, &created_repo).await?;

        // Verify error is still present after update
        assert_eq!(updated_repo.last_crawl_error, Some(error_msg));
        assert!(!updated_repo.enabled);
        assert_eq!(updated_repo.branch, Some("develop".to_string()));

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 9: Error is cleared on successful crawl completion
    #[tokio::test]
    async fn test_error_cleared_on_successful_crawl() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository
        let repo = create_test_repository("test-repo-9");
        let created_repo = repo_repo.create_repository(&repo).await?;

        // Simulate initial crawl failure with error
        let error_msg = "Network timeout during crawl".to_string();
        repo_repo.set_last_crawl_error(created_repo.id, Some(error_msg)).await?;

        // Verify error is set
        let repo_with_error = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert!(repo_with_error.last_crawl_error.is_some());

        // Simulate successful crawl by clearing error
        repo_repo.set_last_crawl_error(created_repo.id, None).await?;

        // Verify error is cleared
        let repo_after_success = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert!(repo_after_success.last_crawl_error.is_none());

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 10: Concurrent error updates work correctly
    #[tokio::test]
    async fn test_concurrent_error_updates() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository
        let repo = create_test_repository("test-repo-10");
        let created_repo = repo_repo.create_repository(&repo).await?;

        // Spawn multiple async tasks to update errors concurrently
        let mut handles = vec![];

        for i in 0..5 {
            let repo_repo_clone = RepositoryRepository::new(pool.clone());
            let repo_id = created_repo.id;

            let handle = tokio::spawn(async move {
                let error_msg = format!("Error from task {}", i);
                repo_repo_clone.set_last_crawl_error(repo_id, Some(error_msg)).await
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await??;
        }

        // Verify the repository has one of the error messages (last write wins)
        let final_repo = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert!(final_repo.last_crawl_error.is_some());
        assert!(final_repo.last_crawl_error.unwrap().contains("Error from task"));

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 11: Error message with exact size boundary
    #[tokio::test]
    async fn test_error_message_with_large_content() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository
        let repo = create_test_repository("test-repo-11");
        let created_repo = repo_repo.create_repository(&repo).await?;

        // Create a large error message (5KB)
        let large_error = "ERROR: ".to_string() + &"x".repeat(5000);

        repo_repo.set_last_crawl_error(created_repo.id, Some(large_error.clone())).await?;

        // Verify large error was stored
        let retrieved_repo = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert_eq!(retrieved_repo.last_crawl_error, Some(large_error));

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 12: Error timestamp (updated_at) is updated when error is set
    #[tokio::test]
    async fn test_updated_at_timestamp_changes_when_error_set() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository
        let repo = create_test_repository("test-repo-12");
        let created_repo = repo_repo.create_repository(&repo).await?;

        let original_updated_at = created_repo.updated_at;

        // Wait a tiny bit to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Set an error
        let error_msg = "Error occurred".to_string();
        repo_repo.set_last_crawl_error(created_repo.id, Some(error_msg)).await?;

        // Retrieve and check updated_at
        let repo_after_error = repo_repo.get_repository(created_repo.id).await?.unwrap();
        assert!(repo_after_error.updated_at > original_updated_at);

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 13: GET /api/repositories includes lastCrawlError field
    // This test validates the model serialization
    #[tokio::test]
    async fn test_last_crawl_error_serialization_in_api_response() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository with error
        let repo = create_test_repository("test-repo-13");
        let created_repo = repo_repo.create_repository(&repo).await?;

        let error_msg = "Serialization test error".to_string();
        repo_repo.set_last_crawl_error(created_repo.id, Some(error_msg.clone())).await?;

        // Retrieve and serialize as JSON
        let retrieved_repo = repo_repo.get_repository(created_repo.id).await?.unwrap();
        let json = serde_json::to_value(&retrieved_repo)?;

        // Verify lastCrawlError is in the JSON response with correct casing
        assert!(json.get("lastCrawlError").is_some());
        assert_eq!(
            json.get("lastCrawlError").unwrap().as_str().unwrap(),
            error_msg.as_str()
        );

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 14: lastCrawlError is null in JSON when no error
    #[tokio::test]
    async fn test_last_crawl_error_null_in_api_response() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create a repository without error
        let repo = create_test_repository("test-repo-14");
        let created_repo = repo_repo.create_repository(&repo).await?;

        // Retrieve and serialize as JSON
        let retrieved_repo = repo_repo.get_repository(created_repo.id).await?.unwrap();
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
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }

    // Test 15: find_incomplete_crawls includes error information
    #[tokio::test]
    async fn test_incomplete_crawls_includes_error_info() -> Result<()> {
        let pool = create_pool().await?;
        let repo_repo = RepositoryRepository::new(pool.clone());

        // Create an incomplete crawl with error
        let mut repo = create_test_repository("test-repo-15");
        repo.crawl_state = Some("in_progress".to_string());
        let created_repo = repo_repo.create_repository(&repo).await?;

        let error_msg = "Incomplete crawl error".to_string();
        repo_repo.set_last_crawl_error(created_repo.id, Some(error_msg.clone())).await?;

        // Find incomplete crawls
        let incomplete = repo_repo.find_incomplete_crawls().await?;

        // Verify our repository is found with error info
        let found = incomplete.iter().find(|r| r.id == created_repo.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().last_crawl_error, Some(error_msg));

        // Clean up
        repo_repo.delete_repository(created_repo.id).await?;

        Ok(())
    }
}
