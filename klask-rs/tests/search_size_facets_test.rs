#[cfg(test)]
mod search_size_facets_tests {
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

    // Test 1: Basic Size Range Facets
    // Verify that size_ranges facets are returned and counts match the indexed files
    #[tokio::test]
    async fn test_basic_size_range_facets() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files with specific sizes distributed across buckets
        let test_files = vec![
            ("file1.txt", 512, "small file"),         // < 1 KB (0-1023)
            ("file2.txt", 2048, "medium file"),       // 1 KB - 10 KB
            ("file3.txt", 50000, "large file"),       // 10 KB - 100 KB
            ("file4.txt", 500000, "very large file"), // 100 KB - 1 MB
            ("file5.txt", 5000000, "huge file"),      // 1 MB - 10 MB
            ("file6.txt", 50000000, "enormous file"), // > 10 MB
        ];

        for (_i, (name, size, content)) in test_files.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: name,
                file_path: &format!("src/{}", name),
                content,
                repository: "test-repo",
                project: "test-repo",
                version: "main",
                extension: "txt",
                size: *size,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with facets enabled
        let query = SearchQuery {
            query: "file".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 6, "Should find all 6 files");

        // Verify facets are present
        let facets = results.facets.expect("Facets should be present");
        assert!(!facets.size_ranges.is_empty(), "Size ranges should not be empty");

        // Verify all 6 size buckets exist (even if some are zero)
        assert_eq!(facets.size_ranges.len(), 6, "Should have exactly 6 size buckets");

        // Expected counts: one file in each bucket
        let expected_counts = vec![1, 1, 1, 1, 1, 1];
        for (i, (label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket '{}' should have count {}, got {}",
                label, expected_counts[i], count
            );
        }
    }

    // Test 2: Size Bucket Boundaries - Test exact boundary conditions
    #[tokio::test]
    async fn test_size_bucket_boundaries() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Test exact boundary values - all with searchable content
        let boundary_files = vec![
            ("boundary_0.txt", 0, "boundary searchable"),           // 0 bytes - in < 1KB
            ("boundary_1023.txt", 1023, "boundary searchable"),     // Exactly 1023 - in < 1KB
            ("boundary_1024.txt", 1024, "boundary searchable"),     // Exactly 1024 - in 1-10KB
            ("boundary_10239.txt", 10239, "boundary searchable"),   // Just before 10KB
            ("boundary_10240.txt", 10240, "boundary searchable"),   // Exactly 10KB
            ("boundary_102399.txt", 102399, "boundary searchable"), // Just before 100KB
            ("boundary_102400.txt", 102400, "boundary searchable"), // Exactly 100KB
            ("boundary_1mb_minus.txt", 1048575, "boundary searchable"), // Just before 1MB
            ("boundary_1mb.txt", 1048576, "boundary searchable"),   // Exactly 1MB
            ("boundary_10mb_minus.txt", 10485759, "boundary searchable"), // Just before 10MB
            ("boundary_10mb.txt", 10485760, "boundary searchable"), // Exactly 10MB
            ("boundary_over_10mb.txt", 10485761, "boundary searchable"), // Just over 10MB
        ];

        for (_i, (name, size, content)) in boundary_files.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: name,
                file_path: &format!("src/{}", name),
                content,
                repository: "boundary-repo",
                project: "boundary-repo",
                version: "main",
                extension: "txt",
                size: *size,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with facets enabled
        let query = SearchQuery {
            query: "searchable".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 100,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        let facets = results.facets.expect("Facets should be present");

        // Expected distribution across buckets
        // < 1 KB: files at 0, 1023 = 2 files
        // 1 KB - 10 KB: files at 1024, 10239 = 2 files
        // 10 KB - 100 KB: files at 10240, 102399 = 2 files
        // 100 KB - 1 MB: files at 102400, 1048575 = 2 files
        // 1 MB - 10 MB: files at 1048576, 10485759 = 2 files
        // > 10 MB: files at 10485760, 10485761 = 2 files
        let expected_counts = vec![2, 2, 2, 2, 2, 2];

        for (i, (label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket '{}' (index {}) should have count {}, got {}",
                label, i, expected_counts[i], count
            );
        }
    }

    // Test 3: Size Distribution with Multiple Files per Bucket
    #[tokio::test]
    async fn test_size_bucket_distribution() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Create multiple files in specific buckets

        // Add 5 files to < 1 KB bucket
        for i in 0..5 {
            let file_id = Uuid::new_v4();
            let size = 100 + (i * 100);
            let file_data = FileData {
                file_id,
                file_name: &format!("small_{}.txt", i),
                file_path: &format!("src/small_{}.txt", i),
                content: "content",
                repository: "dist-repo",
                project: "dist-repo",
                version: "main",
                extension: "txt",
                size: size as u64,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        // Add 3 files to 1 KB - 10 KB bucket
        for i in 0..3 {
            let file_id = Uuid::new_v4();
            let size = 2048 + (i * 1024);
            let file_data = FileData {
                file_id,
                file_name: &format!("medium_{}.txt", i),
                file_path: &format!("src/medium_{}.txt", i),
                content: "content",
                repository: "dist-repo",
                project: "dist-repo",
                version: "main",
                extension: "txt",
                size: size as u64,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        // Add 2 files to 10 KB - 100 KB bucket
        for i in 0..2 {
            let file_id = Uuid::new_v4();
            let size = 50000 + (i * 10000);
            let file_data = FileData {
                file_id,
                file_name: &format!("large_{}.txt", i),
                file_path: &format!("src/large_{}.txt", i),
                content: "content",
                repository: "dist-repo",
                project: "dist-repo",
                version: "main",
                extension: "txt",
                size: size as u64,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with facets
        let query = SearchQuery {
            query: "content".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 100,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        let facets = results.facets.expect("Facets should be present");

        // Verify expected counts
        let expected_counts = vec![5, 3, 2, 0, 0, 0];
        for (i, (label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket '{}' should have {} files, got {}",
                label, expected_counts[i], count
            );
        }
    }

    // Test 4: Size Facets with Project Filter
    // Verify size_ranges reflects filtered results when project filter is applied
    #[tokio::test]
    async fn test_size_facets_with_project_filter() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files from multiple projects
        for project_idx in 0..2 {
            let project = if project_idx == 0 { "project-a" } else { "project-b" };

            // Add files of different sizes to each project
            for size_category in 0..3 {
                let file_id = Uuid::new_v4();
                let size = match size_category {
                    0 => 500,   // < 1 KB
                    1 => 5000,  // 1 KB - 10 KB
                    _ => 50000, // 10 KB - 100 KB
                };
                let file_data = FileData {
                    file_id,
                    file_name: &format!("file_{}_cat_{}.txt", project_idx, size_category),
                    file_path: &format!("src/file_{}_cat_{}.txt", project_idx, size_category),
                    content: "searchable content",
                    repository: project,
                    project,
                    version: "main",
                    extension: "txt",
                    size,
                };
                service.upsert_file(file_data).await.unwrap();
            }
        }

        service.commit().await.unwrap();

        // Search with project-a filter and facets enabled
        let query = SearchQuery {
            query: "content".to_string(),
            project_filter: Some("project-a".to_string()),
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 3, "Should find 3 files from project-a");

        let facets = results.facets.expect("Facets should be present");

        // Verify size_ranges only reflects project-a results
        // < 1 KB: 1, 1 KB - 10 KB: 1, 10 KB - 100 KB: 1, rest: 0
        let expected_counts = vec![1, 1, 1, 0, 0, 0];
        for (i, (label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket '{}' should show {} files from project-a",
                label, expected_counts[i]
            );
        }
    }

    // Test 5: Size Facets with Extension Filter
    // Verify size_ranges reflects filtered results when extension filter is applied
    #[tokio::test]
    async fn test_size_facets_with_extension_filter() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files with different extensions and sizes
        let extensions = vec!["rs", "js", "py"];
        for ext in &extensions {
            for i in 0..2 {
                let file_id = Uuid::new_v4();
                let size = match i {
                    0 => 256,  // < 1 KB
                    _ => 5000, // 1 KB - 10 KB
                };
                let file_data = FileData {
                    file_id,
                    file_name: &format!("file_{}.{}", i, ext),
                    file_path: &format!("src/file_{}.{}", i, ext),
                    content: "test code",
                    repository: "multi-ext-repo",
                    project: "multi-ext-repo",
                    version: "main",
                    extension: ext,
                    size,
                };
                service.upsert_file(file_data).await.unwrap();
            }
        }

        service.commit().await.unwrap();

        // Search with extension filter for "rs" files only
        let query = SearchQuery {
            query: "code".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: Some("rs".to_string()),
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 2, "Should find 2 RS files");

        let facets = results.facets.expect("Facets should be present");

        // Verify size_ranges only reflects RS files: 1 < 1KB, 1 in 1-10KB, rest 0
        let expected_counts = vec![1, 1, 0, 0, 0, 0];
        for (i, (label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket '{}' should show {} RS files",
                label, expected_counts[i]
            );
        }
    }

    // Test 6: Size Facets with Multiple Filters
    #[tokio::test]
    async fn test_size_facets_with_multiple_filters() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index diverse files
        let test_data = vec![
            ("repo-a", "v1.0", "rs", 512),   // < 1 KB
            ("repo-a", "v1.0", "rs", 5000),  // 1-10 KB
            ("repo-a", "v1.0", "js", 50000), // 10-100 KB
            ("repo-a", "v2.0", "rs", 512),   // < 1 KB
            ("repo-b", "v1.0", "rs", 512),   // < 1 KB
        ];

        for (i, (repo, version, ext, size)) in test_data.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file_{}.{}", i, ext),
                file_path: &format!("src/file_{}.{}", i, ext),
                content: "test",
                repository: repo,
                project: repo,
                version,
                extension: ext,
                size: *size,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with multiple filters: repo-a, v1.0, rs only
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
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 2, "Should find 2 matching files");

        let facets = results.facets.expect("Facets should be present");

        // Both files are in different size buckets: 512 < 1KB, 5000 in 1-10KB
        let expected_counts = vec![1, 1, 0, 0, 0, 0];
        for (i, (_label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket {} should have {} files with multiple filters",
                i, expected_counts[i]
            );
        }
    }

    // Test 7: Size Filter Independence - Size facets show ALL buckets regardless of size filter
    // This test demonstrates that size filters don't affect size_ranges facets display
    #[tokio::test]
    async fn test_size_facets_independent_of_size_filter() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files across all size ranges
        let sizes = vec![
            512,      // < 1 KB
            2048,     // 1-10 KB
            50000,    // 10-100 KB
            500000,   // 100KB-1MB
            5000000,  // 1-10MB
            50000000, // > 10MB
        ];

        for (i, size) in sizes.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file_{}.txt", i),
                file_path: &format!("src/file_{}.txt", i),
                content: "searchable",
                repository: "size-repo",
                project: "size-repo",
                version: "main",
                extension: "txt",
                size: *size,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with size filter (min_size) and facets enabled
        // Size ranges facets should NOT be affected by the size filter
        let query = SearchQuery {
            query: "searchable".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: Some(10000), // Only files >= 10KB in results
            max_size: None,
            limit: 100,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        // Only 50KB, 500KB, 5MB, 50MB files should match search results (4 files)
        assert_eq!(results.total, 4, "Should find 4 files >= 10KB");

        let facets = results.facets.expect("Facets should be present");

        // IMPORTANT: Size facets show ALL files matching text query, NOT filtered by size
        // This is by design - size facets are independent of size filters
        // < 1 KB: 1 (512)
        // 1-10 KB: 1 (2048)
        // 10-100 KB: 1 (50KB)
        // 100KB-1MB: 1 (500KB)
        // 1-10MB: 1 (5MB)
        // > 10MB: 1 (50MB)
        let expected_counts = vec![1, 1, 1, 1, 1, 1];
        for (i, (_label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket {} should have {} files (size filter should NOT affect facets)",
                i, expected_counts[i]
            );
        }
    }

    // Test 8: Empty Results
    // Search returning no results should still have size_ranges (all zeros)
    #[tokio::test]
    async fn test_size_facets_with_empty_results() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index a single file
        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "file.txt",
            file_path: "src/file.txt",
            content: "this is a test",
            repository: "empty-repo",
            project: "empty-repo",
            version: "main",
            extension: "txt",
            size: 1024,
        };
        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for something that doesn't exist
        let query = SearchQuery {
            query: "nonexistent_string_12345".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 0, "Should find no results");

        let facets = results.facets.expect("Facets should be present even with no results");

        // All size buckets should have 0 count
        let expected_counts = vec![0, 0, 0, 0, 0, 0];
        for (i, (_label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket {} should have 0 files when no results match",
                i
            );
        }
    }

    // Test 9: No Files in Some Buckets
    // Verify facets show 0 for buckets with no files
    #[tokio::test]
    async fn test_size_facets_with_empty_buckets() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files only in < 1KB and > 10MB buckets
        let sizes = vec![
            512,      // < 1 KB
            50000000, // > 10MB
        ];

        for (i, size) in sizes.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file_{}.txt", i),
                file_path: &format!("src/file_{}.txt", i),
                content: "content",
                repository: "sparse-repo",
                project: "sparse-repo",
                version: "main",
                extension: "txt",
                size: *size,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with facets
        let query = SearchQuery {
            query: "content".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        let facets = results.facets.expect("Facets should be present");

        // Expected: 1 file < 1KB, 0 in middle buckets, 1 file > 10MB
        let expected_counts = vec![1, 0, 0, 0, 0, 1];
        for (i, (_label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket {} should have {} files",
                i, expected_counts[i]
            );
        }
    }

    // Test 10: All Files in One Bucket
    // Verify correct counting when all files fall into a single size bucket
    #[tokio::test]
    async fn test_size_facets_all_files_in_one_bucket() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index 10 files all in the 1-10KB range
        for i in 0..10 {
            let file_id = Uuid::new_v4();
            let size = 2048 + (i * 512);
            let file_data = FileData {
                file_id,
                file_name: &format!("file_{}.txt", i),
                file_path: &format!("src/file_{}.txt", i),
                content: "homogeneous content",
                repository: "homogeneous-repo",
                project: "homogeneous-repo",
                version: "main",
                extension: "txt",
                size: size as u64,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with facets
        let query = SearchQuery {
            query: "content".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 100,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 10, "Should find all 10 files");

        let facets = results.facets.expect("Facets should be present");

        // All 10 files in 1-10KB bucket, rest empty
        let expected_counts = vec![0, 10, 0, 0, 0, 0];
        for (i, (label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket '{}' should have {} files",
                label, expected_counts[i]
            );
        }
    }

    // Test 11: Size Facets with Max Size Filter
    // Demonstrates that size facets are NOT affected by size filters
    #[tokio::test]
    async fn test_size_facets_with_max_size_filter() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files across all size ranges
        let sizes = vec![
            512,      // < 1 KB
            2048,     // 1-10 KB
            50000,    // 10-100 KB
            500000,   // 100KB-1MB
            5000000,  // 1-10MB
            50000000, // > 10MB
        ];

        for (i, size) in sizes.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file_{}.txt", i),
                file_path: &format!("src/file_{}.txt", i),
                content: "searchable",
                repository: "max-size-repo",
                project: "max-size-repo",
                version: "main",
                extension: "txt",
                size: *size,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with max_size filter (only files <= 100KB in search results)
        let query = SearchQuery {
            query: "searchable".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: Some(102400), // Only files <= 100KB in results
            limit: 100,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        // Should find 512, 2048, 50000 (3 files <= 102400)
        assert_eq!(results.total, 3, "Should find 3 files <= 100KB");

        let facets = results.facets.expect("Facets should be present");

        // Size facets show all files, NOT filtered by size constraint
        // This is the key design point - size facets are independent of size filters
        // < 1 KB: 1 (512)
        // 1-10 KB: 1 (2048)
        // 10-100 KB: 1 (50000)
        // 100KB-1MB: 1 (500000 - still shown even though filtered out of results)
        // 1-10MB: 1 (5000000 - still shown even though filtered out of results)
        // > 10MB: 1 (50000000 - still shown even though filtered out of results)
        let expected_counts = vec![1, 1, 1, 1, 1, 1];
        for (i, (_label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket {} should show {} files (size filter does NOT affect facets)",
                i, expected_counts[i]
            );
        }
    }

    // Test 12: Size Facets with Min and Max Size Filters
    // Demonstrates that size facets show all matching files regardless of size filters
    #[tokio::test]
    async fn test_size_facets_with_min_and_max_size_filters() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index files across all size ranges
        let sizes = vec![
            512,      // < 1 KB
            2048,     // 1-10 KB
            50000,    // 10-100 KB
            500000,   // 100KB-1MB
            5000000,  // 1-10MB
            50000000, // > 10MB
        ];

        for (i, size) in sizes.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file_{}.txt", i),
                file_path: &format!("src/file_{}.txt", i),
                content: "data",
                repository: "range-repo",
                project: "range-repo",
                version: "main",
                extension: "txt",
                size: *size,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with range filter (10KB <= size <= 1MB)
        let query = SearchQuery {
            query: "data".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: Some(10240),   // >= 10KB in results only
            max_size: Some(1048576), // <= 1MB in results only
            limit: 100,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        // Should find 50000, 500000 (2 files in the range)
        assert_eq!(results.total, 2, "Should find 2 files in size range");

        let facets = results.facets.expect("Facets should be present");

        // Size facets show all files matching text query, NOT affected by size filters
        // < 1 KB: 1 (512 - still shown even though filtered out of results)
        // 1-10 KB: 1 (2048 - still shown even though filtered out of results)
        // 10-100 KB: 1 (50000 - matches and shown)
        // 100KB-1MB: 1 (500000 - matches and shown)
        // 1-10MB: 1 (5000000 - still shown even though filtered out of results)
        // > 10MB: 1 (50000000 - still shown even though filtered out of results)
        let expected_counts = vec![1, 1, 1, 1, 1, 1];
        for (i, (_label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket {} should have {} files (size range filter does NOT affect facets)",
                i, expected_counts[i]
            );
        }
    }

    // Test 13: Size Facets Order
    // Verify that size_ranges buckets are always in the expected order
    #[tokio::test]
    async fn test_size_facets_bucket_order() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index one file in each bucket to verify order
        let sizes = vec![512, 2048, 50000, 500000, 5000000, 50000000];

        for (i, size) in sizes.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("file_{}.txt", i),
                file_path: &format!("src/file_{}.txt", i),
                content: "order",
                repository: "order-repo",
                project: "order-repo",
                version: "main",
                extension: "txt",
                size: *size,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with facets
        let query = SearchQuery {
            query: "order".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        let facets = results.facets.expect("Facets should be present");

        // Verify the expected order of buckets
        let expected_labels =
            vec!["< 1 KB", "1 KB - 10 KB", "10 KB - 100 KB", "100 KB - 1 MB", "1 MB - 10 MB", "> 10 MB"];

        for (i, (actual_label, _count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                actual_label, expected_labels[i],
                "Bucket at index {} should be '{}', got '{}'",
                i, expected_labels[i], actual_label
            );
        }
    }

    // Test 14: Large File Sizes
    // Verify handling of very large file sizes
    #[tokio::test]
    async fn test_size_facets_with_very_large_files() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index very large files
        let sizes = vec![
            100000000,   // 100 MB (> 10 MB)
            1000000000,  // 1 GB (> 10 MB)
            10000000000, // 10 GB (> 10 MB)
        ];

        for (i, size) in sizes.iter().enumerate() {
            let file_id = Uuid::new_v4();
            let file_data = FileData {
                file_id,
                file_name: &format!("huge_{}.bin", i),
                file_path: &format!("data/huge_{}.bin", i),
                content: "binary",
                repository: "large-repo",
                project: "large-repo",
                version: "main",
                extension: "bin",
                size: *size,
            };
            service.upsert_file(file_data).await.unwrap();
        }

        service.commit().await.unwrap();

        // Search with facets
        let query = SearchQuery {
            query: "binary".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: true,
        };

        let results = service.search(query).await.unwrap();
        assert_eq!(results.total, 3, "Should find all 3 large files");

        let facets = results.facets.expect("Facets should be present");

        // All files should be in > 10 MB bucket
        let expected_counts = vec![0, 0, 0, 0, 0, 3];
        for (i, (_label, count)) in facets.size_ranges.iter().enumerate() {
            assert_eq!(
                *count, expected_counts[i],
                "Bucket {} should have {} files",
                i, expected_counts[i]
            );
        }
    }

    // Test 15: Size Facets without Facets Request
    // Verify facets are None when include_facets is false
    #[tokio::test]
    async fn test_size_facets_not_returned_when_not_requested() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index a file
        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "test.txt",
            file_path: "src/test.txt",
            content: "test content",
            repository: "test-repo",
            project: "test-repo",
            version: "main",
            extension: "txt",
            size: 1024,
        };
        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search WITHOUT requesting facets
        let query = SearchQuery {
            query: "content".to_string(),
            project_filter: None,
            version_filter: None,
            extension_filter: None,
            repository_filter: None,
            min_size: None,
            max_size: None,
            limit: 10,
            offset: 0,
            include_facets: false, // Facets not requested
        };

        let results = service.search(query).await.unwrap();
        assert!(results.facets.is_none(), "Facets should be None when not requested");
    }
}
