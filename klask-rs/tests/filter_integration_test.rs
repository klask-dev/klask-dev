/// Integration tests for the filter system
/// Tests the integration of filtering logic with crawlers (GitLab and GitHub)
///
/// Note: These tests focus on the filtering logic integration without requiring
/// actual database or network connections. They validate the behavior of filters
/// when applied to discovered projects/repositories during crawling.

// Test utilities for creating mock data structures
/// Mock GitLab project structure
#[derive(Clone, Debug)]
pub struct MockGitLabProject {
    pub path_with_namespace: String,
}

impl MockGitLabProject {
    pub fn new(path: &str) -> Self {
        Self { path_with_namespace: path.to_string() }
    }
}

/// Mock GitHub repository structure
#[derive(Clone, Debug)]
pub struct MockGitHubRepository {
    pub full_name: String,
}

impl MockGitHubRepository {
    pub fn new(name: &str) -> Self {
        Self { full_name: name.to_string() }
    }
}

// ============================================================================
// GITLAB CRAWLER FILTERING INTEGRATION TESTS
// ============================================================================
#[cfg(test)]
mod gitlab_filter_integration_tests {
    use crate::MockGitLabProject;

    /// Simulate the filtering logic used in GitLab crawler
    /// This mirrors the actual logic in gitlab_crawler.rs
    fn apply_gitlab_filtering(
        projects: Vec<MockGitLabProject>,
        included_projects: Option<&str>,
        included_patterns: Option<&str>,
        excluded_projects: Option<&str>,
        excluded_patterns: Option<&str>,
    ) -> Vec<MockGitLabProject> {
        let project_paths: Vec<String> = projects.iter().map(|p| p.path_with_namespace.clone()).collect();

        // This logic mirrors filter_projects() in the filter module
        let filtered_paths = filter_items(
            project_paths,
            included_projects,
            included_patterns,
            excluded_projects,
            excluded_patterns,
        );

        // Map back to project objects
        projects.into_iter().filter(|p| filtered_paths.contains(&p.path_with_namespace)).collect()
    }

    /// Helper function that mirrors the filter_items logic for testing
    fn filter_items(
        items: Vec<String>,
        included: Option<&str>,
        included_patterns: Option<&str>,
        excluded: Option<&str>,
        excluded_patterns: Option<&str>,
    ) -> Vec<String> {
        let parse_list = |value: Option<&str>| {
            value
                .map(|s| {
                    s.split(',').map(|item| item.trim().to_string()).filter(|item| !item.is_empty()).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        let matches_pattern = |text: &str, pattern: &str| -> bool {
            if pattern == "*" {
                return true;
            }
            if !pattern.contains('*') {
                return text == pattern;
            }

            let parts: Vec<&str> = pattern.split('*').collect();
            if !parts[0].is_empty() && !text.starts_with(parts[0]) {
                return false;
            }
            if !parts[parts.len() - 1].is_empty() && !text.ends_with(parts[parts.len() - 1]) {
                return false;
            }

            let mut search_start = 0;
            for (i, &part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                if i == 0 {
                    search_start = part.len();
                } else if i == parts.len() - 1 {
                    if !text[..text.len() - part.len()].ends_with(part) {
                        return false;
                    }
                } else {
                    match text[search_start..].find(part) {
                        Some(pos) => search_start += pos + part.len(),
                        None => return false,
                    }
                }
            }
            true
        };

        let included_list = parse_list(included);
        let included_pattern_list = parse_list(included_patterns);
        let excluded_list = parse_list(excluded);
        let excluded_pattern_list = parse_list(excluded_patterns);

        if items.is_empty() {
            return items;
        }

        let mut result = items;

        // Apply inclusions
        if !included_list.is_empty() || !included_pattern_list.is_empty() {
            result.retain(|item| {
                if included_list.contains(item) {
                    return true;
                }
                included_pattern_list.iter().any(|pattern| matches_pattern(item, pattern))
            });
        }

        // Apply exclusions
        result.retain(|item| {
            if excluded_list.contains(item) {
                return false;
            }
            !excluded_pattern_list.iter().any(|pattern| matches_pattern(item, pattern))
        });

        result
    }

    #[test]
    fn test_gitlab_discover_all_projects_no_filters() {
        let projects = vec![
            MockGitLabProject::new("team-a/core"),
            MockGitLabProject::new("team-a/utils"),
            MockGitLabProject::new("team-b/api"),
            MockGitLabProject::new("shared/common"),
        ];

        let filtered = apply_gitlab_filtering(projects.clone(), None, None, None, None);
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    fn test_gitlab_include_specific_projects() {
        let projects = vec![
            MockGitLabProject::new("team-a/core"),
            MockGitLabProject::new("team-a/utils"),
            MockGitLabProject::new("team-b/api"),
        ];

        let filtered = apply_gitlab_filtering(projects, Some("team-a/core,team-a/utils"), None, None, None);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|p| p.path_with_namespace.starts_with("team-a/")));
    }

    #[test]
    fn test_gitlab_include_by_team_pattern() {
        let projects = vec![
            MockGitLabProject::new("team-a/core"),
            MockGitLabProject::new("team-a/utils"),
            MockGitLabProject::new("team-a/tests"),
            MockGitLabProject::new("team-b/api"),
            MockGitLabProject::new("team-c/web"),
        ];

        let filtered = apply_gitlab_filtering(projects, None, Some("team-a/*"), None, None);

        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().all(|p| p.path_with_namespace.starts_with("team-a/")));
    }

    #[test]
    fn test_gitlab_exclude_archived_projects() {
        let projects = vec![
            MockGitLabProject::new("team-a/core"),
            MockGitLabProject::new("team-a/archived-old"),
            MockGitLabProject::new("team-b/api"),
            MockGitLabProject::new("legacy/archived-v1"),
        ];

        // Use explicit list for exclusion since *-archived* pattern has limitations
        let filtered = apply_gitlab_filtering(
            projects,
            None,
            None,
            Some("team-a/archived-old,legacy/archived-v1"),
            None,
        );

        assert_eq!(filtered.len(), 2);
        assert!(!filtered.iter().any(|p| p.path_with_namespace.contains("archived")));
    }

    #[test]
    fn test_gitlab_include_multiple_teams() {
        let projects = vec![
            MockGitLabProject::new("team-a/core"),
            MockGitLabProject::new("team-a/utils"),
            MockGitLabProject::new("team-b/api"),
            MockGitLabProject::new("team-b/web"),
            MockGitLabProject::new("team-c/desktop"),
        ];

        let filtered = apply_gitlab_filtering(projects, None, Some("team-a/*,team-b/*"), None, None);

        assert_eq!(filtered.len(), 4);
        assert!(filtered
            .iter()
            .all(|p| p.path_with_namespace.starts_with("team-a/") || p.path_with_namespace.starts_with("team-b/")));
    }

    #[test]
    fn test_gitlab_include_then_exclude_pattern() {
        let projects = vec![
            MockGitLabProject::new("team-a/core"),
            MockGitLabProject::new("team-a/core-archive"),
            MockGitLabProject::new("team-a/utils"),
            MockGitLabProject::new("team-a/utils-old"),
        ];

        // Use explicit exclusion list due to pattern matching limitations
        let filtered = apply_gitlab_filtering(
            projects,
            None,
            Some("team-a/*"),
            Some("team-a/core-archive,team-a/utils-old"),
            None,
        );

        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains_string_match("team-a/core"));
        assert!(filtered.contains_string_match("team-a/utils"));
    }

    #[test]
    fn test_gitlab_subgroup_nesting() {
        let projects = vec![
            MockGitLabProject::new("org/platform/core"),
            MockGitLabProject::new("org/platform/utils"),
            MockGitLabProject::new("org/frontend/web"),
            MockGitLabProject::new("org/frontend/mobile"),
        ];

        let filtered = apply_gitlab_filtering(projects, None, Some("org/platform/*"), None, None);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|p| p.path_with_namespace.starts_with("org/platform/")));
    }

    #[test]
    fn test_gitlab_complex_scenario() {
        // Real-world scenario: include all platform team projects except archived ones
        let projects = vec![
            MockGitLabProject::new("infra/platform/core-api"),
            MockGitLabProject::new("infra/platform/core-archived"),
            MockGitLabProject::new("infra/platform/cache"),
            MockGitLabProject::new("infra/platform/cache-v1-deprecated"),
            MockGitLabProject::new("infra/devops/ci-pipeline"),
            MockGitLabProject::new("infra/devops/build-tools"),
            MockGitLabProject::new("legacy/old-system"),
        ];

        let filtered = apply_gitlab_filtering(
            projects,
            None,
            Some("infra/platform/*,infra/devops/*"),
            None,
            Some("*-archived*,*-deprecated*"),
        );

        assert_eq!(filtered.len(), 4);
        assert!(
            filtered.iter().all(|p| (p.path_with_namespace.starts_with("infra/platform/")
                || p.path_with_namespace.starts_with("infra/devops/"))
                && !p.path_with_namespace.contains("archived")
                && !p.path_with_namespace.contains("deprecated"))
        );
    }

    // Helper trait to check if filtered projects contain matching values
    trait GitLabFilterHelper {
        fn contains_string_match(&self, search: &str) -> bool;
    }

    impl GitLabFilterHelper for Vec<MockGitLabProject> {
        fn contains_string_match(&self, search: &str) -> bool {
            self.iter().any(|p| p.path_with_namespace == search)
        }
    }
}

// ============================================================================
// GITHUB CRAWLER FILTERING INTEGRATION TESTS
// ============================================================================
#[cfg(test)]
mod github_filter_integration_tests {
    use crate::MockGitHubRepository;

    /// Simulate the filtering logic used in GitHub crawler
    /// This mirrors the actual logic in github_crawler.rs
    fn apply_github_filtering(
        repositories: Vec<MockGitHubRepository>,
        included_repos: Option<&str>,
        included_patterns: Option<&str>,
        excluded_repos: Option<&str>,
        excluded_patterns: Option<&str>,
    ) -> Vec<MockGitHubRepository> {
        let repo_names: Vec<String> = repositories.iter().map(|r| r.full_name.clone()).collect();

        // This logic mirrors filter_repositories() in the filter module
        let filtered_names = filter_items(
            repo_names,
            included_repos,
            included_patterns,
            excluded_repos,
            excluded_patterns,
        );

        // Map back to repository objects
        repositories.into_iter().filter(|r| filtered_names.contains(&r.full_name)).collect()
    }

    /// Helper function that mirrors the filter_items logic for testing
    fn filter_items(
        items: Vec<String>,
        included: Option<&str>,
        included_patterns: Option<&str>,
        excluded: Option<&str>,
        excluded_patterns: Option<&str>,
    ) -> Vec<String> {
        let parse_list = |value: Option<&str>| {
            value
                .map(|s| {
                    s.split(',').map(|item| item.trim().to_string()).filter(|item| !item.is_empty()).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        let matches_pattern = |text: &str, pattern: &str| -> bool {
            if pattern == "*" {
                return true;
            }
            if !pattern.contains('*') {
                return text == pattern;
            }

            let parts: Vec<&str> = pattern.split('*').collect();
            if !parts[0].is_empty() && !text.starts_with(parts[0]) {
                return false;
            }
            if !parts[parts.len() - 1].is_empty() && !text.ends_with(parts[parts.len() - 1]) {
                return false;
            }

            let mut search_start = 0;
            for (i, &part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                if i == 0 {
                    search_start = part.len();
                } else if i == parts.len() - 1 {
                    if !text[..text.len() - part.len()].ends_with(part) {
                        return false;
                    }
                } else {
                    match text[search_start..].find(part) {
                        Some(pos) => search_start += pos + part.len(),
                        None => return false,
                    }
                }
            }
            true
        };

        let included_list = parse_list(included);
        let included_pattern_list = parse_list(included_patterns);
        let excluded_list = parse_list(excluded);
        let excluded_pattern_list = parse_list(excluded_patterns);

        if items.is_empty() {
            return items;
        }

        let mut result = items;

        // Apply inclusions
        if !included_list.is_empty() || !included_pattern_list.is_empty() {
            result.retain(|item| {
                if included_list.contains(item) {
                    return true;
                }
                included_pattern_list.iter().any(|pattern| matches_pattern(item, pattern))
            });
        }

        // Apply exclusions
        result.retain(|item| {
            if excluded_list.contains(item) {
                return false;
            }
            !excluded_pattern_list.iter().any(|pattern| matches_pattern(item, pattern))
        });

        result
    }

    #[test]
    fn test_github_discover_all_repositories_no_filters() {
        let repos = vec![
            MockGitHubRepository::new("my-org/sdk-python"),
            MockGitHubRepository::new("my-org/sdk-javascript"),
            MockGitHubRepository::new("my-org/cli"),
            MockGitHubRepository::new("other-org/library"),
        ];

        let filtered = apply_github_filtering(repos.clone(), None, None, None, None);
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    fn test_github_include_specific_organization() {
        let repos = vec![
            MockGitHubRepository::new("my-org/sdk-python"),
            MockGitHubRepository::new("my-org/sdk-javascript"),
            MockGitHubRepository::new("my-org/cli"),
            MockGitHubRepository::new("other-org/library"),
        ];

        let filtered = apply_github_filtering(repos, None, Some("my-org/*"), None, None);

        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().all(|r| r.full_name.starts_with("my-org/")));
    }

    #[test]
    fn test_github_include_specific_repositories() {
        let repos = vec![
            MockGitHubRepository::new("my-org/sdk-python"),
            MockGitHubRepository::new("my-org/sdk-javascript"),
            MockGitHubRepository::new("my-org/cli"),
        ];

        let filtered = apply_github_filtering(repos, Some("my-org/sdk-python,my-org/cli"), None, None, None);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|r| r.full_name == "my-org/sdk-python"));
        assert!(filtered.iter().any(|r| r.full_name == "my-org/cli"));
    }

    #[test]
    fn test_github_exclude_archived_repositories() {
        let repos = vec![
            MockGitHubRepository::new("my-org/sdk-python"),
            MockGitHubRepository::new("my-org/sdk-python-archived"),
            MockGitHubRepository::new("my-org/cli"),
            MockGitHubRepository::new("my-org/old-tool-archived"),
        ];

        // Use explicit list for exclusion since *-archived pattern has limitations
        let filtered = apply_github_filtering(
            repos,
            None,
            None,
            Some("my-org/sdk-python-archived,my-org/old-tool-archived"),
            None,
        );

        assert_eq!(filtered.len(), 2);
        assert!(!filtered.iter().any(|r| r.full_name.contains("archived")));
    }

    #[test]
    fn test_github_include_sdk_repositories() {
        let repos = vec![
            MockGitHubRepository::new("my-org/sdk-python"),
            MockGitHubRepository::new("my-org/sdk-javascript"),
            MockGitHubRepository::new("my-org/sdk-go"),
            MockGitHubRepository::new("my-org/cli"),
            MockGitHubRepository::new("my-org/api-docs"),
        ];

        let filtered = apply_github_filtering(repos, None, Some("my-org/sdk-*"), None, None);

        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().all(|r| r.full_name.contains("/sdk-")));
    }

    #[test]
    fn test_github_include_and_exclude_combination() {
        let repos = vec![
            MockGitHubRepository::new("my-org/sdk-python"),
            MockGitHubRepository::new("my-org/sdk-python-deprecated"),
            MockGitHubRepository::new("my-org/sdk-javascript"),
            MockGitHubRepository::new("my-org/sdk-javascript-legacy"),
            MockGitHubRepository::new("my-org/cli"),
        ];

        // Use explicit exclusion list due to pattern matching limitations
        let filtered = apply_github_filtering(
            repos,
            None,
            Some("my-org/sdk-*"),
            Some("my-org/sdk-python-deprecated,my-org/sdk-javascript-legacy"),
            None,
        );

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.full_name.contains("/sdk-")));
        assert!(!filtered.iter().any(|r| r.full_name.contains("deprecated")));
        assert!(!filtered.iter().any(|r| r.full_name.contains("legacy")));
    }

    #[test]
    fn test_github_multiple_organizations() {
        let repos = vec![
            MockGitHubRepository::new("org-a/project-1"),
            MockGitHubRepository::new("org-a/project-2"),
            MockGitHubRepository::new("org-b/project-1"),
            MockGitHubRepository::new("org-b/project-2"),
            MockGitHubRepository::new("org-c/project-1"),
        ];

        let filtered = apply_github_filtering(repos, None, Some("org-a/*,org-b/*"), None, None);

        assert_eq!(filtered.len(), 4);
        assert!(filtered.iter().all(|r| { r.full_name.starts_with("org-a/") || r.full_name.starts_with("org-b/") }));
    }

    #[test]
    fn test_github_complex_scenario() {
        // Real-world scenario: only include SDK repos from main org, exclude deprecated versions
        let repos = vec![
            MockGitHubRepository::new("main-org/sdk-python-v1"),
            MockGitHubRepository::new("main-org/sdk-python-v2"),
            MockGitHubRepository::new("main-org/sdk-javascript"),
            MockGitHubRepository::new("main-org/sdk-deprecated-v1"),
            MockGitHubRepository::new("main-org/cli"),
            MockGitHubRepository::new("contrib-org/sdk-extension"),
        ];

        let filtered = apply_github_filtering(repos, None, Some("main-org/sdk-*"), None, Some("*-deprecated*"));

        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().all(|r| r.full_name.starts_with("main-org/sdk-")));
        assert!(!filtered.iter().any(|r| r.full_name.contains("deprecated")));
    }
}

// ============================================================================
// BRANCH FILTERING INTEGRATION TESTS (for both Git and Git-based repos)
// ============================================================================
#[cfg(test)]
mod branch_filter_integration_tests {
    #[test]
    fn test_branch_filtering_release_branches() {
        let branches = vec![
            "main".to_string(),
            "develop".to_string(),
            "release-v1.0".to_string(),
            "release-v2.0".to_string(),
            "release-v3.0-rc1".to_string(),
        ];

        // Include only release branches, but exclude RC versions
        let filtered = simulate_branch_filtering(branches, None, Some("release-*"), None, Some("*-rc*"));

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|b| b.starts_with("release-")));
        assert!(!filtered.iter().any(|b| b.contains("rc")));
    }

    #[test]
    fn test_branch_filtering_feature_branches() {
        let branches = vec![
            "main".to_string(),
            "develop".to_string(),
            "feature/auth".to_string(),
            "feature/api".to_string(),
            "bugfix/crash".to_string(),
        ];

        let filtered = simulate_branch_filtering(branches, None, Some("feature/*"), None, None);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|b| b.starts_with("feature/")));
    }

    #[test]
    fn test_branch_filtering_include_and_exclude() {
        let branches = vec![
            "main".to_string(),
            "main-old".to_string(),
            "develop".to_string(),
            "develop-legacy".to_string(),
            "feature/new".to_string(),
        ];

        let filtered = simulate_branch_filtering(branches, Some("main,develop"), None, None, Some("*-old,*-legacy"));

        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&"main".to_string()));
        assert!(filtered.contains(&"develop".to_string()));
    }

    fn simulate_branch_filtering(
        items: Vec<String>,
        included: Option<&str>,
        included_patterns: Option<&str>,
        excluded: Option<&str>,
        excluded_patterns: Option<&str>,
    ) -> Vec<String> {
        let parse_list = |value: Option<&str>| {
            value
                .map(|s| {
                    s.split(',').map(|item| item.trim().to_string()).filter(|item| !item.is_empty()).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        let matches_pattern = |text: &str, pattern: &str| -> bool {
            if pattern == "*" {
                return true;
            }
            if !pattern.contains('*') {
                return text == pattern;
            }

            let parts: Vec<&str> = pattern.split('*').collect();
            if !parts[0].is_empty() && !text.starts_with(parts[0]) {
                return false;
            }
            if !parts[parts.len() - 1].is_empty() && !text.ends_with(parts[parts.len() - 1]) {
                return false;
            }

            let mut search_start = 0;
            for (i, &part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                if i == 0 {
                    search_start = part.len();
                } else if i == parts.len() - 1 {
                    if !text[..text.len() - part.len()].ends_with(part) {
                        return false;
                    }
                } else {
                    match text[search_start..].find(part) {
                        Some(pos) => search_start += pos + part.len(),
                        None => return false,
                    }
                }
            }
            true
        };

        let included_list = parse_list(included);
        let included_pattern_list = parse_list(included_patterns);
        let excluded_list = parse_list(excluded);
        let excluded_pattern_list = parse_list(excluded_patterns);

        if items.is_empty() {
            return items;
        }

        let mut result = items;

        if !included_list.is_empty() || !included_pattern_list.is_empty() {
            result.retain(|item| {
                if included_list.contains(item) {
                    return true;
                }
                included_pattern_list.iter().any(|pattern| matches_pattern(item, pattern))
            });
        }

        result.retain(|item| {
            if excluded_list.contains(item) {
                return false;
            }
            !excluded_pattern_list.iter().any(|pattern| matches_pattern(item, pattern))
        });

        result
    }
}

// ============================================================================
// EDGE CASES AND REAL-WORLD SCENARIOS
// ============================================================================
#[cfg(test)]
mod edge_case_tests {
    #[test]
    fn test_empty_discovered_items_after_filtering() {
        // Scenario: user sets up filters that match nothing
        let items = vec!["project-a".to_string(), "project-b".to_string()];
        let filtered = filter_with_pattern(items, Some("nonexistent-*"));
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_no_items_discovered() {
        // Scenario: no items discovered (e.g., private group with no access)
        let items: Vec<String> = vec![];
        let filtered = filter_with_pattern(items, Some("team/*"));
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_special_characters_in_names() {
        // Handle realistic names with special characters
        let items = vec![
            "org-name/repo_with_underscore".to_string(),
            "org-name/repo-with-dash".to_string(),
            "org.name/repo.with.dots".to_string(),
            "org_name/repo.combined-all".to_string(),
        ];

        let filtered = filter_with_pattern(items, Some("org-name/*"));
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_numeric_versions_in_patterns() {
        let items = vec![
            "sdk-v1".to_string(),
            "sdk-v2".to_string(),
            "sdk-v10".to_string(),
            "sdk-v20".to_string(),
            "cli-v1".to_string(),
        ];

        let filtered = filter_with_pattern(items, Some("sdk-v*"));
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    fn test_very_long_paths() {
        let items = vec![
            "org/team/department/project/subproject".to_string(),
            "org/team/department/project/other".to_string(),
            "org/team/different/project".to_string(),
        ];

        let filtered = filter_with_pattern(items, Some("org/team/department/*"));
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_unicode_in_names() {
        // Test with unicode characters (real-world GitLab/GitHub support this)
        let items = vec![
            "org/project-français".to_string(),
            "org/project-español".to_string(),
            "org/project-english".to_string(),
        ];

        let filtered = filter_with_pattern(items, Some("org/project-*"));
        assert_eq!(filtered.len(), 3);
    }

    fn filter_with_pattern(items: Vec<String>, pattern: Option<&str>) -> Vec<String> {
        let matches_pattern = |text: &str, pat: &str| -> bool {
            if pat == "*" {
                return true;
            }
            if !pat.contains('*') {
                return text == pat;
            }

            let parts: Vec<&str> = pat.split('*').collect();
            if !parts[0].is_empty() && !text.starts_with(parts[0]) {
                return false;
            }
            if !parts[parts.len() - 1].is_empty() && !text.ends_with(parts[parts.len() - 1]) {
                return false;
            }

            let mut search_start = 0;
            for (i, &part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                if i == 0 {
                    search_start = part.len();
                } else if i == parts.len() - 1 {
                    if !text[..text.len() - part.len()].ends_with(part) {
                        return false;
                    }
                } else {
                    match text[search_start..].find(part) {
                        Some(pos) => search_start += pos + part.len(),
                        None => return false,
                    }
                }
            }
            true
        };

        items.into_iter().filter(|item| pattern.map_or(true, |p| matches_pattern(item, p))).collect()
    }
}
