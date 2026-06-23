use crate::models::Repository;
use crate::services::encryption::EncryptionService;
use anyhow::{Result, anyhow};
use gix::open::Options;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use std::fs::OpenOptions;
use std::io::{Write, Read, Seek, SeekFrom};

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

        // Décrypter le token
        let access_token = if let Some(encrypted_token) = &repository.access_token {
            match self.encryption_service.decrypt(encrypted_token) {
                Ok(token) => Some(token),
                Err(e) => {
                    warn!("Failed to decrypt access token: {}. Proceeding without authentication.", e);
                    None
                }
           }
        } else {
             None
        };

        if repo_path.exists() {
            info!("Updating existing repository at: {:?}", repo_path);

            let result = tokio::time::timeout(
                std::time::Duration::from_secs(180),
                tokio::task::spawn_blocking(move || -> Result<gix::Repository> {
                info!("Opening repository with custom options");
                     let opts = Options::isolated().config_overrides(["gitoxide.credentials.terminalPrompt=0"]);
                     let git_repo = match gix::open_opts(&repo_path_owned, opts) {
                         Ok(repo) => {
                             info!("Repository opened successfully");
                             repo
                         }
                         Err(e) => {
                             error!("Failed to open repository: {}", e);
                             return Err(e.into());
                         }
                     };
     
     
                     info!("Fetching latest changes from remote");
                     let remote = git_repo.find_remote("origin")?;
                     if let Some(url) = remote.url(gix::remote::Direction::Fetch) {
                         info!("Remote 'origin' URL: {}", url);
                     } else {
                         warn!("Remote 'origin' has no URL for fetch direction");
                     }
     
                     info!("Before connect");
                     let mut conn = remote.connect(gix::remote::Direction::Fetch)?;
     
                     // Configure les credentials
                     info!("Before configure credential");
                     if let Some(token) = &access_token {
                         let token_clone = token.clone();
                         conn.set_credentials(move |action| {
                             if let gix::credentials::helper::Action::Get(ctx) = action {
                                 Ok(Some(gix::credentials::protocol::Outcome {
                                     identity: gix::sec::identity::Account {
                                         username: "oauth2".to_string(),
                                         password: token_clone.clone(),
                                         oauth_refresh_token: None,
                                     },
                                     next: ctx.into(),
                                 }))
                             } else {
                                 Ok(None)
                             }
                         });
                     } else {
                         conn.set_credentials(move |_action| {
                             Err(gix::credentials::protocol::Error::Quit)
                         });
                     }
     
                     info!("Before fetch");
                     let fetch_result = conn.prepare_fetch(gix::progress::Discard, Default::default())?;
     
                     info!("Before receive");
                     fetch_result.receive(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;
                                     info!("Successfully fetched latest changes from remote");
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

        info!("repo_path does not exist, cloning fresh repository");
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
                // Prepare clone
                let mut prep = gix::prepare_clone(clone_url, &repo_path_owned)
                    .map_err(|e| anyhow!("Failed to prepare clone: {}", e))?;

                // Configure for non-interactive mode (no credential prompts)
                let mut config_overrides = vec!["gitoxide.credentials.terminalPrompt=0"];
                let accept_invalid_certs = std::env::var("KLASK_GITLAB_ACCEPT_INVALID_CERTS")
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(false);

                if accept_invalid_certs {
                    config_overrides.push("gitoxide.http.sslNoVerify=true");
                }

                prep = prep.with_in_memory_config_overrides(config_overrides.iter().copied());

                // Configure credential helper to provide token or refuse explicitly
                if let Some(ref token) = access_token {
                    let token_for_creds = token.clone();
                    prep = prep.configure_connection(move |connection| {
                        let token_for_closure = token_for_creds.clone();
                        #[allow(clippy::result_large_err)] // external lib should fix this issue
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
                        #[allow(clippy::result_large_err)] // external lib should fix this issue
                        connection.set_credentials(move |_action| Err(gix::credentials::protocol::Error::Quit));
                        Ok(())
                    });
                }

                // Configure shallow clone (depth=1) to speed up large repositories
                prep = prep.configure_remote(|remote| Ok(remote.with_fetch_tags(gix::remote::fetch::Tags::None)));

                // Perform the fetch
                let (_prepared_clone, _outcome) = prep
                    .fetch_only(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                    .map_err(|e| anyhow!("fetch_only failed: {}", e))?;

                let repo = gix::open(&repo_path_owned).map_err(|e| anyhow!("open cloned repo failed: {}", e))?;

                info!("Successfully open repository");

                let config_path = &repo_path_owned.join(".git/config");

                info!("Check name and email exist in : {:?} ",config_path);
                // Open git config file
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)  // Create file is not exist
                    .open(&config_path)?;

                // Read if exist
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                // Verif [user] section
                if !contents.contains("[user]") {
                    // Add user config if not exist
                    file.seek(SeekFrom::End(0))?;
                    writeln!(file, "[user]")?;
                    writeln!(file, "    name = klask")?;
                    writeln!(file, "    email = klask@email.com")?;
                }

                info!("Successfully cloned repository");
                Ok(repo)
            }),
        )
        .await
        .map_err(|_| anyhow!("clone timed out"))??
    }
}
