#[cfg(test)]
mod crawl_edge_cases_tests {
    use klask_rs::services::progress::{CrawlStatus, ProgressTracker};
    use rand::Rng;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::{sleep, timeout};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_extremely_rapid_state_changes() {
        let tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        // Start a crawl
        tracker.start_crawl(repo_id, "rapid-test".to_string()).await;

        // Rapidly change states
        let mut handles = vec![];
        let statuses = vec![
            CrawlStatus::Cloning,
            CrawlStatus::Processing,
            CrawlStatus::Indexing,
            CrawlStatus::Completed,
            CrawlStatus::Failed,
            CrawlStatus::Cancelled,
        ];

        for (i, status) in statuses.into_iter().enumerate() {
            let tracker_clone = Arc::clone(&tracker);
            let handle = tokio::spawn(async move {
                // Introduce tiny delays to create race conditions
                sleep(Duration::from_nanos(i as u64 * 100)).await;
                tracker_clone.update_status(repo_id, status).await;
            });
            handles.push(handle);
        }

        // Wait for all updates to complete
        futures::future::join_all(handles).await;

        // Final state should be consistent
        let progress = tracker.get_progress(repo_id).await;
        assert!(progress.is_some());

        let final_progress = progress.unwrap();
        // Should be in one of the terminal states
        assert!(matches!(
            final_progress.status,
            CrawlStatus::Completed | CrawlStatus::Failed | CrawlStatus::Cancelled
        ));

        // Should not be crawling anymore
        assert!(!tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_concurrent_progress_updates_during_status_changes() {
        let tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        tracker.start_crawl(repo_id, "concurrent-updates".to_string()).await;

        let mut handles = vec![];

        // Spawn status update tasks
        for i in 0..5 {
            let tracker_clone = Arc::clone(&tracker);
            let handle = tokio::spawn(async move {
                let status = match i % 3 {
                    0 => CrawlStatus::Cloning,
                    1 => CrawlStatus::Processing,
                    _ => CrawlStatus::Indexing,
                };
                tracker_clone.update_status(repo_id, status).await;
            });
            handles.push(handle);
        }

        // Spawn progress update tasks
        for i in 0..10 {
            let tracker_clone = Arc::clone(&tracker);
            let handle = tokio::spawn(async move {
                tracker_clone.update_progress(repo_id, i * 10, Some(100), i * 5).await;
            });
            handles.push(handle);
        }

        // Spawn file update tasks
        for i in 0..5 {
            let tracker_clone = Arc::clone(&tracker);
            let handle = tokio::spawn(async move {
                tracker_clone.set_current_file(repo_id, Some(format!("file-{}.rs", i))).await;
            });
            handles.push(handle);
        }

        // Wait for all updates to complete
        futures::future::join_all(handles).await;

        // Verify final state is consistent
        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert!(progress.files_processed <= 100);
        assert!(progress.files_indexed <= 50);
        assert!(progress.current_file.is_some());
    }

    #[tokio::test]
    async fn test_memory_pressure_with_rapid_repository_creation_and_cleanup() {
        let tracker = Arc::new(ProgressTracker::new());

        // Create and immediately clean up many repositories
        for batch in 0..10 {
            let mut repo_ids = Vec::new();

            // Create batch of repositories
            for i in 0..100 {
                let repo_id = Uuid::new_v4();
                repo_ids.push(repo_id);
                tracker.start_crawl(repo_id, format!("batch-{}-repo-{}", batch, i)).await;
            }

            // Complete half, fail quarter, cancel quarter
            for (i, &repo_id) in repo_ids.iter().enumerate() {
                match i % 4 {
                    0 => tracker.complete_crawl(repo_id).await,
                    1 => tracker.set_error(repo_id, "Test error".to_string()).await,
                    2 => tracker.cancel_crawl(repo_id).await,
                    _ => {} // Leave some active
                }
            }

            // Cleanup old progress
            tracker.cleanup_old_progress(0).await;

            // Verify active progress count is reasonable - this is a stress test so we just
            // verify the system doesn't completely break down under memory pressure
            let active = tracker.get_all_active_progress().await;
            // Just ensure we don't have a runaway situation with too many active items
            assert!(
                active.len() < 1000,
                "Batch {}: too many active items, got {}",
                batch,
                active.len()
            );
        }
    }

    #[tokio::test]
    async fn test_timeout_handling_during_concurrent_operations() {
        let tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        tracker.start_crawl(repo_id, "timeout-test".to_string()).await;

        // Test that operations complete within reasonable time even under high concurrency
        let operations_future = async {
            let mut handles = vec![];

            for i in 0..1000 {
                let tracker_clone = Arc::clone(&tracker);
                let handle = tokio::spawn(async move {
                    // Mix of operations
                    match i % 6 {
                        0 => {
                            tracker_clone.is_crawling(repo_id).await;
                        }
                        1 => {
                            tracker_clone.update_progress(repo_id, i, Some(1000), i / 2).await;
                        }
                        2 => {
                            tracker_clone.set_current_file(repo_id, Some(format!("file-{}", i))).await;
                        }
                        3 => {
                            tracker_clone.get_progress(repo_id).await;
                        }
                        4 => {
                            tracker_clone.get_all_active_progress().await;
                        }
                        _ => {
                            tracker_clone.update_status(repo_id, CrawlStatus::Processing).await;
                        }
                    }
                });
                handles.push(handle);
            }

            futures::future::join_all(handles).await;
        };

        // Operations should complete within 5 seconds
        let result = timeout(Duration::from_secs(5), operations_future).await;
        assert!(result.is_ok(), "Operations took too long to complete");
    }

    #[tokio::test]
    async fn test_repository_state_consistency_under_stress() {
        let tracker = Arc::new(ProgressTracker::new());
        let repo_ids: Vec<Uuid> = (0..50).map(|_| Uuid::new_v4()).collect();

        // Start crawls for all repositories
        for &repo_id in &repo_ids {
            tracker.start_crawl(repo_id, format!("stress-test-{}", repo_id)).await;
        }

        // Apply random operations
        let mut handles = vec![];
        for _ in 0..500 {
            let repo_id = repo_ids[rand::rng().random_range(0..repo_ids.len())];
            let tracker_clone = Arc::clone(&tracker);

            let random_operation = rand::rng().random_range(0..8);
            let handle = tokio::spawn(async move {
                match random_operation {
                    0 => tracker_clone.is_crawling(repo_id).await,
                    1 => {
                        tracker_clone.update_status(repo_id, CrawlStatus::Processing).await;
                        false
                    }
                    2 => {
                        tracker_clone.update_progress(repo_id, 50, Some(100), 25).await;
                        false
                    }
                    3 => {
                        tracker_clone.set_current_file(repo_id, Some("test.rs".to_string())).await;
                        false
                    }
                    4 => {
                        tracker_clone.complete_crawl(repo_id).await;
                        false
                    }
                    5 => {
                        tracker_clone.cancel_crawl(repo_id).await;
                        false
                    }
                    6 => {
                        tracker_clone.set_error(repo_id, "Random error".to_string()).await;
                        false
                    }
                    _ => tracker_clone.get_progress(repo_id).await.is_some(),
                }
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let results: Vec<bool> = futures::future::join_all(handles).await.into_iter().map(|r| r.unwrap()).collect();

        // Verify that operations completed without panicking
        assert!(results.len() == 500);

        // Verify that repository states are still consistent
        for &repo_id in &repo_ids {
            let progress = tracker.get_progress(repo_id).await;
            if let Some(p) = progress {
                // Progress should be internally consistent
                assert!(p.progress_percentage <= 100.0);
                assert!(p.files_indexed <= p.files_processed);
                if let Some(total) = p.files_total {
                    assert!(p.files_processed <= total);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_progress_info_immutability_during_updates() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();

        tracker.start_crawl(repo_id, "immutability-test".to_string()).await;

        // Get initial progress
        let initial_progress = tracker.get_progress(repo_id).await.unwrap();
        let initial_started_at = initial_progress.started_at;
        let initial_id = initial_progress.repository_id;
        let initial_name = initial_progress.repository_name.clone();

        // Update progress multiple times
        for i in 0..10 {
            tracker.update_progress(repo_id, i * 10, Some(100), i * 5).await;
            tracker.update_status(repo_id, CrawlStatus::Processing).await;
            tracker.set_current_file(repo_id, Some(format!("file-{}.rs", i))).await;

            let current_progress = tracker.get_progress(repo_id).await.unwrap();

            // Immutable fields should not change
            assert_eq!(current_progress.started_at, initial_started_at);
            assert_eq!(current_progress.repository_id, initial_id);
            assert_eq!(current_progress.repository_name, initial_name);

            // updated_at should change
            assert!(current_progress.updated_at >= initial_progress.updated_at);
        }
    }

    #[tokio::test]
    async fn test_cleanup_during_active_operations() {
        let tracker = Arc::new(ProgressTracker::new());

        // Start multiple crawls with different completion times
        let repo_ids: Vec<Uuid> = (0..20).map(|_| Uuid::new_v4()).collect();

        for &repo_id in &repo_ids {
            tracker.start_crawl(repo_id, format!("cleanup-test-{}", repo_id)).await;
        }

        // Complete some crawls
        for repo_id in repo_ids.iter().take(10) {
            tracker.complete_crawl(*repo_id).await;
        }

        // Start concurrent operations while cleanup is running
        let mut handles = vec![];

        // Spawn cleanup task
        let tracker_cleanup = Arc::clone(&tracker);
        let cleanup_handle = tokio::spawn(async move {
            for _ in 0..5 {
                tracker_cleanup.cleanup_old_progress(0).await;
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(cleanup_handle);

        // Spawn operations on active repositories
        for repo_id in repo_ids.iter().skip(10).take(10) {
            let repo_id = *repo_id;
            let tracker_clone = Arc::clone(&tracker);
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    tracker_clone.update_progress(repo_id, j * 10, Some(100), j * 5).await;
                    tracker_clone.is_crawling(repo_id).await;
                    sleep(Duration::from_millis(1)).await;
                }
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        futures::future::join_all(handles).await;

        // Verify that active repositories are still tracked
        let active_progress = tracker.get_all_active_progress().await;
        assert!(active_progress.len() >= 10); // Should have at least the active ones
    }

    #[tokio::test]
    async fn test_error_handling_during_concurrent_state_changes() {
        let tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        tracker.start_crawl(repo_id, "error-test".to_string()).await;

        // Spawn tasks that will update state and potentially conflict
        let mut handles = vec![];

        // Task that sets error
        let tracker_error = Arc::clone(&tracker);
        let error_handle = tokio::spawn(async move {
            sleep(Duration::from_millis(5)).await;
            tracker_error.set_error(repo_id, "Test error message".to_string()).await;
        });
        handles.push(error_handle);

        // Tasks that try to update progress concurrently
        for i in 0..5 {
            let tracker_clone = Arc::clone(&tracker);
            let handle = tokio::spawn(async move {
                sleep(Duration::from_millis(i * 2)).await;
                tracker_clone.update_progress(repo_id, (i * 20) as usize, Some(100), (i * 10) as usize).await;
            });
            handles.push(handle);
        }

        // Task that tries to complete
        let tracker_complete = Arc::clone(&tracker);
        let complete_handle = tokio::spawn(async move {
            sleep(Duration::from_millis(7)).await;
            tracker_complete.complete_crawl(repo_id).await;
        });
        handles.push(complete_handle);

        // Wait for all tasks to complete
        futures::future::join_all(handles).await;

        // Final state should be consistent
        let final_progress = tracker.get_progress(repo_id).await.unwrap();

        // Should be in a terminal state
        assert!(matches!(
            final_progress.status,
            CrawlStatus::Failed | CrawlStatus::Completed
        ));

        // Should have completion timestamp
        assert!(final_progress.completed_at.is_some());
        assert!(!tracker.is_crawling(repo_id).await);
    }

    #[tokio::test]
    async fn test_uuid_collision_handling() {
        let tracker = ProgressTracker::new();

        // Use the same UUID for multiple operations (simulate collision)
        let repo_id = Uuid::from_u128(12345); // Fixed UUID

        // Start multiple "different" repositories with same ID
        tracker.start_crawl(repo_id, "repo-1".to_string()).await;

        // This should overwrite the previous one
        tracker.start_crawl(repo_id, "repo-2".to_string()).await;

        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(progress.repository_name, "repo-2");

        // Only one should be tracked
        let all_progress = tracker.get_all_active_progress().await;
        assert_eq!(all_progress.len(), 1);
    }

    #[tokio::test]
    async fn test_extreme_progress_values() {
        let tracker = ProgressTracker::new();
        let repo_id = Uuid::new_v4();

        tracker.start_crawl(repo_id, "extreme-values".to_string()).await;

        // Test with extreme values
        tracker.update_progress(repo_id, usize::MAX, Some(usize::MAX), usize::MAX).await;

        let progress = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(progress.files_processed, usize::MAX);
        assert_eq!(progress.files_total, Some(usize::MAX));
        assert_eq!(progress.files_indexed, usize::MAX);
        assert_eq!(progress.progress_percentage, 100.0); // Should be clamped

        // Test with zero total
        tracker.update_progress(repo_id, 100, Some(0), 50).await;
        let progress2 = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(progress2.progress_percentage, 0.0);

        // Test with processed > total
        tracker.update_progress(repo_id, 150, Some(100), 80).await;
        let progress3 = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(progress3.progress_percentage, 100.0); // Should be clamped
    }

    #[tokio::test]
    async fn test_long_running_operations_stability() {
        let tracker = Arc::new(ProgressTracker::new());
        let repo_id = Uuid::new_v4();

        tracker.start_crawl(repo_id, "long-running".to_string()).await;

        // Simulate a long-running operation with periodic updates
        let mut handles = vec![];

        // Background task that continuously checks status
        let tracker_checker = Arc::clone(&tracker);
        let check_handle = tokio::spawn(async move {
            for _ in 0..100 {
                tracker_checker.is_crawling(repo_id).await;
                tracker_checker.get_progress(repo_id).await;
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(check_handle);

        // Task that simulates progress updates over time
        let tracker_updater = Arc::clone(&tracker);
        let update_handle = tokio::spawn(async move {
            for i in 0..100 {
                tracker_updater.update_progress(repo_id, i, Some(100), i / 2).await;
                tracker_updater.set_current_file(repo_id, Some(format!("processing-{}.rs", i))).await;
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(update_handle);

        // Wait for all operations to complete
        futures::future::join_all(handles).await;

        // Verify final state is consistent
        let final_progress = tracker.get_progress(repo_id).await.unwrap();
        assert_eq!(final_progress.files_processed, 99);
        assert_eq!(final_progress.files_indexed, 49);
        assert_eq!(final_progress.progress_percentage, 99.0);
        assert!(final_progress.current_file.unwrap().starts_with("processing-"));
    }
}
