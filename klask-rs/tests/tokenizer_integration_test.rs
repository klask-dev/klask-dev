#[cfg(test)]
mod tokenizer_integration_tests {
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

    fn build_search_query(query: &str) -> SearchQuery {
        SearchQuery {
            query: query.to_string(),
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
            regex_flags: None,
        }
    }

    // ==================== Tokenizer Integration Tests ====================

    #[tokio::test]
    async fn test_tokenizer_acronym_html_parser() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "parser.ts",
            file_path: "src/parser.ts",
            content: "class HTMLParser { public parse() {} }",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "ts",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for "html" - should find HTMLParser because it's tokenized as "html", "parser"
        let results = service.search(build_search_query("html")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find HTMLParser when searching for 'html'"
        );

        // Search for "parser" - should find HTMLParser
        let results = service.search(build_search_query("parser")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find HTMLParser when searching for 'parser'"
        );
    }

    #[tokio::test]
    async fn test_tokenizer_preserves_snake_case() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "config.rs",
            file_path: "src/config.rs",
            content: "const DATABASE_CONNECTION_TIMEOUT: u64 = 30;",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "rs",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for the exact snake_case term
        let results = service.search(build_search_query("database_connection_timeout")).await.unwrap();
        assert!(!results.results.is_empty(), "Should find DATABASE_CONNECTION_TIMEOUT");

        // Search uppercase should also work (case-insensitive)
        let results = service.search(build_search_query("DATABASE_CONNECTION_TIMEOUT")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find DATABASE_CONNECTION_TIMEOUT with uppercase search"
        );
    }

    #[tokio::test]
    async fn test_tokenizer_nginx_url() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "config.rs",
            file_path: "src/config.rs",
            content: "const NETBOX_URL: &str = \"https://nginx.example.com\";",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "rs",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search lowercase (with complete token preservation, both work)
        let results = service.search(build_search_query("netbox_url")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find NETBOX_URL when searching lowercase"
        );

        // Search uppercase
        let results = service.search(build_search_query("NETBOX_URL")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find NETBOX_URL when searching uppercase"
        );
    }

    #[tokio::test]
    async fn test_tokenizer_hyphen_preservation() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "styles.css",
            file_path: "src/styles.css",
            content: ".btn-primary-lg { color: blue; }",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "css",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for the CSS class
        let results = service.search(build_search_query("btn-primary-lg")).await.unwrap();
        assert!(!results.results.is_empty(), "Should find btn-primary-lg");
    }

    #[tokio::test]
    async fn test_tokenizer_http_acronym() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "client.ts",
            file_path: "src/client.ts",
            content: "function getHTTPResponse(): Promise<any> { return fetch('/api'); }",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "ts",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for "http"
        let results = service.search(build_search_query("http")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find getHTTPResponse when searching for 'http'"
        );

        // Search for "get"
        let results = service.search(build_search_query("get")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find getHTTPResponse when searching for 'get'"
        );

        // Search for "response"
        let results = service.search(build_search_query("response")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find getHTTPResponse when searching for 'response'"
        );
    }

    #[tokio::test]
    async fn test_tokenizer_json_acronym() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "serializer.ts",
            file_path: "src/serializer.ts",
            content: "function parseJSONObject(str: string): JSONObject { return JSON.parse(str); }",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "ts",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for "json"
        let results = service.search(build_search_query("json")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find JSONObject and parseJSONObject when searching for 'json'"
        );
    }

    #[tokio::test]
    async fn test_tokenizer_camel_case() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "utils.js",
            file_path: "src/utils.js",
            content: "function parseJSONResponse() { return data; }",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "js",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for individual tokens from parseJSONResponse
        let results = service.search(build_search_query("parse")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find parseJSONResponse when searching for 'parse'"
        );

        let results = service.search(build_search_query("json")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find parseJSONResponse when searching for 'json'"
        );

        let results = service.search(build_search_query("response")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find parseJSONResponse when searching for 'response'"
        );
    }

    #[tokio::test]
    async fn test_tokenizer_kubernetes_resource_names() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "deployment.yaml",
            file_path: "k8s/deployment.yaml",
            content: "metadata:\n  name: klask-backend-prod\n  namespace: production",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "yaml",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for "klask-backend-prod"
        let results = service.search(build_search_query("klask-backend-prod")).await.unwrap();
        assert!(!results.results.is_empty(), "Should find klask-backend-prod");
    }

    #[tokio::test]
    async fn test_tokenizer_pascal_case() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "handler.rs",
            file_path: "src/handler.rs",
            content: "impl RequestHandler { fn handle(&self) {} }",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "rs",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for "request"
        let results = service.search(build_search_query("request")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find RequestHandler when searching for 'request'"
        );

        // Search for "handler"
        let results = service.search(build_search_query("handler")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find RequestHandler when searching for 'handler'"
        );
    }

    #[tokio::test]
    async fn test_tokenizer_case_insensitive() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "utils.ts",
            file_path: "src/utils.ts",
            content: "function CamelCaseFunction() {}",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "ts",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for individual parts (case-insensitive)
        // CamelCaseFunction is split into ["camel", "case", "function"]
        let results = service.search(build_search_query("camel")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find CamelCaseFunction when searching for 'camel'"
        );

        let results = service.search(build_search_query("CASE")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find CamelCaseFunction when searching for 'CASE' (uppercase)"
        );

        let results = service.search(build_search_query("function")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should find CamelCaseFunction when searching for 'function'"
        );
    }

    #[tokio::test]
    async fn test_tokenizer_multiple_files() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        // Index file 1 with HTMLParser
        service
            .upsert_file(FileData {
                file_id: Uuid::new_v4(),
                file_name: "parser.ts",
                file_path: "src/parser.ts",
                content: "class HTMLParser {}",
                repository: "test-project",
                project: "test-project",
                version: "1.0.0",
                extension: "ts",
                size: 1024,
            })
            .await
            .unwrap();

        // Index file 2 with NETBOX_URL
        service
            .upsert_file(FileData {
                file_id: Uuid::new_v4(),
                file_name: "config.rs",
                file_path: "src/config.rs",
                content: "const NETBOX_URL: &str = \"\";",
                repository: "test-project",
                project: "test-project",
                version: "1.0.0",
                extension: "rs",
                size: 1024,
            })
            .await
            .unwrap();

        // Index file 3 with camelCase
        service
            .upsert_file(FileData {
                file_id: Uuid::new_v4(),
                file_name: "utils.js",
                file_path: "src/utils.js",
                content: "function parseJSONData() {}",
                repository: "test-project",
                project: "test-project",
                version: "1.0.0",
                extension: "js",
                size: 1024,
            })
            .await
            .unwrap();

        service.commit().await.unwrap();

        // Verify all can be found by searching for their token parts
        // HTMLParser is tokenized as ["html", "parser", "htmlparser"]
        assert!(!service.search(build_search_query("html")).await.unwrap().results.is_empty());

        // NETBOX_URL is tokenized as ["netbox_url"] (underscores are preserved, no split)
        // With complete token preservation, can search by full identifier
        assert!(!service.search(build_search_query("netbox_url")).await.unwrap().results.is_empty());

        // parseJSONData is tokenized as ["parse", "json", "data", "parsejsondata"]
        assert!(!service.search(build_search_query("parse")).await.unwrap().results.is_empty());
    }

    // ==================== Regression Tests ====================

    #[tokio::test]
    async fn test_normal_text_search() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "readme.md",
            file_path: "readme.md",
            content: "This is a normal English text document with words",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "md",
            size: 1024,
        };

        service.upsert_file(file_data).await.unwrap();
        service.commit().await.unwrap();

        // Search for normal words
        let results = service.search(build_search_query("normal")).await.unwrap();
        assert!(
            !results.results.is_empty(),
            "Should still be able to search for normal text"
        );
    }

    #[tokio::test]
    async fn test_search_service_creation() {
        let (_service, _temp_dir, _guard) = create_test_search_service().await;
        // Service creation itself is the test - it should not panic
    }

    #[tokio::test]
    async fn test_index_and_retrieve_file() {
        let (service, _temp_dir, _guard) = create_test_search_service().await;

        let file_id = Uuid::new_v4();
        let file_data = FileData {
            file_id,
            file_name: "main.rs",
            file_path: "src/main.rs",
            content: "fn main() { println!(\"Hello, world!\"); }",
            repository: "test-project",
            project: "test-project",
            version: "1.0.0",
            extension: "rs",
            size: 1024,
        };

        let result = service.upsert_file(file_data).await;
        assert!(result.is_ok());

        let commit_result = service.commit().await;
        assert!(commit_result.is_ok());

        let doc_count = service.get_document_count().unwrap();
        assert_eq!(doc_count, 1, "Should have one document indexed");
    }
}
