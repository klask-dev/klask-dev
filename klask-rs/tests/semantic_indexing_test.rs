//! Integration test for the semantic indexing write path (Phase 2):
//! real embedding model → real LanceDB store, exercised through the
//! [`VectorIndexer`] worker exactly as the crawler drives it.
//!
//! Loads a real ONNX model (downloaded on first run into target/fastembed-cache)
//! and writes a real LanceDB index, so it is `#[ignore]`d by default:
//!
//! ```bash
//! cargo test --features semantic-search --test semantic_indexing_test -- --ignored --nocapture
//! ```
#![cfg(feature = "semantic-search")]

use klask_rs::config::SemanticSearchConfig;
use klask_rs::services::semantic::chunker::ChunkOptions;
use klask_rs::services::semantic::embedder::{EmbeddingProvider, FastEmbedProvider};
use klask_rs::services::semantic::store::LanceVectorStore;
use klask_rs::services::semantic::{IndexJob, VectorIndexer};
use std::sync::Arc;
use uuid::Uuid;

/// Small model (384 dims) to keep the download reasonable.
fn test_config(cache_dir: &str, vector_dir: &str) -> SemanticSearchConfig {
    SemanticSearchConfig {
        enabled: true,
        model: "Xenova/bge-small-en-v1.5".to_string(),
        cache_dir: cache_dir.to_string(),
        vector_store_dir: vector_dir.to_string(),
        chunk_max_lines: 60,
        chunk_overlap_lines: 15,
        batch_size: 32,
        queue_capacity: 64,
    }
}

fn job(file_id: Uuid, repository: &str, path: &str, content: &str) -> IndexJob {
    IndexJob {
        file_id,
        repository: repository.to_string(),
        project: repository.to_string(),
        version: "main".to_string(),
        path: path.to_string(),
        extension: "rs".to_string(),
        content: content.to_string(),
    }
}

async fn count_eventually(indexer: &VectorIndexer, target: u64) -> u64 {
    for _ in 0..100 {
        let n = indexer.count().await.unwrap();
        if n >= target {
            return n;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    indexer.count().await.unwrap()
}

#[tokio::test]
#[ignore = "requires downloading an ONNX model and writing a real LanceDB index"]
async fn test_full_indexing_lifecycle() {
    let tmp = tempfile::tempdir().unwrap();
    let vector_dir = tmp.path().join("vector-index");
    let config = test_config("target/fastembed-cache", vector_dir.to_str().unwrap());

    let provider = Arc::new(FastEmbedProvider::try_new(&config).expect("load embedding model"));
    let store = Arc::new(LanceVectorStore::open(&config.vector_store_dir, provider.dimension()).await.unwrap());

    let indexer = VectorIndexer::start(
        provider.clone(),
        store.clone(),
        ChunkOptions { max_lines: config.chunk_max_lines, overlap_lines: config.chunk_overlap_lines },
        config.batch_size,
        config.queue_capacity,
    );

    let f1 = Uuid::new_v4();
    let f2 = Uuid::new_v4();

    // 1. Index two files in two repositories.
    indexer
        .index_file(job(
            f1,
            "repo-a",
            "src/auth.rs",
            "fn validate_jwt(token: &str) -> bool {\n    !token.is_empty()\n}",
        ))
        .await
        .unwrap();
    indexer
        .index_file(job(
            f2,
            "repo-b",
            "src/main.rs",
            "fn main() {\n    println!(\"hello\");\n}",
        ))
        .await
        .unwrap();

    let after_index = count_eventually(&indexer, 2).await;
    assert!(
        after_index >= 2,
        "both files should produce at least one chunk each, got {after_index}"
    );

    // 2. Re-index f1 with new content: chunks replaced, not duplicated.
    indexer.index_file(job(f1, "repo-a", "src/auth.rs", "fn validate_jwt() {}")).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let after_reindex = indexer.count().await.unwrap();
    assert_eq!(
        after_reindex, after_index,
        "re-indexing the same file must not add duplicate rows"
    );

    // 3. Delete repo-a: only repo-b's chunks remain.
    let deleted = indexer.delete_project("repo-a").await.unwrap();
    assert!(deleted >= 1, "delete_project should remove repo-a's chunks");
    let remaining = indexer.count().await.unwrap();
    assert!(remaining >= 1, "repo-b chunks should remain after deleting repo-a");

    // 4. Delete repo-b: store is empty.
    indexer.delete_project("repo-b").await.unwrap();
    assert_eq!(
        indexer.count().await.unwrap(),
        0,
        "store should be empty after deleting all repos"
    );
}
