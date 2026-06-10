//! Embedding generation for semantic search.
//!
//! `EmbeddingProvider` abstracts the embedding backend so the model runtime
//! (fastembed/ONNX today, possibly GPU execution providers later) can be
//! swapped without touching the chunking/indexing/query pipeline. Everything
//! runs locally: no cloud API, no code leaves the infrastructure.

use anyhow::Result;

/// A local text-embedding backend.
///
/// Implementations are CPU-bound and synchronous; async callers should wrap
/// calls in `tokio::task::spawn_blocking`.
// Without the semantic-search feature no implementation exists, so the binary
// target flags the methods as never used; that's expected for the opt-in build.
#[cfg_attr(not(feature = "semantic-search"), allow(dead_code))]
pub trait EmbeddingProvider: Send + Sync {
    /// Dimension of the vectors produced by this provider.
    fn dimension(&self) -> usize;

    /// Model identifier (e.g. "jinaai/jina-embeddings-v2-base-code") for
    /// logs, metrics and the admin UI.
    fn model_id(&self) -> &str;

    /// Embed a batch of texts, returning one vector per input, in order.
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

#[cfg(feature = "semantic-search")]
pub use fastembed_provider::FastEmbedProvider;

#[cfg(feature = "semantic-search")]
mod fastembed_provider {
    use super::EmbeddingProvider;
    use crate::config::SemanticSearchConfig;
    use anyhow::{Result, anyhow};
    use fastembed::{EmbeddingModel, ModelInfo, TextEmbedding, TextInitOptions};
    use std::path::PathBuf;
    use std::sync::Mutex;

    /// Embedding provider backed by fastembed (ONNX Runtime, local inference).
    ///
    /// The first initialization downloads the model into `cache_dir`; later
    /// starts load it from disk. Air-gapped deployments can pre-provision the
    /// cache directory (see docs/SEMANTIC_SEARCH_PLAN.md §9).
    pub struct FastEmbedProvider {
        // fastembed's embed() needs &mut self (the ONNX session is stateful)
        model: Mutex<TextEmbedding>,
        dimension: usize,
        model_code: String,
        batch_size: usize,
    }

    /// Resolve a model code (e.g. "jinaai/jina-embeddings-v2-base-code") to a
    /// model supported by fastembed.
    pub(crate) fn resolve_model(model_code: &str) -> Result<ModelInfo<EmbeddingModel>> {
        let mut supported = TextEmbedding::list_supported_models();
        match supported.iter().position(|info| info.model_code.eq_ignore_ascii_case(model_code)) {
            Some(index) => Ok(supported.swap_remove(index)),
            None => {
                let codes = supported.into_iter().map(|m| m.model_code).collect::<Vec<_>>();
                Err(anyhow!(
                    "Unsupported embedding model '{}'. Supported models: {}",
                    model_code,
                    codes.join(", ")
                ))
            }
        }
    }

    impl FastEmbedProvider {
        /// Load (downloading on first use) the configured embedding model.
        pub fn try_new(config: &SemanticSearchConfig) -> Result<Self> {
            let info = resolve_model(&config.model)?;

            let options = TextInitOptions::new(info.model)
                .with_cache_dir(PathBuf::from(&config.cache_dir))
                .with_show_download_progress(false);

            let model = TextEmbedding::try_new(options)
                .map_err(|e| anyhow!("Failed to load embedding model '{}': {e}", info.model_code))?;

            Ok(Self {
                model: Mutex::new(model),
                dimension: info.dim,
                model_code: info.model_code,
                // Bounded to keep a misconfigured value from ballooning ONNX batch buffers
                batch_size: config.batch_size.clamp(1, 512),
            })
        }
    }

    impl EmbeddingProvider for FastEmbedProvider {
        fn dimension(&self) -> usize {
            self.dimension
        }

        fn model_id(&self) -> &str {
            &self.model_code
        }

        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            if texts.is_empty() {
                return Ok(Vec::new());
            }
            let mut model = self.model.lock().map_err(|_| anyhow!("Embedding model mutex poisoned"))?;
            let embeddings =
                model.embed(texts, Some(self.batch_size)).map_err(|e| anyhow!("Embedding generation failed: {e}"))?;
            Ok(embeddings)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_resolve_model_known_codes() {
            // The two models named in docs/SEMANTIC_SEARCH_PLAN.md must stay resolvable
            let jina = resolve_model("jinaai/jina-embeddings-v2-base-code").expect("jina code model supported");
            assert_eq!(jina.dim, 768);

            let bge = resolve_model("Xenova/bge-small-en-v1.5").expect("bge small supported");
            assert_eq!(bge.dim, 384);
        }

        #[test]
        fn test_resolve_model_is_case_insensitive() {
            assert!(resolve_model("JINAAI/JINA-EMBEDDINGS-V2-BASE-CODE").is_ok());
        }

        #[test]
        fn test_resolve_model_unknown_code_lists_alternatives() {
            let err = resolve_model("acme/does-not-exist").unwrap_err().to_string();
            assert!(err.contains("acme/does-not-exist"));
            assert!(
                err.contains("jinaai/jina-embeddings-v2-base-code"),
                "error should list supported models"
            );
        }
    }
}
