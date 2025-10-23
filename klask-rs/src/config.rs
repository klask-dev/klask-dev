use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub search: SearchConfig,
    pub crawler: CrawlerConfig,
    pub auth: AuthConfig,
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
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://klask:klask@localhost/klask".to_string()),
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
                jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string()),
                jwt_expires_in: std::env::var("JWT_EXPIRES_IN").unwrap_or_else(|_| "24h".to_string()),
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
