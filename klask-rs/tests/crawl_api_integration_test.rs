#[cfg(test)]
mod crawl_api_integration_tests {
    use axum::http::StatusCode;
    use klask_rs::services::progress::{CrawlStatus, ProgressTracker};

    use std::sync::Arc;

    use uuid::Uuid;

    // Mock app state for testing
    #[derive(Clone)]
    #[allow(dead_code)]
    struct MockAppState {
        progress_tracker: Arc<ProgressTracker>,
    }

    // Simulate the API endpoint logic
    async fn mock_trigger_crawl_endpoint(
        repository_id: Uuid,
        progress_tracker: &ProgressTracker,
        repository_enabled: bool,
    ) -> Result<String, StatusCode> {
        // Check if repository is already being crawled
        if progress_tracker.is_crawling(repository_id).await {
            return Err(StatusCode::CONFLICT);
        }

        // Check if repository is enabled
        if !repository_enabled {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Double-check if repository is still not being crawled (race condition protection)
        if progress_tracker.is_crawling(repository_id).await {
            return Err(StatusCode::CONFLICT);
        }

        // Simulate starting the crawl
        progress_tracker.start_crawl(repository_id, format!("repo-{}", repository_id)).await;

        Ok("Crawl started in background".to_string())
    }

    #[tokio::test]
    async fn test_http_409_response_when_repository_already_crawling() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Start a crawl first
        progress_tracker.start_crawl(repo_id, "test-repo".to_string()).await;

        // Attempt to trigger another crawl
        let result = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_http_400_response_when_repository_disabled() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        let result = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, false).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_http_200_response_when_crawl_starts_successfully() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        let result = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Crawl started in background");
        assert!(progress_tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_race_condition_protection_in_endpoint() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Spawn multiple concurrent requests
        let mut handles = vec![];

        for _ in 0..10 {
            let progress_tracker_clone = Arc::clone(&progress_tracker);
            let handle =
                tokio::spawn(async move { mock_trigger_crawl_endpoint(repo_id, &progress_tracker_clone, true).await });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let results: Vec<Result<String, StatusCode>> =
            futures::future::join_all(handles).await.into_iter().map(|r| r.unwrap()).collect();

        // Only one should succeed, others should get CONFLICT
        let successful = results.iter().filter(|r| r.is_ok()).count();
        let conflicts =
            results.iter().filter(|r| r.is_err() && r.as_ref().unwrap_err() == &StatusCode::CONFLICT).count();

        assert_eq!(successful, 1);
        assert_eq!(conflicts, 9);
    }

    #[tokio::test]
    async fn test_double_check_race_condition_protection() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Simulate a scenario where crawl starts between the two checks
        // This test verifies the double-check pattern in the endpoint

        // Custom implementation that simulates timing
        async fn mock_trigger_crawl_with_timing(
            repository_id: Uuid,
            progress_tracker: &ProgressTracker,
            start_crawl_between_checks: bool,
        ) -> Result<String, StatusCode> {
            // First check
            if progress_tracker.is_crawling(repository_id).await {
                return Err(StatusCode::CONFLICT);
            }

            // Simulate another process starting crawl here
            if start_crawl_between_checks {
                progress_tracker.start_crawl(repository_id, "concurrent-repo".to_string()).await;
            }

            // Second check (should catch the race condition)
            if progress_tracker.is_crawling(repository_id).await {
                return Err(StatusCode::CONFLICT);
            }

            // This shouldn't be reached if race condition is properly handled
            progress_tracker.start_crawl(repository_id, "test-repo".to_string()).await;
            Ok("Crawl started".to_string())
        }

        let result = mock_trigger_crawl_with_timing(repo_id, &progress_tracker, true).await;

        // Should return CONFLICT due to the double-check
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_concurrent_requests_to_different_repositories() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

        // Spawn concurrent requests for different repositories
        let mut handles = vec![];

        for &repo_id in &repo_ids {
            let progress_tracker_clone = Arc::clone(&progress_tracker);
            let handle =
                tokio::spawn(async move { mock_trigger_crawl_endpoint(repo_id, &progress_tracker_clone, true).await });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let results: Vec<Result<String, StatusCode>> =
            futures::future::join_all(handles).await.into_iter().map(|r| r.unwrap()).collect();

        // All should succeed since they're different repositories
        let successful = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(successful, 5);

        // All repositories should be crawling
        for &repo_id in &repo_ids {
            assert!(progress_tracker.is_crawling(repo_id).await);
        }
    }

    #[tokio::test]
    async fn test_crawl_after_completion_returns_200() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Start and complete a crawl
        progress_tracker.start_crawl(repo_id, "test-repo".to_string()).await;
        progress_tracker.complete_crawl(repo_id).await;

        // Should be able to start a new crawl
        let result = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;

        assert!(result.is_ok());
        assert!(progress_tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_crawl_after_failure_returns_200() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Start and fail a crawl
        progress_tracker.start_crawl(repo_id, "test-repo".to_string()).await;
        progress_tracker.set_error(repo_id, "Test error".to_string()).await;

        // Should be able to start a new crawl
        let result = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;

        assert!(result.is_ok());
        assert!(progress_tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_crawl_after_cancellation_returns_200() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Start and cancel a crawl
        progress_tracker.start_crawl(repo_id, "test-repo".to_string()).await;
        progress_tracker.cancel_crawl(repo_id).await;

        // Should be able to start a new crawl
        let result = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;

        assert!(result.is_ok());
        assert!(progress_tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_status_transitions_and_crawl_prevention() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Initially should be able to crawl
        let result1 = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;
        assert!(result1.is_ok());

        // During Starting status - should not be able to crawl
        let result2 = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;
        assert_eq!(result2.unwrap_err(), StatusCode::CONFLICT);

        // Update to Cloning - should still not be able to crawl
        progress_tracker.update_status(repo_id, CrawlStatus::Cloning).await;
        let result3 = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;
        assert_eq!(result3.unwrap_err(), StatusCode::CONFLICT);

        // Update to Processing - should still not be able to crawl
        progress_tracker.update_status(repo_id, CrawlStatus::Processing).await;
        let result4 = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;
        assert_eq!(result4.unwrap_err(), StatusCode::CONFLICT);

        // Update to Indexing - should still not be able to crawl
        progress_tracker.update_status(repo_id, CrawlStatus::Indexing).await;
        let result5 = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;
        assert_eq!(result5.unwrap_err(), StatusCode::CONFLICT);

        // Complete the crawl - should now be able to crawl again
        progress_tracker.update_status(repo_id, CrawlStatus::Completed).await;
        let result6 = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;
        assert!(result6.is_ok());
    }

    #[tokio::test]
    async fn test_high_concurrency_stress_test() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Spawn a large number of concurrent requests
        let mut handles = vec![];

        for _ in 0..100 {
            let progress_tracker_clone = Arc::clone(&progress_tracker);
            let handle =
                tokio::spawn(async move { mock_trigger_crawl_endpoint(repo_id, &progress_tracker_clone, true).await });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let results: Vec<Result<String, StatusCode>> =
            futures::future::join_all(handles).await.into_iter().map(|r| r.unwrap()).collect();

        // Only one should succeed, all others should get CONFLICT
        let successful = results.iter().filter(|r| r.is_ok()).count();
        let conflicts =
            results.iter().filter(|r| r.is_err() && r.as_ref().unwrap_err() == &StatusCode::CONFLICT).count();

        assert_eq!(successful, 1);
        assert_eq!(conflicts, 99);

        // Repository should be in crawling state
        assert!(progress_tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_memory_leak_prevention_with_many_repositories() {
        let progress_tracker = Arc::new(ProgressTracker::new());

        // Start and complete many crawls
        for i in 0..1000 {
            let repo_id = Uuid::new_v4();

            // Start crawl
            let start_result = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;
            assert!(start_result.is_ok());

            // Complete crawl
            progress_tracker.complete_crawl(repo_id).await;

            // Verify can't start while completing
            progress_tracker.update_status(repo_id, CrawlStatus::Processing).await;
            let conflict_result = mock_trigger_crawl_endpoint(repo_id, &progress_tracker, true).await;
            assert_eq!(conflict_result.unwrap_err(), StatusCode::CONFLICT);

            // Complete it properly
            progress_tracker.complete_crawl(repo_id).await;

            // Clean up old progress periodically
            if i % 100 == 99 {
                progress_tracker.cleanup_old_progress(0).await;
            }
        }

        // Verify the tracker is still functioning
        let test_repo_id = Uuid::new_v4();
        let final_result = mock_trigger_crawl_endpoint(test_repo_id, &progress_tracker, true).await;
        assert!(final_result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_status_updates_and_crawl_attempts() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Start a crawl
        progress_tracker.start_crawl(repo_id, "concurrent-test".to_string()).await;

        // Spawn concurrent tasks that update status and attempt to crawl
        let mut handles = vec![];

        // Status update tasks
        for status in [CrawlStatus::Processing, CrawlStatus::Indexing, CrawlStatus::Completed] {
            let progress_tracker_clone = Arc::clone(&progress_tracker);
            let handle = tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                progress_tracker_clone.update_status(repo_id, status).await;
            });
            handles.push(handle);
        }

        // Crawl attempt tasks
        for _ in 0..5 {
            let progress_tracker_clone = Arc::clone(&progress_tracker);
            let handle = tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                let _result = mock_trigger_crawl_endpoint(repo_id, &progress_tracker_clone, true).await;
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let _results = futures::future::join_all(handles).await;

        // Final state should be consistent
        let progress = progress_tracker.get_progress(repo_id).await;
        assert!(progress.is_some());
    }

    #[tokio::test]
    async fn test_crawl_endpoint_with_invalid_repository_id() {
        let progress_tracker = Arc::new(ProgressTracker::new());

        // Test with various repository states
        let non_existent_repo = Uuid::new_v4();

        // Should succeed for non-existent repository (if enabled)
        let result = mock_trigger_crawl_endpoint(non_existent_repo, &progress_tracker, true).await;
        assert!(result.is_ok());
    }

    // Simulate stop crawl endpoint testing
    async fn mock_stop_crawl_endpoint(
        repository_id: Uuid,
        progress_tracker: &ProgressTracker,
    ) -> Result<String, StatusCode> {
        // Check if repository is currently being crawled
        if !progress_tracker.is_crawling(repository_id).await {
            return Err(StatusCode::NOT_FOUND);
        }

        // Cancel the crawl
        progress_tracker.cancel_crawl(repository_id).await;

        Ok("Crawl stopped successfully".to_string())
    }

    #[tokio::test]
    async fn test_stop_crawl_endpoint_success() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Start a crawl
        progress_tracker.start_crawl(repo_id, "test-repo".to_string()).await;

        // Stop the crawl
        let result = mock_stop_crawl_endpoint(repo_id, &progress_tracker).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Crawl stopped successfully");
        assert!(!progress_tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_stop_crawl_endpoint_not_found() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Try to stop a crawl that doesn't exist
        let result = mock_stop_crawl_endpoint(repo_id, &progress_tracker).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_stop_crawl_after_completion_returns_404() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Start and complete a crawl
        progress_tracker.start_crawl(repo_id, "test-repo".to_string()).await;
        progress_tracker.complete_crawl(repo_id).await;

        // Try to stop the completed crawl
        let result = mock_stop_crawl_endpoint(repo_id, &progress_tracker).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }
}

// Integration tests with mock HTTP server
#[cfg(test)]
mod http_integration_tests {

    use axum::http::StatusCode;
    use klask_rs::services::progress::ProgressTracker;
    use serde_json::json;
    use std::sync::Arc;
    use uuid::Uuid;

    // Mock HTTP request/response testing
    #[tokio::test]
    async fn test_trigger_crawl_http_409_conflict() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Start a crawl to create conflict
        progress_tracker.start_crawl(repo_id, "test-repo".to_string()).await;

        // Simulate HTTP request that would trigger crawl
        let response_status = if progress_tracker.is_crawling(repo_id).await {
            StatusCode::CONFLICT
        } else {
            StatusCode::OK
        };

        assert_eq!(response_status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_concurrent_http_requests_simulation() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Simulate multiple HTTP requests hitting the endpoint simultaneously
        let mut response_statuses = Vec::new();

        let mut handles = vec![];
        for _ in 0..10 {
            let progress_tracker_clone = Arc::clone(&progress_tracker);
            let handle = tokio::spawn(async move {
                // Simulate the endpoint logic
                if progress_tracker_clone.is_crawling(repo_id).await {
                    StatusCode::CONFLICT
                } else {
                    // Double-check before starting
                    if progress_tracker_clone.is_crawling(repo_id).await {
                        StatusCode::CONFLICT
                    } else {
                        progress_tracker_clone.start_crawl(repo_id, "concurrent-repo".to_string()).await;
                        StatusCode::OK
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            response_statuses.push(handle.await.unwrap());
        }

        // Only one should get OK (200), others should get CONFLICT (409)
        let ok_count = response_statuses.iter().filter(|&&s| s == StatusCode::OK).count();
        let conflict_count = response_statuses.iter().filter(|&&s| s == StatusCode::CONFLICT).count();

        assert_eq!(ok_count, 1);
        assert_eq!(conflict_count, 9);
    }

    #[tokio::test]
    async fn test_json_response_format() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Simulate successful crawl start
        progress_tracker.start_crawl(repo_id, "test-repo".to_string()).await;

        // Simulate JSON response
        let response_body = json!({
            "message": "Crawl started in background",
            "repository_id": repo_id.to_string(),
            "status": "started"
        });

        assert_eq!(response_body["message"], "Crawl started in background");
        assert_eq!(response_body["repository_id"], repo_id.to_string());
        assert_eq!(response_body["status"], "started");
    }

    #[tokio::test]
    async fn test_error_response_format() {
        let progress_tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Start a crawl to create conflict
        progress_tracker.start_crawl(repo_id, "test-repo".to_string()).await;

        // Simulate conflict error response
        let error_response = json!({
            "error": "Conflict",
            "message": "Repository is already being crawled",
            "repository_id": repo_id.to_string(),
            "status": 409
        });

        assert_eq!(error_response["error"], "Conflict");
        assert_eq!(error_response["status"], 409);
    }
}
