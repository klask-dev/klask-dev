//! Background embedding worker that feeds the vector store.
//!
//! The crawler hands files to [`VectorIndexer`] via a bounded channel; a single
//! worker task chunks each file, embeds the chunks with the local
//! [`EmbeddingProvider`], and upserts them into the [`VectorStore`]. Decoupling
//! embedding from the crawl keeps ONNX inference off the crawl's hot path and
//! funnels all inference through one ONNX session (the provider serializes
//! internally), avoiding lock contention and unbounded parallel-batch memory.
//!
//! Chunks are accumulated across files and written to the store in batches:
//! every LanceDB write commits a new table version and fragment, so one write
//! per file made bulk indexing collapse. The worker flushes when the batch is
//! full or when the queue goes quiet, so a lagging index still catches up
//! promptly.
//!
//! Backpressure is strict: when the queue is full the crawl *awaits* capacity
//! (it does not drop work), so the vector index stays consistent with what was
//! crawled. See docs/SEMANTIC_SEARCH_PLAN.md §4.
//!
//! Gated on the `semantic-search` feature (it depends on the vector store).
#![cfg(feature = "semantic-search")]

use super::chunker::{ChunkOptions, chunk_file};
use super::embedder::EmbeddingProvider;
use super::store::{ChunkRecord, VectorHit, VectorSearchFilters, VectorStore};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, error, info};
use uuid::Uuid;

/// Number of inference batches worth of chunks buffered before a store write.
const WRITE_FLUSH_BATCHES: usize = 32;

/// Floor for the write batch, so a tiny `batch_size` still writes in bulk.
const MIN_FLUSH_CHUNKS: usize = 256;

/// How a file's chunks are written to the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
    /// The file has no chunks in the store yet, so insert without probing for
    /// them. Both bulk paths qualify: a crawl purges the repository's chunks
    /// before re-indexing and the backfill clears the whole store, and a
    /// `file_id` is produced at most once per run. This is what lets the worker
    /// batch several files into a single write.
    Append,
    /// The file may already have chunks: delete them, then insert. For
    /// re-indexing a single file outside a full crawl or rebuild. Applied on
    /// its own (never batched) so the delete cannot race a buffered insert.
    #[allow(dead_code)] // wiring for incremental re-index (crawler TODO)
    Replace,
}

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
    /// Whether the file's existing chunks must be removed first. See [`WriteMode`].
    pub mode: WriteMode,
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
    // Files enqueued but not yet fully embedded (queued + in-flight). Lets the
    // admin UI show that semantic indexing is still working after the crawl /
    // backfill has finished handing files over.
    pending: Arc<AtomicU64>,
    // Serializes `optimize()`: concurrent crawls each finish with one, and two
    // compactions of the same table race for the same fragments.
    optimizing: Arc<Mutex<()>>,
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
        let pending = Arc::new(AtomicU64::new(0));
        let batch_size = batch_size.max(1);
        let worker = Worker {
            provider,
            store: store.clone(),
            chunk_options,
            batch_size,
            // Buffer roughly 32 inference batches before writing, so one crawl
            // produces thousands of chunks per LanceDB commit instead of one
            // commit per file.
            flush_threshold: (batch_size * WRITE_FLUSH_BATCHES).max(MIN_FLUSH_CHUNKS),
            pending: pending.clone(),
        };
        tokio::spawn(worker.run(rx));
        info!("Semantic embedding worker started (queue capacity {})", capacity.max(1));
        Self { tx, store, pending, optimizing: Arc::new(Mutex::new(())) }
    }

    /// Enqueue a file for embedding.
    ///
    /// Awaits queue capacity when full (strict backpressure). Errors only if the
    /// worker has stopped (channel closed) — the caller logs and continues, as
    /// the Tantivy index remains the source of truth.
    pub async fn index_file(&self, job: IndexJob) -> Result<()> {
        // Count before sending so `pending` never under-reports; roll back if
        // the worker is gone and the job was never queued.
        self.pending.fetch_add(1, Ordering::SeqCst);
        self.tx.send(job).await.map_err(|_| {
            self.pending.fetch_sub(1, Ordering::SeqCst);
            anyhow!("Semantic embedding worker is no longer running")
        })
    }

    /// Files enqueued but not yet embedded (queued + in-flight). Zero means
    /// the semantic index has caught up with everything handed to it.
    pub fn pending(&self) -> u64 {
        self.pending.load(Ordering::SeqCst)
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

    /// Nearest-neighbour search over the stored chunk vectors (Phase 4 query
    /// path). A passthrough to the store so the query path reuses the same
    /// handle the worker writes through, keeping the indexer the sole owner of
    /// the store. Returns up to `limit` hits closest-first, filtered to match.
    pub async fn search(
        &self,
        query_vector: &[f32],
        limit: usize,
        filters: &VectorSearchFilters,
    ) -> Result<Vec<VectorHit>> {
        self.store.search(query_vector, limit, filters).await
    }

    /// Delete every stored chunk. Runs directly against the store (not via the
    /// queue) so the semantic backfill (Phase 3) can wipe the index before a
    /// full rebuild without racing queued inserts. Returns rows removed.
    pub async fn clear(&self) -> Result<u64> {
        self.store.clear().await
    }

    /// Compact the store and refresh its indexes after a bulk write.
    ///
    /// Call once a crawl or backfill has finished and the queue has drained,
    /// never per file. Concurrent calls are skipped rather than queued: a
    /// second compaction of the same table has nothing to add and would only
    /// contend for the same fragments.
    pub async fn optimize(&self) -> Result<()> {
        let Ok(_guard) = self.optimizing.try_lock() else {
            debug!("Vector store optimization already in progress, skipping");
            return Ok(());
        };
        self.store.optimize().await
    }
}

struct Worker {
    provider: Arc<dyn EmbeddingProvider>,
    store: Arc<dyn VectorStore>,
    chunk_options: ChunkOptions,
    batch_size: usize,
    flush_threshold: usize,
    pending: Arc<AtomicU64>,
}

impl Worker {
    async fn run(self, mut rx: mpsc::Receiver<IndexJob>) {
        // Chunks of several files, written to the store as one batch.
        let mut buffer: Vec<ChunkRecord> = Vec::new();
        // Files whose chunks sit in `buffer`; they stay counted in `pending`
        // until the batch is actually stored.
        let mut buffered_files: u64 = 0;

        while let Some(job) = rx.recv().await {
            let file_id = job.file_id;
            let mode = job.mode;

            match self.embed(job).await {
                Ok(records) => match mode {
                    WriteMode::Append => {
                        buffer.extend(records);
                        buffered_files += 1;
                    }
                    WriteMode::Replace => {
                        // Flush first: a buffered insert for this file must not
                        // land after the delete this branch is about to issue.
                        self.flush(&mut buffer, &mut buffered_files).await;
                        if let Err(e) = self.store.upsert_file_chunks(file_id, records).await {
                            error!("Semantic indexing failed for file_id={file_id}: {e}");
                        }
                        self.pending.fetch_sub(1, Ordering::SeqCst);
                    }
                },
                Err(e) => {
                    // One bad file must never kill the worker: log and keep
                    // draining. Failed files also count down, since `pending`
                    // tracks outstanding work, not successes.
                    error!("Semantic indexing failed for file_id={file_id}: {e}");
                    self.pending.fetch_sub(1, Ordering::SeqCst);
                }
            }

            // Write when the batch is full, or when the crawl is no longer
            // feeding us — an idle queue means the write costs nothing we need,
            // and it lets `pending` reach zero so callers see the index caught up.
            if buffer.len() >= self.flush_threshold || rx.is_empty() {
                self.flush(&mut buffer, &mut buffered_files).await;
            }
        }

        self.flush(&mut buffer, &mut buffered_files).await;
        info!("Semantic embedding worker stopped (queue drained)");
    }

    /// Store the buffered chunks and release the files they belong to from
    /// `pending`. A failed write is logged, not retried: Tantivy remains the
    /// source of truth and the backfill reconciles the gap.
    async fn flush(&self, buffer: &mut Vec<ChunkRecord>, buffered_files: &mut u64) {
        if *buffered_files == 0 {
            return;
        }
        let chunks = buffer.len();
        let files = *buffered_files;
        // Empty files legitimately contribute zero chunks; `insert_chunks`
        // short-circuits, and the files still have to leave `pending`.
        match self.store.insert_chunks(std::mem::take(buffer)).await {
            Ok(()) => debug!("Stored {chunks} chunks from {files} files"),
            Err(e) => error!("Semantic indexing failed to store {chunks} chunks from {files} files: {e}"),
        }
        *buffered_files = 0;
        self.pending.fetch_sub(files, Ordering::SeqCst);
    }

    /// Chunk and embed one file. Returns its chunk records without touching the
    /// store, so the caller decides how they are written (batched or replaced).
    /// An empty file yields no records, which still clears its chunks under
    /// [`WriteMode::Replace`].
    async fn embed(&self, job: IndexJob) -> Result<Vec<ChunkRecord>> {
        let chunks = chunk_file(&job.path, &job.content, &self.chunk_options);
        if chunks.is_empty() {
            return Ok(Vec::new());
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

        debug!(
            "Embedded {} chunks for {} (file_id={})",
            records.len(),
            job.path,
            job.file_id
        );
        Ok(records)
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
            mode: WriteMode::Append,
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
        // Same file_id, new content → replaces, not appends. Make the new
        // content span enough lines to produce a *different* chunk count (2),
        // so we can deterministically wait for the re-index to land instead of
        // racing a fixed sleep (the old version asserted count==1 both before
        // and after, so a too-short sleep under load could pass spuriously or
        // catch the transient mid-upsert state).
        let new_content = (0..ChunkOptions::default().max_lines * 2 + 1)
            .map(|i| format!("let v{i} = {i};"))
            .collect::<Vec<_>>()
            .join("\n");
        j.content = new_content.clone();
        // Re-indexing a file outside a crawl/rebuild is the Replace path: the
        // worker applies it on its own and deletes the file's old chunks first.
        j.mode = WriteMode::Replace;
        indexer.index_file(j).await.unwrap();
        // The new content is deterministically multi-chunk; compute the exact
        // expected count by re-chunking it.
        use crate::services::semantic::chunker::chunk_file;
        let expected = chunk_file("src/lib.rs", &new_content, &ChunkOptions::default()).len() as u64;
        assert!(expected >= 2, "test setup should produce a multi-chunk file");
        // The re-index replaces (not appends): wait for the new count to land,
        // then assert it is exactly the new chunk count — an append bug would
        // leave 1 (old) + N (new). Polling (not a fixed sleep) avoids racing the
        // background worker under parallel test load, and waiting for >=2 never
        // catches the transient mid-upsert state.
        wait_until(|| store.count(), expected).await;
        assert_eq!(
            store.count().await.unwrap(),
            expected,
            "replaced count must equal the new content's chunk count (no duplicates)"
        );
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

    /// Provider slow enough that jobs observably sit in the queue.
    struct SlowProvider;

    impl EmbeddingProvider for SlowProvider {
        fn dimension(&self) -> usize {
            DIM
        }
        fn model_id(&self) -> &str {
            "slow-mock"
        }
        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            std::thread::sleep(std::time::Duration::from_millis(100));
            Ok(texts.iter().map(|_| vec![1.0; DIM]).collect())
        }
    }

    #[tokio::test]
    async fn test_pending_tracks_outstanding_work_and_drains_to_zero() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir).await;
        let indexer = VectorIndexer::start(Arc::new(SlowProvider), store.clone(), ChunkOptions::default(), 32, 16);

        assert_eq!(indexer.pending(), 0);
        for _ in 0..3 {
            indexer.index_file(job("fn a() {}")).await.unwrap();
        }
        // Each embed blocks 100ms, so all three jobs cannot have finished yet.
        assert!(
            indexer.pending() >= 1,
            "jobs should still be outstanding right after enqueue"
        );

        for _ in 0..100 {
            if indexer.pending() == 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert_eq!(
            indexer.pending(),
            0,
            "pending must drain to zero once the worker catches up"
        );
    }

    #[tokio::test]
    async fn test_pending_drains_even_when_jobs_fail() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir).await;
        let indexer = VectorIndexer::start(
            Arc::new(MockProvider { calls: Arc::new(AtomicUsize::new(0)), fail: true }),
            store.clone(),
            ChunkOptions::default(),
            32,
            16,
        );
        indexer.index_file(job("fn a() {}")).await.unwrap();
        indexer.index_file(job("fn b() {}")).await.unwrap();
        for _ in 0..100 {
            if indexer.pending() == 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        // pending tracks outstanding work, not successes: failures count down too.
        assert_eq!(indexer.pending(), 0);
    }

    /// Wraps a real store to count how many write calls the worker issues.
    struct CountingStore {
        inner: Arc<dyn VectorStore>,
        inserts: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl VectorStore for CountingStore {
        async fn search(
            &self,
            query_vector: &[f32],
            limit: usize,
            filters: &VectorSearchFilters,
        ) -> Result<Vec<VectorHit>> {
            self.inner.search(query_vector, limit, filters).await
        }
        async fn upsert_file_chunks(&self, file_id: Uuid, records: Vec<ChunkRecord>) -> Result<()> {
            self.inner.upsert_file_chunks(file_id, records).await
        }
        async fn insert_chunks(&self, records: Vec<ChunkRecord>) -> Result<()> {
            self.inserts.fetch_add(1, Ordering::SeqCst);
            self.inner.insert_chunks(records).await
        }
        async fn optimize(&self) -> Result<()> {
            self.inner.optimize().await
        }
        async fn delete_file(&self, file_id: Uuid) -> Result<u64> {
            self.inner.delete_file(file_id).await
        }
        async fn delete_project_chunks(&self, repository: &str) -> Result<u64> {
            self.inner.delete_project_chunks(repository).await
        }
        async fn clear(&self) -> Result<u64> {
            self.inner.clear().await
        }
        async fn count(&self) -> Result<u64> {
            self.inner.count().await
        }
        fn dimension(&self) -> usize {
            self.inner.dimension()
        }
    }

    /// The per-file LanceDB commit was what made bulk indexing collapse: every
    /// write commits a table version and a fragment. While the queue is
    /// non-empty the worker must accumulate chunks and write them once.
    #[tokio::test]
    async fn test_appends_are_batched_into_one_write() {
        let dir = tempfile::tempdir().unwrap();
        let inserts = Arc::new(AtomicUsize::new(0));
        let store: Arc<dyn VectorStore> =
            Arc::new(CountingStore { inner: store(&dir).await, inserts: inserts.clone() });
        // SlowProvider blocks 100ms per embed, so the files enqueued below all
        // sit in the queue while the first is embedded — no scheduling race.
        let indexer = VectorIndexer::start(Arc::new(SlowProvider), store.clone(), ChunkOptions::default(), 32, 16);

        const FILES: usize = 6;
        for i in 0..FILES {
            indexer.index_file(job(&format!("fn f{i}() {{}}"))).await.unwrap();
        }
        wait_until(|| store.count(), FILES as u64).await;

        let writes = inserts.load(Ordering::SeqCst);
        assert!(
            writes < FILES,
            "worker must batch: {writes} writes for {FILES} files is one commit per file"
        );
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
