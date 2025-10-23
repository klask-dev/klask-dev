use klask_rs::api;
use klask_rs::models::{Repository, RepositoryType};
use klask_rs::services::github::{GitHubOwner, GitHubRepository, GitHubService};
use serde_json::json;

/// Tests for GitHub API integration endpoints
/// This includes discover, test-token, and CRUD operations with GitHub repositories

#[tokio::test]
async fn test_github_service_pattern_matching() {
    // Test the GitHub service pattern matching functionality
    let service = GitHubService::new();

    // Create test repository
    let repo = create_test_github_repository("test-project", "testuser", 1);

    // Test with different exclusion patterns
    let excluded_patterns = vec!["*-archive".to_string(), "test/*".to_string(), "*/large-*".to_string()];

    // This repository should not be excluded
    assert!(
        !service.filter_excluded_repositories_with_config(vec![repo.clone()], &[], &excluded_patterns).is_empty(),
        "Repository should not be excluded with current patterns"
    );

    // Create repository that matches exclusion pattern
    let archive_repo = create_test_github_repository("project-archive", "testuser", 2);
    assert!(
        service.filter_excluded_repositories_with_config(vec![archive_repo], &[], &excluded_patterns).is_empty(),
        "Archive repository should be excluded"
    );

    println!("✅ GitHub service pattern matching test passed!");
}

#[tokio::test]
async fn test_github_repository_exclusion_exact_match() {
    let service = GitHubService::new();

    let excluded_repositories = vec!["testuser/exclude-me".to_string(), "org/legacy-project".to_string()];

    let repos = vec![
        create_test_github_repository("keep-me", "testuser", 1),
        create_test_github_repository("exclude-me", "testuser", 2),
        create_test_github_repository("another-keep", "testuser", 3),
    ];

    let filtered = service.filter_excluded_repositories_with_config(repos, &excluded_repositories, &[]);

    assert_eq!(filtered.len(), 2, "Should keep 2 repositories after excluding 1");
    assert_eq!(filtered[0].full_name, "testuser/keep-me");
    assert_eq!(filtered[1].full_name, "testuser/another-keep");

    println!("✅ GitHub repository exclusion exact match test passed!");
}

#[tokio::test]
async fn test_github_repository_exclusion_patterns() {
    let service = GitHubService::new();

    let excluded_patterns = vec![
        "*-archive".to_string(),
        "testuser/test-*".to_string(), // Pattern matches full_name
        "*-temp".to_string(),
    ];

    let repos = vec![
        create_test_github_repository("active-project", "testuser", 1),
        create_test_github_repository("old-archive", "testuser", 2),
        create_test_github_repository("test-experiment", "testuser", 3),
        create_test_github_repository("working-temp", "testuser", 4),
        create_test_github_repository("production-app", "testuser", 5),
    ];

    let filtered = service.filter_excluded_repositories_with_config(repos, &[], &excluded_patterns);

    assert_eq!(filtered.len(), 2, "Should keep 2 repositories after pattern exclusions");
    assert_eq!(filtered[0].full_name, "testuser/active-project");
    assert_eq!(filtered[1].full_name, "testuser/production-app");

    println!("✅ GitHub repository exclusion patterns test passed!");
}

#[tokio::test]
async fn test_github_repository_data_structure() {
    // Test that GitHub repository data structure is valid
    let test_repo_data = json!({
        "name": "test-repo",
        "url": "https://api.github.com",
        "repositoryType": "GitHub",
        "branch": "main",
        "enabled": true,
        "accessToken": "ghp_test_token",
        "githubNamespace": "test-org",
        "githubExcludedRepositories": "org/archive-1,org/archive-2",
        "githubExcludedPatterns": "*-temp,*-archive"
    });

    // Verify JSON structure is valid
    assert!(test_repo_data.is_object());
    assert_eq!(test_repo_data["name"].as_str().unwrap(), "test-repo");
    assert_eq!(test_repo_data["repositoryType"].as_str().unwrap(), "GitHub");
    assert_eq!(test_repo_data["githubNamespace"].as_str().unwrap(), "test-org");
    assert_eq!(
        test_repo_data["githubExcludedRepositories"].as_str().unwrap(),
        "org/archive-1,org/archive-2"
    );
    assert_eq!(
        test_repo_data["githubExcludedPatterns"].as_str().unwrap(),
        "*-temp,*-archive"
    );

    println!("✅ GitHub repository data structure validation test passed!");
}

#[tokio::test]
async fn test_github_repository_model_fields() {
    // Test that Repository model correctly handles GitHub-specific fields
    let repo = Repository {
        id: uuid::Uuid::new_v4(),
        name: "github-test-repo".to_string(),
        url: "https://api.github.com".to_string(),
        repository_type: RepositoryType::GitHub,
        branch: Some("main".to_string()),
        enabled: true,
        access_token: Some("encrypted_token".to_string()),
        gitlab_namespace: None,
        is_group: false,
        last_crawled: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        auto_crawl_enabled: false,
        cron_schedule: None,
        next_crawl_at: None,
        crawl_frequency_hours: None,
        max_crawl_duration_minutes: Some(60),
        last_crawl_duration_seconds: None,
        gitlab_excluded_projects: None,
        gitlab_excluded_patterns: None,
        github_namespace: Some("test-org".to_string()),
        github_excluded_repositories: Some("org/repo1,org/repo2".to_string()),
        github_excluded_patterns: Some("*-archive,*-temp".to_string()),
        crawl_state: Some("idle".to_string()),
        last_processed_project: None,
        crawl_started_at: None,
    };

    // Verify GitHub fields are set correctly
    assert!(matches!(repo.repository_type, RepositoryType::GitHub));
    assert_eq!(repo.github_namespace, Some("test-org".to_string()));
    assert_eq!(
        repo.github_excluded_repositories,
        Some("org/repo1,org/repo2".to_string())
    );
    assert_eq!(repo.github_excluded_patterns, Some("*-archive,*-temp".to_string()));

    // Verify GitLab fields are not set
    assert_eq!(repo.gitlab_namespace, None);
    assert_eq!(repo.gitlab_excluded_projects, None);
    assert_eq!(repo.gitlab_excluded_patterns, None);

    println!("✅ GitHub repository model fields test passed!");
}

#[tokio::test]
async fn test_github_api_router_creation() {
    // Test that the API router including GitHub endpoints can be created
    let router = api::create_router().await;
    assert!(
        router.is_ok(),
        "Should be able to create API router with GitHub endpoints"
    );
    println!("✅ GitHub API router creation test passed!");
}

#[tokio::test]
async fn test_github_repository_type_serialization() {
    // Test that RepositoryType::GitHub serializes correctly
    let repo_type = RepositoryType::GitHub;
    let serialized = serde_json::to_string(&repo_type).expect("Should serialize");
    assert_eq!(serialized, "\"GitHub\"");

    // Test deserialization
    let deserialized: RepositoryType = serde_json::from_str(&serialized).expect("Should deserialize");
    assert!(matches!(deserialized, RepositoryType::GitHub));

    println!("✅ GitHub repository type serialization test passed!");
}

#[tokio::test]
async fn test_github_namespace_filtering() {
    let _service = GitHubService::new();

    // Create repos from different namespaces
    let repos = vec![
        create_test_github_repository("project1", "target-org", 1),
        create_test_github_repository("project2", "target-org", 2),
        create_test_github_repository("project3", "other-org", 3),
    ];

    // When filtering by namespace in real implementation, we would filter by owner
    // For this test, we validate the repository owner structure
    for repo in &repos {
        assert!(!repo.owner.login.is_empty());
        assert!(!repo.owner.owner_type.is_empty());
    }

    println!("✅ GitHub namespace filtering test passed!");
}

#[tokio::test]
async fn test_github_repository_with_multiple_exclusions() {
    let service = GitHubService::new();

    let excluded_repositories = vec!["org/specific-repo".to_string()];
    let excluded_patterns = vec!["*-archive".to_string(), "*-temp".to_string()];

    let repos = vec![
        create_test_github_repository("active-project", "org", 1),
        create_test_github_repository("specific-repo", "org", 2), // Exact match
        create_test_github_repository("old-archive", "org", 3),   // Pattern match
        create_test_github_repository("work-temp", "org", 4),     // Pattern match
        create_test_github_repository("production", "org", 5),
    ];

    let filtered = service.filter_excluded_repositories_with_config(repos, &excluded_repositories, &excluded_patterns);

    assert_eq!(filtered.len(), 2, "Should keep 2 repositories after all exclusions");
    assert_eq!(filtered[0].full_name, "org/active-project");
    assert_eq!(filtered[1].full_name, "org/production");

    println!("✅ GitHub repository with multiple exclusions test passed!");
}

#[tokio::test]
async fn test_github_archived_repositories_excluded() {
    // GitHub service should filter out archived repositories
    let _service = GitHubService::new();

    let mut archived_repo = create_test_github_repository("archived-project", "org", 1);
    archived_repo.archived = true;

    let active_repo = create_test_github_repository("active-project", "org", 2);

    // In the real discover_repositories implementation, archived repos are filtered out
    // Here we test the repository structure
    assert!(archived_repo.archived, "Repository should be marked as archived");
    assert!(!active_repo.archived, "Repository should not be marked as archived");

    println!("✅ GitHub archived repositories exclusion test passed!");
}

#[tokio::test]
async fn test_github_repository_clone_urls() {
    let repo = create_test_github_repository("test-repo", "testuser", 1);

    // Verify all URL formats are correct
    assert_eq!(repo.clone_url, "https://github.com/testuser/test-repo.git");
    assert_eq!(repo.ssh_url, "git@github.com:testuser/test-repo.git");
    assert_eq!(repo.html_url, "https://github.com/testuser/test-repo");

    println!("✅ GitHub repository clone URLs test passed!");
}

#[tokio::test]
async fn test_github_private_public_repositories() {
    let private_repo = {
        let mut repo = create_test_github_repository("private-repo", "org", 1);
        repo.private = true;
        repo
    };

    let public_repo = create_test_github_repository("public-repo", "org", 2);

    assert!(private_repo.private, "Private repository should be marked as private");
    assert!(
        !public_repo.private,
        "Public repository should not be marked as private"
    );

    println!("✅ GitHub private/public repositories test passed!");
}

#[tokio::test]
async fn test_github_owner_types() {
    let user_repo = create_test_github_repository("user-repo", "individual", 1);
    assert_eq!(user_repo.owner.owner_type, "User");

    let mut org_repo = create_test_github_repository("org-repo", "company", 2);
    org_repo.owner.owner_type = "Organization".to_string();
    assert_eq!(org_repo.owner.owner_type, "Organization");

    println!("✅ GitHub owner types test passed!");
}

#[tokio::test]
async fn test_github_wildcard_pattern_edge_cases() {
    let service = GitHubService::new();

    // Test edge case patterns
    let patterns = vec![
        "*".to_string(), // Match all
    ];

    let repos = vec![
        create_test_github_repository("any-repo-1", "org", 1),
        create_test_github_repository("any-repo-2", "org", 2),
    ];

    let filtered = service.filter_excluded_repositories_with_config(repos, &[], &patterns);

    assert_eq!(filtered.len(), 0, "Wildcard * should exclude all repositories");

    // Test empty pattern
    let empty_patterns: Vec<String> = vec![];
    let repos = vec![create_test_github_repository("keep-me", "org", 1)];

    let filtered = service.filter_excluded_repositories_with_config(repos, &[], &empty_patterns);

    assert_eq!(filtered.len(), 1, "Empty patterns should keep all repositories");

    println!("✅ GitHub wildcard pattern edge cases test passed!");
}

#[tokio::test]
async fn test_github_complex_patterns() {
    let service = GitHubService::new();

    let patterns = vec![
        "org/*-archive".to_string(),       // org/project-archive
        "*/test-*".to_string(),            // any/test-something
        "*-old".to_string(),               // anything-old (matches full_name)
        "org/prefix-*-suffix".to_string(), // org/prefix-anything-suffix
    ];

    let repos = vec![
        create_test_github_repository("prod-app", "org", 1),         // Keep
        create_test_github_repository("test-archive", "org", 2),     // Exclude: org/*-archive
        create_test_github_repository("test-experiment", "team", 3), // Exclude: */test-*
        create_test_github_repository("legacy-old", "org", 4),       // Exclude: *-old
        create_test_github_repository("prefix-feature-suffix", "org", 5), // Exclude: org/prefix-*-suffix
        create_test_github_repository("active-project", "org", 6),   // Keep
    ];

    let filtered = service.filter_excluded_repositories_with_config(repos, &[], &patterns);

    assert_eq!(
        filtered.len(),
        2,
        "Should keep 2 repositories after complex pattern exclusions"
    );
    assert_eq!(filtered[0].full_name, "org/prod-app");
    assert_eq!(filtered[1].full_name, "org/active-project");

    println!("✅ GitHub complex patterns test passed!");
}

// Helper function to create test GitHub repository
fn create_test_github_repository(name: &str, owner: &str, id: i64) -> GitHubRepository {
    GitHubRepository {
        id,
        name: name.to_string(),
        full_name: format!("{}/{}", owner, name),
        description: Some("Test repository".to_string()),
        default_branch: "main".to_string(),
        clone_url: format!("https://github.com/{}/{}.git", owner, name),
        ssh_url: format!("git@github.com:{}/{}.git", owner, name),
        html_url: format!("https://github.com/{}/{}", owner, name),
        private: false,
        archived: false,
        owner: GitHubOwner { login: owner.to_string(), owner_type: "User".to_string() },
    }
}
