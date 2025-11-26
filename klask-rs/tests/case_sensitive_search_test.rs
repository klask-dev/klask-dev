use klask_rs::services::{SearchQuery, SearchService};
use std::sync::LazyLock;
use tempfile::TempDir;
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

// Global mutex to ensure tests don't interfere with each other
static TEST_MUTEX: LazyLock<AsyncMutex<()>> = LazyLock::new(|| AsyncMutex::new(()));

async fn create_test_search_service() -> (SearchService, TempDir, tokio::sync::MutexGuard<'static, ()>) {
    let _guard = TEST_MUTEX.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let test_id = Uuid::new_v4().to_string()[..8].to_string();
    let index_path = temp_dir.path().join(format!("test_index_{}", test_id));
    let service = SearchService::new(&index_path).expect("Failed to create search service");
    (service, temp_dir, _guard)
}

// ============================================================================
// BASIC CASE-SENSITIVE MATCHING TESTS
// ============================================================================

#[tokio::test]
async fn test_case_sensitive_search_finds_exact_case() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "readerTemplate.rs",
        file_path: "src/readerTemplate.rs",
        content: "let template = readerTemplate.parse();",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 38,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Case-sensitive search with EXACT case should find the file
    let query = SearchQuery::new("readerTemplate".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 1,
        "Case-sensitive search for 'readerTemplate' should find 'readerTemplate'"
    );
}

#[tokio::test]
async fn test_case_sensitive_search_rejects_wrong_case() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "readerTemplate.rs",
        file_path: "src/readerTemplate.rs",
        content: "let template = readerTemplate.parse();",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 38,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    let query = SearchQuery::new("readertemplate".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 0,
        "Case-sensitive search for 'readertemplate' should not find 'readerTemplate'"
    );
}

#[tokio::test]
async fn test_case_sensitive_camelcase_variant_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "HTMLParser.rs",
        file_path: "src/HTMLParser.rs",
        content: "impl HTMLParser { fn parse() {} }",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 33,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    let query = SearchQuery::new("htmlparser".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 0,
        "Case-sensitive search for 'htmlparser' should not find 'HTMLParser'"
    );
}

#[tokio::test]
async fn test_case_sensitive_uppercase_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "MAX_SIZE.rs",
        file_path: "src/MAX_SIZE.rs",
        content: "const MAX_SIZE: usize = 1024;",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 30,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Lowercase query should not find uppercase constant
    let query = SearchQuery::new("max_size".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 0,
        "Case-sensitive search for 'max_size' should not find 'MAX_SIZE'"
    );
}

// ============================================================================
// FILE NAME AND PATH MATCHING TESTS
// ============================================================================

#[tokio::test]
async fn test_case_sensitive_file_name_match() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "MyComponent.tsx",
        file_path: "src/MyComponent.tsx",
        content: "export function MyComponent() {}",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "tsx",
        size: 32,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Case-sensitive search with EXACT case should find it
    let query = SearchQuery::new("MyComponent".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 1,
        "Case-sensitive search for 'MyComponent' should find exact match"
    );
}

#[tokio::test]
async fn test_case_sensitive_file_name_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "MyComponent.tsx",
        file_path: "src/MyComponent.tsx",
        content: "export function MyComponent() {}",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "tsx",
        size: 32,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Different case should not match
    let query = SearchQuery::new("mycomponent".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 0,
        "Case-sensitive search for 'mycomponent' should not find 'MyComponent'"
    );
}

#[tokio::test]
async fn test_case_sensitive_file_path_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "index.ts",
        file_path: "src/Components/Button/index.ts",
        content: "export class Button {}",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "ts",
        size: 22,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Different case should not match in path
    let query = SearchQuery::new("components".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 0,
        "Case-sensitive search for 'components' should not find 'Components' in path"
    );
}

// ============================================================================
// MULTIPLE CASE VARIANTS TEST
// ============================================================================

#[tokio::test]
async fn test_case_sensitive_multiple_case_variants_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    // Index file with multiple case variants in content
    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "handlers.rs",
        file_path: "src/handlers.rs",
        content: "fn getUserData() { } fn getUserdata() { } fn GETUSERDATA() { }",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 60,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Case-sensitive search for wrong case should not match any variant
    let query = SearchQuery::new("getuserdAta".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();
    assert_eq!(result.total, 0, "Should not find 'getuserdAta' with wrong case");
}

// ============================================================================
// FILTER INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_case_sensitive_with_repository_filter_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data1 = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "MyReader.rs",
        file_path: "src/MyReader.rs",
        content: "struct MyReader {}",
        repository: "repo1",
        project: "project1",
        version: "1.0.0",
        extension: "rs",
        size: 18,
    };

    service.upsert_file(file_data1).await.unwrap();
    service.commit().await.unwrap();

    // Should not find in different repository
    let mut query = SearchQuery::new("MyReader".to_string()).with_case_sensitive(true);
    query.repository_filter = Some("repo3".to_string());

    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 0,
        "Case-sensitive search in non-existent repo should find nothing"
    );
}

#[tokio::test]
async fn test_case_sensitive_with_extension_filter_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data1 = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "handleRequest.rs",
        file_path: "src/handleRequest.rs",
        content: "fn handleRequest() {}",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 21,
    };

    service.upsert_file(file_data1).await.unwrap();
    service.commit().await.unwrap();

    let mut query = SearchQuery::new("handleRequest".to_string()).with_case_sensitive(true);
    query.extension_filter = Some("py".to_string());

    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 0,
        "Case-sensitive search with py extension filter should find nothing"
    );
}

// ============================================================================
// PAGINATION TESTS
// ============================================================================

#[tokio::test]
async fn test_case_sensitive_pagination_with_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    // Index multiple files
    for i in 0..5 {
        let file_data = klask_rs::services::search::FileData {
            file_id: Uuid::new_v4(),
            file_name: &format!("TestValue{}.rs", i),
            file_path: &format!("src/TestValue{}.rs", i),
            content: "let x = test;",
            repository: "test-repo",
            project: "test-project",
            version: "1.0.0",
            extension: "rs",
            size: 13,
        };
        service.upsert_file(file_data).await.unwrap();
    }
    service.commit().await.unwrap();

    // Case-sensitive search with wrong case
    let mut query = SearchQuery::new("testvalue".to_string()).with_case_sensitive(true);
    query.limit = 2;
    query.offset = 0;

    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 0,
        "Case-sensitive search for 'testvalue' should not find 'TestValue'"
    );
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[tokio::test]
async fn test_case_sensitive_with_special_characters_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "__init__.py",
        file_path: "src/__init__.py",
        content: "fn __init__() { } fn _private() { } fn PublicFunc() { }",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "py",
        size: 52,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Case-sensitive search with wrong case
    let query = SearchQuery::new("INIT".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();

    // INIT should not match __init__ case-sensitively
    assert_eq!(result.total, 0, "Case-sensitive search for 'INIT' should not match");
}

#[tokio::test]
async fn test_case_sensitive_with_numbers_and_letters_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "v1Alpha.rs",
        file_path: "src/v1Alpha.rs",
        content: "const v1Alpha = 1; const V1_ALPHA = 2; const v1_alpha = 3;",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 60,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Case-sensitive search for different case
    let query = SearchQuery::new("v1ALPHA".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();
    assert_eq!(result.total, 0, "Should not find 'v1ALPHA' with wrong case");
}

#[tokio::test]
async fn test_case_sensitive_empty_query_handling() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "test.rs",
        file_path: "src/test.rs",
        content: "fn test() {}",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 12,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Empty case-sensitive query should handle gracefully
    let query = SearchQuery::new("".to_string()).with_case_sensitive(true);
    let result = service.search(query).await;

    // Result can be either Ok with 0 results or Err - both are acceptable
    if let Ok(res) = result {
        assert_eq!(res.total, 0, "Empty query should return no results");
    }
}

#[tokio::test]
async fn test_case_sensitive_simple_word_match() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "permissions.rs",
        file_path: "src/permissions.rs",
        content: "const ALLOW: &str = \"allow\"; fn Allow() {}",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 44,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Case-sensitive search for "Allow" (with capital A)
    let query = SearchQuery::new("Allow".to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 1,
        "Case-sensitive search for 'Allow' should find 'Allow' in function name"
    );
}

#[tokio::test]
async fn test_case_sensitive_very_long_identifier_mismatch() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let long_id = "VeryLongIdentifierNameThatIsUsedInMultiplePlacesWithSpecificCasing";
    let file_name = format!("{}.rs", long_id);

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: &file_name,
        file_path: &format!("src/{}", file_name),
        content: &format!("fn {}() {{}}", long_id),
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: file_name.len() as u64,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Wrong case should not match
    let wrong_case = "verylongidentifiernameusedmultipleplaceswithspecificcasing";
    let query = SearchQuery::new(wrong_case.to_string()).with_case_sensitive(true);
    let result = service.search(query).await.unwrap();

    assert_eq!(
        result.total, 0,
        "Case-sensitive search should not find long identifier with wrong case"
    );
}

// ============================================================================
// VLANID SPECIFIC TESTS - Testing camelCase search behavior
// ============================================================================

#[tokio::test]
async fn test_case_insensitive_search_vlan_id_mixed_case_query() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "network.rs",
        file_path: "src/network.rs",
        content: "let vlanID = 100; // configure vlanID here",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 42,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Case-insensitive search with exact case "vlanID" should find it
    // (QueryParser will lowercase and split the query)
    let query = SearchQuery::new("vlanID".to_string());
    let result = service.search(query).await.unwrap();

    println!("Search 'vlanID' case-insensitive: {} results", result.total);
    assert_eq!(
        result.total, 1,
        "Case-insensitive search for 'vlanID' should find 'vlanID' in content"
    );
}

#[tokio::test]
async fn test_case_insensitive_search_vlanid_lowercase_query() {
    let (service, _temp_dir, _guard) = create_test_search_service().await;

    let file_data = klask_rs::services::search::FileData {
        file_id: Uuid::new_v4(),
        file_name: "network.rs",
        file_path: "src/network.rs",
        content: "let vlanID = 100;",
        repository: "test-repo",
        project: "test-project",
        version: "1.0.0",
        extension: "rs",
        size: 17,
    };

    service.upsert_file(file_data).await.unwrap();
    service.commit().await.unwrap();

    // Case-insensitive search with lowercase "vlanid" should find "vlanID"
    let query = SearchQuery::new("vlanid".to_string());
    let result = service.search(query).await.unwrap();

    println!("Search 'vlanid' case-insensitive: {} results", result.total);
    assert_eq!(
        result.total, 1,
        "Case-insensitive search for 'vlanid' should find 'vlanID' in content"
    );
}
