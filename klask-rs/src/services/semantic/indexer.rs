//! Background embedding worker that feeds the vector store.
//!
//! The crawler hands files to [`VectorIndexer`] via a bounded channel; a single
//! worker task chunks each file, embeds the chunks with the local
//! [`EmbeddingProvider`], and upserts them into the [`VectorStore`]. Decoupling
//! embedding from the crawl keeps ONNX inference off the crawl's hot path and
//! funnels all inference through one ONNX session (the provider serializes
//! internally), avoiding lock contention and unbounded parallel-batch memory.
//!
//! Backpressure is strict: when the queue is full the crawl *awaits* capacity
//! (it does not drop work), so the vector index stays consistent with what was
//! crawled. See docs/SEMANTIC_SEARCH_PLAN.md §4.
//!
//! Gated on the `semantic-search` feature (it depends on the vector store).
#![cfg(feature = "semantic-search")]

use super::chunker::{ChunkOptions, chunk_file};
use super::embedder::EmbeddingProvider;
use super::store::{ChunkRecord, VectorStore};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use uuid::Uuid;

/// A single file to embed and store. Owns its content so the crawl can move on.
#[derive(Debug, Clone)]
pub struct IndexJob {
    pub file_id: Uuid,
    pub repository: String,
    pub project: String,
    pub version: String,
    pub path: String,
    pub extension: String,
    pub content: String,
}

/// Handle to the background embedding worker.
///
/// Cheap to clone (it only holds an `mpsc::Sender`). Held by `AppState` and the
/// crawler. Dropping all clones closes the channel, which drains and stops the
/// worker gracefully.
#[derive(Clone)]
pub struct VectorIndexer {
    tx: mpsc::Sender<IndexJob>,
    store: Arc<dyn VectorStore>,
}

impl VectorIndexer {
    /// Spawn the worker and return a handle.
    ///
    /// `batch_size` caps how many chunks are embedded per inference call;
    /// `capacity` bounds the queue (and thus the crawl's backpressure point).
    pub fn start(
        provider: Arc<dyn EmbeddingProvider>,
        store: Arc<dyn VectorStore>,
        chunk_options: ChunkOptions,
        batch_size: usize,
        capacity: usize,
    ) -> Self {
        let (tx, rx) = mpsc::channel(capacity.max(1));
        let worker = Worker { provider, store: store.clone(), chunk_options, batch_size: batch_size.max(1) };
        tokio::spawn(worker.run(rx));
        info!("Semantic embedding worker started (queue capacity {})", capacity.max(1));
        Self { tx, store }
    }

    /// Enqueue a file for embedding.
    ///
    /// Awaits queue capacity when full (strict backpressure). Errors only if the
    /// worker has stopped (channel closed) — the caller logs and continues, as
    /// the Tantivy index remains the source of truth.
    pub async fn index_file(&self, job: IndexJob) -> Result<()> {
        self.tx.send(job).await.map_err(|_| anyhow!("Semantic embedding worker is no longer running"))
    }

    /// Delete all chunks of a repository.
    ///
    /// Runs directly against the store (not via the queue) so a delete issued
    /// before a re-crawl is applied before the re-crawl's inserts, matching the
    /// crawler's "delete then re-index" ordering for Tantivy.
    pub async fn delete_project(&self, repository: &str) -> Result<u64> {
        self.store.delete_project_chunks(repository).await
    }

    /// Current number of stored chunks (for logging / admin card).
    pub async fn count(&self) -> Result<u64> {
        self.store.count().await
    }
}

struct Worker {
    provider: Arc<dyn EmbeddingProvider>,
    store: Arc<dyn VectorStore>,
    chunk_options: ChunkOptions,
    batch_size: usize,
}

impl Worker {
    async fn run(self, mut rx: mpsc::Receiver<IndexJob>) {
        while let Some(job) = rx.recv().await {
            let file_id = job.file_id;
            if let Err(e) = self.process(job).await {
                // One bad file must never kill the worker: log and keep draining.
                error!("Semantic indexing failed for file_id={file_id}: {e}");
            }
        }
        info!("Semantic embedding worker stopped (queue drained)");
    }

    async fn process(&self, job: IndexJob) -> Result<()> {
        let chunks = chunk_file(&job.path, &job.content, &self.chunk_options);
        if chunks.is_empty() {
            // Empty file: still upsert (with no records) so a file that became
            // empty has its old chunks removed.
            return self.store.upsert_file_chunks(job.file_id, Vec::new()).await;
        }

        let mut records: Vec<ChunkRecord> = Vec::with_capacity(chunks.len());
        for batch in chunks.chunks(self.batch_size) {
            let texts: Vec<String> = batch.iter().map(|c| c.text.clone()).collect();
            // Embedding is CPU-bound and synchronous; keep it off the async runtime.
            let provider = self.provider.clone();
            let vectors = tokio::task::spawn_blocking(move || provider.embed(&texts))
                .await
                .map_err(|e| anyhow!("embedding task panicked: {e}"))??;

            if vectors.len() != batch.len() {
                return Err(anyhow!(
                    "embedder returned {} vectors for {} chunks",
                    vectors.len(),
                    batch.len()
                ));
            }

            for (chunk, vector) in batch.iter().zip(vectors) {
                // Metadata is identical for every chunk of a file; the per-field
                // clone here is one String alloc per chunk (unavoidable while
                // ChunkRecord owns its strings), not per batch field × chunk.
                records.push(ChunkRecord {
                    file_id: job.file_id,
                    repository: job.repository.clone(),
                    project: job.project.clone(),
                    version: job.version.clone(),
                    path: job.path.clone(),
                    extension: job.extension.clone(),
                    start_line: chunk.start_line as u32,
                    end_line: chunk.end_line as u32,
                    vector,
                });
            }
        }

        let n = records.len();
        self.store.upsert_file_chunks(job.file_id, records).await?;
        debug!("Embedded {n} chunks for {} (file_id={})", job.path, job.file_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::semantic::store::LanceVectorStore;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const DIM: usize = 8;

    /// Deterministic, ONNX-free provider so worker tests run in normal CI
    /// without downloading a model.
    struct MockProvider {
        calls: Arc<AtomicUsize>,
        fail: bool,
    }

    impl EmbeddingProvider for MockProvider {
        fn dimension(&self) -> usize {
            DIM
        }
        fn model_id(&self) -> &str {
            "mock"
        }
        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                return Err(anyhow!("mock embed failure"));
            }
            // A trivial but input-dependent vector so identical texts map alike.
            Ok(texts
                .iter()
                .map(|t| {
                    let seed = (t.len() % 7) as f32;
                    vec![seed; DIM]
                })
                .collect())
        }
    }

    async fn store(dir: &tempfile::TempDir) -> Arc<dyn VectorStore> {
        Arc::new(LanceVectorStore::open(dir.path(), DIM).await.unwrap())
    }

    fn job(content: &str) -> IndexJob {
        IndexJob {
            file_id: Uuid::new_v4(),
            repository: "repo".to_string(),
            project: "repo".to_string(),
            version: "main".to_string(),
            path: "src/lib.rs".to_string(),
            extension: "rs".to_string(),
            content: content.to_string(),
        }
    }

    #[tokio::test]
    async fn test_indexes_a_file_end_to_end() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir).await;
        let calls = Arc::new(AtomicUsize::new(0));
        let indexer = VectorIndexer::start(
            Arc::new(MockProvider { calls: calls.clone(), fail: false }),
            store.clone(),
            ChunkOptions::default(),
            32,
            16,
        );

        indexer.index_file(job("fn main() {\n    println!(\"hi\");\n}")).await.unwrap();
        // Drop the indexer to close the channel and let the worker drain+finish.
        drop(indexer);
        // The store reflects the work once the worker has processed it.
        wait_until(|| store.count(), 1).await;
        assert!(calls.load(Ordering::SeqCst) >= 1);
    }

    #[tokio::test]
    async fn test_reindex_same_file_no_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir).await;
        let indexer = VectorIndexer::start(
            Arc::new(MockProvider { calls: Arc::new(AtomicUsize::new(0)), fail: false }),
            store.clone(),
            ChunkOptions::default(),
            32,
            16,
        );
        let mut j = job("fn a() {}");
        indexer.index_file(j.clone()).await.unwrap();
        wait_until(|| store.count(), 1).await;
        // Same file_id, new content → replaces, not appends.
        j.content = "fn b() {}\nfn c() {}".to_string();
        indexer.index_file(j).await.unwrap();
        // Still a single chunk for a tiny file → count stays 1.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(store.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_bad_file_does_not_kill_worker() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir).await;
        let indexer = VectorIndexer::start(
            Arc::new(MockProvider { calls: Arc::new(AtomicUsize::new(0)), fail: true }),
            store.clone(),
            ChunkOptions::default(),
            32,
            16,
        );
        // Failing embed: worker logs and continues; nothing stored, no panic.
        indexer.index_file(job("fn a() {}")).await.unwrap();
        indexer.index_file(job("fn b() {}")).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(store.count().await.unwrap(), 0);
        // Worker still alive: enqueue succeeds.
        assert!(indexer.index_file(job("fn c() {}")).await.is_ok());
    }

    #[tokio::test]
    async fn test_delete_project_passthrough() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir).await;
        let indexer = VectorIndexer::start(
            Arc::new(MockProvider { calls: Arc::new(AtomicUsize::new(0)), fail: false }),
            store.clone(),
            ChunkOptions::default(),
            32,
            16,
        );
        indexer.index_file(job("fn a() {}")).await.unwrap();
        wait_until(|| store.count(), 1).await;
        assert_eq!(indexer.delete_project("repo").await.unwrap(), 1);
        assert_eq!(store.count().await.unwrap(), 0);
    }

    /// Poll an async count fn until it reaches `target` (bounded), so tests
    /// don't race the background worker.
    async fn wait_until<F, Fut>(mut f: F, target: u64)
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<u64>>,
    {
        for _ in 0..50 {
            if f().await.unwrap() >= target {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        panic!("count did not reach {target} in time");
    }
}
