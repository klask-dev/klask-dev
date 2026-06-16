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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
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
    pub allow_registration: bool,
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

impl AppConfig {
    pub fn new() -> Result<Self> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        // Set default values
        let config = Self {
            server: ServerConfig {
                host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse().unwrap_or(3000),
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
                jwt_expires_in: std::env::var("JWT_EXPIRES_IN").unwrap_or_else(|_| "24h".to_string()),
                allow_registration: std::env::var("ALLOW_REGISTRATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
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
                chunk_max_lines: std::env::var("SEMANTIC_SEARCH_CHUNK_MAX_LINES")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
                chunk_overlap_lines: std::env::var("SEMANTIC_SEARCH_CHUNK_OVERLAP_LINES")
                    .unwrap_or_else(|_| "15".to_string())
                    .parse()
                    .unwrap_or(15),
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
