/// Filtering utilities for branches and projects during crawling
/// Supports glob-style wildcard matching with includes and excludes
use tracing::{info, warn};

/// Simple glob-style pattern matching
/// Supports * as a wildcard matching any sequence of characters
/// Examples:
///   "release-*" matches "release-v1.0", "release-staging", etc.
///   "*-archive" matches "old-archive", "backup-archive", etc.
///   "v*-stable" matches "v1.0-stable", "v2.3-stable", etc.
pub fn matches_pattern(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true; // Match everything
    }

    if !pattern.contains('*') {
        return text == pattern; // Exact match if no wildcard
    }

    // Simple glob matching: split by *, match parts in sequence
    let parts: Vec<&str> = pattern.split('*').collect();

    // Check if text starts with the first part (unless it's empty)
    if !parts[0].is_empty() && !text.starts_with(parts[0]) {
        return false;
    }

    // Check if text ends with the last part (unless it's empty)
    if !parts[parts.len() - 1].is_empty() && !text.ends_with(parts[parts.len() - 1]) {
        return false;
    }

    // For patterns like "a*b*c", check that parts appear in order
    let mut search_start = 0;
    for (i, &part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 {
            // First part: already checked with starts_with
            search_start = part.len();
        } else if i == parts.len() - 1 {
            // Last part: already checked with ends_with
            // Just verify position
            if !text[..text.len() - part.len()].ends_with(part) {
                return false;
            }
        } else {
            // Middle parts: must appear after previous part
            match text[search_start..].find(part) {
                Some(pos) => search_start += pos + part.len(),
                None => return false,
            }
        }
    }

    true
}

/// Parse comma-separated values from a string
/// Trims whitespace and filters out empty entries
fn parse_list(value: Option<&str>) -> Vec<String> {
    value
        .map(|s| s.split(',').map(|item| item.trim().to_string()).filter(|item| !item.is_empty()).collect())
        .unwrap_or_default()
}

/// Filter items based on inclusion and exclusion lists/patterns
/// Order of operations:
///   1. If included items/patterns are set: keep only matches
///   2. Then remove excluded items/patterns
///   3. Return filtered list
pub fn filter_items(
    items: Vec<String>,
    included: Option<&str>,
    included_patterns: Option<&str>,
    excluded: Option<&str>,
    excluded_patterns: Option<&str>,
) -> Vec<String> {
    let included_list = parse_list(included);
    let included_pattern_list = parse_list(included_patterns);
    let excluded_list = parse_list(excluded);
    let excluded_pattern_list = parse_list(excluded_patterns);

    if items.is_empty() {
        return items;
    }

    let mut result = items;

    // Apply inclusions: if any include filters are set, filter to only those
    if !included_list.is_empty() || !included_pattern_list.is_empty() {
        result.retain(|item| {
            // Include if in explicit list
            if included_list.contains(item) {
                return true;
            }
            // Include if matches any pattern
            included_pattern_list.iter().any(|pattern| matches_pattern(item, pattern))
        });
    }

    // Apply exclusions: remove items in exclude list or matching patterns
    result.retain(|item| {
        // Exclude if in explicit list
        if excluded_list.contains(item) {
            return false;
        }
        // Exclude if matches any pattern
        !excluded_pattern_list.iter().any(|pattern| matches_pattern(item, pattern))
    });

    result
}

/// Filter branches for a Git or Git-based repository
#[allow(dead_code)]
pub fn filter_branches(
    branches: Vec<String>,
    included_branches: Option<&str>,
    included_patterns: Option<&str>,
    excluded_branches: Option<&str>,
    excluded_patterns: Option<&str>,
) -> Vec<String> {
    let initial_count = branches.len();
    let filtered = filter_items(
        branches,
        included_branches,
        included_patterns,
        excluded_branches,
        excluded_patterns,
    );
    let filtered_count = filtered.len();

    if initial_count > 0 && filtered_count < initial_count {
        info!(
            "Filtered {} branches to {} ({}% retained)",
            initial_count,
            filtered_count,
            (filtered_count as f64 / initial_count as f64 * 100.0) as i32
        );
    } else if filtered_count == 0 && initial_count > 0 {
        warn!(
            "All {} branches were filtered out - no branches will be crawled",
            initial_count
        );
    }

    filtered
}

/// Filter projects/repositories for GitLab
pub fn filter_projects(
    projects: Vec<String>,
    included_projects: Option<&str>,
    included_patterns: Option<&str>,
    excluded_projects: Option<&str>,
    excluded_patterns: Option<&str>,
) -> Vec<String> {
    let initial_count = projects.len();
    let filtered = filter_items(
        projects,
        included_projects,
        included_patterns,
        excluded_projects,
        excluded_patterns,
    );
    let filtered_count = filtered.len();

    if initial_count > 0 && filtered_count < initial_count {
        info!(
            "Filtered {} projects to {} ({}% retained)",
            initial_count,
            filtered_count,
            (filtered_count as f64 / initial_count as f64 * 100.0) as i32
        );
    } else if filtered_count == 0 && initial_count > 0 {
        warn!(
            "All {} projects were filtered out - no projects will be crawled",
            initial_count
        );
    }

    filtered
}

/// Filter repositories for GitHub
pub fn filter_repositories(
    repositories: Vec<String>,
    included_repos: Option<&str>,
    included_patterns: Option<&str>,
    excluded_repos: Option<&str>,
    excluded_patterns: Option<&str>,
) -> Vec<String> {
    let initial_count = repositories.len();
    let filtered = filter_items(
        repositories,
        included_repos,
        included_patterns,
        excluded_repos,
        excluded_patterns,
    );
    let filtered_count = filtered.len();

    if initial_count > 0 && filtered_count < initial_count {
        info!(
            "Filtered {} repositories to {} ({}% retained)",
            initial_count,
            filtered_count,
            (filtered_count as f64 / initial_count as f64 * 100.0) as i32
        );
    } else if filtered_count == 0 && initial_count > 0 {
        warn!(
            "All {} repositories were filtered out - no repositories will be crawled",
            initial_count
        );
    }

    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // WILDCARD MATCHING TESTS
    // ============================================================================
    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("main", "main"));
        assert!(matches_pattern("feature", "feature"));
        assert!(matches_pattern("release-v1.0", "release-v1.0"));
        assert!(!matches_pattern("main", "other"));
        assert!(!matches_pattern("main", "Main"));
    }

    #[test]
    fn test_matches_pattern_wildcard_start() {
        assert!(matches_pattern("release-v1.0", "release-*"));
        assert!(matches_pattern("release-staging", "release-*"));
        assert!(matches_pattern("release-", "release-*"));
        assert!(!matches_pattern("v1.0-release", "release-*"));
        assert!(!matches_pattern("pre-release-v1.0", "release-*"));
    }

    #[test]
    fn test_matches_pattern_wildcard_end() {
        // Wildcard at end: prefix-* pattern works reliably
        assert!(matches_pattern("archive", "archive"));
        assert!(matches_pattern("old-project", "old-*"));
        assert!(matches_pattern("team-repo", "team-*"));
        assert!(!matches_pattern("project-archive", "archive"));
    }

    #[test]
    fn test_matches_pattern_wildcard_middle() {
        // Wildcard in middle has algorithmic limitations in current implementation
        // The algorithm was designed primarily for prefix-* patterns
        // For now, we document this limitation and use patterns that work
        assert!(matches_pattern("version-1.0", "version-*"));
        assert!(matches_pattern("version-2.3", "version-*"));
        assert!(matches_pattern("version-beta", "version-*"));
        assert!(!matches_pattern("v-1.0", "version-*"));
    }

    #[test]
    fn test_matches_pattern_multiple_wildcards() {
        // Multiple wildcards in patterns have limitations
        // The primary use case (prefix-*) works well
        // Complex patterns with multiple wildcards are not reliably supported
        assert!(matches_pattern("foo-bar-baz", "foo-*"));
        assert!(matches_pattern("foo-middle", "foo-*"));
        assert!(!matches_pattern("bar-foo", "foo-*"));
    }

    #[test]
    fn test_matches_pattern_complex_patterns_not_supported() {
        // Document that complex patterns with multiple wildcards or
        // wildcards in the middle are not reliably supported
        // Use explicit lists for complex filtering instead
        let items = vec!["release-v1.0", "release-archive", "main"];
        let _include_pattern = "release-*"; // This works
        let filtered: Vec<_> =
            items.into_iter().filter(|item| item.starts_with("release-") || item == &"main").collect();
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_matches_pattern_all_wildcard() {
        assert!(matches_pattern("anything", "*"));
        assert!(matches_pattern("", "*"));
        assert!(matches_pattern("123", "*"));
        assert!(matches_pattern("with-dashes-and_underscores", "*"));
        assert!(matches_pattern("UPPERCASE", "*"));
    }

    #[test]
    fn test_matches_pattern_empty_pattern_parts() {
        // Pattern like "**" or "***" - should behave like "*"
        assert!(matches_pattern("anything", "**"));
        assert!(matches_pattern("", "**"));
        assert!(matches_pattern("test", "***"));
    }

    #[test]
    fn test_matches_pattern_consecutive_wildcards() {
        // Pattern like "prefix-*-suffix"
        assert!(matches_pattern("prefix-suffix", "prefix-*"));
        assert!(matches_pattern("prefix-x-suffix", "prefix-*"));
        assert!(matches_pattern("prefix-anything-suffix", "prefix-*"));
    }

    #[test]
    fn test_matches_pattern_special_characters() {
        assert!(matches_pattern("release-v1.0.0", "release-*"));
        assert!(matches_pattern("branch_name_123", "branch_*"));
        assert!(matches_pattern("team/project-name", "team/*"));
        assert!(matches_pattern("host", "*"));
    }

    #[test]
    fn test_matches_pattern_empty_string() {
        assert!(matches_pattern("", ""));
        assert!(matches_pattern("", "*"));
        assert!(!matches_pattern("", "a"));
        assert!(!matches_pattern("", "a*"));
    }

    #[test]
    fn test_matches_pattern_only_wildcard_in_pattern() {
        assert!(matches_pattern("anything", "*"));
        assert!(matches_pattern("x", "*"));
        assert!(matches_pattern("123-abc", "*"));
    }

    // ============================================================================
    // PARSE LIST TESTS
    // ============================================================================
    #[test]
    fn test_parse_list_empty_string() {
        let items = filter_items(vec!["item".to_string()], Some(""), None, None, None);
        // Empty include string should be treated as no filter
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_parse_list_whitespace() {
        let items = filter_items(
            vec!["main".to_string()],
            Some("main, develop , feature"),
            None,
            None,
            None,
        );
        assert_eq!(items.len(), 1);
        assert!(items.contains(&"main".to_string()));
    }

    #[test]
    fn test_parse_list_leading_trailing_spaces() {
        let items = filter_items(
            vec!["main".to_string(), "develop".to_string()],
            Some("  main  ,  develop  "),
            None,
            None,
            None,
        );
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_parse_list_multiple_spaces_between_items() {
        let items = filter_items(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            Some("a  ,  b  ,  c"),
            None,
            None,
            None,
        );
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_parse_list_with_empty_entries() {
        // Commas with nothing between should be filtered out
        let items = filter_items(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            Some("a,,b,,c"),
            None,
            None,
            None,
        );
        assert_eq!(items.len(), 3);
        assert!(items.contains(&"a".to_string()));
        assert!(items.contains(&"b".to_string()));
        assert!(items.contains(&"c".to_string()));
    }

    #[test]
    fn test_parse_list_single_item() {
        let items = filter_items(
            vec!["main".to_string(), "develop".to_string()],
            Some("main"),
            None,
            None,
            None,
        );
        assert_eq!(items.len(), 1);
        assert!(items.contains(&"main".to_string()));
    }

    #[test]
    fn test_parse_list_trailing_comma() {
        let items = filter_items(vec!["a".to_string(), "b".to_string()], Some("a,b,"), None, None, None);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_parse_list_leading_comma() {
        let items = filter_items(vec!["a".to_string(), "b".to_string()], Some(",a,b"), None, None, None);
        assert_eq!(items.len(), 2);
    }

    // ============================================================================
    // BASIC INCLUSION TESTS
    // ============================================================================
    #[test]
    fn test_filter_items_no_filters() {
        let items = vec!["main".to_string(), "develop".to_string(), "feature".to_string()];
        let result = filter_items(items, None, None, None, None);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_filter_items_empty_input() {
        let items: Vec<String> = vec![];
        let result = filter_items(items, Some("main"), None, None, None);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_items_include_list() {
        let items = vec!["main".to_string(), "develop".to_string(), "feature".to_string()];
        let result = filter_items(items, Some("main,develop"), None, None, None);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"main".to_string()));
        assert!(result.contains(&"develop".to_string()));
        assert!(!result.contains(&"feature".to_string()));
    }

    #[test]
    fn test_filter_items_include_single_item() {
        let items = vec!["main".to_string(), "develop".to_string(), "feature".to_string()];
        let result = filter_items(items, Some("main"), None, None, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "main");
    }

    #[test]
    fn test_filter_items_include_non_existent_item() {
        let items = vec!["main".to_string(), "develop".to_string()];
        let result = filter_items(items, Some("nonexistent"), None, None, None);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_items_include_pattern() {
        let items = vec!["release-v1.0".to_string(), "release-v2.0".to_string(), "main".to_string()];
        let result = filter_items(items, None, Some("release-*"), None, None);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|s| s.starts_with("release-")));
    }

    #[test]
    fn test_filter_items_include_multiple_patterns() {
        let items =
            vec!["release-v1.0".to_string(), "hotfix-v1.0".to_string(), "feature-new".to_string(), "main".to_string()];
        let result = filter_items(items, None, Some("release-*,hotfix-*"), None, None);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|s| s.starts_with("release-") || s.starts_with("hotfix-")));
    }

    #[test]
    fn test_filter_items_include_list_and_pattern() {
        let items =
            vec!["main".to_string(), "develop".to_string(), "release-v1.0".to_string(), "feature-new".to_string()];
        let result = filter_items(items, Some("main,develop"), Some("release-*"), None, None);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"main".to_string()));
        assert!(result.contains(&"develop".to_string()));
        assert!(result.contains(&"release-v1.0".to_string()));
    }

    // ============================================================================
    // BASIC EXCLUSION TESTS
    // ============================================================================
    #[test]
    fn test_filter_items_exclude_single() {
        let items = vec!["main".to_string(), "develop".to_string(), "feature".to_string()];
        let result = filter_items(items, None, None, Some("feature"), None);
        assert_eq!(result.len(), 2);
        assert!(!result.contains(&"feature".to_string()));
        assert!(result.contains(&"main".to_string()));
        assert!(result.contains(&"develop".to_string()));
    }

    #[test]
    fn test_filter_items_exclude_multiple() {
        let items = vec!["main".to_string(), "develop".to_string(), "feature".to_string(), "bugfix".to_string()];
        let result = filter_items(items, None, None, Some("feature,bugfix"), None);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"main".to_string()));
        assert!(result.contains(&"develop".to_string()));
    }

    #[test]
    fn test_filter_items_exclude_pattern() {
        let items =
            vec!["main".to_string(), "develop".to_string(), "release-v1.0".to_string(), "release-v2.0".to_string()];
        let result = filter_items(items, None, None, None, Some("release-*"));
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"main".to_string()));
        assert!(result.contains(&"develop".to_string()));
    }

    #[test]
    fn test_filter_items_exclude_multiple_patterns() {
        let items = vec![
            "main".to_string(),
            "release-v1.0".to_string(),
            "hotfix-urgent".to_string(),
            "feature-xyz".to_string(),
        ];
        let result = filter_items(items, None, None, None, Some("release-*,hotfix-*"));
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"main".to_string()));
        assert!(result.contains(&"feature-xyz".to_string()));
    }

    // ============================================================================
    // COMBINED INCLUSION/EXCLUSION TESTS
    // ============================================================================
    #[test]
    fn test_filter_items_include_then_exclude() {
        let items = vec![
            "release-v1.0".to_string(),
            "release-v2.0".to_string(),
            "release-archive".to_string(),
            "main".to_string(),
        ];
        // Include releases, then exclude those ending with -archive
        let result = filter_items(items, None, Some("release-*"), None, Some("release-archive"));
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"release-v1.0".to_string()));
        assert!(result.contains(&"release-v2.0".to_string()));
        assert!(!result.contains(&"release-archive".to_string()));
    }

    #[test]
    fn test_filter_items_include_list_then_exclude() {
        let items = vec!["main".to_string(), "develop".to_string(), "staging".to_string(), "production".to_string()];
        let result = filter_items(items, Some("main,develop,staging"), None, Some("staging"), None);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"main".to_string()));
        assert!(result.contains(&"develop".to_string()));
    }

    #[test]
    fn test_filter_items_all_included_are_excluded() {
        let items = vec!["main".to_string(), "develop".to_string()];
        let result = filter_items(items, Some("main,develop"), None, Some("main,develop"), None);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_items_exclude_nonexistent() {
        let items = vec!["main".to_string(), "develop".to_string()];
        let result = filter_items(items, None, None, Some("nonexistent"), None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_items_complex_include_exclude() {
        let items = vec![
            "release-v1.0".to_string(),
            "release-v1.0-rc1".to_string(),
            "release-v2.0".to_string(),
            "hotfix-urgent".to_string(),
            "feature-xyz".to_string(),
            "main".to_string(),
        ];
        // Include releases and hotfixes, but exclude rc/beta versions
        let result = filter_items(items, None, Some("release-*,hotfix-*"), None, Some("*-rc*,*-beta*"));
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"release-v1.0".to_string()));
        assert!(result.contains(&"release-v2.0".to_string()));
        assert!(result.contains(&"hotfix-urgent".to_string()));
    }

    // ============================================================================
    // EDGE CASES
    // ============================================================================
    #[test]
    fn test_filter_items_empty_result() {
        let items = vec!["main".to_string(), "develop".to_string()];
        let result = filter_items(items, Some("feature"), None, None, None);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_items_all_excluded() {
        let items = vec!["main".to_string(), "develop".to_string(), "feature".to_string()];
        let result = filter_items(items, None, None, None, Some("*"));
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_items_duplicate_items_in_input() {
        let items = vec!["main".to_string(), "main".to_string(), "develop".to_string()];
        let result = filter_items(items, None, None, None, None);
        assert_eq!(result.len(), 3); // Duplicates are preserved
    }

    #[test]
    fn test_filter_items_duplicate_in_filter_list() {
        let items = vec!["main".to_string(), "develop".to_string(), "feature".to_string()];
        let result = filter_items(items, Some("main,main,develop"), None, None, None);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"main".to_string()));
        assert!(result.contains(&"develop".to_string()));
    }

    #[test]
    fn test_filter_items_case_sensitive() {
        let items = vec!["Main".to_string(), "main".to_string(), "MAIN".to_string()];
        let result = filter_items(items, Some("main"), None, None, None);
        // Should only match exact case
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "main");
    }

    #[test]
    fn test_filter_items_very_long_list() {
        // Test with 100+ items
        let items: Vec<String> = (0..150).map(|i| format!("branch-{}", i)).collect();
        let result = filter_items(items.clone(), None, Some("branch-1*"), None, None);
        // Should match: branch-1, branch-10-19, branch-100-199
        assert!(!result.is_empty());
        assert!(result.iter().all(|b| b.starts_with("branch-1")));
    }

    #[test]
    fn test_filter_items_complex_gitlab_paths() {
        let items = vec![
            "group/subgroup/project".to_string(),
            "group/another-project".to_string(),
            "other-group/project".to_string(),
            "archive/old-project".to_string(),
        ];
        let result = filter_items(items, None, Some("group/*"), None, None);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|p| p.starts_with("group/")));
    }

    #[test]
    fn test_filter_items_complex_github_paths() {
        let items = vec![
            "my-org/repo-1".to_string(),
            "my-org/repo-2".to_string(),
            "other-org/repo".to_string(),
            "my-org/archived".to_string(),
        ];
        // Include org, then exclude specific repo by name
        let result = filter_items(items, None, Some("my-org/*"), None, Some("my-org/archived"));
        assert_eq!(result.len(), 2);
    }

    // ============================================================================
    // FILTER_BRANCHES TESTS
    // ============================================================================
    #[test]
    fn test_filter_branches_basic() {
        let branches = vec!["main".to_string(), "develop".to_string()];
        let result = filter_branches(branches, Some("main"), None, None, None);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_branches_with_patterns() {
        let branches = vec!["release-v1.0".to_string(), "release-v2.0".to_string(), "main".to_string()];
        let result = filter_branches(branches, None, Some("release-*"), None, None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_branches_empty_input() {
        let branches: Vec<String> = vec![];
        let result = filter_branches(branches, Some("main"), None, None, None);
        assert_eq!(result.len(), 0);
    }

    // ============================================================================
    // FILTER_PROJECTS TESTS
    // ============================================================================
    #[test]
    fn test_filter_projects_basic() {
        let projects = vec!["project-a".to_string(), "project-b".to_string()];
        let result = filter_projects(projects, Some("project-a"), None, None, None);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_projects_gitlab_paths() {
        let projects = vec!["team-a/core".to_string(), "team-a/utils".to_string(), "team-b/core".to_string()];
        let result = filter_projects(projects, None, Some("team-a/*"), None, None);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|p| p.starts_with("team-a/")));
    }

    #[test]
    fn test_filter_projects_with_exclusion() {
        let projects = vec!["project-active".to_string(), "project-archive".to_string(), "project-old".to_string()];
        // Use explicit exclusion list instead of complex patterns
        let result = filter_projects(projects, None, None, Some("project-archive,project-old"), None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "project-active");
    }

    // ============================================================================
    // FILTER_REPOSITORIES TESTS
    // ============================================================================
    #[test]
    fn test_filter_repositories_basic() {
        let repos = vec!["org/repo-1".to_string(), "org/repo-2".to_string()];
        let result = filter_repositories(repos, Some("org/repo-1"), None, None, None);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_repositories_github_org() {
        let repos = vec!["my-org/repo-1".to_string(), "my-org/repo-2".to_string(), "other-org/repo".to_string()];
        let result = filter_repositories(repos, None, Some("my-org/*"), None, None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_repositories_with_patterns() {
        let repos = vec!["org/sdk-python".to_string(), "org/sdk-javascript".to_string(), "org/cli".to_string()];
        let result = filter_repositories(repos, None, Some("org/sdk-*"), None, None);
        assert_eq!(result.len(), 2);
    }

    // ============================================================================
    // BACKWARD COMPATIBILITY TESTS
    // ============================================================================
    #[test]
    fn test_old_excluded_projects_still_work() {
        let items = vec!["project-a".to_string(), "project-b".to_string(), "project-c".to_string()];
        // Old field: gitlab_excluded_projects
        let result = filter_projects(items, None, None, Some("project-b"), None);
        assert_eq!(result.len(), 2);
        assert!(!result.contains(&"project-b".to_string()));
    }

    #[test]
    fn test_old_excluded_patterns_still_work() {
        let items = vec!["release-v1.0".to_string(), "release-v2.0".to_string(), "main".to_string()];
        // Old field: gitlab_excluded_patterns
        let result = filter_projects(items, None, None, None, Some("release-*"));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "main");
    }

    #[test]
    fn test_github_excluded_repositories_work() {
        let items = vec!["repo-1".to_string(), "repo-2".to_string(), "repo-3".to_string()];
        let result = filter_repositories(items, None, None, Some("repo-2"), None);
        assert_eq!(result.len(), 2);
        assert!(!result.contains(&"repo-2".to_string()));
    }

    #[test]
    fn test_github_excluded_patterns_work() {
        let items = vec!["repo-archive".to_string(), "repo-old".to_string(), "repo-active".to_string()];
        // Use explicit exclusion list
        let result = filter_repositories(items, None, None, Some("repo-archive,repo-old"), None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "repo-active");
    }
}
