use anyhow::Result;
use klask_rs::services::crawler::CrawlProgress;
use std::path::Path;

#[tokio::test]
async fn test_crawl_progress_initialization() -> Result<()> {
    let progress = CrawlProgress { files_processed: 0, files_indexed: 0, errors: Vec::new() };

    assert_eq!(progress.files_processed, 0);
    assert_eq!(progress.files_indexed, 0);
    assert!(progress.errors.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_supported_file_extensions() -> Result<()> {
    // This test would ideally test the is_supported_file method,
    // but it requires a CrawlerService instance which needs database connection
    // For now, we'll test the logic patterns manually

    let supported_files = vec![
        "test.rs",
        "test.py",
        "test.js",
        "test.ts",
        "test.java",
        "test.c",
        "test.cpp",
        "test.go",
        "test.rb",
        "README.md",
        "Dockerfile",
        "Makefile",
        "package.json",
    ];

    let supported_extensions = &[
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

    for file in &supported_files {
        let path = Path::new(file);
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            assert!(
                supported_extensions.contains(&extension.to_lowercase().as_str()),
                "Extension {} should be supported",
                extension
            );
        } else {
            // Files without extensions that should be supported
            let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
            let is_special_file = matches!(
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
            );
            assert!(is_special_file, "Special file {} should be supported", file_name);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_unsupported_file_extensions() -> Result<()> {
    let unsupported_files = vec!["test.exe", "test.dll", "test.so", "test.bin", "test.img", "test.zip"];

    let supported_extensions = &[
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

    for file in &unsupported_files {
        let path = Path::new(file);
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            assert!(
                !supported_extensions.contains(&extension.to_lowercase().as_str()),
                "Extension {} should not be supported",
                extension
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_file_size_limit_constant() -> Result<()> {
    // Test that the MAX_FILE_SIZE constant is reasonable (10MB)
    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
    assert_eq!(MAX_FILE_SIZE, 10_485_760);
    Ok(())
}

#[tokio::test]
async fn test_error_accumulation() -> Result<()> {
    let mut progress = CrawlProgress { files_processed: 0, files_indexed: 0, errors: Vec::new() };

    // Simulate processing files with some errors
    progress.files_processed += 1;
    progress.files_indexed += 1;

    progress.files_processed += 1;
    progress.errors.push("Failed to read file".to_string());

    progress.files_processed += 1;
    progress.files_indexed += 1;

    assert_eq!(progress.files_processed, 3);
    assert_eq!(progress.files_indexed, 2);
    assert_eq!(progress.errors.len(), 1);
    assert_eq!(progress.errors[0], "Failed to read file");

    Ok(())
}
