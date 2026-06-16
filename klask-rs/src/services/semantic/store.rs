//! Persistent vector store for semantic search (LanceDB).
//!
//! Stores one row per code chunk: the embedding vector plus the metadata
//! columns needed to filter (mirroring Tantivy facets) and to render a snippet
//! (`path`, `start_line`, `end_line`). The store is embedded — it persists to a
//! local directory next to the Tantivy index, so Klask stays a single binary
//! plus volumes (see docs/SEMANTIC_SEARCH_PLAN.md §3.2).
//!
//! The whole module is gated on the `semantic-search` feature; without it the
//! binary pulls neither LanceDB nor Arrow. The query path (ANN search, RRF
//! wiring) lands in plan phase 4 — phase 2 is the write/lifecycle path only,
//! so no ANN index is created here yet.
#![cfg(feature = "semantic-search")]

use anyhow::Result;
use uuid::Uuid;

/// One embedded code chunk ready to persist.
///
/// Field semantics mirror `search::FileData` so the two indexes describe the
/// same universe and filters behave identically across engines.
#[derive(Debug, Clone)]
pub struct ChunkRecord {
    pub file_id: Uuid,
    /// Parent repository (mass-deletion + facet key), mirrors Tantivy `repository`.
    pub repository: String,
    /// Individual project name (facet), mirrors Tantivy `project`.
    pub project: String,
    /// Branch/version (facet).
    pub version: String,
    /// File path relative to the repo root (for snippet display).
    pub path: String,
    /// File extension (facet).
    pub extension: String,
    /// First line of the chunk, 1-based inclusive.
    pub start_line: u32,
    /// Last line of the chunk, 1-based inclusive.
    pub end_line: u32,
    /// Embedding vector; length must equal the store's configured dimension.
    pub vector: Vec<f32>,
}

/// Persistent vector store abstraction.
///
/// A trait (like [`super::EmbeddingProvider`]) so the indexer and future query
/// path depend on behaviour, not on LanceDB directly — which keeps tests able
/// to swap in an in-memory fake and lets the backend change later.
#[async_trait::async_trait]
pub trait VectorStore: Send + Sync {
    /// Replace all chunks of `file_id` with `records` (delete-then-insert),
    /// mirroring Tantivy's `upsert_file`. `records` may be empty (just deletes).
    /// Every record's `vector` must have length [`VectorStore::dimension`].
    async fn upsert_file_chunks(&self, file_id: Uuid, records: Vec<ChunkRecord>) -> Result<()>;

    /// Delete all chunks of a single file. Returns the number of rows removed.
    /// Part of the lifecycle API for per-file removal (a file deleted from a
    /// repo between crawls); the incremental-delete wiring lands in a later
    /// phase, so it is currently exercised only by tests.
    #[allow(dead_code)]
    async fn delete_file(&self, file_id: Uuid) -> Result<u64>;

    /// Delete all chunks of a repository (mirrors `delete_project_documents`).
    /// Returns the number of rows removed.
    async fn delete_project_chunks(&self, repository: &str) -> Result<u64>;

    /// Delete every chunk in the store. Returns the number of rows removed.
    /// Used by the semantic backfill (Phase 3) to start a full rebuild from a
    /// clean slate so re-running it never leaves stale or duplicated rows.
    async fn clear(&self) -> Result<u64>;

    /// Total number of stored chunks (for the admin index card, phases 3/5).
    async fn count(&self) -> Result<u64>;

    /// Embedding dimension the store was opened with. Used by tests and the
    /// query path (phase 4); kept on the trait for backend swappability.
    #[allow(dead_code)]
    fn dimension(&self) -> usize;
}

/// Escape a string for safe inclusion as a single-quoted SQL literal in a
/// LanceDB filter predicate.
///
/// LanceDB's `delete` takes a raw SQL predicate string, so user-controlled
/// values (repository names) must be escaped to avoid predicate injection.
/// SQL escapes a single quote by doubling it.
pub(crate) fn sql_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub use lance_store::LanceVectorStore;

mod lance_store {
    use super::{ChunkRecord, VectorStore, sql_quote};
    use anyhow::{Context, Result, anyhow};
    use arrow_array::{
        Array, FixedSizeListArray, RecordBatch, StringArray, UInt32Array,
        builder::{FixedSizeListBuilder, Float32Builder},
    };
    use arrow_schema::{DataType, Field, Schema, SchemaRef};
    use lancedb::{Connection, Table, connect};
    use std::path::Path;
    use std::sync::Arc;
    use uuid::Uuid;

    const TABLE_NAME: &str = "chunks";

    /// LanceDB-backed [`VectorStore`].
    pub struct LanceVectorStore {
        #[allow(dead_code)] // kept alive for the table; future phases reopen tables
        connection: Connection,
        table: Table,
        schema: SchemaRef,
        dimension: usize,
    }

    /// Build the Arrow schema for the chunks table at a given vector dimension.
    fn chunks_schema(dimension: usize) -> SchemaRef {
        let vector_field = Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), dimension as i32),
            true,
        );
        Arc::new(Schema::new(vec![
            Field::new("chunk_id", DataType::Utf8, false),
            Field::new("file_id", DataType::Utf8, false),
            Field::new("repository", DataType::Utf8, false),
            Field::new("project", DataType::Utf8, false),
            Field::new("version", DataType::Utf8, false),
            Field::new("path", DataType::Utf8, false),
            Field::new("extension", DataType::Utf8, false),
            Field::new("start_line", DataType::UInt32, false),
            Field::new("end_line", DataType::UInt32, false),
            vector_field,
        ]))
    }

    /// Extract the vector dimension from an existing table's schema, if present.
    fn schema_dimension(schema: &Schema) -> Option<usize> {
        schema.field_with_name("vector").ok().and_then(|f| match f.data_type() {
            DataType::FixedSizeList(_, len) => Some(*len as usize),
            _ => None,
        })
    }

    impl LanceVectorStore {
        /// Open (creating on first use) the chunks table under `path`.
        ///
        /// If a table already exists with a different vector dimension than
        /// `dimension` (e.g. the embedding model changed), opening fails with a
        /// clear error rather than silently corrupting the index — the operator
        /// must rebuild it. This mirrors the strict model resolution in
        /// `embedder::resolve_model`.
        pub async fn open(path: impl AsRef<Path>, dimension: usize) -> Result<Self> {
            if dimension == 0 {
                return Err(anyhow!("Vector store dimension must be non-zero"));
            }
            let uri = path.as_ref().to_string_lossy().to_string();
            let connection =
                connect(&uri).execute().await.with_context(|| format!("Failed to open LanceDB at '{uri}'"))?;

            let schema = chunks_schema(dimension);

            let existing = connection.table_names().execute().await.context("Failed to list LanceDB tables")?;

            let table = if existing.iter().any(|n| n == TABLE_NAME) {
                let table = connection.open_table(TABLE_NAME).execute().await.context("Failed to open chunks table")?;
                let existing_schema = table.schema().await.context("Failed to read chunks table schema")?;
                match schema_dimension(&existing_schema) {
                    Some(existing_dim) if existing_dim != dimension => {
                        return Err(anyhow!(
                            "Vector store at '{uri}' was built with dimension {existing_dim}, but the configured \
                             embedding model produces dimension {dimension}. The embedding model changed; rebuild \
                             the semantic index (delete '{uri}') before continuing."
                        ));
                    }
                    Some(_) => table,
                    None => {
                        return Err(anyhow!(
                            "Existing LanceDB table '{TABLE_NAME}' at '{uri}' has no recognizable vector column"
                        ));
                    }
                }
            } else {
                connection
                    .create_empty_table(TABLE_NAME, schema.clone())
                    .execute()
                    .await
                    .context("Failed to create chunks table")?
            };

            Ok(Self { connection, table, schema, dimension })
        }

        /// Convert chunk records into a single Arrow `RecordBatch`.
        fn records_to_batch(&self, records: &[ChunkRecord]) -> Result<RecordBatch> {
            let mut chunk_id = Vec::with_capacity(records.len());
            let mut file_id = Vec::with_capacity(records.len());
            let mut repository = Vec::with_capacity(records.len());
            let mut project = Vec::with_capacity(records.len());
            let mut version = Vec::with_capacity(records.len());
            let mut path = Vec::with_capacity(records.len());
            let mut extension = Vec::with_capacity(records.len());
            let mut start_line = Vec::with_capacity(records.len());
            let mut end_line = Vec::with_capacity(records.len());

            let mut vector_builder = FixedSizeListBuilder::new(
                Float32Builder::with_capacity(records.len() * self.dimension),
                self.dimension as i32,
            );

            for record in records {
                if record.vector.len() != self.dimension {
                    return Err(anyhow!(
                        "Chunk vector length {} does not match store dimension {}",
                        record.vector.len(),
                        self.dimension
                    ));
                }
                // chunk_id is deterministic so a re-index of the same file produces the
                // same row identity; file_id delete-then-insert keeps it dedup-correct.
                chunk_id.push(format!("{}:{}", record.file_id, record.start_line));
                file_id.push(record.file_id.to_string());
                repository.push(record.repository.clone());
                project.push(record.project.clone());
                version.push(record.version.clone());
                path.push(record.path.clone());
                extension.push(record.extension.clone());
                start_line.push(record.start_line);
                end_line.push(record.end_line);

                vector_builder.values().append_slice(&record.vector);
                vector_builder.append(true);
            }

            let vector: FixedSizeListArray = vector_builder.finish();

            RecordBatch::try_new(
                self.schema.clone(),
                vec![
                    Arc::new(StringArray::from(chunk_id)),
                    Arc::new(StringArray::from(file_id)),
                    Arc::new(StringArray::from(repository)),
                    Arc::new(StringArray::from(project)),
                    Arc::new(StringArray::from(version)),
                    Arc::new(StringArray::from(path)),
                    Arc::new(StringArray::from(extension)),
                    Arc::new(UInt32Array::from(start_line)),
                    Arc::new(UInt32Array::from(end_line)),
                    Arc::new(vector) as Arc<dyn Array>,
                ],
            )
            .context("Failed to build Arrow RecordBatch for chunks")
        }

        async fn delete_where(&self, predicate: &str) -> Result<u64> {
            let result =
                self.table.delete(predicate).await.with_context(|| format!("LanceDB delete failed: {predicate}"))?;
            Ok(result.num_deleted_rows)
        }
    }

    #[async_trait::async_trait]
    impl VectorStore for LanceVectorStore {
        async fn upsert_file_chunks(&self, file_id: Uuid, records: Vec<ChunkRecord>) -> Result<()> {
            // Delete-then-insert: remove any prior chunks of this file first so a
            // re-index never leaves stale rows behind (mirrors Tantivy upsert_file).
            self.delete_where(&format!("file_id = {}", sql_quote(&file_id.to_string()))).await?;

            if records.is_empty() {
                return Ok(());
            }
            let batch = self.records_to_batch(&records)?;
            self.table.add(batch).execute().await.context("Failed to add chunk vectors to LanceDB")?;
            Ok(())
        }

        async fn delete_file(&self, file_id: Uuid) -> Result<u64> {
            self.delete_where(&format!("file_id = {}", sql_quote(&file_id.to_string()))).await
        }

        async fn delete_project_chunks(&self, repository: &str) -> Result<u64> {
            self.delete_where(&format!("repository = {}", sql_quote(repository))).await
        }

        async fn clear(&self) -> Result<u64> {
            // `true` is a constant predicate matching every row — no
            // user-controlled input, so no injection surface.
            self.delete_where("true").await
        }

        async fn count(&self) -> Result<u64> {
            let n = self.table.count_rows(None).await.context("Failed to count LanceDB rows")?;
            Ok(n as u64)
        }

        fn dimension(&self) -> usize {
            self.dimension
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn record(file_id: Uuid, repository: &str, start: u32, dim: usize) -> ChunkRecord {
            ChunkRecord {
                file_id,
                repository: repository.to_string(),
                project: repository.to_string(),
                version: "main".to_string(),
                path: "src/lib.rs".to_string(),
                extension: "rs".to_string(),
                start_line: start,
                end_line: start + 5,
                vector: vec![0.1_f32; dim],
            }
        }

        async fn temp_store(dim: usize) -> (tempfile::TempDir, LanceVectorStore) {
            let dir = tempfile::tempdir().unwrap();
            let store = LanceVectorStore::open(dir.path(), dim).await.unwrap();
            (dir, store)
        }

        #[tokio::test]
        async fn test_open_creates_empty_table() {
            let (_dir, store) = temp_store(8).await;
            assert_eq!(store.dimension(), 8);
            assert_eq!(store.count().await.unwrap(), 0);
        }

        #[tokio::test]
        async fn test_upsert_and_count() {
            let (_dir, store) = temp_store(8).await;
            let fid = Uuid::new_v4();
            store
                .upsert_file_chunks(fid, vec![record(fid, "repo-a", 1, 8), record(fid, "repo-a", 7, 8)])
                .await
                .unwrap();
            assert_eq!(store.count().await.unwrap(), 2);
        }

        #[tokio::test]
        async fn test_upsert_same_file_replaces_no_duplicates() {
            let (_dir, store) = temp_store(8).await;
            let fid = Uuid::new_v4();
            store
                .upsert_file_chunks(fid, vec![record(fid, "repo-a", 1, 8), record(fid, "repo-a", 7, 8)])
                .await
                .unwrap();
            // Re-index the same file with a single chunk: old rows must be gone.
            store.upsert_file_chunks(fid, vec![record(fid, "repo-a", 1, 8)]).await.unwrap();
            assert_eq!(store.count().await.unwrap(), 1);
        }

        #[tokio::test]
        async fn test_upsert_empty_only_deletes() {
            let (_dir, store) = temp_store(8).await;
            let fid = Uuid::new_v4();
            store.upsert_file_chunks(fid, vec![record(fid, "repo-a", 1, 8)]).await.unwrap();
            store.upsert_file_chunks(fid, vec![]).await.unwrap();
            assert_eq!(store.count().await.unwrap(), 0);
        }

        #[tokio::test]
        async fn test_delete_file_only_removes_that_file() {
            let (_dir, store) = temp_store(8).await;
            let a = Uuid::new_v4();
            let b = Uuid::new_v4();
            store.upsert_file_chunks(a, vec![record(a, "repo-a", 1, 8)]).await.unwrap();
            store.upsert_file_chunks(b, vec![record(b, "repo-a", 1, 8)]).await.unwrap();
            assert_eq!(store.delete_file(a).await.unwrap(), 1);
            assert_eq!(store.count().await.unwrap(), 1);
        }

        #[tokio::test]
        async fn test_delete_project_only_removes_that_repo() {
            let (_dir, store) = temp_store(8).await;
            let a = Uuid::new_v4();
            let b = Uuid::new_v4();
            store.upsert_file_chunks(a, vec![record(a, "repo-a", 1, 8)]).await.unwrap();
            store.upsert_file_chunks(b, vec![record(b, "repo-b", 1, 8)]).await.unwrap();
            assert_eq!(store.delete_project_chunks("repo-a").await.unwrap(), 1);
            assert_eq!(store.count().await.unwrap(), 1);
        }

        #[tokio::test]
        async fn test_clear_removes_everything() {
            let (_dir, store) = temp_store(8).await;
            let a = Uuid::new_v4();
            let b = Uuid::new_v4();
            store.upsert_file_chunks(a, vec![record(a, "repo-a", 1, 8)]).await.unwrap();
            store.upsert_file_chunks(b, vec![record(b, "repo-b", 1, 8)]).await.unwrap();
            assert_eq!(store.clear().await.unwrap(), 2);
            assert_eq!(store.count().await.unwrap(), 0);
            // Clearing an empty store removes nothing and does not error.
            assert_eq!(store.clear().await.unwrap(), 0);
        }

        #[tokio::test]
        async fn test_dimension_mismatch_on_reopen_errors() {
            let dir = tempfile::tempdir().unwrap();
            {
                let store = LanceVectorStore::open(dir.path(), 8).await.unwrap();
                let fid = Uuid::new_v4();
                store.upsert_file_chunks(fid, vec![record(fid, "repo-a", 1, 8)]).await.unwrap();
            }
            // `unwrap_err` would require the Ok type to be `Debug`, which the
            // LanceDB-backed store is not; match the error out explicitly.
            let err = match LanceVectorStore::open(dir.path(), 16).await {
                Ok(_) => panic!("opening with a mismatched dimension must fail"),
                Err(e) => e.to_string(),
            };
            assert!(
                err.contains("dimension 8"),
                "error should mention the existing dimension: {err}"
            );
        }

        #[tokio::test]
        async fn test_wrong_vector_length_rejected() {
            let (_dir, store) = temp_store(8).await;
            let fid = Uuid::new_v4();
            let mut rec = record(fid, "repo-a", 1, 8);
            rec.vector = vec![0.0; 4]; // wrong length
            assert!(store.upsert_file_chunks(fid, vec![rec]).await.is_err());
        }

        #[tokio::test]
        async fn test_repository_name_with_quote_is_safe() {
            // A repository name containing a single quote must not break the
            // delete predicate (injection guard).
            let (_dir, store) = temp_store(8).await;
            let fid = Uuid::new_v4();
            let nasty = "repo' OR '1'='1";
            store.upsert_file_chunks(fid, vec![record(fid, nasty, 1, 8)]).await.unwrap();
            // Deleting a different repo must remove nothing despite the quote.
            assert_eq!(store.delete_project_chunks("other").await.unwrap(), 0);
            assert_eq!(store.count().await.unwrap(), 1);
            // Deleting the real (quoted) name removes exactly its row.
            assert_eq!(store.delete_project_chunks(nasty).await.unwrap(), 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_quote_escapes_single_quotes() {
        assert_eq!(sql_quote("repo"), "'repo'");
        assert_eq!(sql_quote("a'b"), "'a''b'");
        assert_eq!(sql_quote("' OR '1'='1"), "''' OR ''1''=''1'");
    }
}
