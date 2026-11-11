use crate::models::Repository;
use crate::services::encryption::EncryptionService;
use anyhow::{Result, anyhow};
use gix::bstr::ByteSlice;
use gix::open::Options;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// Default time allowed for a fetch into an existing clone (overridable with `KLASK_GIT_FETCH_TIMEOUT_SECS`).
const DEFAULT_FETCH_TIMEOUT_SECS: u64 = 1800;
/// Default time allowed for a fresh clone (overridable with `KLASK_GIT_CLONE_TIMEOUT_SECS`).
const DEFAULT_CLONE_TIMEOUT_SECS: u64 = 3600;
const FETCH_TIMEOUT_ENV: &str = "KLASK_GIT_FETCH_TIMEOUT_SECS";
const CLONE_TIMEOUT_ENV: &str = "KLASK_GIT_CLONE_TIMEOUT_SECS";

/// Parses a timeout in seconds. Missing, non-numeric or zero values fall back to `default_secs`.
fn parse_timeout_secs(raw: Option<&str>, default_secs: u64) -> Duration {
    let secs = raw.and_then(|value| value.trim().parse::<u64>().ok()).filter(|&secs| secs > 0).unwrap_or(default_secs);
    Duration::from_secs(secs)
}

fn timeout_from_env(var: &str, default_secs: u64) -> Duration {
    parse_timeout_secs(std::env::var(var).ok().as_deref(), default_secs)
}

fn accept_invalid_certs_from_env() -> bool {
    std::env::var("KLASK_GITLAB_ACCEPT_INVALID_CERTS").map(|v| v.to_lowercase() == "true").unwrap_or(false)
}

/// Config overrides shared by clone and fetch: never prompt for credentials, and optionally skip TLS checks.
fn config_overrides(accept_invalid_certs: bool) -> Vec<&'static str> {
    // Fetching writes reflogs, which requires a committer. The server usually has no git identity
    // configured, and the isolated open ignores user config: without a fallback every fetch fails with
    // "MissingCommitter". The fallback only applies when no identity is configured.
    let mut overrides = vec![
        "gitoxide.credentials.terminalPrompt=0",
        "gitoxide.committer.nameFallback=klask",
        "gitoxide.committer.emailFallback=klask@localhost",
    ];
    if accept_invalid_certs {
        overrides.push("gitoxide.http.sslNoVerify=true");
    }
    overrides
}

/// Credentials callback shared by clone and fetch. Without a token every request is refused
/// (no prompt); with a token, it is answered on `get` only.
#[allow(clippy::result_large_err)] // external lib should fix this issue
fn credentials_callback(
    token: Option<String>,
) -> impl FnMut(gix::credentials::helper::Action) -> gix::credentials::protocol::Result {
    move |action| match &token {
        None => Err(gix::credentials::protocol::Error::Quit),
        Some(token) => match action {
            gix::credentials::helper::Action::Get(ctx) => Ok(Some(gix::credentials::protocol::Outcome {
                identity: gix::sec::identity::Account {
                    username: "oauth2".to_string(),
                    password: token.clone(),
                    oauth_refresh_token: None,
                },
                next: ctx.into(),
            })),
            // Ignore store/erase operations
            _ => Ok(None),
        },
    }
}

/// Opens an existing clone with the same options a clone uses.
fn open_existing(repo_path: &Path, accept_invalid_certs: bool) -> Result<gix::Repository> {
    let opts = Options::isolated().config_overrides(config_overrides(accept_invalid_certs));
    gix::open_opts(repo_path, opts).map_err(|e| anyhow!("failed to open existing clone: {}", e))
}

/// Fetches `origin` with the repository's credentials, then drops local tracking refs
/// (`refs/remotes/origin/*`) for branches the remote no longer advertises.
fn fetch_origin(git_repo: &gix::Repository, access_token: Option<String>) -> Result<()> {
    let remote = git_repo.find_remote("origin").map_err(|e| anyhow!("remote 'origin' not found: {}", e))?;
    let mut connection =
        remote.connect(gix::remote::Direction::Fetch).map_err(|e| anyhow!("failed to connect to remote: {}", e))?;
    connection.set_credentials(credentials_callback(access_token));

    let prep = connection
        .prepare_fetch(gix::progress::Discard, Default::default())
        .map_err(|e| anyhow!("failed to negotiate fetch: {}", e))?;

    let advertised_branches: HashSet<String> = prep
        .ref_map()
        .remote_refs
        .iter()
        .filter_map(|remote_ref| remote_ref.unpack().0.to_str().ok())
        .filter_map(|name| name.strip_prefix("refs/heads/"))
        .map(str::to_owned)
        .collect();

    prep.receive(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
        .map_err(|e| anyhow!("fetch failed: {}", e))?;

    prune_stale_tracking_refs(git_repo, &advertised_branches)
}

/// Deletes `refs/remotes/origin/<branch>` for every branch missing from `advertised_branches`.
fn prune_stale_tracking_refs(git_repo: &gix::Repository, advertised_branches: &HashSet<String>) -> Result<()> {
    if advertised_branches.is_empty() {
        // An empty advertisement is more likely a protocol problem than a repository without branches:
        // keep the existing tracking refs rather than dropping every branch.
        warn!("Remote advertised no branches; skipping prune of stale tracking refs");
        return Ok(());
    }

    let mut stale_refs = Vec::new();
    for reference in git_repo.references()?.all()? {
        let reference = reference.map_err(|e| anyhow!("failed to iterate references: {}", e))?;
        let name = reference.name().as_bstr().to_string();
        if let Some(branch) = name.strip_prefix("refs/remotes/origin/")
            && branch != "HEAD"
            && !advertised_branches.contains(branch)
        {
            stale_refs.push(name);
        }
    }

    for name in stale_refs {
        info!("Removing tracking ref {} (branch deleted on remote)", name);
        git_repo
            .find_reference(&name)
            .map_err(|e| anyhow!("failed to find stale ref {}: {}", name, e))?
            .delete()
            .map_err(|e| anyhow!("failed to delete stale ref {}: {}", name, e))?;
    }
    Ok(())
}

#[derive(Clone)]
pub struct GitOperations {
    encryption_service: Arc<EncryptionService>,
}

impl GitOperations {
    pub fn new(encryption_service: Arc<EncryptionService>) -> Self {
        Self { encryption_service }
    }

    /// Decrypts the repository's access token, if any. A decryption failure is logged and
    /// the operation proceeds unauthenticated (the remote then refuses it if it is private).
    fn decrypt_access_token(&self, repository: &Repository) -> Option<String> {
        let encrypted_token = repository.access_token.as_ref()?;
        match self.encryption_service.decrypt(encrypted_token) {
            Ok(token) => Some(token),
            Err(e) => {
                warn!(
                    "Failed to decrypt access token: {}. Proceeding without authentication.",
                    e
                );
                None
            }
        }
    }

    /// Clones the repository when `repo_path` does not exist yet, otherwise fetches into the existing clone.
    ///
    /// A fetch failure or timeout is returned as an error (the crawl then fails and records it);
    /// the existing clone is kept so that the next attempt can retry. Only a clone that cannot be
    /// opened at all is removed and cloned again.
    pub async fn clone_or_update_repository(
        &self,
        repository: &Repository,
        repo_path: &Path,
    ) -> Result<gix::Repository> {
        if repo_path.exists() {
            info!("Updating existing repository at: {:?}", repo_path);
            let accept_invalid_certs = accept_invalid_certs_from_env();
            let path_for_open = repo_path.to_owned();
            let opened = tokio::task::spawn_blocking(move || open_existing(&path_for_open, accept_invalid_certs))
                .await
                .map_err(|e| anyhow!("open task failed: {}", e))?;

            match opened {
                Ok(git_repo) => return self.fetch_existing_repository(repository, git_repo).await,
                Err(e) => {
                    warn!(
                        "Existing clone at {:?} cannot be opened ({}); deleting and re-cloning",
                        repo_path, e
                    );
                    std::fs::remove_dir_all(repo_path)?;
                }
            }
        }

        self.clone_fresh_repository(repository, repo_path).await
    }

    async fn fetch_existing_repository(
        &self,
        repository: &Repository,
        git_repo: gix::Repository,
    ) -> Result<gix::Repository> {
        let access_token = self.decrypt_access_token(repository);
        let timeout = timeout_from_env(FETCH_TIMEOUT_ENV, DEFAULT_FETCH_TIMEOUT_SECS);

        // The blocking task cannot be cancelled, so on timeout it is left to finish on its own and
        // the clone directory is NOT removed (removing it under a running fetch would corrupt it).
        let handle = tokio::task::spawn_blocking(move || -> Result<gix::Repository> {
            fetch_origin(&git_repo, access_token)?;
            info!("Successfully fetched latest changes");
            Ok(git_repo)
        });

        match tokio::time::timeout(timeout, handle).await {
            Ok(Ok(result)) => result.map_err(|e| anyhow!("Failed to fetch repository updates: {:#}", e)),
            Ok(Err(join_error)) => Err(anyhow!(
                "Failed to fetch repository updates: task failed: {}",
                join_error
            )),
            Err(_) => Err(anyhow!(
                "Failed to fetch repository updates: timed out after {}s (set {} to raise it)",
                timeout.as_secs(),
                FETCH_TIMEOUT_ENV
            )),
        }
    }

    pub async fn clone_fresh_repository(&self, repository: &Repository, repo_path: &Path) -> Result<gix::Repository> {
        tracing::debug!("Cloning repository to: {:?}", repo_path);

        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create parent directories for {:?}: {}", parent, e))?;
        }

        let access_token = self.decrypt_access_token(repository);
        let clone_url = repository.url.clone();
        let repo_path_owned = repo_path.to_owned();
        let timeout = timeout_from_env(CLONE_TIMEOUT_ENV, DEFAULT_CLONE_TIMEOUT_SECS);

        tokio::time::timeout(
            timeout,
            tokio::task::spawn_blocking(move || -> Result<gix::Repository> {
                // Prepare clone
                let mut prep = gix::prepare_clone(clone_url, &repo_path_owned)
                    .map_err(|e| anyhow!("Failed to prepare clone: {}", e))?;

                let overrides = config_overrides(accept_invalid_certs_from_env());
                prep = prep.with_in_memory_config_overrides(overrides.iter().copied());

                // Token if any, otherwise refuse credentials (no prompt)
                prep = prep.configure_connection(move |connection| {
                    connection.set_credentials(credentials_callback(access_token.clone()));
                    Ok(())
                });

                // Only branch heads are fetched (no tags). The full history of each branch is kept:
                // the tree walker needs complete commits, this is not a shallow clone.
                prep = prep.configure_remote(|remote| Ok(remote.with_fetch_tags(gix::remote::fetch::Tags::None)));

                // Perform the fetch
                let (_prepared_clone, _outcome) = prep
                    .fetch_only(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                    .map_err(|e| anyhow!("fetch_only failed: {}", e))?;

                let repo = gix::open(&repo_path_owned).map_err(|e| anyhow!("open cloned repo failed: {}", e))?;

                info!("Successfully cloned repository");
                Ok(repo)
            }),
        )
        .await
        .map_err(|_| {
            anyhow!(
                "clone timed out after {}s (set {} to raise it)",
                timeout.as_secs(),
                CLONE_TIMEOUT_ENV
            )
        })?
        .map_err(|e| anyhow!("clone task failed: {}", e))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_timeout_uses_default_for_missing_or_invalid_values() {
        assert_eq!(parse_timeout_secs(None, 1800), Duration::from_secs(1800));
        assert_eq!(parse_timeout_secs(Some("abc"), 1800), Duration::from_secs(1800));
        assert_eq!(parse_timeout_secs(Some("0"), 1800), Duration::from_secs(1800));
        assert_eq!(parse_timeout_secs(Some(" 90 "), 1800), Duration::from_secs(90));
    }

    #[test]
    fn config_overrides_skip_tls_check_only_when_requested() {
        assert!(!config_overrides(false).contains(&"gitoxide.http.sslNoVerify=true"));
        assert!(config_overrides(true).contains(&"gitoxide.http.sslNoVerify=true"));
    }

    /// Runs the git CLI in `dir` with a fixed identity, panicking on failure (test helper only).
    fn git(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .args(["-c", "user.name=test", "-c", "user.email=test@example.invalid"])
            .args(args)
            .current_dir(dir)
            .status()
            .expect("git must be installed to run these tests");
        assert!(status.success(), "git {:?} failed", args);
    }

    /// Creates a source repository with branches `main` and `feature`, returns its path.
    fn make_source_repo(root: &Path) -> std::path::PathBuf {
        let source = root.join("source");
        std::fs::create_dir_all(&source).unwrap();
        git(&source, &["init", "-q", "-b", "main"]);
        std::fs::write(source.join("a.txt"), "hello\n").unwrap();
        git(&source, &["add", "."]);
        git(&source, &["commit", "-q", "-m", "init"]);
        git(&source, &["branch", "feature"]);
        source
    }

    /// Clones `source` into `clone_dir` the same way `clone_fresh_repository` does, then opens it.
    fn clone_local(source: &Path, clone_dir: &Path) -> gix::Repository {
        let mut prep = gix::prepare_clone(source.to_str().unwrap(), clone_dir).expect("prepare clone");
        let (_prepared, _outcome) =
            prep.fetch_only(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED).expect("initial clone");
        open_existing(clone_dir, false).expect("open clone")
    }

    fn has_tracking_ref(repo: &gix::Repository, branch: &str) -> bool {
        repo.find_reference(&format!("refs/remotes/origin/{branch}")).is_ok()
    }

    #[test]
    fn fetch_picks_up_new_branches_and_prunes_deleted_ones() {
        let root = tempfile::tempdir().unwrap();
        let source = make_source_repo(root.path());
        let repo = clone_local(&source, &root.path().join("clone"));
        assert!(has_tracking_ref(&repo, "feature"));

        git(&source, &["branch", "new-branch"]);
        git(&source, &["branch", "-D", "feature"]);

        fetch_origin(&repo, None).expect("fetch must succeed");

        assert!(
            has_tracking_ref(&repo, "new-branch"),
            "new remote branch must be fetched"
        );
        assert!(
            !has_tracking_ref(&repo, "feature"),
            "branch deleted on the remote must be pruned"
        );
        assert!(has_tracking_ref(&repo, "main"));
    }

    #[test]
    fn fetch_failure_is_returned_as_error() {
        let root = tempfile::tempdir().unwrap();
        let source = make_source_repo(root.path());
        let repo = clone_local(&source, &root.path().join("clone"));

        std::fs::rename(&source, root.path().join("moved")).unwrap();

        assert!(
            fetch_origin(&repo, None).is_err(),
            "an unreachable remote must not look like a successful fetch"
        );
        assert!(
            has_tracking_ref(&repo, "main"),
            "the existing tracking refs must be left untouched"
        );
    }
}
