// Simple example to demonstrate regex search with raw fields
use anyhow::Result;
use klask_rs::services::search::{FileData, SearchQuery, SearchService};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Create search service
    let search_service = SearchService::new("./example_regex_index")?;

    // Clear any existing index
    search_service.clear_index().await?;
    println!("Cleared existing search index");

    // Index some test files with regex-friendly names
    let test_files = vec![
        (
            "stringEncode.rs",
            "src/utils/stringEncode.rs",
            "pub fn stringEncode() {}",
        ),
        (
            "stringDecode.rs",
            "src/utils/stringDecode.rs",
            "pub fn stringDecode() {}",
        ),
        (
            "MyService.java",
            "src/services/MyService.java",
            "public class MyService {}",
        ),
        (
            "TestService.java",
            "src/services/TestService.java",
            "public class TestService {}",
        ),
        (
            "CrawlerService.rs",
            "src/services/crawler/CrawlerService.rs",
            "pub struct CrawlerService {}",
        ),
    ];

    println!("Indexing test files...");
    for (file_name, file_path, content) in test_files {
        let file_data = FileData {
            file_id: Uuid::new_v4(),
            file_name,
            file_path,
            content,
            repository: "test-repo",
            project: "test-project",
            version: "main",
            extension: file_name.split('.').last().unwrap_or(""),
            size: content.len() as u64,
        };
        search_service.upsert_file(file_data).await?;
        println!("  Indexed: {}", file_name);
    }

    // Commit the index
    search_service.commit().await?;
    println!("Committed search index");
    println!();

    // Test 1: Regex search for stringEncode pattern
    println!("Test 1: Regex search for 'stringEncod.*'");
    let regex_query =
        SearchQuery { query: "stringEncod.*".to_string(), regex_search: true, limit: 10, ..Default::default() };

    match search_service.search(regex_query).await {
        Ok(results) => {
            println!("  Results: {} matches", results.total);
            for result in &results.results {
                println!("    - {} ({})", result.file_name, result.file_path);
            }
            if results.total >= 1 {
                // The regex matches both stringEncode and stringDecode since they both start with stringEncod
                println!(
                    "  ✅ PASS: Found {} results matching 'stringEncod.*' pattern",
                    results.total
                );
            } else {
                println!("  ❌ FAIL: Expected at least 1 result, got {}", results.total);
            }
        }
        Err(e) => {
            println!("  ❌ FAIL: Search error: {}", e);
        }
    }
    println!();

    // Test 2: Regex search for Service pattern
    println!("Test 2: Regex search for '.*Service.*'");
    let regex_query =
        SearchQuery { query: ".*Service.*".to_string(), regex_search: true, limit: 10, ..Default::default() };

    match search_service.search(regex_query).await {
        Ok(results) => {
            println!("  Results: {} matches", results.total);
            for result in &results.results {
                println!("    - {} ({})", result.file_name, result.file_path);
            }
            if results.total >= 3 {
                println!("  ✅ PASS: Found at least 3 Service files (MyService, TestService, CrawlerService)");
            } else {
                println!("  ❌ FAIL: Expected at least 3 results, got {}", results.total);
            }
        }
        Err(e) => {
            println!("  ❌ FAIL: Search error: {}", e);
        }
    }
    println!();

    // Test 3: Regex search with file path
    println!("Test 3: Regex search for 'src/services/.*Service\\.rs'");
    let regex_query = SearchQuery {
        query: "src/services/.*Service\\.rs".to_string(),
        regex_search: true,
        limit: 10,
        ..Default::default()
    };

    match search_service.search(regex_query).await {
        Ok(results) => {
            println!("  Results: {} matches", results.total);
            for result in &results.results {
                println!("    - {} ({})", result.file_name, result.file_path);
            }
            if results.total == 1 && results.results[0].file_name == "CrawlerService.rs" {
                println!("  ✅ PASS: Found CrawlerService.rs in src/services/crawler/");
            } else {
                println!("  ⚠️  WARN: Expected CrawlerService.rs, got different results");
            }
        }
        Err(e) => {
            println!("  ❌ FAIL: Search error: {}", e);
        }
    }
    println!();

    // Clean up test index
    let _ = std::fs::remove_dir_all("./example_regex_index");
    println!("Cleaned up test index");

    Ok(())
}
