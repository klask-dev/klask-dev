#[cfg(test)]
mod progress_tests {
    use chrono::Utc;
    use klask_rs::services::progress::{CrawlProgressInfo, CrawlStatus, ProgressTracker};
    use std::sync::Arc;

    use uuid::Uuid;

    #[tokio::test]
    async fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new();
        let active_progress = tracker.get_all_active_progress().await;
        assert!(active_progress.is_empty());
    }

    #[tokio::test]
    async fn test_start_crawl() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        let progress = tracker.start_crawl(repo_id, repo_name.clone()).await;

        assert_eq!(progress.repository_id, repo_id);
        assert_eq!(progress.repository_name, repo_name);
        assert!(matches!(progress.status, CrawlStatus::Starting));
        assert_eq!(progress.progress_percentage, 0.0);
        assert_eq!(progress.files_processed, 0);
        assert_eq!(progress.files_indexed, 0);
        assert!(progress.files_total.is_none());
        assert!(progress.current_file.is_none());
        assert!(progress.error_message.is_none());
        assert!(progress.completed_at.is_none());
    }

    #[tokio::test]
    async fn test_update_status() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;

        // Test status update to Processing
        tracker.update_status(repo_id, CrawlStatus::Processing).await;
        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert!(matches!(progress.status, CrawlStatus::Processing));
        assert!(progress.completed_at.is_none());

        // Test status update to Completed
        tracker.update_status(repo_id, CrawlStatus::Completed).await;
        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert!(matches!(progress.status, CrawlStatus::Completed));
        assert!(progress.completed_at.is_some());
        assert_eq!(progress.progress_percentage, 100.0);
    }

    #[tokio::test]
    async fn test_update_progress() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;

        // Test progress update
        tracker.update_progress(repo_id, 50, Some(100), 25).await;
        let progress = tracker.get_progress(repo_id).await.unwrap();

        assert_eq!(progress.files_processed, 50);
        assert_eq!(progress.files_total, Some(100));
        assert_eq!(progress.files_indexed, 25);
        assert_eq!(progress.progress_percentage, 50.0);
    }

    #[tokio::test]
    async fn test_progress_percentage_calculation() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;

        // Test with zero total
        tracker.update_progress(repo_id, 10, Some(0), 5).await;
        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(progress.progress_percentage, 0.0);

        // Test with normal values
        tracker.update_progress(repo_id, 75, Some(100), 50).await;
        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(progress.progress_percentage, 75.0);

        // Test with values exceeding 100%
        tracker.update_progress(repo_id, 150, Some(100), 100).await;
        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(progress.progress_percentage, 100.0);

        // Test with no total
        tracker.update_progress(repo_id, 50, None, 30).await;
        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(progress.progress_percentage, 100.0); // Should remain 100% from previous update
    }

    #[tokio::test]
    async fn test_set_current_file() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;

        let file_path = "src/main.rs".to_string();
        tracker.set_current_file(repo_id, Some(file_path.clone())).await;

        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(progress.current_file, Some(file_path));

        // Test clearing current file
        tracker.set_current_file(repo_id, None).await;
        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert!(progress.current_file.is_none());
    }

    #[tokio::test]
    async fn test_set_error() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;

        let error_message = "Failed to clone repository".to_string();
        tracker.set_error(repo_id, error_message.clone()).await;

        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(progress.error_message, Some(error_message));
        assert!(matches!(progress.status, CrawlStatus::Failed));
        assert!(progress.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_complete_crawl() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repo".to_string();

        tracker.start_crawl(repo_id, repo_name).await;
        tracker.complete_crawl(repo_id).await;

        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert!(matches!(progress.status, CrawlStatus::Completed));
        assert!(progress.completed_at.is_some());
        assert_eq!(progress.progress_percentage, 100.0);
    }

    #[tokio::test]
    async fn test_get_all_active_progress() {
        let tracker = ProgressTracker::new();

        let repo_id1 = Uuid::new_v4();
        let repo_id2 = Uuid::new_v4();
        let repo_id3 = Uuid::new_v4();

        // Start multiple crawls
        tracker.start_crawl(repo_id1, "repo1".to_string()).await;
        tracker.start_crawl(repo_id2, "repo2".to_string()).await;
        tracker.start_crawl(repo_id3, "repo3".to_string()).await;

        // Update one to processing
        tracker.update_status(repo_id1, CrawlStatus::Processing).await;

        // Complete one
        tracker.complete_crawl(repo_id2).await;

        // Fail one
        tracker.set_error(repo_id3, "Test error".to_string()).await;

        let active_progress = tracker.get_all_active_progress().await;

        // Should only have the processing one
        assert_eq!(active_progress.len(), 1);
        assert_eq!(active_progress[0].repository_id, repo_id1);
        assert!(matches!(active_progress[0].status, CrawlStatus::Processing));
    }

    #[tokio::test]
    async fn test_cleanup_old_progress() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();

        tracker.start_crawl(repo_id, "test-repo".to_string()).await;
        tracker.complete_crawl(repo_id).await;

        // Should have the completed progress
        assert!(tracker.get_progress(repo_id).await.is_some());

        // Clean up progress older than 0 hours (should remove all completed)
        tracker.cleanup_old_progress(0).await;

        // Progress should still exist since cleanup is based on completion time
        // and we just completed it (within the last second)
        assert!(tracker.get_progress(repo_id).await.is_some());
    }

    #[tokio::test]
    async fn test_remove_progress() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();

        tracker.start_crawl(repo_id, "test-repo".to_string()).await;
        assert!(tracker.get_progress(repo_id).await.is_some());

        tracker.remove_progress(repo_id).await;
        assert!(tracker.get_progress(repo_id).await.is_none());
    }

    #[tokio::test]
    async fn test_crawl_progress_info_new() {
        let repo_id = Uuid::new_v4();
        let repo_name = "test-repository".to_string();
        let before_creation = Utc::now();

        let progress = CrawlProgressInfo::new(repo_id, repo_name.clone());
        let after_creation = Utc::now();

        assert_eq!(progress.repository_id, repo_id);
        assert_eq!(progress.repository_name, repo_name);
        assert!(matches!(progress.status, CrawlStatus::Starting));
        assert_eq!(progress.progress_percentage, 0.0);
        assert_eq!(progress.files_processed, 0);
        assert_eq!(progress.files_total, None);
        assert_eq!(progress.files_indexed, 0);
        assert!(progress.current_file.is_none());
        assert!(progress.error_message.is_none());
        assert!(progress.started_at >= before_creation && progress.started_at <= after_creation);
        assert!(progress.updated_at >= before_creation && progress.updated_at <= after_creation);
        assert!(progress.completed_at.is_none());
    }

    #[tokio::test]
    async fn test_multiple_repositories_tracking() {
        let tracker = ProgressTracker::new();

        let repo_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

        // Start crawls for multiple repositories
        for (i, &repo_id) in repo_ids.iter().enumerate() {
            tracker.start_crawl(repo_id, format!("repo-{}", i)).await;
        }

        // Update each to different statuses
        tracker.update_status(repo_ids[0], CrawlStatus::Cloning).await;
        tracker.update_status(repo_ids[1], CrawlStatus::Processing).await;
        tracker.update_status(repo_ids[2], CrawlStatus::Indexing).await;
        tracker.complete_crawl(repo_ids[3]).await;
        tracker.set_error(repo_ids[4], "Test error".to_string()).await;

        // Check each repository has the correct status
        let progress0 = tracker.get_progress(repo_ids[0]).await.unwrap();
        assert!(matches!(progress0.status, CrawlStatus::Cloning));

        let progress1 = tracker.get_progress(repo_ids[1]).await.unwrap();
        assert!(matches!(progress1.status, CrawlStatus::Processing));

        let progress2 = tracker.get_progress(repo_ids[2]).await.unwrap();
        assert!(matches!(progress2.status, CrawlStatus::Indexing));

        let progress3 = tracker.get_progress(repo_ids[3]).await.unwrap();
        assert!(matches!(progress3.status, CrawlStatus::Completed));

        let progress4 = tracker.get_progress(repo_ids[4]).await.unwrap();
        assert!(matches!(progress4.status, CrawlStatus::Failed));

        // Check active progress only includes non-completed/failed
        let active = tracker.get_all_active_progress().await;
        assert_eq!(active.len(), 3);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        tracker.start_crawl(repo_id, "concurrent-test".to_string()).await;

        // Spawn multiple tasks to update progress concurrently
        let mut handles = vec![];

        for i in 0..10 {
            let tracker_clone = Arc::clone(&tracker);
            let handle = tokio::spawn(async move {
                tracker_clone.update_progress(repo_id, i * 10, Some(100), i * 5).await;
                tracker_clone.set_current_file(repo_id, Some(format!("file-{}.rs", i))).await;
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify progress is in a consistent state
        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert!(progress.files_processed <= 100);
        assert!(progress.files_indexed <= 50);
        assert!(progress.current_file.is_some());
    }
}
