use crate::models::Repository;
use crate::services::encryption::EncryptionService;
use anyhow::{anyhow, Result};
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

    /// Clone or update a Git repository using gix
    pub async fn clone_or_update_repository(
        &self,
        repository: &Repository,
        repo_path: &Path,
    ) -> Result<gix::Repository> {
        let repo_path_owned = repo_path.to_owned();

        if repo_path.exists() {
            info!("Updating existing repository at: {:?}", repo_path);

            // Use spawn_blocking to fetch updates
            match tokio::time::timeout(
                std::time::Duration::from_secs(180), // 3 minutes timeout for fetch
                tokio::task::spawn_blocking(move || -> Result<gix::Repository> {
                    let git_repo = gix::open(&repo_path_owned)?;

                    // Fetch latest changes from remote using gix
                    info!("Fetching latest changes from remote");

                    // Fetch from remote using gix
                    // We attempt to fetch but continue with existing data if it fails
                    match git_repo.find_remote("origin") {
                        Ok(remote) => {
                            // Configure for non-interactive mode
                            let connection_result = remote.connect(gix::remote::Direction::Fetch);

                            match connection_result {
                                Ok(connection) => {
                                    match connection.prepare_fetch(gix::progress::Discard, Default::default()) {
                                        Ok(prepare) => {
                                            match prepare
                                                .receive(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                                            {
                                                Ok(_outcome) => {
                                                    info!("Successfully fetched latest changes");
                                                }
                                                Err(e) => {
                                                    warn!("Failed to receive fetch: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Failed to prepare fetch: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to connect to remote: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to find remote 'origin': {}", e);
                        }
                    }

                    Ok(git_repo)
                }),
            )
            .await
            {
                Ok(Ok(Ok(git_repo))) => Ok(git_repo),
                Ok(Ok(Err(e))) => {
                    // If we can't open/fetch, delete and re-clone
                    warn!("Failed to update existing repository, will delete and re-clone: {}", e);
                    std::fs::remove_dir_all(repo_path)?;
                    self.clone_fresh_repository(repository, repo_path).await
                }
                Ok(Err(e)) => {
                    warn!("Spawn blocking error: {}", e);
                    std::fs::remove_dir_all(repo_path)?;
                    self.clone_fresh_repository(repository, repo_path).await
                }
                Err(_) => {
                    warn!("Fetch operation timed out after 3 minutes, deleting and re-cloning");
                    std::fs::remove_dir_all(repo_path)?;
                    self.clone_fresh_repository(repository, repo_path).await
                }
            }
        } else {
            self.clone_fresh_repository(repository, repo_path).await
        }
    }

    /// Clone a fresh repository using gix with authentication
    pub async fn clone_fresh_repository(&self, repository: &Repository, repo_path: &Path) -> Result<gix::Repository> {
        debug!("Cloning repository to: {:?}", repo_path);

        // Create parent directories if they don't exist
        // This is necessary for nested paths like "group/subgroup/project"
        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create parent directories for {:?}: {}", parent, e))?;
        }

        // Prepare authentication configuration
        // We use http.extraHeader instead of embedding tokens in URLs for better security
        let (clone_url, auth_config) = if let Some(encrypted_token) = &repository.access_token {
            match self.encryption_service.decrypt(encrypted_token) {
                Ok(token) => {
                    let auth_header = if repository.url.contains("gitlab.com") || repository.url.contains("gitlab") {
                        // GitLab: Authorization: Bearer TOKEN
                        format!("Authorization: Bearer {}", token)
                    } else if repository.url.contains("github.com") {
                        // GitHub: Authorization: Bearer TOKEN (or token TOKEN for older APIs)
                        format!("Authorization: Bearer {}", token)
                    } else {
                        // Generic: Bearer token
                        format!("Authorization: Bearer {}", token)
                    };

                    // Return the clean URL (without token) and the auth header
                    (repository.url.clone(), Some(auth_header))
                }
                Err(e) => {
                    warn!(
                        "Failed to decrypt access token, attempting clone without authentication: {}",
                        e
                    );
                    (repository.url.clone(), None)
                }
            }
        } else {
            (repository.url.clone(), None)
        };

        let repo_path_owned = repo_path.to_owned();
        debug!("Starting clone operation with secure authentication (credentials not in URL)");

        // Use spawn_blocking for the clone operation with timeout
        let git_repo = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minutes timeout
            tokio::task::spawn_blocking(move || -> Result<gix::Repository> {
                // Prepare clone with authentication header if provided
                let mut prepare_clone = gix::prepare_clone(clone_url, &repo_path_owned)
                    .map_err(|e| anyhow!("Failed to prepare clone: {}", e))?;

                // Configure for non-interactive mode (no credential prompts)
                // This is critical for cloud-native environments and public repositories
                let mut config_overrides = vec![
                    "credential.helper=".to_string(),     // Disable credential helpers
                    "core.askPass=".to_string(),          // Disable interactive password prompts
                    "GIT_ASKPASS=/bin/false".to_string(), // Prevent any prompts
                ];

                // Configure authentication using http.extraHeader if we have a token
                // This is more secure than embedding the token in the URL
                if let Some(header) = auth_config {
                    config_overrides.push(format!("http.extraHeader={}", header));
                }

                // Set environment variables to prevent interactive prompts
                std::env::set_var("GIT_ASKPASS", "/bin/false");
                std::env::set_var("SSH_ASKPASS_REQUIRE", "force");

                prepare_clone =
                    prepare_clone.with_in_memory_config_overrides(config_overrides.iter().map(|s| s.as_str()));

                // Configure shallow clone (depth=1) to speed up large repositories
                // This clones only the latest commit, significantly reducing download time
                prepare_clone =
                    prepare_clone.configure_remote(|remote| Ok(remote.with_fetch_tags(gix::remote::fetch::Tags::None)));

                // Perform the fetch with shallow clone
                let (_prepared_clone, _outcome) = prepare_clone
                    .fetch_only(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                    .map_err(|e| anyhow!("Failed to fetch: {}", e))?;

                // Open the cloned repository
                let repo =
                    gix::open(&repo_path_owned).map_err(|e| anyhow!("Failed to open cloned repository: {}", e))?;

                info!("Successfully cloned repository with secure authentication");
                Ok(repo)
            }),
        )
        .await
        .map_err(|_| anyhow!("Git clone operation timed out after 5 minutes"))??;

        git_repo
    }
}
