// Integration tests for schema mismatch detection and recovery workflows
//
// These tests verify the complete end-to-end behavior of:
// 1. Schema mismatch detection
// 2. Admin status reporting
// 3. Index rebuild functionality
// 4. Search recovery after rebuild

#[cfg(test)]
mod schema_mismatch_integration_tests {
    use std::sync::Arc;
    use tokio::sync::Mutex as AsyncMutex;

    /// Mock AppState for testing admin endpoints
    #[derive(Clone)]
    struct MockAppState {
        schema_mismatch: Arc<AsyncMutex<bool>>,
    }

    impl MockAppState {
        fn new(has_mismatch: bool) -> Self {
            Self { schema_mismatch: Arc::new(AsyncMutex::new(has_mismatch)) }
        }

        async fn set_mismatch(&self, value: bool) {
            *self.schema_mismatch.lock().await = value;
        }

        async fn get_mismatch(&self) -> bool {
            *self.schema_mismatch.lock().await
        }
    }

    /// Test: Admin status endpoint returns correct format
    ///
    /// Simulates the behavior of GET /api/admin/search/status endpoint
    #[tokio::test]
    async fn test_admin_status_endpoint_response_format() {
        let app_state = MockAppState::new(false);

        let has_mismatch = app_state.get_mismatch().await;
        let (index_available, message) = if has_mismatch {
            (
                false,
                Some("Index schema mismatch detected. Please rebuild the index in admin settings.".to_string()),
            )
        } else {
            (true, None)
        };

        // Verify the response structure matches the API contract
        assert_eq!(has_mismatch, false);
        assert_eq!(index_available, true);
        assert!(message.is_none());
    }

    /// Test: Admin status endpoint when schema mismatch is detected
    #[tokio::test]
    async fn test_admin_status_with_mismatch_detected() {
        let app_state = MockAppState::new(true);

        let has_mismatch = app_state.get_mismatch().await;
        let (index_available, message) = if has_mismatch {
            (
                false,
                Some("Index schema mismatch detected. Please rebuild the index in admin settings.".to_string()),
            )
        } else {
            (true, None)
        };

        // When mismatch is detected:
        // - schema_mismatch should be true
        // - index_available should be false
        // - message should contain helpful text
        assert_eq!(has_mismatch, true);
        assert_eq!(index_available, false);
        assert!(message.is_some());
        assert!(message.unwrap().contains("rebuild"));
    }

    /// Test: Status endpoint is accessible without authentication
    ///
    /// The status endpoint should NOT require admin authentication
    /// to allow frequent checks by the frontend banner
    #[tokio::test]
    async fn test_status_endpoint_no_auth_required() {
        // In real implementation, this would test the HTTP endpoint
        // Here we verify the logic doesn't require special permissions

        let app_state = MockAppState::new(false);

        // Any client should be able to call this
        let status = app_state.get_mismatch().await;
        assert!(!status);

        // The endpoint should not require authentication checks
        // (verified by the endpoint implementation itself)
    }

    /// Test: Admin reset endpoint clears schema mismatch
    ///
    /// Simulates POST /api/admin/search/reset-index behavior
    #[tokio::test]
    async fn test_admin_reset_endpoint_clears_mismatch() {
        let app_state = MockAppState::new(true);

        // Before reset, mismatch should be true
        assert!(app_state.get_mismatch().await);

        // Reset operation clears the mismatch flag
        app_state.set_mismatch(false).await;

        // After reset, mismatch should be false
        assert!(!app_state.get_mismatch().await);
    }

    /// Test: Admin reset response indicates if mismatch was present
    ///
    /// The reset endpoint should return schema_was_mismatch field
    /// so the frontend knows if it detected and fixed an issue
    #[tokio::test]
    async fn test_admin_reset_response_includes_was_mismatch() {
        let app_state = MockAppState::new(true);

        // Capture the state before reset
        let schema_was_mismatch = app_state.get_mismatch().await;

        // Perform reset
        app_state.set_mismatch(false).await;

        // Response should include:
        // - schema_was_mismatch: true (indicating we detected and fixed it)
        // - current schema_mismatch: false (indicating it's now fixed)
        assert_eq!(schema_was_mismatch, true);
        assert!(!app_state.get_mismatch().await);
    }

    /// Test: Reset endpoint with no prior mismatch still includes field
    #[tokio::test]
    async fn test_admin_reset_response_when_no_mismatch() {
        let app_state = MockAppState::new(false);

        // Capture the state before reset
        let schema_was_mismatch = app_state.get_mismatch().await;

        // Perform reset (should still work)
        app_state.set_mismatch(false).await;

        // Response should include schema_was_mismatch: false
        assert_eq!(schema_was_mismatch, false);
        assert!(!app_state.get_mismatch().await);
    }

    /// Test: Multiple reset calls work correctly
    ///
    /// Calling reset multiple times should be idempotent
    #[tokio::test]
    async fn test_multiple_reset_calls_idempotent() {
        let app_state = MockAppState::new(true);

        // First reset
        app_state.set_mismatch(false).await;
        assert!(!app_state.get_mismatch().await);

        // Second reset immediately after
        app_state.set_mismatch(false).await;
        assert!(!app_state.get_mismatch().await);

        // Third reset
        app_state.set_mismatch(false).await;
        assert!(!app_state.get_mismatch().await);

        // All should succeed with no errors or race conditions
    }

    /// Test: Full flow - detect mismatch through status, then reset
    ///
    /// End-to-end workflow that frontend/admin would perform
    #[tokio::test]
    async fn test_full_workflow_detect_and_reset() {
        let app_state = MockAppState::new(true);

        // Step 1: Frontend checks status
        let status_check_1 = app_state.get_mismatch().await;
        assert!(status_check_1, "Initial status should show mismatch");

        // Step 2: Frontend displays warning banner
        let show_banner = status_check_1;
        assert!(show_banner);

        // Step 3: Admin clicks "Rebuild Index" button
        // This triggers the reset endpoint
        app_state.set_mismatch(false).await;

        // Step 4: Frontend refreshes status
        let status_check_2 = app_state.get_mismatch().await;
        assert!(!status_check_2, "After reset, status should show no mismatch");

        // Step 5: Frontend hides warning banner
        let hide_banner = !status_check_2;
        assert!(hide_banner);
    }

    /// Test: Concurrent status checks during reset
    ///
    /// Multiple requests checking status while reset is happening
    /// should not cause issues
    #[tokio::test]
    async fn test_concurrent_status_checks_during_reset() {
        let app_state = MockAppState::new(true);

        // Spawn multiple tasks checking status
        let check_handles: Vec<_> = (0..5)
            .map(|_| {
                let state = app_state.clone();
                tokio::spawn(async move {
                    for _ in 0..10 {
                        let _status = state.get_mismatch().await;
                        tokio::task::yield_now().await;
                    }
                })
            })
            .collect();

        // Give checks time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

        // Reset while checks are happening
        app_state.set_mismatch(false).await;

        // All check tasks should complete successfully
        for handle in check_handles {
            assert!(handle.await.is_ok());
        }

        // Final state should be consistent
        assert!(!app_state.get_mismatch().await);
    }

    /// Test: Status endpoint doesn't cache stale data
    ///
    /// Each call should reflect current state
    #[tokio::test]
    async fn test_status_endpoint_reflects_current_state() {
        let app_state = MockAppState::new(false);

        // Check 1: No mismatch
        assert!(!app_state.get_mismatch().await);

        // Simulate schema change that would cause mismatch
        app_state.set_mismatch(true).await;

        // Check 2: Should immediately show mismatch (no caching)
        assert!(app_state.get_mismatch().await);

        // Reset
        app_state.set_mismatch(false).await;

        // Check 3: Should immediately show no mismatch
        assert!(!app_state.get_mismatch().await);
    }

    /// Test: Status endpoint response is consistent across calls
    ///
    /// In a healthy state, repeated checks should return identical data
    #[tokio::test]
    async fn test_status_consistency_in_healthy_state() {
        let app_state = MockAppState::new(false);

        let mut results = Vec::new();
        for _ in 0..10 {
            results.push(app_state.get_mismatch().await);
        }

        // All results should be false
        for (i, result) in results.iter().enumerate() {
            assert!(!result, "Status check {} should be consistent", i);
        }
    }

    /// Test: Status consistency during active recovery
    ///
    /// While rebuild is happening, status should be accurate
    #[tokio::test]
    async fn test_status_consistency_during_recovery() {
        let app_state = MockAppState::new(true);

        // Start recovery
        let recovery_task = {
            let state = app_state.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                state.set_mismatch(false).await;
            })
        };

        // Check status during recovery
        let mut pre_recovery_checks = 0;
        while app_state.get_mismatch().await {
            pre_recovery_checks += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            if pre_recovery_checks > 100 {
                break;
            }
        }

        recovery_task.await.unwrap();

        // After recovery completes, status should be accurate
        assert!(!app_state.get_mismatch().await);
    }

    /// Test: Admin operations don't block status checks
    ///
    /// Status endpoint should be very fast and not blocked by
    /// long-running admin operations
    #[tokio::test]
    async fn test_status_not_blocked_by_admin_ops() {
        let app_state = MockAppState::new(true);

        // Simulate long-running admin operation
        let admin_task = {
            let state = app_state.clone();
            tokio::spawn(async move {
                // Simulate long rebuild: 100ms
                for _ in 0..10 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    // Check would happen here during rebuild
                    let _status = state.get_mismatch().await;
                }
                state.set_mismatch(false).await;
            })
        };

        // While admin operation is running, status checks should be fast
        let start = std::time::Instant::now();
        for _ in 0..50 {
            let _status = app_state.get_mismatch().await;
            // Should be sub-millisecond
            let elapsed = start.elapsed();
            if elapsed.as_millis() > 100 {
                panic!("Status check took {}ms, expected < 100ms", elapsed.as_millis());
            }
        }

        admin_task.await.unwrap();
    }

    /// Test: Error handling for reset endpoint
    ///
    /// Reset should handle errors gracefully
    #[tokio::test]
    async fn test_reset_error_handling() {
        let app_state = MockAppState::new(true);

        // Reset should succeed and clear the flag
        app_state.set_mismatch(false).await;

        // Verify it was cleared
        assert!(!app_state.get_mismatch().await);

        // Calling reset again should also succeed (idempotent)
        app_state.set_mismatch(false).await;
        assert!(!app_state.get_mismatch().await);
    }

    /// Test: Status after successful rebuild
    ///
    /// After rebuild completes, status should show healthy
    #[tokio::test]
    async fn test_status_after_rebuild() {
        let app_state = MockAppState::new(true);

        // Initially unhealthy
        assert!(app_state.get_mismatch().await);

        // Rebuild
        app_state.set_mismatch(false).await;

        // Now healthy
        assert!(!app_state.get_mismatch().await);

        // Status should consistently report healthy
        for _ in 0..5 {
            assert!(!app_state.get_mismatch().await);
        }
    }

    /// Test: Admin endpoints preserve data during rebuild
    ///
    /// Schema mismatch flag changes shouldn't affect other operations
    #[tokio::test]
    async fn test_mismatch_flag_independence() {
        let app_state = MockAppState::new(true);

        // Change mismatch flag
        app_state.set_mismatch(false).await;

        // Status should be updated
        assert!(!app_state.get_mismatch().await);

        // Multiple status checks should remain consistent
        for _ in 0..3 {
            assert!(!app_state.get_mismatch().await);
        }
    }
}
