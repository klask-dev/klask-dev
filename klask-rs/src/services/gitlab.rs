use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GitLabService {
    client: Client,
    #[allow(dead_code)]
    excluded_projects: Vec<String>,
    #[allow(dead_code)]
    excluded_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabProject {
    pub id: i64,
    pub name: String,
    pub path_with_namespace: String,
    pub description: Option<String>,
    pub default_branch: Option<String>,
    pub http_url_to_repo: String,
    pub ssh_url_to_repo: String,
    pub web_url: String,
    pub visibility: String,
    pub archived: bool,
}

impl Default for GitLabService {
    fn default() -> Self {
        Self::new()
    }
}

impl GitLabService {
    pub fn new() -> Self {
        let accept_invalid_certs =
            std::env::var("KLASK_GITLAB_ACCEPT_INVALID_CERTS").map(|v| v.to_lowercase() == "true").unwrap_or(false);

        let mut builder = Client::builder().user_agent("klask-rs/2.0").timeout(std::time::Duration::from_secs(30));

        if accept_invalid_certs {
            tracing::warn!(
                "GitLab client configured to accept invalid certificates (KLASK_GITLAB_ACCEPT_INVALID_CERTS=true)"
            );
            builder = builder.danger_accept_invalid_certs(true);
        }

        // Parse excluded projects from environment variables
        let excluded_projects = std::env::var("KLASK_GITLAB_EXCLUDED_PROJECTS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        let excluded_patterns = std::env::var("KLASK_GITLAB_EXCLUDED_PATTERNS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        if !excluded_projects.is_empty() {
            tracing::info!(
                "GitLab exclusion configured - excluded projects: {:?}",
                excluded_projects
            );
        }
        if !excluded_patterns.is_empty() {
            tracing::info!(
                "GitLab exclusion configured - excluded patterns: {:?}",
                excluded_patterns
            );
        }

        Self { client: builder.build().unwrap_or_else(|_| Client::new()), excluded_projects, excluded_patterns }
    }

    /// Discover all accessible projects for a GitLab instance
    pub async fn discover_projects(
        &self,
        gitlab_url: &str,
        access_token: &str,
        namespace: Option<&str>,
    ) -> Result<Vec<GitLabProject>> {
        let mut projects = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let mut url = format!(
                "{}/api/v4/projects?page={}&per_page={}&membership=true&simple=false",
                gitlab_url.trim_end_matches('/'),
                page,
                per_page
            );

            // If namespace is provided, filter by it
            if let Some(ns) = namespace {
                url.push_str(&format!("&search_namespaces=true&search={}", ns));
            }

            tracing::debug!("Making GitLab API request to: {}", url);
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await
                .context("Failed to fetch GitLab projects")?;

            let status = response.status();
            if !status.is_success() {
                let error_body = response.text().await.unwrap_or_default();
                tracing::error!(
                    "GitLab API request failed - URL: {}, Status: {}, Body: {}",
                    url,
                    status,
                    error_body
                );
                return Err(anyhow::anyhow!("GitLab API error: {} - {}", status, error_body));
            }

            let page_projects: Vec<GitLabProject> =
                response.json().await.context("Failed to parse GitLab projects response")?;

            if page_projects.is_empty() {
                break;
            }

            projects.extend(page_projects);
            page += 1;
        }

        // Filter out archived projects
        projects.retain(|p| !p.archived);

        Ok(projects)
    }

    /// Get all projects in a specific group
    #[allow(dead_code)]
    pub async fn get_group_projects(
        &self,
        gitlab_url: &str,
        access_token: &str,
        group_id: &str,
    ) -> Result<Vec<GitLabProject>> {
        let mut projects = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let url = format!(
                "{}/api/v4/groups/{}/projects?page={}&per_page={}&include_subgroups=true&archived=false",
                gitlab_url.trim_end_matches('/'),
                group_id,
                page,
                per_page
            );

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await
                .context("Failed to fetch GitLab group projects")?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                    "GitLab API error: {} - {}",
                    response.status(),
                    response.text().await.unwrap_or_default()
                ));
            }

            let page_projects: Vec<GitLabProject> =
                response.json().await.context("Failed to parse GitLab group projects response")?;

            if page_projects.is_empty() {
                break;
            }

            projects.extend(page_projects);
            page += 1;
        }

        Ok(projects)
    }

    /// Get information about a specific project
    #[allow(dead_code)]
    pub async fn get_project(&self, gitlab_url: &str, access_token: &str, project_id: &str) -> Result<GitLabProject> {
        let url = format!(
            "{}/api/v4/projects/{}",
            gitlab_url.trim_end_matches('/'),
            urlencoding::encode(project_id)
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Failed to fetch GitLab project")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "GitLab API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let project: GitLabProject = response.json().await.context("Failed to parse GitLab project response")?;

        Ok(project)
    }

    /// Check if a project should be excluded from crawling
    #[allow(dead_code)]
    pub fn should_exclude_project(&self, project: &GitLabProject) -> bool {
        self.should_exclude_project_with_config(project, &self.excluded_projects, &self.excluded_patterns)
    }

    /// Check if a project should be excluded with custom exclusion config
    fn should_exclude_project_with_config(
        &self,
        project: &GitLabProject,
        excluded_projects: &[String],
        excluded_patterns: &[String],
    ) -> bool {
        // Check exact matches
        if excluded_projects.contains(&project.path_with_namespace) {
            tracing::info!("Excluding project (exact match): {}", project.path_with_namespace);
            return true;
        }

        // Check pattern matches
        for pattern in excluded_patterns {
            if self.matches_pattern(pattern, &project.path_with_namespace) {
                tracing::info!(
                    "Excluding project (pattern '{}'): {}",
                    pattern,
                    project.path_with_namespace
                );
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

    /// Filter out excluded projects from a list
    #[allow(dead_code)]
    pub fn filter_excluded_projects(&self, projects: Vec<GitLabProject>) -> Vec<GitLabProject> {
        self.filter_excluded_projects_with_config(projects, &self.excluded_projects, &self.excluded_patterns)
    }

    /// Filter out excluded projects with custom exclusion config
    pub fn filter_excluded_projects_with_config(
        &self,
        projects: Vec<GitLabProject>,
        excluded_projects: &[String],
        excluded_patterns: &[String],
    ) -> Vec<GitLabProject> {
        let initial_count = projects.len();
        let filtered: Vec<GitLabProject> = projects
            .into_iter()
            .filter(|project| !self.should_exclude_project_with_config(project, excluded_projects, excluded_patterns))
            .collect();

        let excluded_count = initial_count - filtered.len();
        if excluded_count > 0 {
            tracing::info!("Excluded {} out of {} GitLab projects", excluded_count, initial_count);
        }

        filtered
    }

    /// Test if the access token is valid
    pub async fn test_token(&self, gitlab_url: &str, access_token: &str) -> Result<bool> {
        let url = format!("{}/api/v4/user", gitlab_url.trim_end_matches('/'));

        // Test both authentication methods to compare
        tracing::debug!("Testing GitLab token with URL: {}", url);

        // Test with PRIVATE-TOKEN first
        tracing::info!("Testing with PRIVATE-TOKEN header");
        let response_private = self.client.get(&url).header("PRIVATE-TOKEN", access_token).send().await;

        // Test with Bearer token
        tracing::info!("Testing with Authorization Bearer header");
        let response_bearer =
            self.client.get(&url).header("Authorization", format!("Bearer {}", access_token)).send().await;

        // Compare results
        match (response_private, response_bearer) {
            (Ok(resp_private), Ok(resp_bearer)) => {
                let status_private = resp_private.status();
                let status_bearer = resp_bearer.status();

                tracing::info!("PRIVATE-TOKEN status: {}", status_private);
                tracing::info!("Bearer token status: {}", status_bearer);

                if status_private.is_success() && status_bearer.is_success() {
                    // Both work, let's compare response bodies
                    let body_private = resp_private.text().await.unwrap_or_default();
                    let body_bearer = resp_bearer.text().await.unwrap_or_default();

                    if body_private == body_bearer {
                        tracing::info!("Both auth methods return identical responses");
                    } else {
                        tracing::warn!("Auth methods return different responses!");
                        tracing::debug!("PRIVATE-TOKEN response: {}", body_private);
                        tracing::debug!("Bearer response: {}", body_bearer);
                    }

                    // Use PRIVATE-TOKEN as primary (more explicit for GitLab)
                    Ok(true)
                } else if status_private.is_success() {
                    tracing::warn!("Only PRIVATE-TOKEN works, Bearer failed with: {}", status_bearer);
                    Ok(true)
                } else if status_bearer.is_success() {
                    tracing::warn!("Only Bearer works, PRIVATE-TOKEN failed with: {}", status_private);
                    Ok(true)
                } else {
                    tracing::error!(
                        "Both auth methods failed - PRIVATE-TOKEN: {}, Bearer: {}",
                        status_private,
                        status_bearer
                    );
                    let error_body = resp_private.text().await.unwrap_or_default();
                    tracing::error!("Error body: {}", error_body);
                    Ok(false)
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                tracing::error!("GitLab API request failed for URL: {}", url);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_project(name: &str, id: i64) -> GitLabProject {
        GitLabProject {
            id,
            name: name.to_string(),
            path_with_namespace: name.to_string(),
            description: Some("Test project".to_string()),
            default_branch: Some("main".to_string()),
            http_url_to_repo: format!("https://gitlab.example.com/{}.git", name),
            ssh_url_to_repo: format!("git@gitlab.example.com:{}.git", name),
            web_url: format!("https://gitlab.example.com/{}", name),
            visibility: "private".to_string(),
            archived: false,
        }
    }

    #[test]
    fn test_pattern_matching() {
        let service = GitLabService { client: Client::new(), excluded_projects: vec![], excluded_patterns: vec![] };

        // Exact match
        assert!(service.matches_pattern("project", "project"));
        assert!(!service.matches_pattern("project", "other"));

        // Wildcard at end
        assert!(service.matches_pattern("test-*", "test-project"));
        assert!(service.matches_pattern("test-*", "test-large"));
        assert!(!service.matches_pattern("test-*", "other-project"));

        // Wildcard at beginning
        assert!(service.matches_pattern("*-archive", "project-archive"));
        assert!(service.matches_pattern("*-archive", "test-archive"));
        assert!(!service.matches_pattern("*-archive", "project-active"));

        // Wildcard in middle
        assert!(service.matches_pattern("test-*-archive", "test-project-archive"));
        assert!(service.matches_pattern("test-*-archive", "test-large-archive"));
        assert!(!service.matches_pattern("test-*-archive", "test-project-active"));

        // Multiple wildcards
        assert!(service.matches_pattern("*-*-*", "a-b-c"));
        assert!(service.matches_pattern("*-*-*", "test-project-archive"));
        assert!(!service.matches_pattern("*-*-*", "single"));

        // Match all
        assert!(service.matches_pattern("*", "anything"));
        assert!(service.matches_pattern("*", ""));
    }

    #[test]
    fn test_should_exclude_project_exact_match() {
        let service = GitLabService {
            client: Client::new(),
            excluded_projects: vec!["team/large-project".to_string(), "archive/old-system".to_string()],
            excluded_patterns: vec![],
        };

        let project1 = create_test_project("team/large-project", 1);
        let project2 = create_test_project("team/small-project", 2);
        let project3 = create_test_project("archive/old-system", 3);

        assert!(service.should_exclude_project(&project1));
        assert!(!service.should_exclude_project(&project2));
        assert!(service.should_exclude_project(&project3));
    }

    #[test]
    fn test_should_exclude_project_pattern_match() {
        let service = GitLabService {
            client: Client::new(),
            excluded_projects: vec![],
            excluded_patterns: vec!["*-archive".to_string(), "test-*".to_string(), "*-large-*".to_string()],
        };

        let project1 = create_test_project("team/project-archive", 1);
        let project2 = create_test_project("team/project-active", 2);
        let project3 = create_test_project("test-experiment", 3);
        let project4 = create_test_project("team/very-large-project", 4);
        let project5 = create_test_project("team/small-project", 5);

        assert!(service.should_exclude_project(&project1)); // matches *-archive
        assert!(!service.should_exclude_project(&project2)); // no match
        assert!(service.should_exclude_project(&project3)); // matches test-*
        assert!(service.should_exclude_project(&project4)); // matches *-large-*
        assert!(!service.should_exclude_project(&project5)); // no match
    }

    #[test]
    fn test_filter_excluded_projects() {
        let service = GitLabService {
            client: Client::new(),
            excluded_projects: vec!["team/exclude-me".to_string()],
            excluded_patterns: vec!["*-archive".to_string()],
        };

        let projects = vec![
            create_test_project("team/keep-me", 1),
            create_test_project("team/exclude-me", 2),
            create_test_project("team/project-archive", 3),
            create_test_project("team/another-active", 4),
        ];

        let filtered = service.filter_excluded_projects(projects);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].path_with_namespace, "team/keep-me");
        assert_eq!(filtered[1].path_with_namespace, "team/another-active");
    }

    #[test]
    fn test_empty_exclusion_config() {
        let service = GitLabService { client: Client::new(), excluded_projects: vec![], excluded_patterns: vec![] };

        let projects = vec![create_test_project("team/project1", 1), create_test_project("team/project2", 2)];

        let filtered = service.filter_excluded_projects(projects.clone());
        assert_eq!(filtered.len(), projects.len());
    }

    #[test]
    fn test_exclude_all_projects() {
        let service = GitLabService {
            client: Client::new(),
            excluded_projects: vec![],
            excluded_patterns: vec!["*".to_string()],
        };

        let projects = vec![create_test_project("team/project1", 1), create_test_project("team/project2", 2)];

        let filtered = service.filter_excluded_projects(projects);
        assert_eq!(filtered.len(), 0);
    }
}
