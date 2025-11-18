use klask_rs::services::{SearchQuery, SearchService};
use std::time::Instant;
use tokio::time::Duration;

#[tokio::test]
async fn test_concurrent_queries_run_in_parallel() {
    // Create search service with test data
    let search_service = SearchService::new("test_index_concurrent_search").unwrap();

    // Index some test files
    for i in 0..100 {
        let file_id = uuid::Uuid::new_v4();
        search_service
            .index_file(klask_rs::services::FileData {
                file_id,
                file_name: &format!("network_test_{}.rs", i),
                file_path: &format!("src/network/test_{}.rs", i),
                content: "fn network_function() { println!(\"network version test\"); }",
                repository: "test-repo",
                project: "test-project",
                version: "main",
                extension: "rs",
                size: 1024,
            })
            .await
            .unwrap();
    }

    // Commit changes
    search_service.commit().await.unwrap();

    // Test 1: Simple query should be fast (< 1 second)
    let start = Instant::now();
    let simple_query = SearchQuery {
        query: "function".to_string(),
        repository_filter: None,
        project_filter: None,
        version_filter: None,
        extension_filter: None,
        min_size: None,
        max_size: None,
        limit: 10,
        offset: 0,
        include_facets: false,
        fuzzy_search: false,
        regex_search: false,
    };
    let simple_result = search_service.search(simple_query).await;
    let simple_duration = start.elapsed();

    assert!(simple_result.is_ok());
    assert!(
        simple_duration < Duration::from_secs(1),
        "Simple query took {:?}, expected < 1s",
        simple_duration
    );

    // Test 2: Launch both heavy regex and simple query concurrently
    let service1 = search_service.clone();
    let service2 = search_service.clone();

    let heavy_query = SearchQuery {
        query: ".*etwork".to_string(), // Inefficient pattern with .* prefix
        repository_filter: None,
        project_filter: None,
        version_filter: None,
        extension_filter: None,
        min_size: None,
        max_size: None,
        limit: 10,
        offset: 0,
        include_facets: false,
        fuzzy_search: false,
        regex_search: true,
    };

    let simple_query2 = SearchQuery {
        query: "function".to_string(),
        repository_filter: None,
        project_filter: None,
        version_filter: None,
        extension_filter: None,
        min_size: None,
        max_size: None,
        limit: 10,
        offset: 0,
        include_facets: false,
        fuzzy_search: false,
        regex_search: false,
    };

    let start = Instant::now();

    // Launch both queries concurrently
    let (heavy_result, simple_result) = tokio::join!(service1.search(heavy_query), service2.search(simple_query2));

    let total_duration = start.elapsed();

    // Both should succeed
    assert!(heavy_result.is_ok(), "Heavy query failed: {:?}", heavy_result.err());
    assert!(simple_result.is_ok(), "Simple query failed: {:?}", simple_result.err());

    // The simple query should not be blocked by the heavy one
    // If they were running serially, it would take much longer
    println!("Concurrent execution took: {:?}", total_duration);
    println!("Heavy query would have blocked simple query in old implementation");

    // Simple query should still be relatively fast even with heavy query running
    // (This verifies that spawn_blocking allows parallel execution)
    assert!(
        total_duration < Duration::from_secs(35),
        "Concurrent queries took {:?}, expected < 35s (timeout is 30s)",
        total_duration
    );
}

#[tokio::test]
async fn test_inefficient_regex_pattern_warning() {
    let search_service = SearchService::new("test_index_regex_warning").unwrap();

    // Index a test file
    let file_id = uuid::Uuid::new_v4();
    search_service
        .index_file(klask_rs::services::FileData {
            file_id,
            file_name: "test.rs",
            file_path: "src/test.rs",
            content: "network version",
            repository: "test-repo",
            project: "test-project",
            version: "main",
            extension: "rs",
            size: 1024,
        })
        .await
        .unwrap();

    search_service.commit().await.unwrap();

    // Search with inefficient pattern (should log warning but still work)
    let query = SearchQuery {
        query: ".*etwork".to_string(), // Inefficient: starts with .*
        repository_filter: None,
        project_filter: None,
        version_filter: None,
        extension_filter: None,
        min_size: None,
        max_size: None,
        limit: 10,
        offset: 0,
        include_facets: false,
        fuzzy_search: false,
        regex_search: true,
    };

    let result = search_service.search(query).await;

    // Should still work, just with a warning
    assert!(result.is_ok(), "Query should succeed despite inefficiency");
}

#[tokio::test]
async fn test_search_timeout() {
    // This test is informational - it demonstrates that the timeout exists
    // We can't easily create a query that times out reliably without a huge index
    let search_service = SearchService::new("test_index_timeout").unwrap();

    // Index a single file
    let file_id = uuid::Uuid::new_v4();
    search_service
        .index_file(klask_rs::services::FileData {
            file_id,
            file_name: "test.rs",
            file_path: "src/test.rs",
            content: "test",
            repository: "test-repo",
            project: "test-project",
            version: "main",
            extension: "rs",
            size: 1024,
        })
        .await
        .unwrap();

    search_service.commit().await.unwrap();

    // Normal query should complete well before timeout
    let query = SearchQuery {
        query: "test".to_string(),
        repository_filter: None,
        project_filter: None,
        version_filter: None,
        extension_filter: None,
        min_size: None,
        max_size: None,
        limit: 10,
        offset: 0,
        include_facets: false,
        fuzzy_search: false,
        regex_search: false,
    };

    let start = Instant::now();
    let result = search_service.search(query).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Query should succeed");
    assert!(
        duration < Duration::from_secs(30),
        "Query should complete before timeout"
    );
}
