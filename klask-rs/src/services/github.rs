use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GitHubService {
    #[allow(dead_code)]
    client: Client,
    #[allow(dead_code)]
    excluded_repositories: Vec<String>,
    #[allow(dead_code)]
    excluded_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub html_url: String,
    pub private: bool,
    pub archived: bool,
    pub owner: GitHubOwner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
    #[serde(rename = "type")]
    pub owner_type: String, // "User" or "Organization"
}

impl Default for GitHubService {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubService {
    /// Check GitHub API rate limit from response headers and log warnings/errors
    fn check_github_rate_limit(response: &reqwest::Response) {
        if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
            if let Ok(remaining_str) = remaining.to_str() {
                if let Ok(remaining_count) = remaining_str.parse::<i32>() {
                    if remaining_count == 0 {
                        tracing::error!("GitHub rate limit exhausted! Check x-ratelimit-reset header.");

                        // Try to get reset time for better error message
                        if let Some(reset) = response.headers().get("x-ratelimit-reset") {
                            if let Ok(reset_str) = reset.to_str() {
                                tracing::error!("Rate limit will reset at timestamp: {}", reset_str);
                            }
                        }
                    } else if remaining_count < 100 {
                        tracing::warn!("GitHub rate limit low: {} requests remaining", remaining_count);
                    }
                }
            }
        }
    }

    pub fn new() -> Self {
        let mut builder = Client::builder().user_agent("klask-rs/2.0").timeout(std::time::Duration::from_secs(30));

        // GitHub doesn't typically need invalid cert acceptance, but include it for consistency
        let accept_invalid_certs =
            std::env::var("KLASK_GITHUB_ACCEPT_INVALID_CERTS").map(|v| v.to_lowercase() == "true").unwrap_or(false);

        if accept_invalid_certs {
            tracing::warn!(
                "GitHub client configured to accept invalid certificates (KLASK_GITHUB_ACCEPT_INVALID_CERTS=true)"
            );
            builder = builder.danger_accept_invalid_certs(true);
        }

        // Parse excluded repositories from environment variables
        let excluded_repositories = std::env::var("KLASK_GITHUB_EXCLUDED_REPOSITORIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        let excluded_patterns = std::env::var("KLASK_GITHUB_EXCLUDED_PATTERNS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        if !excluded_repositories.is_empty() {
            tracing::info!(
                "GitHub exclusion configured - excluded repositories: {:?}",
                excluded_repositories
            );
        }
        if !excluded_patterns.is_empty() {
            tracing::info!(
                "GitHub exclusion configured - excluded patterns: {:?}",
                excluded_patterns
            );
        }

        Self { client: builder.build().unwrap_or_else(|_| Client::new()), excluded_repositories, excluded_patterns }
    }

    /// Discover all accessible repositories for the authenticated user
    /// This includes repositories owned by the user and repositories where they are a member of an organization
    #[allow(dead_code)]
    pub async fn discover_repositories(
        &self,
        access_token: &str,
        namespace: Option<&str>,
    ) -> Result<Vec<GitHubRepository>> {
        // Treat empty string as None (no namespace filter)
        let namespace = namespace.filter(|s| !s.is_empty());

        let mut repositories = if let Some(org) = namespace {
            // Fetch organization repositories
            self.fetch_org_repositories(access_token, org).await?
        } else {
            // Fetch user repositories (owned + organization member)
            self.fetch_user_repositories(access_token).await?
        };

        // Filter out archived repositories and log how many were filtered
        let initial_count = repositories.len();
        repositories.retain(|r| !r.archived);
        let archived_count = initial_count - repositories.len();

        if archived_count > 0 {
            tracing::info!(
                "Filtered out {} archived GitHub repositories (kept {} active)",
                archived_count,
                repositories.len()
            );
        }

        Ok(repositories)
    }

    /// Fetch repositories accessible to the authenticated user
    #[allow(dead_code)]
    async fn fetch_user_repositories(&self, access_token: &str) -> Result<Vec<GitHubRepository>> {
        let mut repositories = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let url = format!(
                "https://api.github.com/user/repos?affiliation=owner,organization_member&per_page={}&page={}",
                per_page, page
            );

            tracing::debug!("Making GitHub API request to: {}", url);
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Accept", "application/vnd.github.v3+json")
                .send()
                .await
                .context("Failed to fetch GitHub repositories")?;

            let status = response.status();

            // Check rate limit before processing response
            Self::check_github_rate_limit(&response);

            if !status.is_success() {
                let error_body = response.text().await.unwrap_or_default();
                tracing::error!(
                    "GitHub API request failed - URL: {}, Status: {}, Body: {}",
                    url,
                    status,
                    error_body
                );
                return Err(anyhow::anyhow!("GitHub API error: {} - {}", status, error_body));
            }

            let page_repos: Vec<GitHubRepository> =
                response.json().await.context("Failed to parse GitHub repositories response")?;

            if page_repos.is_empty() {
                break;
            }

            repositories.extend(page_repos);
            page += 1;

            // GitHub has a rate limit, but for authenticated requests it's 5000/hour
            // which should be sufficient for pagination
        }

        Ok(repositories)
    }

    /// Fetch repositories for a specific organization
    #[allow(dead_code)]
    async fn fetch_org_repositories(&self, access_token: &str, org: &str) -> Result<Vec<GitHubRepository>> {
        let mut repositories = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let url = format!(
                "https://api.github.com/orgs/{}/repos?per_page={}&page={}",
                org, per_page, page
            );

            tracing::debug!("Making GitHub API request to: {}", url);
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Accept", "application/vnd.github.v3+json")
                .send()
                .await
                .context("Failed to fetch GitHub organization repositories")?;

            let status = response.status();

            // Check rate limit before processing response
            Self::check_github_rate_limit(&response);

            if !status.is_success() {
                let error_body = response.text().await.unwrap_or_default();
                tracing::error!(
                    "GitHub API request failed - URL: {}, Status: {}, Body: {}",
                    url,
                    status,
                    error_body
                );
                return Err(anyhow::anyhow!("GitHub API error: {} - {}", status, error_body));
            }

            let page_repos: Vec<GitHubRepository> =
                response.json().await.context("Failed to parse GitHub organization repositories response")?;

            if page_repos.is_empty() {
                break;
            }

            repositories.extend(page_repos);
            page += 1;
        }

        Ok(repositories)
    }

    /// Test if the access token is valid
    #[allow(dead_code)]
    pub async fn test_token(&self, access_token: &str) -> Result<bool> {
        let url = "https://api.github.com/user";

        tracing::debug!("Testing GitHub token with URL: {}", url);

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                tracing::info!("GitHub token test status: {}", status);

                // Check rate limit
                Self::check_github_rate_limit(&resp);

                if status.is_success() {
                    let user_info: serde_json::Value =
                        resp.json().await.context("Failed to parse GitHub user response")?;
                    tracing::info!(
                        "GitHub token valid for user: {}",
                        user_info["login"].as_str().unwrap_or("unknown")
                    );
                    Ok(true)
                } else {
                    let error_body = resp.text().await.unwrap_or_default();
                    tracing::error!("GitHub token test failed: {} - {}", status, error_body);
                    Ok(false)
                }
            }
            Err(e) => {
                tracing::error!("GitHub API request failed for URL: {}", url);
                tracing::error!("Request error: {}", e);
                if let Some(source) = std::error::Error::source(&e) {
                    tracing::error!("Error source: {:?}", source);
                }
                if let Some(status) = e.status() {
                    tracing::error!("HTTP status: {}", status);
                }
                Err(anyhow!("Network error: {} (URL: {})", e, url))
            }
        }
    }

    /// Check if a repository should be excluded from crawling
    #[allow(dead_code)]
    pub fn should_exclude_repository(&self, repository: &GitHubRepository) -> bool {
        self.should_exclude_repository_with_config(repository, &self.excluded_repositories, &self.excluded_patterns)
    }

    /// Check if a repository should be excluded with custom exclusion config
    fn should_exclude_repository_with_config(
        &self,
        repository: &GitHubRepository,
        excluded_repositories: &[String],
        excluded_patterns: &[String],
    ) -> bool {
        // Check exact matches
        if excluded_repositories.contains(&repository.full_name) {
            tracing::info!("Excluding repository (exact match): {}", repository.full_name);
            return true;
        }

        // Check pattern matches
        for pattern in excluded_patterns {
            if self.matches_pattern(pattern, &repository.full_name) {
                tracing::info!("Excluding repository (pattern '{}'): {}", pattern, repository.full_name);
                return true;
            }
        }

        false
    }

    /// Simple wildcard pattern matching (supports * as wildcard)
    fn matches_pattern(&self, pattern: &str, text: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if !pattern.contains('*') {
            return pattern == text;
        }

        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.is_empty() {
            return false;
        }

        let mut current_pos = 0;

        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            if i == 0 {
                // First part - must match beginning
                if !text.starts_with(part) {
                    return false;
                }
                current_pos = part.len();
            } else if i == parts.len() - 1 {
                // Last part - must match end
                if !text[current_pos..].ends_with(part) {
                    return false;
                }
            } else {
                // Middle part - must be found somewhere
                if let Some(pos) = text[current_pos..].find(part) {
                    current_pos += pos + part.len();
                } else {
                    return false;
                }
            }
        }

        true
    }

    /// Filter out excluded repositories from a list
    #[allow(dead_code)]
    pub fn filter_excluded_repositories(&self, repositories: Vec<GitHubRepository>) -> Vec<GitHubRepository> {
        self.filter_excluded_repositories_with_config(
            repositories,
            &self.excluded_repositories,
            &self.excluded_patterns,
        )
    }

    /// Filter out excluded repositories with custom exclusion config
    pub fn filter_excluded_repositories_with_config(
        &self,
        repositories: Vec<GitHubRepository>,
        excluded_repositories: &[String],
        excluded_patterns: &[String],
    ) -> Vec<GitHubRepository> {
        let initial_count = repositories.len();
        let filtered: Vec<GitHubRepository> = repositories
            .into_iter()
            .filter(|repository| {
                !self.should_exclude_repository_with_config(repository, excluded_repositories, excluded_patterns)
            })
            .collect();

        let excluded_count = initial_count - filtered.len();
        if excluded_count > 0 {
            tracing::info!(
                "Excluded {} out of {} GitHub repositories",
                excluded_count,
                initial_count
            );
        }

        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_repository(name: &str, owner: &str, id: i64) -> GitHubRepository {
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

    #[test]
    fn test_pattern_matching() {
        let service = GitHubService { client: Client::new(), excluded_repositories: vec![], excluded_patterns: vec![] };

        // Exact match
        assert!(service.matches_pattern("user/project", "user/project"));
        assert!(!service.matches_pattern("user/project", "other/project"));

        // Wildcard at end
        assert!(service.matches_pattern("user/*", "user/project"));
        assert!(service.matches_pattern("user/*", "user/another"));
        assert!(!service.matches_pattern("user/*", "other/project"));

        // Wildcard at beginning
        assert!(service.matches_pattern("*-archive", "project-archive"));
        assert!(service.matches_pattern("*-archive", "test-archive"));
        assert!(!service.matches_pattern("*-archive", "project-active"));

        // Wildcard in middle
        assert!(service.matches_pattern("user/*-archive", "user/project-archive"));
        assert!(service.matches_pattern("user/*-archive", "user/test-archive"));
        assert!(!service.matches_pattern("user/*-archive", "user/project-active"));

        // Multiple wildcards
        assert!(service.matches_pattern("*/*-*", "user/test-archive"));
        assert!(service.matches_pattern("*/*-*", "org/project-old"));
        assert!(!service.matches_pattern("*/*-*", "user/simple"));

        // Match all
        assert!(service.matches_pattern("*", "anything"));
        assert!(service.matches_pattern("*", ""));
    }

    #[test]
    fn test_should_exclude_repository_exact_match() {
        let service = GitHubService {
            client: Client::new(),
            excluded_repositories: vec!["user/large-project".to_string(), "org/archive".to_string()],
            excluded_patterns: vec![],
        };

        let repo1 = create_test_repository("large-project", "user", 1);
        let repo2 = create_test_repository("small-project", "user", 2);
        let repo3 = create_test_repository("archive", "org", 3);

        assert!(service.should_exclude_repository(&repo1));
        assert!(!service.should_exclude_repository(&repo2));
        assert!(service.should_exclude_repository(&repo3));
    }

    #[test]
    fn test_should_exclude_repository_pattern_match() {
        let service = GitHubService {
            client: Client::new(),
            excluded_repositories: vec![],
            excluded_patterns: vec!["*-archive".to_string(), "test/*".to_string(), "*/large-*".to_string()],
        };

        let repo1 = create_test_repository("project-archive", "user", 1);
        let repo2 = create_test_repository("project-active", "user", 2);
        let repo3 = create_test_repository("experiment", "test", 3);
        let repo4 = create_test_repository("large-project", "org", 4);
        let repo5 = create_test_repository("small-project", "org", 5);

        assert!(service.should_exclude_repository(&repo1)); // matches *-archive
        assert!(!service.should_exclude_repository(&repo2)); // no match
        assert!(service.should_exclude_repository(&repo3)); // matches test/*
        assert!(service.should_exclude_repository(&repo4)); // matches */large-*
        assert!(!service.should_exclude_repository(&repo5)); // no match
    }

    #[test]
    fn test_filter_excluded_repositories() {
        let service = GitHubService {
            client: Client::new(),
            excluded_repositories: vec!["user/exclude-me".to_string()],
            excluded_patterns: vec!["*-archive".to_string()],
        };

        let repositories = vec![
            create_test_repository("keep-me", "user", 1),
            create_test_repository("exclude-me", "user", 2),
            create_test_repository("project-archive", "user", 3),
            create_test_repository("another-active", "user", 4),
        ];

        let filtered = service.filter_excluded_repositories(repositories);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].full_name, "user/keep-me");
        assert_eq!(filtered[1].full_name, "user/another-active");
    }

    #[test]
    fn test_empty_exclusion_config() {
        let service = GitHubService { client: Client::new(), excluded_repositories: vec![], excluded_patterns: vec![] };

        let repositories =
            vec![create_test_repository("project1", "user", 1), create_test_repository("project2", "user", 2)];

        let filtered = service.filter_excluded_repositories(repositories.clone());
        assert_eq!(filtered.len(), repositories.len());
    }

    #[test]
    fn test_exclude_all_repositories() {
        let service = GitHubService {
            client: Client::new(),
            excluded_repositories: vec![],
            excluded_patterns: vec!["*".to_string()],
        };

        let repositories =
            vec![create_test_repository("project1", "user", 1), create_test_repository("project2", "user", 2)];

        let filtered = service.filter_excluded_repositories(repositories);
        assert_eq!(filtered.len(), 0);
    }
}
