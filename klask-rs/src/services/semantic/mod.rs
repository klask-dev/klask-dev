//! Hybrid semantic search (Phase 1: embedding infrastructure).
//!
//! Adds natural-language code search alongside Tantivy BM25, fully
//! self-hosted: chunking (`chunker`), local embedding generation behind the
//! `semantic-search` cargo feature (`embedder`), and rank fusion for the
//! future hybrid query path (`fusion`).
//!
//! Roadmap and design decisions: docs/SEMANTIC_SEARCH_PLAN.md. The vector
//! store, indexing worker and query path land in later phases.

pub mod chunker;
pub mod embedder;
pub mod fusion;

pub use embedder::EmbeddingProvider;
#[cfg(feature = "semantic-search")]
pub use embedder::FastEmbedProvider;

use crate::config::SemanticSearchConfig;
use std::sync::Arc;

/// Initialize the embedding provider from configuration.
///
/// Returns `None` when semantic search is disabled, when the binary was built
/// without the `semantic-search` feature, or when the model fails to load —
/// the server then degrades gracefully to keyword-only search (with an error
/// in the logs), instead of refusing to start.
pub fn init_embedding_provider(config: &SemanticSearchConfig) -> Option<Arc<dyn EmbeddingProvider>> {
    if !config.enabled {
        tracing::debug!("Semantic search is disabled (SEMANTIC_SEARCH_ENABLED=false)");
        return None;
    }

    #[cfg(feature = "semantic-search")]
    {
        tracing::info!(
            "Semantic search enabled: loading embedding model '{}' (cache: {}). First start downloads the model.",
            config.model,
            config.cache_dir
        );
        let started = std::time::Instant::now();
        match FastEmbedProvider::try_new(config) {
            Ok(provider) => {
                // Warm-up: the first inference initializes the ONNX session
                match provider.embed(&["fn main() {}".to_string()]) {
                    Ok(_) => {
                        tracing::info!(
                            "Embedding model '{}' ready: dimension={}, loaded+warmed in {:.1}s",
                            provider.model_id(),
                            provider.dimension(),
                            started.elapsed().as_secs_f32()
                        );
                        Some(Arc::new(provider))
                    }
                    Err(e) => {
                        tracing::error!("Embedding model warm-up failed, semantic search disabled: {e}");
                        None
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to initialize embedding provider, semantic search disabled: {e}");
                None
            }
        }
    }

    #[cfg(not(feature = "semantic-search"))]
    {
        tracing::warn!(
            "SEMANTIC_SEARCH_ENABLED=true but this binary was built without the 'semantic-search' \
             cargo feature; semantic search stays disabled. Rebuild with --features semantic-search."
        );
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disabled_config() -> SemanticSearchConfig {
        SemanticSearchConfig {
            enabled: false,
            model: "jinaai/jina-embeddings-v2-base-code".to_string(),
            cache_dir: "./models".to_string(),
            chunk_max_lines: 60,
            chunk_overlap_lines: 15,
            batch_size: 32,
        }
    }

    #[test]
    fn test_disabled_config_yields_no_provider() {
        assert!(init_embedding_provider(&disabled_config()).is_none());
    }

    #[cfg(not(feature = "semantic-search"))]
    #[test]
    fn test_enabled_without_feature_yields_no_provider() {
        let config = SemanticSearchConfig { enabled: true, ..disabled_config() };
        assert!(init_embedding_provider(&config).is_none());
    }
}
