/// Version information for Klask
/// The version is determined at compile time from environment variables
/// or falls back to the Cargo.toml version

/// Get the current Klask version
/// Tries to read from KLASK_VERSION env var first, then falls back to CARGO_PKG_VERSION
pub fn get_version() -> &'static str {
    option_env!("KLASK_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))
}

/// Get the current Klask git commit hash
pub fn get_commit_hash() -> Option<&'static str> {
    option_env!("GIT_COMMIT_HASH")
}

/// Get the current Klask build timestamp
pub fn get_build_timestamp() -> Option<&'static str> {
    option_env!("BUILD_TIMESTAMP")
}

/// Get full version information
pub fn get_full_version_string() -> String {
    let mut version = format!("v{}", get_version());

    if let Some(commit) = get_commit_hash() {
        version.push_str(&format!(" ({})", &commit[..8.min(commit.len())]));
    }

    version
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_get_full_version_string() {
        let full = get_full_version_string();
        assert!(full.starts_with('v'));
    }
}
