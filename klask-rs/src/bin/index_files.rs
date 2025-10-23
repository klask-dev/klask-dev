use anyhow::Result;
use klask_rs::models::{Repository, RepositoryType};
use klask_rs::services::search::{FileData, SearchService};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize search service
    let search_service = Arc::new(SearchService::new("./index")?);

    // Create a test repository pointing to the klask-rs project
    let repository = Repository {
        id: Uuid::new_v4(),
        name: "klask-rs".to_string(),
        url: "/home/jeremie/git/github/klask-dev/klask-rs".to_string(),
        repository_type: RepositoryType::FileSystem,
        branch: None,
        enabled: true,
        access_token: None,
        gitlab_namespace: None,
        is_group: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_crawled: None,
        // Scheduling fields
        auto_crawl_enabled: false,
        cron_schedule: None,
        next_crawl_at: None,
        crawl_frequency_hours: None,
        max_crawl_duration_minutes: Some(60),
        last_crawl_duration_seconds: None,
        gitlab_excluded_projects: None,
        gitlab_excluded_patterns: None,
        github_namespace: None,
        github_excluded_repositories: None,
        github_excluded_patterns: None,
        crawl_state: None,
        last_processed_project: None,
        crawl_started_at: None,
    };

    // Initialize crawler service (database is optional for this direct indexing)
    let database_url = std::env::var("DATABASE_URL").unwrap_or_default();
    if database_url.is_empty() {
        println!("No DATABASE_URL provided, proceeding without database connection");

        // Manually process files directly with SearchService
        let mut progress =
            klask_rs::services::crawler::CrawlProgress { files_processed: 0, files_indexed: 0, errors: Vec::new() };

        // Use a minimal crawler setup to process files
        let repo_path = std::path::PathBuf::from(&repository.url);

        // Walk through all files and index them directly
        use walkdir::WalkDir;

        println!("Starting to index files from: {}", repository.url);

        for entry in WalkDir::new(repo_path).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
            let file_path = entry.path();
            let relative_path = file_path.strip_prefix(&repository.url)?;
            let relative_path_str = relative_path.to_string_lossy().to_string();

            // Skip hidden files
            if relative_path_str.starts_with('.') {
                continue;
            }

            // Check if file is supported
            if !is_supported_file(file_path) {
                continue;
            }

            // Skip large files
            if let Ok(metadata) = file_path.metadata() {
                if metadata.len() > 10 * 1024 * 1024 {
                    // 10MB
                    continue;
                }
            }

            progress.files_processed += 1;

            // Read and index the file
            match tokio::fs::read_to_string(file_path).await {
                Ok(content) => {
                    // Skip binary files
                    if content.chars().any(|c| c == '\0') {
                        continue;
                    }

                    let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_string();

                    let file_name = file_path.file_name().and_then(|name| name.to_str()).unwrap_or("").to_string();

                    let file_id = Uuid::new_v4();

                    if let Err(e) = search_service
                        .index_file(FileData {
                            file_id,
                            file_name: &file_name,
                            file_path: &relative_path_str,
                            content: &content,
                            repository: &repository.name,
                            project: &repository.name,
                            version: "HEAD",
                            extension: &extension,
                        })
                        .await
                    {
                        progress.errors.push(format!("Failed to index {}: {}", relative_path_str, e));
                    } else {
                        progress.files_indexed += 1;
                        println!("Indexed: {}", relative_path_str);
                    }
                }
                Err(e) => {
                    progress.errors.push(format!("Failed to read {}: {}", relative_path_str, e));
                }
            }
        }

        // Commit the changes
        search_service.commit().await?;

        println!("\nIndexing completed!");
        println!("Files processed: {}", progress.files_processed);
        println!("Files indexed: {}", progress.files_indexed);
        println!("Errors: {}", progress.errors.len());

        if !progress.errors.is_empty() {
            println!("\nErrors encountered:");
            for error in progress.errors {
                println!("  {}", error);
            }
        }
    } else {
        println!("DATABASE_URL provided, this script is for direct indexing without database");
    }

    Ok(())
}

fn is_supported_file(file_path: &Path) -> bool {
    const SUPPORTED_EXTENSIONS: &[&str] = &[
        "rs",
        "py",
        "js",
        "ts",
        "java",
        "c",
        "cpp",
        "h",
        "hpp",
        "go",
        "rb",
        "php",
        "cs",
        "swift",
        "kt",
        "scala",
        "clj",
        "hs",
        "ml",
        "fs",
        "elm",
        "dart",
        "vue",
        "jsx",
        "tsx",
        "html",
        "css",
        "scss",
        "less",
        "sql",
        "sh",
        "bash",
        "zsh",
        "fish",
        "ps1",
        "bat",
        "cmd",
        "dockerfile",
        "yaml",
        "yml",
        "json",
        "toml",
        "xml",
        "md",
        "txt",
        "cfg",
        "conf",
        "ini",
        "properties",
        "gradle",
        "maven",
        "pom",
        "sbt",
        "cmake",
        "makefile",
        "r",
        "m",
        "perl",
        "pl",
        "lua",
    ];

    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
        SUPPORTED_EXTENSIONS.contains(&extension.to_lowercase().as_str())
    } else {
        // Support files without extensions that might be scripts or config files
        if let Some(file_name) = file_path.file_name().and_then(|name| name.to_str()) {
            matches!(
                file_name.to_lowercase().as_str(),
                "dockerfile"
                    | "makefile"
                    | "rakefile"
                    | "gemfile"
                    | "vagrantfile"
                    | "procfile"
                    | "readme"
                    | "license"
                    | "changelog"
                    | "authors"
                    | "contributors"
                    | "copying"
                    | "install"
                    | "news"
                    | "todo"
            )
        } else {
            false
        }
    }
}
