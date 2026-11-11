use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub search: SearchConfig,
    pub crawler: CrawlerConfig,
    pub auth: AuthConfig,
    pub cors: CorsConfig,
    pub semantic: SemanticSearchConfig,
    pub repositories: RepositoryPolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// Maximum accepted request body size in bytes (`MAX_REQUEST_BODY_BYTES`).
    pub max_body_bytes: usize,
    /// Per-request timeout in seconds, 0 disables it (`REQUEST_TIMEOUT_SECS`).
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub index_dir: String,
    pub max_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerConfig {
    pub temp_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    /// Self-service registration (`ALLOW_REGISTRATION`, default `false`).
    pub allow_registration: bool,
    /// Adds the `Secure` attribute to the auth cookie (`COOKIE_SECURE`, default `false`).
    /// Enable it whenever the instance is served over HTTPS.
    pub cookie_secure: bool,
    /// Number of reverse proxies in front of the backend that append to
    /// `X-Forwarded-For` (`TRUSTED_PROXY_HOPS`, default `0` = header ignored).
    /// Count only the proxies the request really goes through: `1` behind an
    /// ingress or gateway that routes `/api` to the backend, one more for each
    /// extra proxy. See `api::auth` / `auth::rate_limit::client_key`.
    pub trusted_proxy_hops: usize,
    /// Optional secret required by `POST /api/auth/setup` (`SETUP_TOKEN`).
    /// When unset, the first-run setup is open to anyone who can reach the instance.
    pub setup_token: Option<String>,
}

/// Policy applied to repository definitions supplied through the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPolicyConfig {
    /// Directories a `FileSystem` repository may point into
    /// (`FILESYSTEM_ALLOWED_ROOTS`, comma-separated, default empty = no
    /// FileSystem repository can be created or updated).
    pub filesystem_allowed_roots: Vec<String>,
    /// Allow `http://` URLs and private/loopback hosts for Git, GitLab and
    /// GitHub repositories and GitLab discovery (`ALLOW_PRIVATE_NETWORK_URLS`,
    /// default `true`: Klask is self-hosted and usually indexes an internal
    /// GitLab or Gitea). Set `false` on exposed or shared instances.
    pub allow_private_network_urls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
}

/// Hybrid semantic search settings (see docs/SEMANTIC_SEARCH_PLAN.md).
/// Disabled by default; also requires a binary built with the
/// `semantic-search` cargo feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchConfig {
    pub enabled: bool,
    /// Embedding model code, e.g. "jinaai/jina-embeddings-v2-base-code"
    /// or "Xenova/bge-small-en-v1.5" (smaller/faster).
    pub model: String,
    /// Directory where the ONNX model is cached. Pre-provision it for
    /// air-gapped deployments.
    pub cache_dir: String,
    /// Directory where the embedded vector store (LanceDB) persists, sibling of
    /// the Tantivy index.
    pub vector_store_dir: String,
    /// Maximum number of file lines per embedding chunk.
    pub chunk_max_lines: usize,
    /// Lines of overlap between consecutive chunks.
    pub chunk_overlap_lines: usize,
    /// Inference batch size.
    pub batch_size: usize,
    /// Bounded embedding-queue capacity. The crawl blocks when the queue is
    /// full (strict backpressure), so this caps in-flight files awaiting
    /// embedding rather than dropping work.
    pub queue_capacity: usize,
}

/// Parse a boolean environment variable. Unset yields `default`; any value
/// other than the usual truthy/falsy spellings is an error, so a typo cannot
/// silently change a security setting.
fn env_bool(name: &str, default: bool) -> Result<bool> {
    match std::env::var(name) {
        Err(_) => Ok(default),
        Ok(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            other => Err(anyhow::anyhow!("{name} must be true or false, got '{other}'")),
        },
    }
}

/// Parse a duration such as `"90s"`, `"30m"`, `"24h"`, `"7d"` into seconds.
/// A number without unit is read as hours (historical default of `JWT_EXPIRES_IN`).
/// Used both for JWT lifetime and the auth cookie `Max-Age`, so they cannot diverge.
pub fn parse_duration_secs(input: &str) -> Result<i64> {
    let trimmed = input.trim();
    let (digits, multiplier) = match trimmed.chars().last() {
        Some('s') => (&trimmed[..trimmed.len() - 1], 1),
        Some('m') => (&trimmed[..trimmed.len() - 1], 60),
        Some('h') => (&trimmed[..trimmed.len() - 1], 3600),
        Some('d') => (&trimmed[..trimmed.len() - 1], 86_400),
        _ => (trimmed, 3600),
    };
    let value: i64 =
        digits.parse().map_err(|_| anyhow::anyhow!("invalid duration '{input}' (expected e.g. 90s, 30m, 24h, 7d)"))?;
    if value <= 0 {
        return Err(anyhow::anyhow!("duration '{input}' must be strictly positive"));
    }
    value.checked_mul(multiplier).ok_or_else(|| anyhow::anyhow!("duration '{input}' is too large"))
}

impl AppConfig {
    pub fn new() -> Result<Self> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        // Set default values
        let config = Self {
            server: ServerConfig {
                host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse().unwrap_or(3000),
                max_body_bytes: std::env::var("MAX_REQUEST_BODY_BYTES")
                    .unwrap_or_else(|_| "4194304".to_string())
                    .parse()
                    .map_err(|_| anyhow::anyhow!("MAX_REQUEST_BODY_BYTES must be a positive integer"))?,
                request_timeout_secs: std::env::var("REQUEST_TIMEOUT_SECS")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .map_err(|_| anyhow::anyhow!("REQUEST_TIMEOUT_SECS must be a non-negative integer"))?,
            },
            database: DatabaseConfig {
                url: {
                    let url = std::env::var("DATABASE_URL")
                        .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable must be set"))?;
                    if url.is_empty() {
                        return Err(anyhow::anyhow!("DATABASE_URL environment variable cannot be empty"));
                    }
                    url
                },
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            },
            search: SearchConfig {
                index_dir: std::env::var("SEARCH_INDEX_DIR").unwrap_or_else(|_| "./index".to_string()),
                max_results: std::env::var("SEARCH_MAX_RESULTS")
                    .unwrap_or_else(|_| "10000".to_string())
                    .parse()
                    .unwrap_or(10000),
            },
            crawler: CrawlerConfig {
                temp_dir: std::env::var("CRAWLER_TEMP_DIR")
                    .unwrap_or_else(|_| std::env::temp_dir().join("klask-crawler").to_string_lossy().to_string()),
            },
            auth: AuthConfig {
                jwt_secret: {
                    let secret = std::env::var("JWT_SECRET")
                        .map_err(|_| anyhow::anyhow!("JWT_SECRET environment variable must be set"))?;
                    if secret.is_empty() {
                        return Err(anyhow::anyhow!("JWT_SECRET environment variable cannot be empty"));
                    }
                    if secret.len() < 32 {
                        return Err(anyhow::anyhow!(
                            "JWT_SECRET must be at least 32 characters long for adequate security. \
                            Current length: {}. Generate a strong secret with: \
                            openssl rand -hex 32",
                            secret.len()
                        ));
                    }
                    secret
                },
                jwt_expires_in: {
                    let raw = std::env::var("JWT_EXPIRES_IN").unwrap_or_else(|_| "24h".to_string());
                    // Validate at startup rather than failing on the first login
                    parse_duration_secs(&raw).map_err(|e| anyhow::anyhow!("JWT_EXPIRES_IN: {e}"))?;
                    raw
                },
                allow_registration: env_bool("ALLOW_REGISTRATION", false)?,
                cookie_secure: env_bool("COOKIE_SECURE", false)?,
                trusted_proxy_hops: std::env::var("TRUSTED_PROXY_HOPS")
                    .unwrap_or_else(|_| "0".to_string())
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| anyhow::anyhow!("TRUSTED_PROXY_HOPS must be a non-negative integer"))?,
                setup_token: std::env::var("SETUP_TOKEN").ok().filter(|t| !t.trim().is_empty()),
            },
            repositories: RepositoryPolicyConfig {
                filesystem_allowed_roots: std::env::var("FILESYSTEM_ALLOWED_ROOTS")
                    .unwrap_or_default()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                allow_private_network_urls: env_bool("ALLOW_PRIVATE_NETWORK_URLS", true)?,
            },
            cors: CorsConfig {
                allowed_origins: std::env::var("ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:5173,http://localhost:8080".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            semantic: SemanticSearchConfig {
                enabled: std::env::var("SEMANTIC_SEARCH_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                model: std::env::var("SEMANTIC_SEARCH_MODEL")
                    .unwrap_or_else(|_| "jinaai/jina-embeddings-v2-base-code".to_string()),
                cache_dir: std::env::var("SEMANTIC_SEARCH_CACHE_DIR").unwrap_or_else(|_| "./models".to_string()),
                vector_store_dir: std::env::var("SEMANTIC_SEARCH_VECTOR_DIR")
                    .unwrap_or_else(|_| "./vector-index".to_string()),
                // Sized so a typical code chunk stays within the embedder's
                // 512-token truncation window (see MAX_SEQUENCE_TOKENS in
                // semantic::embedder) — lines past that budget are silently
                // invisible to semantic search.
                chunk_max_lines: std::env::var("SEMANTIC_SEARCH_CHUNK_MAX_LINES")
                    .unwrap_or_else(|_| "45".to_string())
                    .parse()
                    .unwrap_or(45),
                chunk_overlap_lines: std::env::var("SEMANTIC_SEARCH_CHUNK_OVERLAP_LINES")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                batch_size: std::env::var("SEMANTIC_SEARCH_BATCH_SIZE")
                    .unwrap_or_else(|_| "32".to_string())
                    .parse()
                    .unwrap_or(32),
                queue_capacity: std::env::var("SEMANTIC_SEARCH_QUEUE_CAPACITY")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
            },
        };

        Ok(config)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new().expect("Failed to create default config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_secs_units() {
        assert_eq!(parse_duration_secs("90s").unwrap(), 90);
        assert_eq!(parse_duration_secs("30m").unwrap(), 1800);
        assert_eq!(parse_duration_secs("24h").unwrap(), 86_400);
        assert_eq!(parse_duration_secs("7d").unwrap(), 604_800);
        // Bare number: hours, as before
        assert_eq!(parse_duration_secs("2").unwrap(), 7200);
    }

    #[test]
    fn test_parse_duration_secs_rejects_invalid() {
        assert!(parse_duration_secs("").is_err());
        assert!(parse_duration_secs("abc").is_err());
        assert!(parse_duration_secs("0h").is_err());
        assert!(parse_duration_secs("-5m").is_err());
        assert!(parse_duration_secs("999999999999999d").is_err());
    }
}

#[cfg(test)]
mod env_bool_tests {
    use super::*;

    #[test]
    fn test_env_bool_default_when_unset() {
        // Unique variable name: never set by the environment or a .env file
        assert!(!env_bool("KLASK_TEST_UNSET_BOOL_VARIABLE", false).expect("default"));
        assert!(env_bool("KLASK_TEST_UNSET_BOOL_VARIABLE", true).expect("default"));
    }
}
