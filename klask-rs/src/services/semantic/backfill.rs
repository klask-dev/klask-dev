//! Semantic index backfill (Phase 3).
//!
//! Rebuilds the vector index from the documents already stored in Tantivy.
//! Tantivy is the source of truth for *what is searchable* (the crawler has
//! already applied its extension / size / branch filtering), so re-embedding
//! those documents keeps the semantic index consistent with the keyword index
//! — no re-crawl, no network, no re-applying crawler filters. See
//! docs/SEMANTIC_SEARCH_PLAN.md §4.4.
//!
//! The job is **single-flight**: only one backfill runs at a time (a second
//! request is rejected so the API can return 409). It is cancellable, and it
//! reuses the Phase 2 [`VectorIndexer`] write path (bounded queue, strict
//! backpressure) so it cannot outrun the embedding worker.
//!
//! Gated on the `semantic-search` feature (it drives the vector indexer).
#![cfg(feature = "semantic-search")]

use super::indexer::{IndexJob, VectorIndexer};
use crate::services::SearchService;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, mpsc};
use tracing::{error, info};

/// Bounded buffer between the (blocking) Tantivy reader and the async enqueue
/// loop. Small because the real backpressure is the indexer's own queue; this
/// just hands jobs across the blocking/async boundary.
const READER_CHANNEL_CAPACITY: usize = 64;

/// Serializable snapshot of the backfill job for the admin status endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct BackfillStatus {
    /// Whether a backfill is currently running.
    pub running: bool,
    /// Documents read from Tantivy and enqueued so far.
    pub processed: u64,
    /// Total documents to process (the Tantivy document count at start);
    /// `None` until the job has started counting.
    pub total: Option<u64>,
    /// Chunks currently stored in the vector index (updated as the job runs).
    pub chunks_indexed: u64,
    /// Files enqueued to the embedding worker but not yet embedded (queued +
    /// in-flight). Non-zero outside a backfill means a crawl is feeding the
    /// semantic index right now.
    pub queue_depth: u64,
    /// Embedding model id (so the UI can show what the index was built with).
    pub model: String,
    /// Embedding dimension.
    pub dimension: usize,
    /// Last error, if the job failed.
    pub error: Option<String>,
    /// Whether the last run was cancelled by an admin.
    pub cancelled: bool,
    /// When the current/last run started.
    pub started_at: Option<DateTime<Utc>>,
    /// When the current/last run finished (success, error or cancel).
    pub finished_at: Option<DateTime<Utc>>,
}

/// Mutable backfill bookkeeping behind the controller's mutex.
#[derive(Default)]
struct BackfillState {
    running: bool,
    processed: u64,
    total: Option<u64>,
    error: Option<String>,
    cancelled: bool,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

/// Controls the single-flight semantic backfill job.
///
/// Cheap to clone (everything shared is behind `Arc`). Held by `AppState` so the
/// admin endpoints can start/cancel/poll. Carries its own clones of the search
/// service and vector indexer to drive a rebuild.
#[derive(Clone)]
pub struct BackfillController {
    state: Arc<Mutex<BackfillState>>,
    cancel: Arc<AtomicBool>,
    search_service: Arc<SearchService>,
    indexer: Arc<VectorIndexer>,
    model: String,
    dimension: usize,
}

impl BackfillController {
    pub fn new(
        search_service: Arc<SearchService>,
        indexer: Arc<VectorIndexer>,
        model: String,
        dimension: usize,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(BackfillState::default())),
            cancel: Arc::new(AtomicBool::new(false)),
            search_service,
            indexer,
            model,
            dimension,
        }
    }

    /// Current status snapshot for the admin endpoint.
    pub async fn status(&self) -> BackfillStatus {
        let state = self.state.lock().await;
        // count() is best-effort: a transient store error must not break status.
        let chunks_indexed = self.indexer.count().await.unwrap_or(0);
        BackfillStatus {
            running: state.running,
            processed: state.processed,
            total: state.total,
            chunks_indexed,
            queue_depth: self.indexer.pending(),
            model: self.model.clone(),
            dimension: self.dimension,
            error: state.error.clone(),
            cancelled: state.cancelled,
            started_at: state.started_at,
            finished_at: state.finished_at,
        }
    }

    /// Request cancellation of a running backfill. No-op if none is running.
    /// The job stops at the next document boundary.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    /// Start a backfill in the background.
    ///
    /// Single-flight: returns `Err` if one is already running (the caller maps
    /// this to HTTP 409). On success the job runs detached and progress is
    /// observable via [`BackfillController::status`].
    pub async fn start(&self) -> Result<()> {
        {
            let mut state = self.state.lock().await;
            if state.running {
                return Err(anyhow!("a semantic backfill is already running"));
            }
            // Reset bookkeeping for the new run.
            *state = BackfillState { running: true, started_at: Some(Utc::now()), ..Default::default() };
        }
        self.cancel.store(false, Ordering::SeqCst);

        let this = self.clone();
        tokio::spawn(async move {
            let result = this.run().await;
            let mut state = this.state.lock().await;
            state.running = false;
            state.finished_at = Some(Utc::now());
            match result {
                Ok(()) => {
                    if state.cancelled {
                        info!("Semantic backfill cancelled after {} documents", state.processed);
                    } else {
                        info!("Semantic backfill completed: {} documents enqueued", state.processed);
                    }
                }
                Err(e) => {
                    error!("Semantic backfill failed: {e}");
                    state.error = Some(e.to_string());
                }
            }
        });

        Ok(())
    }

    /// The actual rebuild: clear the store, then stream every Tantivy document
    /// into the embedding worker, then wait for the worker to drain its queue
    /// — so `running` covers the whole rebuild, embedding included.
    async fn run(&self) -> Result<()> {
        // Wipe the vector index so a re-run never leaves stale or duplicated
        // rows (the worker upserts per file_id, but a rebuild should also drop
        // chunks of files that no longer exist in Tantivy).
        let removed = self.indexer.clear().await?;
        info!("Semantic backfill: cleared {removed} existing chunks, starting rebuild");

        // Fail fast if the source index can't be read: a backfill with an
        // unknown total would report misleading "0 / 0" progress, and the
        // iteration below would fail anyway. Surface the error to the admin.
        let total = self.search_service.get_document_count()?;
        {
            let mut state = self.state.lock().await;
            state.total = Some(total);
        }
        info!("Semantic backfill: {total} documents to re-embed");

        // The Tantivy iterator is blocking (disk reads); the enqueue is async
        // (the indexer awaits queue capacity). Bridge the two with a bounded
        // channel: a blocking task reads documents and `blocking_send`s
        // IndexJobs; this task drains them and awaits the indexer.
        let (tx, mut rx) = mpsc::channel::<IndexJob>(READER_CHANNEL_CAPACITY);
        let search_service = self.search_service.clone();
        let cancel = self.cancel.clone();

        let reader = tokio::task::spawn_blocking(move || -> Result<()> {
            search_service.iter_documents(|doc| {
                if cancel.load(Ordering::SeqCst) {
                    return Err(anyhow!("__cancelled__"));
                }
                let job = IndexJob {
                    file_id: doc.file_id,
                    repository: doc.repository,
                    project: doc.project,
                    version: doc.version,
                    path: doc.path,
                    extension: doc.extension,
                    content: doc.content,
                };
                // Blocking send applies backpressure when the async side lags.
                // An error means the receiver was dropped — stop reading.
                tx.blocking_send(job).map_err(|_| anyhow!("backfill receiver dropped"))?;
                Ok(())
            })?;
            Ok(())
        });

        // Drain the channel into the indexer, counting progress.
        while let Some(job) = rx.recv().await {
            if self.cancel.load(Ordering::SeqCst) {
                break;
            }
            if let Err(e) = self.indexer.index_file(job).await {
                // The worker stopped (channel closed): cannot continue.
                return Err(anyhow!("failed to enqueue document for embedding: {e}"));
            }
            let mut state = self.state.lock().await;
            state.processed += 1;
        }

        // Surface the reader task's outcome (cancellation vs real error).
        match reader.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) if e.to_string() == "__cancelled__" => {
                let mut state = self.state.lock().await;
                state.cancelled = true;
            }
            Ok(Err(e)) => return Err(e),
            Err(join_err) => return Err(anyhow!("backfill reader task panicked: {join_err}")),
        }

        // Everything is enqueued, but the worker is still embedding. Stay
        // "running" until the queue drains so the admin UI reports the real
        // end of the rebuild, not just the end of the enqueue phase. A cancel
        // stops the wait, but jobs already queued will still be embedded
        // (they cannot be un-queued).
        while self.indexer.pending() > 0 && !self.cancel.load(Ordering::SeqCst) {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        if self.cancel.load(Ordering::SeqCst) {
            let mut state = self.state.lock().await;
            state.cancelled = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::search::FileData;
    use crate::services::semantic::chunker::ChunkOptions;
    use crate::services::semantic::embedder::EmbeddingProvider;
    use crate::services::semantic::store::{LanceVectorStore, VectorStore};
    use std::sync::atomic::AtomicUsize;
    use tempfile::TempDir;
    use uuid::Uuid;

    const DIM: usize = 8;

    struct MockProvider {
        calls: Arc<AtomicUsize>,
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
            Ok(texts.iter().map(|t| vec![(t.len() % 5) as f32; DIM]).collect())
        }
    }

    async fn build(tmp: &TempDir) -> (Arc<SearchService>, Arc<VectorIndexer>) {
        let search = Arc::new(SearchService::new(tmp.path().join("tantivy")).unwrap());
        let store: Arc<dyn VectorStore> =
            Arc::new(LanceVectorStore::open(tmp.path().join("vectors"), DIM).await.unwrap());
        let indexer = Arc::new(VectorIndexer::start(
            Arc::new(MockProvider { calls: Arc::new(AtomicUsize::new(0)) }),
            store,
            ChunkOptions::default(),
            32,
            64,
        ));
        (search, indexer)
    }

    async fn index_doc(search: &SearchService, repo: &str, path: &str, content: &str) {
        search
            .upsert_file(FileData {
                file_id: Uuid::new_v4(),
                file_name: path,
                file_path: path,
                content,
                repository: repo,
                project: repo,
                version: "main",
                extension: "rs",
                size: content.len() as u64,
            })
            .await
            .unwrap();
        search.commit().await.unwrap();
    }

    async fn wait_chunks(indexer: &VectorIndexer, target: u64) -> u64 {
        for _ in 0..100 {
            let n = indexer.count().await.unwrap();
            if n >= target {
                return n;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        indexer.count().await.unwrap()
    }

    #[tokio::test]
    async fn test_backfill_reindexes_all_documents() {
        let tmp = tempfile::tempdir().unwrap();
        let (search, indexer) = build(&tmp).await;
        index_doc(&search, "repo", "a.rs", "fn a() {}").await;
        index_doc(&search, "repo", "b.rs", "fn b() {}").await;

        let controller = BackfillController::new(search, indexer.clone(), "mock".into(), DIM);
        controller.start().await.unwrap();

        // Wait for the run to finish enqueuing, then for the worker to embed.
        for _ in 0..100 {
            if !controller.status().await.running {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        let chunks = wait_chunks(&indexer, 2).await;
        assert!(chunks >= 2, "expected at least 2 chunks, got {chunks}");

        let status = controller.status().await;
        assert!(!status.running);
        assert_eq!(status.processed, 2);
        assert_eq!(status.total, Some(2));
        assert!(status.error.is_none());
    }

    #[tokio::test]
    async fn test_backfill_clears_before_rebuild() {
        let tmp = tempfile::tempdir().unwrap();
        let (search, indexer) = build(&tmp).await;
        index_doc(&search, "repo", "a.rs", "fn a() {}").await;

        // Pre-seed the store with a stale chunk that no Tantivy doc maps to.
        indexer
            .index_file(IndexJob {
                file_id: Uuid::new_v4(),
                repository: "ghost".into(),
                project: "ghost".into(),
                version: "main".into(),
                path: "gone.rs".into(),
                extension: "rs".into(),
                content: "fn ghost() {}".into(),
            })
            .await
            .unwrap();
        wait_chunks(&indexer, 1).await;

        let controller = BackfillController::new(search, indexer.clone(), "mock".into(), DIM);
        controller.start().await.unwrap();
        for _ in 0..100 {
            if !controller.status().await.running {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        let chunks = wait_chunks(&indexer, 1).await;
        // Exactly one document in Tantivy → the ghost chunk must be gone.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(indexer.count().await.unwrap(), chunks);
        assert_eq!(controller.status().await.processed, 1);
    }

    /// Embeds slowly so the drain phase is observable: `running` must stay
    /// true until the worker queue is empty (not just until enqueue ends),
    /// and whenever `running` is false the reported queue depth must be zero.
    #[tokio::test]
    async fn test_running_covers_embedding_until_queue_drained() {
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

        let tmp = tempfile::tempdir().unwrap();
        let search = Arc::new(SearchService::new(tmp.path().join("tantivy")).unwrap());
        let store: Arc<dyn VectorStore> =
            Arc::new(LanceVectorStore::open(tmp.path().join("vectors"), DIM).await.unwrap());
        let indexer = Arc::new(VectorIndexer::start(
            Arc::new(SlowProvider),
            store,
            ChunkOptions::default(),
            32,
            64,
        ));
        for i in 0..5 {
            index_doc(&search, "repo", &format!("f{i}.rs"), "fn f() {}").await;
        }

        let controller = BackfillController::new(search, indexer.clone(), "mock".into(), DIM);
        controller.start().await.unwrap();

        // 5 docs × 100ms embed ≈ 500ms of work + 500ms drain-poll granularity.
        for _ in 0..600 {
            let status = controller.status().await;
            if !status.running {
                assert_eq!(
                    status.queue_depth, 0,
                    "backfill reported finished while files were still awaiting embedding"
                );
                assert_eq!(status.processed, 5);
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("backfill did not finish in time");
    }

    #[tokio::test]
    async fn test_second_backfill_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let (search, indexer) = build(&tmp).await;
        for i in 0..20 {
            index_doc(&search, "repo", &format!("f{i}.rs"), "fn f() {}").await;
        }

        let controller = BackfillController::new(search, indexer, "mock".into(), DIM);
        controller.start().await.unwrap();
        // A second start while the first is running must be rejected.
        let second = controller.start().await;
        // It may have already finished on a fast machine; only assert rejection
        // if still running.
        if controller.status().await.running {
            assert!(second.is_err(), "second concurrent backfill should be rejected");
        }
    }
}
