//! Tantivy configuration loading and management.
//!
//! This module handles loading Tantivy configuration from environment variables
//! with sensible defaults and validation.

use crate::models::TantivyConfig;

/// Load and validate Tantivy configuration.
#[allow(dead_code)]
pub fn load_config() -> TantivyConfig {
    let config = TantivyConfig::from_env();
    match config.validate() {
        Ok(()) => {
            tracing::info!(
                "Tantivy config loaded: memory_mb={}, num_threads={:?}, cpu_cores={}",
                config.memory_mb,
                config.num_threads,
                config.cpu_cores
            );
            config
        }
        Err(e) => {
            tracing::warn!("Invalid Tantivy configuration: {}. Using defaults.", e);
            TantivyConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TantivyConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.memory_mb, 200);
        assert!(config.num_threads.is_none());
    }

    #[test]
    fn test_config_validation_low_memory() {
        let config = TantivyConfig { memory_mb: 10, num_threads: None, cpu_cores: 4 };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_high_memory() {
        let config = TantivyConfig { memory_mb: 10000, num_threads: None, cpu_cores: 4 };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_excessive_threads() {
        let config = TantivyConfig { memory_mb: 200, num_threads: Some(100), cpu_cores: 4 };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = TantivyConfig { memory_mb: 300, num_threads: Some(4), cpu_cores: 4 };
        assert!(config.validate().is_ok());
    }
}
