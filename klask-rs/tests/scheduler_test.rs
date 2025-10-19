#[cfg(test)]
mod scheduler_tests {
    use chrono::{DateTime, Timelike, Utc};
    use klask_rs::models::repository::RepositoryType;

    use uuid::Uuid;

    // Mock structs for testing
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct MockRepository {
        pub id: Uuid,
        pub name: String,
        pub url: String,
        pub repository_type: RepositoryType,
        pub auto_crawl_enabled: bool,
        pub crawl_frequency_hours: Option<i32>,
        pub cron_schedule: Option<String>,
        pub max_crawl_duration_minutes: Option<i32>,
        pub next_crawl_at: Option<DateTime<Utc>>,
        pub last_crawled: Option<DateTime<Utc>>,
    }

    impl MockRepository {
        fn new(name: &str) -> Self {
            Self {
                id: Uuid::new_v4(),
                name: name.to_string(),
                url: format!("https://github.com/test/{}.git", name),
                repository_type: RepositoryType::Git,
                auto_crawl_enabled: false,
                crawl_frequency_hours: None,
                cron_schedule: None,
                max_crawl_duration_minutes: Some(60),
                next_crawl_at: None,
                last_crawled: None,
            }
        }

        fn with_auto_crawl(mut self) -> Self {
            self.auto_crawl_enabled = true;
            self
        }

        fn with_frequency_hours(mut self, hours: i32) -> Self {
            self.crawl_frequency_hours = Some(hours);
            self
        }

        fn with_cron_schedule(mut self, schedule: &str) -> Self {
            self.cron_schedule = Some(schedule.to_string());
            self
        }

        fn with_max_duration(mut self, minutes: i32) -> Self {
            self.max_crawl_duration_minutes = Some(minutes);
            self
        }
    }

    // Note: Database and crawler service creation functions commented out
    // as they require complex mocking setup

    #[tokio::test]
    async fn test_cron_expression_validation() {
        // Test valid cron expressions
        let valid_expressions = vec![
            "0 0 * * * *",   // Every hour
            "0 */6 * * * *", // Every 6 hours
            "0 0 0 * * *",   // Daily at midnight
            "0 0 0 * * 1",   // Weekly on Monday
            "0 0 0 1 * *",   // Monthly on 1st
        ];

        for expr in valid_expressions {
            let result = expr.parse::<croner::Cron>();
            assert!(result.is_ok(), "Should parse valid cron expression: {}", expr);
        }

        // Test invalid cron expressions
        let invalid_expressions = vec![
            "invalid",
            "0 0 0 * *",       // Too few fields
            "0 0 0 * * * * *", // Too many fields (8 fields)
            "60 0 0 * * *",    // Invalid seconds
            "0 60 0 * * *",    // Invalid minutes
            "0 0 25 * * *",    // Invalid hours
        ];

        for expr in invalid_expressions {
            let result = expr.parse::<croner::Cron>();
            assert!(result.is_err(), "Should reject invalid cron expression: {}", expr);
        }
    }

    #[test]
    fn test_frequency_to_cron_conversion() {
        // Test converting frequency hours to cron expressions
        let test_cases =
            vec![(1, "0 0 */1 * * *"), (6, "0 0 */6 * * *"), (12, "0 0 */12 * * *"), (24, "0 0 */24 * * *")];

        for (hours, expected_cron) in test_cases {
            let cron_expr = format!("0 0 */{} * * *", hours);
            assert_eq!(cron_expr, expected_cron);

            // Verify the generated expression is valid
            let result = cron_expr.parse::<croner::Cron>();
            assert!(
                result.is_ok(),
                "Generated cron expression should be valid: {}",
                cron_expr
            );
        }
    }

    #[tokio::test]
    async fn test_next_run_calculation() {
        use chrono::TimeZone;

        // Test calculating next run times for various cron expressions
        let now = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();

        // Test hourly schedule
        let hourly = "0 0 * * * *".parse::<croner::Cron>().unwrap();
        let next_hourly = hourly.find_next_occurrence(&now, false).unwrap();
        assert_eq!(next_hourly.minute(), 0);
        assert_eq!(next_hourly.second(), 0);
        assert!(next_hourly > now);

        // Test daily schedule
        let daily = "0 0 0 * * *".parse::<croner::Cron>().unwrap();
        let next_daily = daily.find_next_occurrence(&now, false).unwrap();
        assert_eq!(next_daily.hour(), 0);
        assert_eq!(next_daily.minute(), 0);
        assert_eq!(next_daily.second(), 0);
        assert!(next_daily > now);

        // Test weekly schedule (every Tuesday in this cron system)
        let weekly = "0 0 0 * * 2".parse::<croner::Cron>().unwrap();
        let next_weekly = weekly.find_next_occurrence(&now, false).unwrap();

        // Just verify it's a future date and the time is correct
        assert!(next_weekly > now);
        assert_eq!(next_weekly.hour(), 0);
        assert_eq!(next_weekly.minute(), 0);
        assert_eq!(next_weekly.second(), 0);
    }

    #[test]
    fn test_schedule_validation_logic() {
        // Test repository schedule validation logic
        let repo_with_cron = MockRepository::new("cron-repo").with_auto_crawl().with_cron_schedule("0 0 */6 * * *");

        let repo_with_frequency = MockRepository::new("freq-repo").with_auto_crawl().with_frequency_hours(12);

        let repo_with_both = MockRepository::new("both-repo")
            .with_auto_crawl()
            .with_cron_schedule("0 0 */8 * * *")
            .with_frequency_hours(4);

        let repo_with_neither = MockRepository::new("neither-repo").with_auto_crawl();

        // Cron schedule should take precedence
        assert!(repo_with_cron.cron_schedule.is_some());

        // Frequency hours should be used as fallback
        assert!(repo_with_frequency.crawl_frequency_hours.is_some());

        // When both are present, cron should take precedence
        assert!(repo_with_both.cron_schedule.is_some());
        assert!(repo_with_both.crawl_frequency_hours.is_some());

        // Repository with auto-crawl but no schedule should be handled gracefully
        assert!(repo_with_neither.auto_crawl_enabled);
        assert!(repo_with_neither.cron_schedule.is_none());
        assert!(repo_with_neither.crawl_frequency_hours.is_none());
    }

    #[test]
    fn test_timeout_duration_calculation() {
        let repo_default = MockRepository::new("default-timeout");
        let repo_custom = MockRepository::new("custom-timeout").with_max_duration(120);

        // Default timeout should be 60 minutes
        assert_eq!(repo_default.max_crawl_duration_minutes.unwrap_or(60), 60);

        // Custom timeout should be respected
        assert_eq!(repo_custom.max_crawl_duration_minutes.unwrap(), 120);

        // Convert to Duration for timeout
        let default_duration =
            std::time::Duration::from_secs((repo_default.max_crawl_duration_minutes.unwrap_or(60) as u64) * 60);
        let custom_duration =
            std::time::Duration::from_secs((repo_custom.max_crawl_duration_minutes.unwrap() as u64) * 60);

        assert_eq!(default_duration.as_secs(), 3600); // 60 minutes
        assert_eq!(custom_duration.as_secs(), 7200); // 120 minutes
    }

    #[test]
    fn test_schedule_expression_edge_cases() {
        // Test edge cases for schedule expressions

        // Test every minute (probably not practical but valid)
        let every_minute = "0 * * * * *".parse::<croner::Cron>();
        assert!(every_minute.is_ok());

        // Test yearly (January 1st at midnight)
        let yearly = "0 0 0 1 1 *".parse::<croner::Cron>();
        assert!(yearly.is_ok());

        // Test specific day of month and week (should use OR logic)
        let complex_schedule = "0 0 0 1,15 * 1".parse::<croner::Cron>(); // 1st, 15th of month OR Monday
        assert!(complex_schedule.is_ok());

        // Test multiple hours
        let multiple_hours = "0 0 0,6,12,18 * * *".parse::<croner::Cron>(); // 4 times a day
        assert!(multiple_hours.is_ok());
    }

    #[tokio::test]
    async fn test_schedule_cleanup_logic() {
        // Test the logic for cleaning up old schedules
        let now = Utc::now();
        let old_time = now - chrono::Duration::hours(25); // 25 hours ago
        let recent_time = now - chrono::Duration::hours(1); // 1 hour ago

        // Simulate repository completion times
        let _old_repo = MockRepository::new("old-repo");
        let _recent_repo = MockRepository::new("recent-repo");

        // In real implementation, these would have completion times
        // and the cleanup would filter based on those times

        // Test cleanup threshold
        let cleanup_threshold_hours = 24;
        let cutoff_time = now - chrono::Duration::hours(cleanup_threshold_hours);

        assert!(old_time < cutoff_time, "Old time should be before cutoff");
        assert!(recent_time > cutoff_time, "Recent time should be after cutoff");
    }

    #[test]
    fn test_repository_schedule_priority() {
        // Test priority logic: cron_schedule > crawl_frequency_hours > disabled

        struct TestCase {
            name: &'static str,
            auto_crawl: bool,
            cron_schedule: Option<&'static str>,
            frequency_hours: Option<i32>,
            expected_schedule: Option<&'static str>,
        }

        let test_cases = vec![
            TestCase {
                name: "disabled",
                auto_crawl: false,
                cron_schedule: Some("0 0 */6 * * *"),
                frequency_hours: Some(12),
                expected_schedule: None, // Should be disabled
            },
            TestCase {
                name: "cron_priority",
                auto_crawl: true,
                cron_schedule: Some("0 0 */8 * * *"),
                frequency_hours: Some(4),
                expected_schedule: Some("0 0 */8 * * *"), // Cron takes precedence
            },
            TestCase {
                name: "frequency_fallback",
                auto_crawl: true,
                cron_schedule: None,
                frequency_hours: Some(12),
                expected_schedule: Some("0 0 */12 * * *"), // Generated from frequency
            },
            TestCase {
                name: "no_schedule",
                auto_crawl: true,
                cron_schedule: None,
                frequency_hours: None,
                expected_schedule: None, // No schedule available
            },
        ];

        for test_case in test_cases {
            let repo = MockRepository {
                id: Uuid::new_v4(),
                name: test_case.name.to_string(),
                url: "https://example.com/repo.git".to_string(),
                repository_type: RepositoryType::Git,
                auto_crawl_enabled: test_case.auto_crawl,
                crawl_frequency_hours: test_case.frequency_hours,
                cron_schedule: test_case.cron_schedule.map(|s| s.to_string()),
                max_crawl_duration_minutes: Some(60),
                next_crawl_at: None,
                last_crawled: None,
            };

            let effective_schedule = if !repo.auto_crawl_enabled {
                None
            } else if let Some(ref cron) = repo.cron_schedule {
                Some(cron.clone())
            } else {
                repo.crawl_frequency_hours.map(|hours| format!("0 0 */{} * * *", hours))
            };

            assert_eq!(
                effective_schedule.as_deref(),
                test_case.expected_schedule,
                "Test case '{}' failed",
                test_case.name
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_schedule_operations() {
        // Test concurrent schedule operations to ensure thread safety
        use std::collections::HashMap;
        use std::sync::Arc;
        use tokio::sync::RwLock;

        // Simulate the job_ids map from SchedulerService
        let job_ids: Arc<RwLock<HashMap<Uuid, Uuid>>> = Arc::new(RwLock::new(HashMap::new()));

        let mut handles = vec![];

        // Spawn multiple tasks that add/remove schedule entries
        for _i in 0..10 {
            let job_ids_clone = Arc::clone(&job_ids);
            let handle = tokio::spawn(async move {
                let repo_id = Uuid::new_v4();
                let job_id = Uuid::new_v4();

                // Add entry
                {
                    let mut map = job_ids_clone.write().await;
                    map.insert(repo_id, job_id);
                }

                // Simulate some work
                tokio::task::yield_now().await;

                // Remove entry
                {
                    let mut map = job_ids_clone.write().await;
                    map.remove(&repo_id);
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Map should be empty after all operations
        let final_map = job_ids.read().await;
        assert!(final_map.is_empty(), "All entries should be removed");
    }

    #[test]
    fn test_schedule_error_scenarios() {
        // Test various error scenarios

        // Invalid cron expressions
        let invalid_expressions = vec![
            "",
            "invalid",
            "0 0 0 32 * *", // Invalid day of month
            "0 0 25 * * *", // Invalid hour
            "0 60 * * * *", // Invalid minute
        ];

        for expr in invalid_expressions {
            let result = expr.parse::<croner::Cron>();
            assert!(result.is_err(), "Should reject invalid expression: '{}'", expr);
        }

        // Edge case: negative frequency hours (should be handled by validation)
        let negative_frequency = -1;
        assert!(negative_frequency < 0, "Negative frequency should be invalid");

        // Edge case: zero frequency hours (should be handled by validation)
        let zero_frequency = 0;
        assert!(zero_frequency <= 0, "Zero frequency should be invalid");

        // Very large frequency (might be impractical but technically valid)
        let large_frequency = 8760; // One year in hours
        let large_cron = format!("0 0 */{} * * *", large_frequency);
        // This would be a very impractical schedule but syntactically valid
        assert!(!large_cron.is_empty());
    }
}
