use klask_rs::services::search::{FileData, SearchQuery, SearchService};
use std::fs;
use tempfile::tempdir;
use uuid::Uuid;

/// Test case-sensitive and case-insensitive regex search with flags
#[tokio::test]
async fn test_regex_case_sensitivity() {
    let temp_dir = tempdir().unwrap();
    let index_dir = temp_dir.path().join("test_index");
    fs::create_dir_all(&index_dir).unwrap();

    let service = SearchService::new(&index_dir).unwrap();

    // Index files with mixed case names
    let files = vec![
        FileData {
            file_id: Uuid::new_v4(),
            file_name: "MyFile.txt",
            file_path: "src/MyFile.txt",
            content: "Test content",
            repository: "test-repo",
            project: "test-project",
            version: "main",
            extension: "txt",
            size: 12,
        },
        FileData {
            file_id: Uuid::new_v4(),
            file_name: "myfile.txt",
            file_path: "src/myfile.txt",
            content: "Test content",
            repository: "test-repo",
            project: "test-project",
            version: "main",
            extension: "txt",
            size: 12,
        },
        FileData {
            file_id: Uuid::new_v4(),
            file_name: "MYFILE.txt",
            file_path: "src/MYFILE.txt",
            content: "Test content",
            repository: "test-repo",
            project: "test-project",
            version: "main",
            extension: "txt",
            size: 12,
        },
    ];

    for file in &files {
        service.upsert_file(file.clone()).await.unwrap();
    }
    service.commit().await.unwrap();

    // Test 1: Case-sensitive search (default, no flags) - should only match exact case
    let case_sensitive_query = SearchQuery {
        query: "MyFile.*".to_string(),
        regex_search: true,
        regex_flags: None, // No flags = case-sensitive
        limit: 100,
        ..Default::default()
    };

    let results = service.search(case_sensitive_query).await.unwrap();
    assert_eq!(results.total, 1, "Case-sensitive search should only find 'MyFile.txt'");
    assert!(
        results.results[0].file_name.starts_with("MyFile"),
        "Should match 'MyFile.txt' exactly"
    );

    // Test 2: Case-insensitive search with 'i' flag - should match all cases
    let case_insensitive_query = SearchQuery {
        query: "MyFile.*".to_string(),
        regex_search: true,
        regex_flags: Some("i".to_string()), // 'i' flag = case-insensitive
        limit: 100,
        ..Default::default()
    };

    let results = service.search(case_insensitive_query).await.unwrap();
    assert_eq!(
        results.total, 3,
        "Case-insensitive search should find all 3 files (MyFile, myfile, MYFILE)"
    );

    // Test 3: Case-insensitive search with different pattern
    let lower_case_query = SearchQuery {
        query: "myfile.*".to_string(),
        regex_search: true,
        regex_flags: Some("i".to_string()),
        limit: 100,
        ..Default::default()
    };

    let results = service.search(lower_case_query).await.unwrap();
    assert_eq!(results.total, 3, "Case-insensitive 'myfile' should match all 3 files");

    // Test 4: Case-sensitive with lowercase pattern - should only match lowercase
    let lower_case_sensitive = SearchQuery {
        query: "myfile.*".to_string(),
        regex_search: true,
        regex_flags: None, // No flags = case-sensitive
        limit: 100,
        ..Default::default()
    };

    let results = service.search(lower_case_sensitive).await.unwrap();
    assert_eq!(
        results.total, 1,
        "Case-sensitive 'myfile' should only match 'myfile.txt'"
    );
}

/// Test multiple regex flags (i, m, s)
#[tokio::test]
async fn test_regex_multiple_flags() {
    let temp_dir = tempdir().unwrap();
    let index_dir = temp_dir.path().join("test_index");
    fs::create_dir_all(&index_dir).unwrap();

    let service = SearchService::new(&index_dir).unwrap();

    let file = FileData {
        file_id: Uuid::new_v4(),
        file_name: "TestFile.txt",
        file_path: "src/TestFile.txt",
        content: "Line 1\nLine 2\nLine 3",
        repository: "test-repo",
        project: "test-project",
        version: "main",
        extension: "txt",
        size: 22,
    };

    service.upsert_file(file).await.unwrap();
    service.commit().await.unwrap();

    // Test with multiple flags: "ims"
    let multi_flag_query = SearchQuery {
        query: "testfile.*".to_string(),
        regex_search: true,
        regex_flags: Some("ims".to_string()), // Multiple flags
        limit: 100,
        ..Default::default()
    };

    let results = service.search(multi_flag_query).await.unwrap();
    assert_eq!(results.total, 1, "Multi-flag search should find the file");
}

/// Test invalid regex flags are ignored
#[tokio::test]
async fn test_regex_invalid_flags_ignored() {
    let temp_dir = tempdir().unwrap();
    let index_dir = temp_dir.path().join("test_index");
    fs::create_dir_all(&index_dir).unwrap();

    let service = SearchService::new(&index_dir).unwrap();

    let file = FileData {
        file_id: Uuid::new_v4(),
        file_name: "Test.txt",
        file_path: "src/Test.txt",
        content: "Content",
        repository: "test-repo",
        project: "test-project",
        version: "main",
        extension: "txt",
        size: 7,
    };

    service.upsert_file(file).await.unwrap();
    service.commit().await.unwrap();

    // Test with invalid flags mixed with valid ones
    let query = SearchQuery {
        query: "test.*".to_string(),
        regex_search: true,
        regex_flags: Some("ixyz".to_string()), // 'i' is valid, 'xyz' should be ignored
        limit: 100,
        ..Default::default()
    };

    let results = service.search(query).await.unwrap();
    assert_eq!(results.total, 1, "Should still work with invalid flags ignored");
}
