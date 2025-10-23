#[cfg(test)]
mod admin_api_tests {

    use axum_test::TestServer;
    use klask_rs::{
        api::admin::{AdminDashboardData, ContentStats, RecentActivity, RepositoryStats, SearchStats, SystemStats},
        auth::extractors::AppState,
    };
    use serde_json::Value;

    // Mock app state for testing
    #[allow(dead_code)]
    async fn create_test_app_state() -> AppState {
        // This would typically require setting up a test database and services
        // For now, we'll create a minimal mock
        unimplemented!("Test app state creation needed")
    }

    #[allow(dead_code)]
    async fn create_admin_test_server() -> TestServer {
        // Create a test server with admin routes
        // This requires proper setup of the admin router and authentication
        unimplemented!("Admin test server setup needed")
    }

    #[tokio::test]
    async fn test_system_stats_structure() {
        // Test the system stats data structure
        let stats = SystemStats {
            uptime_seconds: 3600,
            version: "0.1.0".to_string(),
            environment: "test".to_string(),
            database_status: "Connected".to_string(),
        };

        assert_eq!(stats.uptime_seconds, 3600);
        assert_eq!(stats.version, "0.1.0");
        assert_eq!(stats.environment, "test");
        assert_eq!(stats.database_status, "Connected");

        // Test serialization
        let json_value = serde_json::to_value(&stats).unwrap();
        assert!(json_value.is_object());
        assert_eq!(json_value["uptime_seconds"], 3600);
        assert_eq!(json_value["version"], "0.1.0");
        assert_eq!(json_value["environment"], "test");
        assert_eq!(json_value["database_status"], "Connected");
    }

    #[tokio::test]
    async fn test_repository_stats_structure() {
        let stats = RepositoryStats {
            total_repositories: 10,
            enabled_repositories: 8,
            disabled_repositories: 2,
            git_repositories: 7,
            gitlab_repositories: 2,
            filesystem_repositories: 1,
            recently_crawled: 5,
            never_crawled: 3,
        };

        // Test calculations
        assert_eq!(
            stats.total_repositories,
            stats.enabled_repositories + stats.disabled_repositories
        );
        assert_eq!(
            stats.total_repositories,
            stats.git_repositories + stats.gitlab_repositories + stats.filesystem_repositories
        );
        assert!(stats.recently_crawled + stats.never_crawled <= stats.total_repositories);

        // Test serialization
        let json_value = serde_json::to_value(&stats).unwrap();
        assert!(json_value.is_object());
        assert_eq!(json_value["total_repositories"], 10);
        assert_eq!(json_value["enabled_repositories"], 8);
        assert_eq!(json_value["disabled_repositories"], 2);
    }

    #[tokio::test]
    async fn test_content_stats_structure() {
        let stats = ContentStats {
            total_files: 1000,
            total_size_bytes: 5000000, // 5MB
            files_by_extension: vec![
                klask_rs::api::admin::ExtensionStat { extension: "rs".to_string(), count: 500, total_size: 2500000 },
                klask_rs::api::admin::ExtensionStat { extension: "js".to_string(), count: 300, total_size: 1500000 },
                klask_rs::api::admin::ExtensionStat { extension: "py".to_string(), count: 200, total_size: 1000000 },
            ],
            files_by_project: vec![
                klask_rs::api::admin::ProjectStat {
                    project: "project-a".to_string(),
                    file_count: 600,
                    total_size: 3000000,
                    disk_size_mb: 3.0,
                },
                klask_rs::api::admin::ProjectStat {
                    project: "project-b".to_string(),
                    file_count: 400,
                    total_size: 2000000,
                    disk_size_mb: 2.0,
                },
            ],
            recent_additions: 50,
        };

        // Test data consistency
        let extension_file_sum: i64 = stats.files_by_extension.iter().map(|e| e.count).sum();
        assert_eq!(extension_file_sum, stats.total_files);

        let extension_size_sum: i64 = stats.files_by_extension.iter().map(|e| e.total_size).sum();
        assert_eq!(extension_size_sum, stats.total_size_bytes);

        let project_file_sum: i64 = stats.files_by_project.iter().map(|p| p.file_count).sum();
        assert_eq!(project_file_sum, stats.total_files);

        let project_size_sum: i64 = stats.files_by_project.iter().map(|p| p.total_size).sum();
        assert_eq!(project_size_sum, stats.total_size_bytes);

        // Test serialization
        let json_value = serde_json::to_value(&stats).unwrap();
        assert!(json_value.is_object());
        assert_eq!(json_value["total_files"], 1000);
        assert_eq!(json_value["total_size_bytes"], 5000000);
        assert!(json_value["files_by_extension"].is_array());
        assert!(json_value["files_by_project"].is_array());
    }

    #[tokio::test]
    async fn test_search_stats_structure() {
        let stats = SearchStats {
            total_documents: 1000,
            index_size_mb: 50.5,
            avg_search_time_ms: Some(25.3),
            popular_queries: vec![
                klask_rs::api::admin::QueryStat { query: "function".to_string(), count: 150 },
                klask_rs::api::admin::QueryStat { query: "class".to_string(), count: 120 },
                klask_rs::api::admin::QueryStat { query: "main".to_string(), count: 100 },
            ],
            documents_by_repository: vec![],
        };

        assert_eq!(stats.total_documents, 1000);
        assert_eq!(stats.index_size_mb, 50.5);
        assert_eq!(stats.avg_search_time_ms, Some(25.3));
        assert_eq!(stats.popular_queries.len(), 3);

        // Test ordering of popular queries (should be by count)
        for i in 1..stats.popular_queries.len() {
            assert!(stats.popular_queries[i - 1].count >= stats.popular_queries[i].count);
        }

        // Test serialization
        let json_value = serde_json::to_value(&stats).unwrap();
        assert!(json_value.is_object());
        assert_eq!(json_value["total_documents"], 1000);
        assert_eq!(json_value["index_size_mb"], 50.5);
        assert_eq!(json_value["avg_search_time_ms"], 25.3);
    }

    #[tokio::test]
    async fn test_recent_activity_structure() {
        use chrono::Utc;

        let now = Utc::now();
        let activity = RecentActivity {
            recent_users: vec![
                klask_rs::api::admin::RecentUser {
                    username: "alice".to_string(),
                    email: "alice@example.com".to_string(),
                    last_seen: now,
                    role: "Admin".to_string(),
                },
                klask_rs::api::admin::RecentUser {
                    username: "bob".to_string(),
                    email: "bob@example.com".to_string(),
                    last_seen: now - chrono::Duration::hours(2),
                    role: "User".to_string(),
                },
            ],
            recent_repositories: vec![klask_rs::api::admin::RecentRepository {
                name: "new-repo".to_string(),
                url: "https://github.com/example/new-repo.git".to_string(),
                repository_type: "Git".to_string(),
                created_at: now,
            }],
            recent_crawls: vec![
                klask_rs::api::admin::RecentCrawl {
                    repository_name: "active-repo".to_string(),
                    last_crawled: Some(now - chrono::Duration::minutes(30)),
                    status: "Completed".to_string(),
                },
                klask_rs::api::admin::RecentCrawl {
                    repository_name: "old-repo".to_string(),
                    last_crawled: Some(now - chrono::Duration::hours(5)),
                    status: "Completed".to_string(),
                },
            ],
        };

        assert_eq!(activity.recent_users.len(), 2);
        assert_eq!(activity.recent_repositories.len(), 1);
        assert_eq!(activity.recent_crawls.len(), 2);

        // Test that recent activities are ordered by time (most recent first)
        for i in 1..activity.recent_users.len() {
            assert!(activity.recent_users[i - 1].last_seen >= activity.recent_users[i].last_seen);
        }

        // Test serialization
        let json_value = serde_json::to_value(&activity).unwrap();
        assert!(json_value.is_object());
        assert!(json_value["recent_users"].is_array());
        assert!(json_value["recent_repositories"].is_array());
        assert!(json_value["recent_crawls"].is_array());
    }

    #[tokio::test]
    async fn test_admin_dashboard_data_structure() {
        use klask_rs::repositories::user_repository::UserStats;

        let dashboard_data = AdminDashboardData {
            system: SystemStats {
                uptime_seconds: 3600,
                version: "0.1.0".to_string(),
                environment: "test".to_string(),
                database_status: "Connected".to_string(),
            },
            users: UserStats { total_users: 100, active_users: 85, admin_users: 5, recent_registrations: 10 },
            repositories: RepositoryStats {
                total_repositories: 50,
                enabled_repositories: 45,
                disabled_repositories: 5,
                git_repositories: 30,
                gitlab_repositories: 15,
                filesystem_repositories: 5,
                recently_crawled: 20,
                never_crawled: 10,
            },
            content: ContentStats {
                total_files: 10000,
                total_size_bytes: 50000000,
                files_by_extension: vec![],
                files_by_project: vec![],
                recent_additions: 500,
            },
            search: SearchStats {
                total_documents: 10000,
                index_size_mb: 100.5,
                avg_search_time_ms: Some(15.2),
                popular_queries: vec![],
                documents_by_repository: vec![],
            },
            recent_activity: RecentActivity {
                recent_users: vec![],
                recent_repositories: vec![],
                recent_crawls: vec![],
            },
        };

        // Test data consistency across sections
        assert_eq!(
            dashboard_data.content.total_files,
            dashboard_data.search.total_documents
        );

        // Test serialization of complete dashboard
        let json_value = serde_json::to_value(&dashboard_data).unwrap();
        assert!(json_value.is_object());
        assert!(json_value["system"].is_object());
        assert!(json_value["users"].is_object());
        assert!(json_value["repositories"].is_object());
        assert!(json_value["content"].is_object());
        assert!(json_value["search"].is_object());
        assert!(json_value["recent_activity"].is_object());
    }

    #[test]
    fn test_admin_api_error_handling() {
        // Test error scenarios for admin API functions

        // Test invalid database status
        let stats_with_error = SystemStats {
            uptime_seconds: 0,
            version: "0.1.0".to_string(),
            environment: "test".to_string(),
            database_status: "Disconnected".to_string(),
        };

        assert_eq!(stats_with_error.database_status, "Disconnected");

        // Test zero values
        let empty_repo_stats = RepositoryStats {
            total_repositories: 0,
            enabled_repositories: 0,
            disabled_repositories: 0,
            git_repositories: 0,
            gitlab_repositories: 0,
            filesystem_repositories: 0,
            recently_crawled: 0,
            never_crawled: 0,
        };

        assert_eq!(empty_repo_stats.total_repositories, 0);

        // Test empty collections
        let empty_content_stats = ContentStats {
            total_files: 0,
            total_size_bytes: 0,
            files_by_extension: vec![],
            files_by_project: vec![],
            recent_additions: 0,
        };

        assert!(empty_content_stats.files_by_extension.is_empty());
        assert!(empty_content_stats.files_by_project.is_empty());
    }

    #[test]
    fn test_admin_stats_calculations() {
        // Test various calculation scenarios

        // Test percentage calculations (would be used in UI)
        let repo_stats = RepositoryStats {
            total_repositories: 100,
            enabled_repositories: 85,
            disabled_repositories: 15,
            git_repositories: 60,
            gitlab_repositories: 30,
            filesystem_repositories: 10,
            recently_crawled: 40,
            never_crawled: 25,
        };

        let enabled_percentage =
            (repo_stats.enabled_repositories as f64 / repo_stats.total_repositories as f64) * 100.0;
        assert_eq!(enabled_percentage, 85.0);

        let git_percentage = (repo_stats.git_repositories as f64 / repo_stats.total_repositories as f64) * 100.0;
        assert_eq!(git_percentage, 60.0);

        // Test size calculations
        let content_stats = ContentStats {
            total_files: 1000,
            total_size_bytes: 1073741824, // 1GB in bytes
            files_by_extension: vec![],
            files_by_project: vec![],
            recent_additions: 100,
        };

        let size_in_mb = content_stats.total_size_bytes as f64 / 1024.0 / 1024.0;
        assert_eq!(size_in_mb, 1024.0);

        let avg_file_size = content_stats.total_size_bytes / content_stats.total_files;
        assert_eq!(avg_file_size, 1073741); // ~1MB per file
    }

    #[test]
    fn test_admin_data_validation() {
        // Test validation scenarios for admin data

        // Test that totals match sums
        let extension_stats = vec![
            klask_rs::api::admin::ExtensionStat { extension: "rs".to_string(), count: 100, total_size: 500000 },
            klask_rs::api::admin::ExtensionStat { extension: "js".to_string(), count: 50, total_size: 250000 },
        ];

        let total_files: i64 = extension_stats.iter().map(|e| e.count).sum();
        let total_size: i64 = extension_stats.iter().map(|e| e.total_size).sum();

        assert_eq!(total_files, 150);
        assert_eq!(total_size, 750000);

        // Test consistency between different views of the same data
        let content_stats = ContentStats {
            total_files,
            total_size_bytes: total_size,
            files_by_extension: extension_stats,
            files_by_project: vec![
                klask_rs::api::admin::ProjectStat {
                    project: "project-1".to_string(),
                    file_count: 120,
                    total_size: 600000,
                    disk_size_mb: 1.0,
                },
                klask_rs::api::admin::ProjectStat {
                    project: "project-2".to_string(),
                    file_count: 30,
                    total_size: 150000,
                    disk_size_mb: 1.0,
                },
            ],
            recent_additions: 20,
        };

        let project_total_files: i64 = content_stats.files_by_project.iter().map(|p| p.file_count).sum();
        let project_total_size: i64 = content_stats.files_by_project.iter().map(|p| p.total_size).sum();

        assert_eq!(project_total_files, content_stats.total_files);
        assert_eq!(project_total_size, content_stats.total_size_bytes);
    }

    #[tokio::test]
    async fn test_admin_api_response_format() {
        // Test the expected JSON response format

        // Test system stats response format
        let system_stats = SystemStats {
            uptime_seconds: 7200,
            version: "1.0.0".to_string(),
            environment: "production".to_string(),
            database_status: "Connected".to_string(),
        };

        let json = serde_json::to_string_pretty(&system_stats).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["uptime_seconds"].is_number());
        assert!(parsed["version"].is_string());
        assert!(parsed["environment"].is_string());
        assert!(parsed["database_status"].is_string());

        // Test that the JSON can be deserialized back
        let deserialized: SystemStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.uptime_seconds, system_stats.uptime_seconds);
        assert_eq!(deserialized.version, system_stats.version);
        assert_eq!(deserialized.environment, system_stats.environment);
        assert_eq!(deserialized.database_status, system_stats.database_status);
    }

    #[test]
    fn test_admin_stats_edge_cases() {
        // Test edge cases and boundary conditions

        // Test maximum values
        let max_stats = RepositoryStats {
            total_repositories: i64::MAX,
            enabled_repositories: i64::MAX - 1,
            disabled_repositories: 1,
            git_repositories: i64::MAX / 2,
            gitlab_repositories: i64::MAX / 2,
            filesystem_repositories: 0,
            recently_crawled: 0,
            never_crawled: i64::MAX,
        };

        // Should not panic with large values
        let json_result = serde_json::to_value(&max_stats);
        assert!(json_result.is_ok());

        // Test zero values
        let zero_stats = SearchStats {
            total_documents: 0,
            index_size_mb: 0.0,
            avg_search_time_ms: None,
            popular_queries: vec![],
            documents_by_repository: vec![],
        };

        assert_eq!(zero_stats.total_documents, 0);
        assert_eq!(zero_stats.index_size_mb, 0.0);
        assert!(zero_stats.avg_search_time_ms.is_none());
        assert!(zero_stats.popular_queries.is_empty());

        // Test negative values (shouldn't occur in practice but test handling)
        let content_with_negatives = ContentStats {
            total_files: -1, // This shouldn't happen but test serialization
            total_size_bytes: -1000,
            files_by_extension: vec![],
            files_by_project: vec![],
            recent_additions: -5,
        };

        let negative_json = serde_json::to_value(&content_with_negatives);
        assert!(negative_json.is_ok()); // Should serialize even if values are invalid
    }
}
