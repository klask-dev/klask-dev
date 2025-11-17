#[cfg(test)]
mod search_repository_tests {
    use klask_rs::services::search::{FileData, SearchQuery, SearchService};
    use std::sync::LazyLock;
    use tempfile::TempDir;
    use tokio::sync::Mutex as AsyncMutex;
    use uuid::Uuid;

    // Global mutex to ensure tests don't interfere with each other
    static TEST_MUTEX: LazyLock<AsyncMutex<()>> = LazyLock::new(|| AsyncMutex::new(()));

    async fn create_test_search_service() -> (SearchService, TempDir, tokio::sync::MutexGuard<'static, ()>) {
        let _guard = TEST_MUTEX.lock().await;
        let temp_dir = TempDir::new().unwrap();
        let test_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
        let index_path = temp_dir.path().join(format!("test_index_{}", test_id));
        let service = SearchService::new(&index_path).expect("Failed to create search service");
        (service, temp_dir, _guard)
    }

    #[tokio::test]
    async fn test_index_files_with_different_repositories() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files from different repositories
        let repositories = [
            ("klask-io/klask", "Klask search engine"),
            ("rust-lang/rust", "Rust compiler"),
            ("facebook/react", "React framework"),
        ];

        for (i, (repo_name, content)) in repositories.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file{}.rs", i),
                file_path: &format!("src/file{}.rs", i),
                content,
                repository: repo_name,
                project: repo_name,
                version: "main",
                extension: "rs",
                size: 1024,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Verify all files are indexed
        let doc_count = service.get_document_count().unwrap();
        assert_eq!(doc_count, 3, "Should have three documents indexed");

        // Search without filter - should find all
        let query = SearchQuery {
            query: "search OR compiler OR framework".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: true,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let results = service.search(query).await.unwrap();
        assert!(results.total >= 1, "Should find results from all repositories");

        // Verify facets include all projects
        if let Some(facets) = results.facets {
            assert!(!facets.projects.is_empty(), "Should have project facets");
            let project_names: Vec<String> = facets.projects.iter().map(|(name, _)| name.clone()).collect();
            assert!(
                project_names.contains(&"klask-io/klask".to_string()),
                "Should include klask-io/klask in facets"
            );
        }
    }

    #[tokio::test]
    async fn test_search_with_repository_filter() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files from different repositories with common search term
        let repositories = [
            ("klask-io/klask", "fn search() { /* Klask search */ }"),
            ("rust-lang/rust", "fn search() { /* Rust search */ }"),
            ("facebook/react", "function search() { /* React search */ }"),
        ];

        for (i, (repo_name, content)) in repositories.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("search{}.rs", i),
                file_path: &format!("src/search{}.rs", i),
                content,
                repository: repo_name,
                project: repo_name,
                version: "main",
                extension: "rs",
                size: 512,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with repository filter
        let query = SearchQuery {
            query: "search".to_string(),
            project_filter: Some("klask-io/klask".to_string()),
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: false,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 1, "Should find exactly one result from klask-io/klask");
        assert_eq!(results.results[0].project, "klask-io/klask");
        assert!(results.results[0].content_snippet.contains("Klask"));
    }

    #[tokio::test]
    async fn test_search_returns_repository_name() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let repo_name = "test-org/test-repo";
        let file_data = FileData {
            file_id,
            file_name: "main.rs",
            file_path: "src/main.rs",
            content: "fn main() { println!(\"Hello\"); }",
            repository: repo_name,
            project: repo_name,
            version: "v1.0.0",
            extension: "rs",
            size: 256,
        };
        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search and verify repository name is returned
        let query = SearchQuery {
            query: "main".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: false,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.results.len(), 1);
        assert_eq!(
            results.results[0].project, repo_name,
            "Result should include repository name"
        );
    }

    #[tokio::test]
    async fn test_filter_by_multiple_repositories() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files from multiple repositories
        let repositories = ["repo-a", "repo-b", "repo-c", "repo-d"];

        for (i, repo) in repositories.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file{}.rs", i),
                file_path: &format!("src/file{}.rs", i),
                content: "test content",
                repository: repo,
                project: repo,
                version: "main",
                extension: "rs",
                size: 1024,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Filter by multiple repositories (comma-separated)
        let query = SearchQuery {
            query: "test".to_string(),
            project_filter: Some("repo-a,repo-c".to_string()),
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: false,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(
            results.total, 2,
            "Should find exactly two results from repo-a and repo-c"
        );

        let project_names: Vec<String> = results.results.iter().map(|r| r.project.clone()).collect();
        assert!(project_names.contains(&"repo-a".to_string()));
        assert!(project_names.contains(&"repo-c".to_string()));
        assert!(!project_names.contains(&"repo-b".to_string()));
        assert!(!project_names.contains(&"repo-d".to_string()));
    }

    #[tokio::test]
    async fn test_files_without_repository_legacy() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index a file with empty project (simulating legacy data)
        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "legacy.rs",
            file_path: "src/legacy.rs",
            content: "legacy content",
            repository: "",
            project: "", // Empty repository
            version: "main",
            extension: "rs",
            size: 768,
        };
        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search should still work
        let query = SearchQuery {
            query: "legacy".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: false,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 1);
        assert_eq!(results.results[0].project, "");
    }

    #[tokio::test]
    async fn test_repository_metrics_in_facets() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index multiple files in same repository
        for i in 0..5 {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file{}.rs", i),
                file_path: &format!("src/file{}.rs", i),
                content: "test content",
                repository: "klask-io/klask",
                project: "klask-io/klask",
                version: "main",
                extension: "rs",
                size: 2048,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        // Index files in another repository
        for i in 0..3 {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file{}.rs", i),
                file_path: &format!("src/file{}.rs", i),
                content: "test content",
                repository: "rust-lang/rust",
                project: "rust-lang/rust",
                version: "main",
                extension: "rs",
                size: 1536,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with facets enabled
        let query = SearchQuery {
            query: "test".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: true,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let results = service.search(query).await.unwrap();

        // Verify facets include correct counts
        if let Some(facets) = results.facets {
            let klask_count =
                facets.projects.iter().find(|(name, _)| name == "klask-io/klask").map(|(_, count)| *count);

            let rust_count = facets.projects.iter().find(|(name, _)| name == "rust-lang/rust").map(|(_, count)| *count);

            assert_eq!(klask_count, Some(5), "Should have 5 files in klask-io/klask");
            assert_eq!(rust_count, Some(3), "Should have 3 files in rust-lang/rust");
        } else {
            panic!("Facets should be present");
        }
    }

    #[tokio::test]
    async fn test_performance_with_many_repositories() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files from many repositories
        let num_repos = 50;
        let files_per_repo = 10;

        for repo_idx in 0..num_repos {
            for file_idx in 0..files_per_repo {
                let file_id = Uuid::new_v4();
                let repo_name = format!("org-{}/repo-{}", repo_idx / 10, repo_idx);
                let file_data = FileData {
                    file_id,
                    file_name: &format!("file{}.rs", file_idx),
                    file_path: &format!("src/file{}.rs", file_idx),
                    content: "fn test() { println!(\"test\"); }",
                    repository: &repo_name,
                    project: &repo_name,
                    version: "main",
                    extension: "rs",
                    size: 4096,
                };
                service.upsert_file(file_data).await.unwrap();
            }
        }

        service.commit().await.unwrap();

        // Measure search performance
        let start = std::time::Instant::now();

        let query = SearchQuery {
            query: "test".to_string(),
            project_filter: Some("org-2/repo-25".to_string()),
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: true,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let results = service.search(query).await.unwrap();
        let duration = start.elapsed();

        // Verify results
        assert_eq!(results.total, 10, "Should find 10 files in org-2/repo-25");

        // Performance check - should be fast even with many repos
        assert!(
            duration.as_millis() < 1000,
            "Search should complete in less than 1 second, took {}ms",
            duration.as_millis()
        );

        println!("Search with {} repositories took {}ms", num_repos, duration.as_millis());
    }

    #[tokio::test]
    async fn test_delete_project_documents() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files from multiple repositories
        let repos = vec!["repo-to-delete", "repo-to-keep"];

        for repo in &repos {
            for i in 0..3 {
                let file_id = Uuid::new_v4();
                let file_data = FileData {
                    file_id,
                    file_name: &format!("file{}.rs", i),
                    file_path: &format!("src/file{}.rs", i),
                    content: "test content",
                    repository: repo,
                    project: repo,
                    version: "main",
                    extension: "rs",
                    size: 1024,
                };
                service.upsert_file(file_data).await.unwrap();
            }
        }

        service.commit().await.unwrap();

        // Verify initial count
        assert_eq!(service.get_document_count().unwrap(), 6);

        // Delete one repository
        let deleted_count = service.delete_project_documents("repo-to-delete").await.unwrap();
        assert_eq!(deleted_count, 3, "Should delete 3 documents");

        // Verify remaining count
        assert_eq!(service.get_document_count().unwrap(), 3, "Should have 3 documents left");

        // Search should only find repo-to-keep
        let query = SearchQuery {
            query: "test".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: false,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 3);
        assert!(results.results.iter().all(|r| r.project == "repo-to-keep"));
    }

    #[tokio::test]
    async fn test_update_project_name() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let old_name = "old-repo-name";
        let new_name = "new-repo-name";

        // Index files with old repository name
        for i in 0..3 {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file{}.rs", i),
                file_path: &format!("src/file{}.rs", i),
                content: "test content",
                repository: old_name,
                project: old_name,
                version: "main",
                extension: "rs",
                size: 2048,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Update repository name
        let updated_count = service.update_project_name(old_name, new_name).await.unwrap();
        assert_eq!(updated_count, 3, "Should update 3 documents");

        // Search with new name should find results
        let query = SearchQuery {
            query: "test".to_string(),
            project_filter: Some(new_name.to_string()),
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: false,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 3);
        assert!(results.results.iter().all(|r| r.project == new_name));

        // Search with old name should find nothing
        let old_query = SearchQuery {
            query: "test".to_string(),
            project_filter: Some(old_name.to_string()),
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: false,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let old_results = service.search(old_query).await.unwrap();
        assert_eq!(old_results.total, 0, "Should not find results with old name");
    }

    #[tokio::test]
    async fn test_combined_filters_with_repository() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files with different combinations
        let files = vec![
            ("repo-a", "v1.0", "rs"),
            ("repo-a", "v1.0", "js"),
            ("repo-a", "v2.0", "rs"),
            ("repo-b", "v1.0", "rs"),
            ("repo-b", "v2.0", "js"),
        ];

        for (i, (repo, version, ext)) in files.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file{}.{}", i, ext),
                file_path: &format!("src/file{}.{}", i, ext),
                content: "test content",
                repository: repo,
                project: repo,
                version,
                extension: ext,
                size: 3072,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Test: repo-a + v1.0 + rs
        let query = SearchQuery {
            query: "test".to_string(),
            project_filter: Some("repo-a".to_string()),
            version_filter: Some("v1.0".to_string()),
            extension_filter: Some("rs".to_string()),
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: false,
            fuzzy_search: false,
            regex_search: false,
        regex_flags: None,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 1, "Should find exactly one result matching all filters");
        assert_eq!(results.results[0].project, "repo-a");
        assert_eq!(results.results[0].version, "v1.0");
        assert_eq!(results.results[0].extension, "rs");
    }
}
