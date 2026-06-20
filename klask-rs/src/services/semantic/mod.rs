//! Hybrid semantic search (Phase 1: embedding infrastructure).
//!
//! Adds natural-language code search alongside Tantivy BM25, fully
//! self-hosted: chunking (`chunker`), local embedding generation behind the
//! `semantic-search` cargo feature (`embedder`), and rank fusion for the
//! future hybrid query path (`fusion`).
//!
//! Roadmap and design decisions: docs/SEMANTIC_SEARCH_PLAN.md. The vector
//! store, indexing worker and query path land in later phases.

#[cfg(feature = "semantic-search")]
pub mod backfill;
pub mod chunker;
pub mod embedder;
pub mod fusion;
#[cfg(feature = "semantic-search")]
pub mod indexer;
#[cfg(feature = "semantic-search")]
pub mod query;
#[cfg(feature = "semantic-search")]
pub mod store;

#[cfg(feature = "semantic-search")]
pub use backfill::BackfillController;
pub use embedder::EmbeddingProvider;
#[cfg(feature = "semantic-search")]
pub use embedder::FastEmbedProvider;
#[cfg(feature = "semantic-search")]
pub use indexer::{IndexJob, VectorIndexer};
#[cfg(feature = "semantic-search")]
pub use store::VectorStore;

use crate::config::SemanticSearchConfig;
use std::sync::Arc;

/// Zero-sized indexer placeholder for builds without the `semantic-search`
/// feature. Deliberately `Clone` but **not** `Copy` so that the `.clone()`
/// calls threading [`MaybeIndexer`] through the crawler compile identically in
/// both build modes (a `Copy` type would make those clones a clippy warning).
#[cfg(not(feature = "semantic-search"))]
#[derive(Clone)]
pub struct DisabledIndexer;

/// Optional handle to the embedding worker, carried by `AppState` and the
/// crawler. Resolves to `Option<Arc<VectorIndexer>>` with the feature and to a
/// zero-sized always-`None` type without it, so call sites stay feature-agnostic
/// (no `#[cfg]` at every struct field / constructor) and the no-feature build
/// pays nothing.
#[cfg(feature = "semantic-search")]
pub type MaybeIndexer = Option<Arc<VectorIndexer>>;
#[cfg(not(feature = "semantic-search"))]
pub type MaybeIndexer = Option<DisabledIndexer>;

/// Optional handle to the semantic backfill controller, carried by `AppState`
/// so the admin endpoints can start/cancel/poll a rebuild. Resolves to the
/// real controller with the feature and to an always-`None` zero-sized type
/// without it (same pattern as [`MaybeIndexer`]).
#[cfg(feature = "semantic-search")]
pub type MaybeBackfill = Option<BackfillController>;
#[cfg(not(feature = "semantic-search"))]
pub type MaybeBackfill = Option<DisabledIndexer>;

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

/// Open the vector store and start the background embedding worker.
///
/// Returns `None` (degrading to keyword-only indexing) when semantic search is
/// off, the feature is not compiled in, or the store fails to open — the server
/// keeps crawling and serving keyword search instead of refusing to start.
/// Requires the embedding `provider` produced by [`init_embedding_provider`].
#[cfg(feature = "semantic-search")]
pub async fn init_vector_indexer(
    config: &SemanticSearchConfig,
    provider: Arc<dyn EmbeddingProvider>,
) -> Option<Arc<VectorIndexer>> {
    use chunker::ChunkOptions;
    use store::LanceVectorStore;

    // These are clamped to >=1 by VectorIndexer::start, but a zero value almost
    // certainly means a misconfiguration — surface it rather than silently
    // running inference one chunk at a time / with a length-1 queue.
    if config.batch_size == 0 {
        tracing::warn!("SEMANTIC_SEARCH_BATCH_SIZE is 0; using 1 (this cripples embedding throughput)");
    }
    if config.queue_capacity == 0 {
        tracing::warn!("SEMANTIC_SEARCH_QUEUE_CAPACITY is 0; using 1");
    }

    let dimension = provider.dimension();
    let store = match LanceVectorStore::open(&config.vector_store_dir, dimension).await {
        Ok(store) => Arc::new(store),
        Err(e) => {
            tracing::error!(
                "Failed to open vector store at '{}', semantic indexing disabled: {e}",
                config.vector_store_dir
            );
            return None;
        }
    };

    let chunk_options = ChunkOptions { max_lines: config.chunk_max_lines, overlap_lines: config.chunk_overlap_lines };

    let indexer = VectorIndexer::start(provider, store, chunk_options, config.batch_size, config.queue_capacity);

    match indexer.count().await {
        Ok(n) => tracing::info!(
            "Vector store ready at '{}' ({} chunks, dimension {})",
            config.vector_store_dir,
            n,
            dimension
        ),
        Err(e) => tracing::warn!("Vector store opened but count() failed: {e}"),
    }

    Some(Arc::new(indexer))
}

/// Build the semantic backfill controller from a running indexer.
///
/// Returns `None` when there is no indexer (semantic search disabled or failed
/// to start). The controller lets admins rebuild the vector index from the
/// existing Tantivy documents (Phase 3); it carries clones of the search
/// service and indexer to drive the rebuild.
#[cfg(feature = "semantic-search")]
pub fn init_backfill_controller(
    indexer: &MaybeIndexer,
    search_service: Arc<crate::services::SearchService>,
    provider: &Arc<dyn EmbeddingProvider>,
) -> MaybeBackfill {
    indexer.as_ref().map(|indexer| {
        BackfillController::new(
            search_service,
            indexer.clone(),
            provider.model_id().to_string(),
            provider.dimension(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disabled_config() -> SemanticSearchConfig {
        SemanticSearchConfig {
            enabled: false,
            model: "jinaai/jina-embeddings-v2-base-code".to_string(),
            cache_dir: "./models".to_string(),
            vector_store_dir: "./vector-index".to_string(),
            chunk_max_lines: 60,
            chunk_overlap_lines: 15,
            batch_size: 32,
            queue_capacity: 1000,
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
