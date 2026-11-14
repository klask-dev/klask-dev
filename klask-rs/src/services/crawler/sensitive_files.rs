//! Exclusion of sensitive files (credentials, private keys, local env files) from the index.
//!
//! Matching is done on the file name, or on the trailing path components for entries that contain a
//! `/` (e.g. `.kube/config`). File contents are never inspected. The exclusion is active by default.
//!
//! Configuration, read once per process on first use:
//! - `KLASK_INDEX_SENSITIVE_FILES=true` disables the built-in list (custom patterns still apply);
//! - `KLASK_EXCLUDED_FILE_PATTERNS=pattern1,pattern2` adds glob patterns (`*` wildcard only).

use super::filter::matches_pattern;
use std::sync::OnceLock;
use tracing::info;

/// Built-in patterns. Deliberately narrow: `*secret*` or `credentials.*` would also drop source code.
const DEFAULT_PATTERNS: &[&str] = &[
    ".env",
    ".env.*",
    ".npmrc",
    ".pypirc",
    ".netrc",
    ".pgpass",
    ".git-credentials",
    ".htpasswd",
    "id_rsa*",
    "id_dsa*",
    "id_ecdsa*",
    "id_ed25519*",
    "*.pem",
    "*.key",
    "*.p12",
    "*.pfx",
    "*.jks",
    "*.keystore",
    "*.kdbx",
    ".docker/config.json",
    ".aws/credentials",
    ".kube/config",
];

/// Environment templates that document variables without holding values: always indexed.
const TEMPLATE_NAMES: &[&str] = &[".env.example", ".env.sample", ".env.template", ".env.dist", ".env.defaults"];

/// Decides whether a file path is excluded from the index.
#[derive(Debug, Clone)]
pub struct SensitiveFileFilter {
    /// Lowercased patterns; entries containing `/` match the trailing path components.
    patterns: Vec<String>,
}

impl SensitiveFileFilter {
    /// Builds a filter from the built-in list (when `include_defaults`) and extra patterns.
    pub fn new(include_defaults: bool, custom_patterns: &[String]) -> Self {
        let mut all_patterns: Vec<String> = if include_defaults {
            DEFAULT_PATTERNS.iter().map(|pattern| (*pattern).to_string()).collect()
        } else {
            Vec::new()
        };
        all_patterns.extend(custom_patterns.iter().cloned());

        let patterns = all_patterns
            .into_iter()
            .map(|pattern| pattern.trim().trim_start_matches("./").to_lowercase())
            .filter(|pattern| !pattern.is_empty())
            .collect();
        Self { patterns }
    }

    /// Reads the configuration from the environment.
    pub fn from_env() -> Self {
        let disable_defaults = std::env::var("KLASK_INDEX_SENSITIVE_FILES")
            .map(|value| matches!(value.trim().to_lowercase().as_str(), "true" | "1" | "yes"))
            .unwrap_or(false);
        let custom: Vec<String> =
            std::env::var("KLASK_EXCLUDED_FILE_PATTERNS").unwrap_or_default().split(',').map(str::to_owned).collect();
        Self::new(!disable_defaults, &custom)
    }

    /// Number of active patterns (built-in and custom).
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Returns true if `path` (relative to the repository, `/` or `\` separated) must not be indexed.
    pub fn is_excluded(&self, path: &str) -> bool {
        let normalized = path.replace('\\', "/").to_lowercase();
        let name = normalized.rsplit('/').next().unwrap_or(&normalized);

        if TEMPLATE_NAMES.contains(&name) {
            return false;
        }
        // Public halves of key pairs are not secrets.
        if name.starts_with("id_") && name.ends_with(".pub") {
            return false;
        }

        self.patterns.iter().any(|pattern| {
            if pattern.contains('/') {
                normalized == pattern.as_str() || normalized.ends_with(&format!("/{pattern}"))
            } else {
                matches_pattern(name, pattern)
            }
        })
    }
}

static GLOBAL_FILTER: OnceLock<SensitiveFileFilter> = OnceLock::new();

/// Process-wide filter, configured from the environment on first use.
pub fn global() -> &'static SensitiveFileFilter {
    GLOBAL_FILTER.get_or_init(|| {
        let filter = SensitiveFileFilter::from_env();
        info!("Sensitive file exclusion: {} active pattern(s)", filter.pattern_count());
        filter
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> SensitiveFileFilter {
        SensitiveFileFilter::new(true, &[])
    }

    #[test]
    fn default_patterns_exclude_secret_files() {
        let filter = defaults();
        for path in [
            ".env",
            ".env.local",
            ".env.production",
            ".npmrc",
            ".pypirc",
            ".netrc",
            ".pgpass",
            ".git-credentials",
            ".htpasswd",
            "id_rsa",
            "id_dsa",
            "id_ecdsa",
            "id_ed25519",
            "id_ed25519_sk",
            "server.pem",
            "tls/server.key",
            "cert.p12",
            "cert.pfx",
            "store.jks",
            "app.keystore",
            "passwords.kdbx",
            ".docker/config.json",
            ".aws/credentials",
            ".kube/config",
        ] {
            assert!(filter.is_excluded(path), "{path} must be excluded");
        }
    }

    #[test]
    fn templates_and_public_keys_are_kept() {
        let filter = defaults();
        for path in [
            ".env.example",
            "config/.env.sample",
            ".env.template",
            ".env.dist",
            ".env.defaults",
            "id_rsa.pub",
            "keys/id_ed25519.pub",
        ] {
            assert!(!filter.is_excluded(path), "{path} must be indexed");
        }
    }

    #[test]
    fn nested_paths_match_on_file_name() {
        let filter = defaults();
        assert!(filter.is_excluded("services/app/.env.production"));
        assert!(filter.is_excluded("deploy/keys/server.pem"));
        assert!(filter.is_excluded("ops/.docker/config.json"));
        assert!(filter.is_excluded("home/user/.aws/credentials"));
    }

    #[test]
    fn source_code_is_not_excluded() {
        let filter = defaults();
        for path in [
            "src/credentials.rs",
            "src/secrets_manager.rs",
            "src/key.rs",
            "src/keys.go",
            "src/env.rs",
            "src/config.rs",
            "app/config.json",
            "kube/config",
            "docs/env-setup.md",
            "lib/.env_loader.rb",
        ] {
            assert!(!filter.is_excluded(path), "{path} must be indexed");
        }
    }

    #[test]
    fn matching_is_case_insensitive_and_accepts_windows_separators() {
        let filter = defaults();
        assert!(filter.is_excluded("ID_RSA"));
        assert!(filter.is_excluded("services\\app\\.env.local"));
    }

    #[test]
    fn custom_patterns_are_added() {
        let filter = SensitiveFileFilter::new(
            true,
            &["*.tfstate".to_string(), " ./infra/terraform.tfvars ".to_string()],
        );
        assert!(filter.is_excluded("prod/terraform.tfstate"));
        assert!(filter.is_excluded("infra/terraform.tfvars"));
        assert!(!filter.is_excluded("other/terraform.tfvars"));
    }

    #[test]
    fn disabling_defaults_keeps_custom_patterns_only() {
        let filter = SensitiveFileFilter::new(false, &["*.tfstate".to_string()]);
        assert!(!filter.is_excluded(".env.local"));
        assert!(!filter.is_excluded("server.pem"));
        assert!(filter.is_excluded("prod/terraform.tfstate"));
        assert_eq!(filter.pattern_count(), 1);
    }

    #[test]
    fn empty_custom_entries_are_ignored() {
        let filter = SensitiveFileFilter::new(false, &[String::new(), "  ".to_string()]);
        assert_eq!(filter.pattern_count(), 0);
        assert!(!filter.is_excluded(".env"));
    }
}
