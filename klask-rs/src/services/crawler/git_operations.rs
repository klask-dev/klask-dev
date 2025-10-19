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
                    std::env::set_var("GIT_TERMINAL_PROMPT", "0");
                    std::env::set_var("GIT_ASKPASS", "");
                    std::env::set_var("SSH_ASKPASS", "");

                    let git_repo = gix::open(&repo_path_owned)?;

                    info!("Fetching latest changes from remote");

                    if let Ok(remote) = git_repo.find_remote("origin") {
                        if let Ok(conn) = remote.connect(gix::remote::Direction::Fetch) {
                            if let Ok(prep) = conn.prepare_fetch(gix::progress::Discard, Default::default()) {
                                if let Err(e) = prep.receive(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED) {
                                    warn!("Failed to receive fetch: {}", e);
                                } else {
                                    info!("Successfully fetched latest changes");
                                }
                            }
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
                // Disable ALL interactive prompts for server-mode operation
                std::env::set_var("GIT_TERMINAL_PROMPT", "0");
                std::env::set_var("GIT_ASKPASS", "");
                std::env::set_var("SSH_ASKPASS", "");

                let mut prep = gix::prepare_clone(clone_url, &repo_path_owned)
                    .map_err(|e| anyhow!("prepare_clone failed: {}", e))?;

                // Configure credential helper to provide token or refuse explicitly
                if let Some(ref token) = access_token {
                    let token_for_creds = token.clone();
                    prep = prep.configure_connection(move |connection| {
                        let token_for_closure = token_for_creds.clone();
                        connection.set_credentials(move |action| {
                            // Extract context from the action
                            if let gix::credentials::helper::Action::Get(ctx) = action {
                                Ok(Some(gix::credentials::protocol::Outcome {
                                    identity: gix::sec::identity::Account {
                                        username: "oauth2".to_string(),
                                        password: token_for_closure.clone(),
                                        oauth_refresh_token: None,
                                    },
                                    next: ctx.into(),
                                }))
                            } else {
                                // Ignore store/erase operations
                                Ok(None)
                            }
                        });
                        Ok(())
                    });
                } else {
                    // No token - refuse credentials to prevent prompting
                    prep = prep.configure_connection(move |connection| {
                        connection.set_credentials(move |_action| Err(gix::credentials::protocol::Error::Quit));
                        Ok(())
                    });
                }

                prep = prep.configure_remote(|remote| Ok(remote.with_fetch_tags(gix::remote::fetch::Tags::None)));

                let (_prep, _outcome) = prep
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
