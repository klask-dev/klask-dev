#[cfg(test)]
mod crawl_prevention_tests {
    use klask_rs::services::progress::{CrawlStatus, ProgressTracker};
    use std::sync::Arc;

    use uuid::Uuid;

    #[tokio::test]
    async fn test_is_crawling_returns_false_for_new_tracker() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();

        let is_crawling = tracker.is_crawling(repo_id).await;
        assert!(!is_crawling);
    }

    #[tokio::test]
    async fn test_is_crawling_returns_true_for_active_crawl() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        // Start a crawl
        tracker.start_crawl(repo_id, repo_name).await;

        let is_crawling = tracker.is_crawling(repo_id).await;
        assert!(is_crawling);
    }

    #[tokio::test]
    async fn test_is_crawling_returns_true_for_processing_status() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;
        tracker.update_status(repo_id, CrawlStatus::Processing).await;

        let is_crawling = tracker.is_crawling(repo_id).await;
        assert!(is_crawling);
    }

    #[tokio::test]
    async fn test_is_crawling_returns_true_for_cloning_status() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;
        tracker.update_status(repo_id, CrawlStatus::Cloning).await;

        let is_crawling = tracker.is_crawling(repo_id).await;
        assert!(is_crawling);
    }

    #[tokio::test]
    async fn test_is_crawling_returns_true_for_indexing_status() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;
        tracker.update_status(repo_id, CrawlStatus::Indexing).await;

        let is_crawling = tracker.is_crawling(repo_id).await;
        assert!(is_crawling);
    }

    #[tokio::test]
    async fn test_is_crawling_returns_false_for_completed_status() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;
        tracker.update_status(repo_id, CrawlStatus::Completed).await;

        let is_crawling = tracker.is_crawling(repo_id).await;
        assert!(!is_crawling);
    }

    #[tokio::test]
    async fn test_is_crawling_returns_false_for_failed_status() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;
        tracker.set_error(repo_id, "Test error".to_string()).await;

        let is_crawling = tracker.is_crawling(repo_id).await;
        assert!(!is_crawling);
    }

    #[tokio::test]
    async fn test_is_crawling_returns_false_for_cancelled_status() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;
        tracker.cancel_crawl(repo_id).await;

        let is_crawling = tracker.is_crawling(repo_id).await;
        assert!(!is_crawling);
    }

    #[tokio::test]
    async fn test_is_crawling_returns_false_after_completion() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;
        assert!(tracker.is_crawling(repo_id).await);

        tracker.complete_crawl(repo_id).await;
        assert!(!tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_is_crawling_multiple_repositories() {
        let tracker = ProgressTracker::new();
        let repo_id1 = Uuid::new_v4();
        let repo_id2 = Uuid::new_v4();
        let repo_id3 = Uuid::new_v4();

        // Start crawls for multiple repositories
        tracker.start_crawl(repo_id1, "repo1".to_string()).await;
        tracker.start_crawl(repo_id2, "repo2".to_string()).await;

        assert!(tracker.is_crawling(repo_id1).await);
        assert!(tracker.is_crawling(repo_id2).await);
        assert!(!tracker.is_crawling(repo_id3).await); // Not started

        // Complete one crawl
        tracker.complete_crawl(repo_id1).await;

        assert!(!tracker.is_crawling(repo_id1).await);
        assert!(tracker.is_crawling(repo_id2).await);
        assert!(!tracker.is_crawling(repo_id3).await);
    }

    #[tokio::test]
    async fn test_is_crawling_race_condition_protection() {
        let tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();
        let repo_name = "race-test-repo".to_string();

        // Simulate concurrent checks
        let mut handles = vec![];

        for _ in 0..10 {
            let tracker_clone = Arc::clone(&tracker);
            let repo_id_clone = repo_id;
            let handle = tokio::spawn(async move { tracker_clone.is_crawling(repo_id_clone).await });
            handles.push(handle);
        }

        // Start crawl after spawning check tasks
        tracker.start_crawl(repo_id, repo_name).await;

        // Wait for all checks to complete
        let _results: Vec<bool> = futures::future::join_all(handles).await.into_iter().map(|r| r.unwrap()).collect();

        // Some checks might return false (if they ran before start_crawl)
        // But the final state should be that it's crawling
        assert!(tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_is_crawling_concurrent_status_updates() {
        let tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();
        let repo_name = "concurrent-test".to_string();

        tracker.start_crawl(repo_id, repo_name).await;

        // Spawn multiple tasks to update status concurrently
        let mut handles = vec![];
        let statuses = vec![
            CrawlStatus::Cloning,
            CrawlStatus::Processing,
            CrawlStatus::Indexing,
            CrawlStatus::Processing,
            CrawlStatus::Cloning,
        ];

        for status in statuses {
            let tracker_clone = Arc::clone(&tracker);
            let handle = tokio::spawn(async move {
                tracker_clone.update_status(repo_id, status).await;
                tracker_clone.is_crawling(repo_id).await
            });
            handles.push(handle);
        }

        // Wait for all updates to complete
        let results: Vec<bool> = futures::future::join_all(handles).await.into_iter().map(|r| r.unwrap()).collect();

        // All should return true since we're only updating to active statuses
        assert!(results.iter().all(|&x| x));
        assert!(tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_is_crawling_after_progress_removal() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;
        assert!(tracker.is_crawling(repo_id).await);

        tracker.remove_progress(repo_id).await;
        assert!(!tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_is_crawling_performance_with_many_repositories() {
        let tracker = ProgressTracker::new();
        let mut repo_ids = Vec::new();

        // Start many crawls
        for i in 0..1000 {
            let repo_id = Uuid::new_v4();
            tracker.start_crawl(repo_id, format!("repo-{}", i)).await;
            repo_ids.push(repo_id);
        }

        // Complete half of them
        for repo_id in repo_ids.iter().take(500) {
            tracker.complete_crawl(*repo_id).await;
        }

        // Check is_crawling for all repositories
        let start = std::time::Instant::now();
        for (i, &repo_id) in repo_ids.iter().enumerate() {
            let is_crawling = tracker.is_crawling(repo_id).await;
            if i < 500 {
                assert!(!is_crawling, "Repository {} should not be crawling", i);
            } else {
                assert!(is_crawling, "Repository {} should be crawling", i);
            }
        }
        let duration = start.elapsed();

        // This should complete reasonably quickly (adjust threshold as needed)
        assert!(
            duration.as_millis() < 1000,
            "is_crawling checks took too long: {:?}",
            duration
        );
    }

    #[tokio::test]
    async fn test_is_crawling_with_status_transitions() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "transition-test".to_string();

        // Test all active status transitions
        tracker.start_crawl(repo_id, repo_name).await;
        assert!(tracker.is_crawling(repo_id).await);

        tracker.update_status(repo_id, CrawlStatus::Cloning).await;
        assert!(tracker.is_crawling(repo_id).await);

        tracker.update_status(repo_id, CrawlStatus::Processing).await;
        assert!(tracker.is_crawling(repo_id).await);

        tracker.update_status(repo_id, CrawlStatus::Indexing).await;
        assert!(tracker.is_crawling(repo_id).await);

        // Test terminal status transitions
        tracker.update_status(repo_id, CrawlStatus::Completed).await;
        assert!(!tracker.is_crawling(repo_id).await);

        // Once completed, should remain non-crawling
        assert!(!tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_is_crawling_memory_safety() {
        let tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Spawn many concurrent tasks that start and check crawling status
        let mut handles = vec![];

        for i in 0..100 {
            let tracker_clone = Arc::clone(&tracker);
            let repo_id_clone = if i % 2 == 0 { repo_id } else { Uuid::new_v4() };
            let handle = tokio::spawn(async move {
                // Mix of operations to test memory safety
                if i % 3 == 0 {
                    tracker_clone.start_crawl(repo_id_clone, format!("repo-{}", i)).await;
                }

                let is_crawling = tracker_clone.is_crawling(repo_id_clone).await;

                if i % 5 == 0 {
                    tracker_clone.complete_crawl(repo_id_clone).await;
                }

                is_crawling
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete without panicking
        let _results: Vec<bool> = futures::future::join_all(handles).await.into_iter().map(|r| r.unwrap()).collect();

        // If we reach here without panicking, memory safety is maintained
        // Test passed if no panic occurred
    }
}

#[cfg(test)]
mod trigger_crawl_conflict_tests {
    use axum::http::StatusCode;
    use klask_rs::services::progress::{CrawlStatus, ProgressTracker};
    use std::sync::Arc;
    use uuid::Uuid;

    // Mock repository for testing
    #[derive(Clone, Debug)]
    struct MockRepository {
        pub id: Uuid,
        pub name: String,
        pub enabled: bool,
    }

    // Simulate the trigger_crawl logic for testing
    async fn simulate_trigger_crawl(
        progress_tracker: &ProgressTracker,
        repository: &MockRepository,
    ) -> Result<String, StatusCode> {
        // Check if repository is already being crawled
        if progress_tracker.is_crawling(repository.id).await {
            return Err(StatusCode::CONFLICT);
        }

        // Check if repository is enabled
        if !repository.enabled {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Double-check if repository is still not being crawled (race condition protection)
        if progress_tracker.is_crawling(repository.id).await {
            return Err(StatusCode::CONFLICT);
        }

        // Simulate starting the crawl
        progress_tracker.start_crawl(repository.id, repository.name.clone()).await;

        Ok("Crawl started in background".to_string())
    }

    #[tokio::test]
    async fn test_trigger_crawl_success_when_not_crawling() {
        let progress_tracker = ProgressTracker::new();
        let repository = MockRepository { id: Uuid::new_v4(), name: "test-repo".to_string(), enabled: true };

        let result = simulate_trigger_crawl(&progress_tracker, &repository).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Crawl started in background");
        assert!(progress_tracker.is_crawling(repository.id).await);
    }

    #[tokio::test]
    async fn test_trigger_crawl_conflict_when_already_crawling() {
        let progress_tracker = ProgressTracker::new();
        let repository = MockRepository { id: Uuid::new_v4(), name: "test-repo".to_string(), enabled: true };

        // Start initial crawl
        progress_tracker.start_crawl(repository.id, repository.name.clone()).await;

        // Try to trigger another crawl
        let result = simulate_trigger_crawl(&progress_tracker, &repository).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_trigger_crawl_conflict_when_processing() {
        let progress_tracker = ProgressTracker::new();
        let repository = MockRepository { id: Uuid::new_v4(), name: "test-repo".to_string(), enabled: true };

        // Start and update to processing
        progress_tracker.start_crawl(repository.id, repository.name.clone()).await;
        progress_tracker.update_status(repository.id, CrawlStatus::Processing).await;

        let result = simulate_trigger_crawl(&progress_tracker, &repository).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_trigger_crawl_success_after_completion() {
        let progress_tracker = ProgressTracker::new();
        let repository = MockRepository { id: Uuid::new_v4(), name: "test-repo".to_string(), enabled: true };

        // Start and complete a crawl
        progress_tracker.start_crawl(repository.id, repository.name.clone()).await;
        progress_tracker.complete_crawl(repository.id).await;

        // Should be able to trigger a new crawl
        let result = simulate_trigger_crawl(&progress_tracker, &repository).await;

        assert!(result.is_ok());
        assert!(progress_tracker.is_crawling(repository.id).await);
    }

    #[tokio::test]
    async fn test_trigger_crawl_success_after_failure() {
        let progress_tracker = ProgressTracker::new();
        let repository = MockRepository { id: Uuid::new_v4(), name: "test-repo".to_string(), enabled: true };

        // Start and fail a crawl
        progress_tracker.start_crawl(repository.id, repository.name.clone()).await;
        progress_tracker.set_error(repository.id, "Test error".to_string()).await;

        // Should be able to trigger a new crawl
        let result = simulate_trigger_crawl(&progress_tracker, &repository).await;

        assert!(result.is_ok());
        assert!(progress_tracker.is_crawling(repository.id).await);
    }

    #[tokio::test]
    async fn test_trigger_crawl_success_after_cancellation() {
        let progress_tracker = ProgressTracker::new();
        let repository = MockRepository { id: Uuid::new_v4(), name: "test-repo".to_string(), enabled: true };

        // Start and cancel a crawl
        progress_tracker.start_crawl(repository.id, repository.name.clone()).await;
        progress_tracker.cancel_crawl(repository.id).await;

        // Should be able to trigger a new crawl
        let result = simulate_trigger_crawl(&progress_tracker, &repository).await;

        assert!(result.is_ok());
        assert!(progress_tracker.is_crawling(repository.id).await);
    }

    #[tokio::test]
    async fn test_trigger_crawl_bad_request_when_disabled() {
        let progress_tracker = ProgressTracker::new();
        let repository = MockRepository {
            id: Uuid::new_v4(),
            name: "test-repo".to_string(),
            enabled: false, // Disabled repository
        };

        let result = simulate_trigger_crawl(&progress_tracker, &repository).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
        assert!(!progress_tracker.is_crawling(repository.id).await);
    }

    #[tokio::test]
    async fn test_trigger_crawl_race_condition_protection() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repository =
            Arc::new(MockRepository { id: Uuid::new_v4(), name: "race-test-repo".to_string(), enabled: true });

        // Spawn multiple concurrent trigger attempts
        let mut handles = vec![];

        for _ in 0..10 {
            let progress_tracker_clone = Arc::clone(&progress_tracker);
            let repository_clone = Arc::clone(&repository);
            let handle =
                tokio::spawn(async move { simulate_trigger_crawl(&progress_tracker_clone, &repository_clone).await });
            handles.push(handle);
        }

        // Wait for all attempts to complete
        let results: Vec<Result<String, StatusCode>> =
            futures::future::join_all(handles).await.into_iter().map(|r| r.unwrap()).collect();

        // Only one should succeed, others should get CONFLICT
        let successful = results.iter().filter(|r| r.is_ok()).count();
        let conflicts =
            results.iter().filter(|r| r.is_err() && r.as_ref().unwrap_err() == &StatusCode::CONFLICT).count();

        assert_eq!(successful, 1, "Exactly one crawl should succeed");
        assert_eq!(conflicts, 9, "Nine attempts should get CONFLICT");
        assert!(progress_tracker.is_crawling(repository.id).await);
    }

    #[tokio::test]
    async fn test_trigger_crawl_multiple_repositories_no_interference() {
        let progress_tracker = ProgressTracker::new();
        let repo1 = MockRepository { id: Uuid::new_v4(), name: "repo1".to_string(), enabled: true };
        let repo2 = MockRepository { id: Uuid::new_v4(), name: "repo2".to_string(), enabled: true };

        // Start crawl for repo1
        let result1 = simulate_trigger_crawl(&progress_tracker, &repo1).await;
        assert!(result1.is_ok());

        // Should still be able to start crawl for repo2
        let result2 = simulate_trigger_crawl(&progress_tracker, &repo2).await;
        assert!(result2.is_ok());

        // Both should be crawling
        assert!(progress_tracker.is_crawling(repo1.id).await);
        assert!(progress_tracker.is_crawling(repo2.id).await);

        // Trying to start repo1 again should fail
        let result1_again = simulate_trigger_crawl(&progress_tracker, &repo1).await;
        assert!(result1_again.is_err());
        assert_eq!(result1_again.unwrap_err(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_trigger_crawl_rapid_successive_attempts() {
        let progress_tracker = ProgressTracker::new();
        let repository = MockRepository { id: Uuid::new_v4(), name: "rapid-test-repo".to_string(), enabled: true };

        // First attempt should succeed
        let result1 = simulate_trigger_crawl(&progress_tracker, &repository).await;
        assert!(result1.is_ok());

        // Immediate subsequent attempts should fail
        for _ in 0..5 {
            let result = simulate_trigger_crawl(&progress_tracker, &repository).await;
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), StatusCode::CONFLICT);
        }

        // Complete the crawl
        progress_tracker.complete_crawl(repository.id).await;

        // Now should be able to trigger again
        let result_after_completion = simulate_trigger_crawl(&progress_tracker, &repository).await;
        assert!(result_after_completion.is_ok());
    }
}

// Add a dependency for the futures crate in your Cargo.toml if not already present:
// [dev-dependencies]
// futures = "0.3"
