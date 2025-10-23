use anyhow::Result;
use klask_rs::services::progress::{CrawlProgressInfo, CrawlStatus, ProgressTracker};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_progress_tracker_initialization() -> Result<()> {
    let tracker = ProgressTracker::new();

    // Should have no active progress initially
    let active_progress = tracker.get_all_active_progress().await;
    assert!(active_progress.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_start_crawl_creates_progress() -> Result<()> {
    let tracker = ProgressTracker::new();
    let repository_id = Uuid::new_v4();
    let repository_name = "test-repo".to_string();

    let progress = tracker.start_crawl(repository_id, repository_name.clone()).await;

    assert_eq!(progress.repository_id, repository_id);
    assert_eq!(progress.repository_name, repository_name);
    assert!(matches!(progress.status, CrawlStatus::Starting));
    assert_eq!(progress.progress_percentage, 0.0);
    assert_eq!(progress.files_processed, 0);
    assert_eq!(progress.files_indexed, 0);
    assert!(progress.files_total.is_none());
    assert!(progress.current_file.is_none());
    assert!(progress.error_message.is_none());
    assert!(progress.completed_at.is_none());

    Ok(())
}

#[tokio::test]
async fn test_cancelled_status_handling() -> Result<()> {
    let tracker = ProgressTracker::new();
    let repository_id = Uuid::new_v4();
    let repository_name = "test-repo".to_string();

    // Start a crawl
    let _progress = tracker.start_crawl(repository_id, repository_name).await;

    // Cancel the crawl
    tracker.cancel_crawl(repository_id).await;

    // Verify status is cancelled
    let progress = tracker.get_progress(repository_id).await.unwrap();
    assert!(matches!(progress.status, CrawlStatus::Cancelled));
    assert_eq!(progress.progress_percentage, 100.0);
    assert!(progress.completed_at.is_some());

    // Should not be included in active progress
    let active_progress = tracker.get_all_active_progress().await;
    assert!(active_progress.is_empty());

    // Should not be considered as crawling
    assert!(!tracker.is_crawling(repository_id).await);

    Ok(())
}

#[tokio::test]
async fn test_update_status_to_cancelled() -> Result<()> {
    let tracker = ProgressTracker::new();
    let repository_id = Uuid::new_v4();
    let repository_name = "test-repo".to_string();

    // Start crawl and update to processing
    tracker.start_crawl(repository_id, repository_name).await;
    tracker.update_status(repository_id, CrawlStatus::Processing).await;

    // Verify it's active
    assert!(tracker.is_crawling(repository_id).await);
    let active_progress = tracker.get_all_active_progress().await;
    assert_eq!(active_progress.len(), 1);

    // Update to cancelled
    tracker.update_status(repository_id, CrawlStatus::Cancelled).await;

    // Verify status changed
    let progress = tracker.get_progress(repository_id).await.unwrap();
    assert!(matches!(progress.status, CrawlStatus::Cancelled));
    assert_eq!(progress.progress_percentage, 100.0);
    assert!(progress.completed_at.is_some());

    // Should no longer be active or crawling
    assert!(!tracker.is_crawling(repository_id).await);
    let active_progress = tracker.get_all_active_progress().await;
    assert!(active_progress.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_cancelled_status_with_completed_failed() -> Result<()> {
    let tracker = ProgressTracker::new();

    // Test with multiple repositories in different final states
    let repo_cancelled = Uuid::new_v4();
    let repo_completed = Uuid::new_v4();
    let repo_failed = Uuid::new_v4();
    let repo_active = Uuid::new_v4();

    // Start crawls
    tracker.start_crawl(repo_cancelled, "cancelled-repo".to_string()).await;
    tracker.start_crawl(repo_completed, "completed-repo".to_string()).await;
    tracker.start_crawl(repo_failed, "failed-repo".to_string()).await;
    tracker.start_crawl(repo_active, "active-repo".to_string()).await;

    // Set different final states
    tracker.update_status(repo_cancelled, CrawlStatus::Cancelled).await;
    tracker.update_status(repo_completed, CrawlStatus::Completed).await;
    tracker.update_status(repo_failed, CrawlStatus::Failed).await;
    tracker.update_status(repo_active, CrawlStatus::Processing).await;

    // Verify only active repo is crawling
    assert!(!tracker.is_crawling(repo_cancelled).await);
    assert!(!tracker.is_crawling(repo_completed).await);
    assert!(!tracker.is_crawling(repo_failed).await);
    assert!(tracker.is_crawling(repo_active).await);

    // Verify only active repo in active progress
    let active_progress = tracker.get_all_active_progress().await;
    assert_eq!(active_progress.len(), 1);
    assert_eq!(active_progress[0].repository_id, repo_active);

    Ok(())
}

#[tokio::test]
async fn test_cleanup_old_progress_with_cancelled() -> Result<()> {
    let tracker = ProgressTracker::new();
    let repository_id = Uuid::new_v4();
    let repository_name = "test-repo".to_string();

    // Start and cancel a crawl
    tracker.start_crawl(repository_id, repository_name).await;
    tracker.cancel_crawl(repository_id).await;

    // Verify progress exists
    assert!(tracker.get_progress(repository_id).await.is_some());

    // Cleanup with 0 hours (should remove everything old)
    tracker.cleanup_old_progress(0).await;

    // Progress should still exist as it was just completed
    // (cleanup only removes items older than the cutoff)
    assert!(tracker.get_progress(repository_id).await.is_some());

    Ok(())
}

#[tokio::test]
async fn test_remove_progress() -> Result<()> {
    let tracker = ProgressTracker::new();
    let repository_id = Uuid::new_v4();
    let repository_name = "test-repo".to_string();

    // Start and cancel crawl
    tracker.start_crawl(repository_id, repository_name).await;
    tracker.cancel_crawl(repository_id).await;

    // Verify progress exists
    assert!(tracker.get_progress(repository_id).await.is_some());

    // Remove progress
    tracker.remove_progress(repository_id).await;

    // Verify progress is removed
    assert!(tracker.get_progress(repository_id).await.is_none());
    assert!(!tracker.is_crawling(repository_id).await);

    Ok(())
}

#[tokio::test]
async fn test_multiple_repositories_cancellation() -> Result<()> {
    let tracker = ProgressTracker::new();
    let num_repos = 5;
    let mut repo_ids = Vec::new();

    // Start multiple crawls
    for i in 0..num_repos {
        let repo_id = Uuid::new_v4();
        repo_ids.push(repo_id);
        tracker.start_crawl(repo_id, format!("repo-{}", i)).await;
        tracker.update_status(repo_id, CrawlStatus::Processing).await;
    }

    // Verify all are active
    let active_progress = tracker.get_all_active_progress().await;
    assert_eq!(active_progress.len(), num_repos);

    // Cancel half of them
    for repo_id in repo_ids.iter().take(num_repos / 2) {
        tracker.cancel_crawl(*repo_id).await;
    }

    // Verify correct number are still active
    let active_progress = tracker.get_all_active_progress().await;
    assert_eq!(active_progress.len(), num_repos - num_repos / 2);

    // Verify correct repos are cancelled vs active
    for (i, repo_id) in repo_ids.iter().enumerate() {
        let expected_crawling = i >= num_repos / 2;
        assert_eq!(tracker.is_crawling(*repo_id).await, expected_crawling);
    }

    Ok(())
}

#[tokio::test]
async fn test_cancel_nonexistent_repository() -> Result<()> {
    let tracker = ProgressTracker::new();
    let nonexistent_id = Uuid::new_v4();

    // Cancel non-existent repository (should not panic)
    tracker.cancel_crawl(nonexistent_id).await;

    // Should still return None for progress
    assert!(tracker.get_progress(nonexistent_id).await.is_none());
    assert!(!tracker.is_crawling(nonexistent_id).await);

    Ok(())
}

#[tokio::test]
async fn test_progress_info_update_status_cancelled() -> Result<()> {
    let repository_id = Uuid::new_v4();
    let mut progress = CrawlProgressInfo::new(repository_id, "test-repo".to_string());

    // Initially not completed
    assert!(progress.completed_at.is_none());
    assert_eq!(progress.progress_percentage, 0.0);
    assert!(matches!(progress.status, CrawlStatus::Starting));

    // Update to cancelled
    progress.update_status(CrawlStatus::Cancelled);

    // Should be marked as completed with 100% progress
    assert!(progress.completed_at.is_some());
    assert_eq!(progress.progress_percentage, 100.0);
    assert!(matches!(progress.status, CrawlStatus::Cancelled));

    Ok(())
}

#[tokio::test]
async fn test_concurrent_cancellation() -> Result<()> {
    let tracker = Arc::new(ProgressTracker::new());
    let repository_id = Uuid::new_v4();
    let repository_name = "test-repo".to_string();

    // Start a crawl
    tracker.start_crawl(repository_id, repository_name).await;

    // Simulate concurrent cancellation attempts
    let tracker_clone1 = tracker.clone();
    let tracker_clone2 = tracker.clone();
    let tracker_for_assert = tracker.clone();

    let task1 = tokio::spawn(async move {
        tracker_clone1.cancel_crawl(repository_id).await;
    });

    let task2 = tokio::spawn(async move {
        tracker_clone2.cancel_crawl(repository_id).await;
    });

    // Both should complete without panicking
    task1.await.unwrap();
    task2.await.unwrap();

    // Repository should be cancelled
    assert!(!tracker_for_assert.is_crawling(repository_id).await);
    let progress = tracker_for_assert.get_progress(repository_id).await.unwrap();
    assert!(matches!(progress.status, CrawlStatus::Cancelled));

    Ok(())
}
