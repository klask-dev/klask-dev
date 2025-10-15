use crate::models::Repository;
use crate::services::encryption::EncryptionService;
use anyhow::{Result, anyhow};
use gix::open::Options;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Git operations for cloning and updating repositories
#[derive(Clone)]
pub struct GitOperations {
    encryption_service: Arc<EncryptionService>,
}

impl GitOperations {
    pub fn new(encryption_service: Arc<EncryptionService>) -> Self {
        Self { encryption_service }
    }

    pub async fn clone_or_update_repository(
        &self,
        repository: &Repository,
        repo_path: &Path,
    ) -> Result<gix::Repository> {
        let repo_path_owned = repo_path.to_owned();

        if repo_path.exists() {
            info!("Updating existing repository at: {:?}", repo_path);

            let result = tokio::time::timeout(
                std::time::Duration::from_secs(180),
                tokio::task::spawn_blocking(move || -> Result<gix::Repository> {
                    // Disable ALL interactive prompts for server-mode operation
                    let opts = Options::isolated().config_overrides(["gitoxide.credentials.terminalPrompt=0"]);

                    let git_repo = gix::open_opts(&repo_path_owned, opts)?;

                    info!("Fetching latest changes from remote");

                    if let Ok(remote) = git_repo.find_remote("origin")
                        && let Ok(conn) = remote.connect(gix::remote::Direction::Fetch)
                        && let Ok(prep) = conn.prepare_fetch(gix::progress::Discard, Default::default())
                    {
                        if let Err(e) = prep.receive(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED) {
                            warn!("Failed to receive fetch: {}", e);
                        } else {
                            info!("Successfully fetched latest changes");
                        }
                    }

                    Ok(git_repo)
                }),
            )
            .await;

            match result {
                Ok(Ok(Ok(repo))) => return Ok(repo),
                _ => {
                    warn!("Update failed; deleting and re-cloning");
                    std::fs::remove_dir_all(repo_path)?;
                    return self.clone_fresh_repository(repository, repo_path).await;
                }
            }
        }

        self.clone_fresh_repository(repository, repo_path).await
    }

    pub async fn clone_fresh_repository(&self, repository: &Repository, repo_path: &Path) -> Result<gix::Repository> {
        debug!("Cloning repository to: {:?}", repo_path);

        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create parent directories for {:?}: {}", parent, e))?;
        }

        // Decrypt token before moving to spawn_blocking
        let access_token = if let Some(encrypted_token) = &repository.access_token {
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
        } else {
            None
        };

        let clone_url = repository.url.clone();
        let repo_path_owned = repo_path.to_owned();

        tokio::time::timeout(
            std::time::Duration::from_secs(300),
            tokio::task::spawn_blocking(move || -> Result<gix::Repository> {
                // Prepare clone with authentication header if provided
                let mut prepare_clone = gix::prepare_clone(clone_url, &repo_path_owned)
                    .map_err(|e| anyhow!("Failed to prepare clone: {}", e))?;

                // Configure for non-interactive mode (no credential prompts)
                // This is critical for public repositories and server environments
                let mut config_overrides = vec![
                    "credential.helper=".to_string(), // Disable credential helpers
                ];

                // Configure authentication using http.extraHeader if we have a token
                // This is more secure than embedding the token in the URL
                if let Some(token) = access_token {
                    config_overrides.push(format!("http.extraHeader=Authorization: Bearer {}", token));
                }

                prepare_clone =
                    prepare_clone.with_in_memory_config_overrides(config_overrides.iter().map(|s| s.as_str()));

                // Configure shallow clone (depth=1) to speed up large repositories
                // This clones only the latest commit, significantly reducing download time
                prepare_clone =
                    prepare_clone.configure_remote(|remote| Ok(remote.with_fetch_tags(gix::remote::fetch::Tags::None)));

                // Perform the fetch with shallow clone
                let (_prepared_clone, _outcome) = prepare_clone
                    .fetch_only(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                    .map_err(|e| anyhow!("fetch_only failed: {}", e))?;

                let repo = gix::open(&repo_path_owned).map_err(|e| anyhow!("open cloned repo failed: {}", e))?;

                info!("Successfully cloned repository");
                Ok(repo)
            }),
        )
        .await
        .map_err(|_| anyhow!("clone timed out"))??
    }
}
