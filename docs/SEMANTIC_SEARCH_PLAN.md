# 🧠 Hybrid Semantic Search Plan — Natural-Language Code Search

## 1. Goal

Let users (and AI agents via MCP) search by **meaning**, not just keywords:

> *"where do we validate JWT tokens?"* → finds `extract_authenticated_user()`,
> even though neither "validate" nor "JWT token" appears literally.

This is a **hybrid** system: Tantivy BM25 keyword search (existing strength) **+**
vector similarity search over code-chunk embeddings, fused into one ranked result list.
Keyword search stays the default; semantic is an opt-in mode that makes Klask
qualitatively different from grep-style competitors (Hound, OpenGrok).

**Hard constraint: stays self-hosted.** Embeddings are computed locally with an ONNX
model — no cloud API, no API keys, no code leaving the infra.

---

## 2. Architecture Overview

```
                       ┌─────────────────────────────────────────┐
 crawler (existing)    │  file content                           │
 ──────────────────────┤                                         │
                       ▼                                         ▼
                Tantivy index (existing)              Chunker (tree-sitter)
                       │                                         │
                       │                              EmbeddingService (fastembed/ONNX)
                       │                                         │
                       │                              Vector index (LanceDB, embedded)
                       │                                         │
                       ▼                                         ▼
                 BM25 top-k ────────► RRF fusion ◄──── ANN top-k (cosine)
                                          │
                                          ▼
                                   ranked results
```

## 3. Technical Decisions

### 3.1 Embedding runtime: `fastembed` (ONNX Runtime, pure local)

- Rust crate, batch inference on CPU, no Python, no network at query time.
- Model download happens once at startup (cacheable in the PVC / baked into the Docker
  image for air-gapped deployments).
- **Model choice** (benchmark in Phase 1 on our own eval set):
  - `jina-embeddings-v2-base-code` — code-specialized, 768 dims, ~160M params (first pick);
  - `BAAI/bge-small-en-v1.5` — fallback, 384 dims, much faster/smaller if latency or
    index size is a problem.
- Wrapped in an `EmbeddingService` trait so the model (and dims) is swappable via config.

### 3.2 Vector store: LanceDB (embedded)

- Embedded columnar vector DB, Rust-native, persists to local disk next to the Tantivy
  index — **keeps Klask a single binary + volumes**, no new infra service.
- Alternatives considered:
  - *pgvector*: reuses PostgreSQL, but ANN performance degrades at tens of millions of
    chunks and bloats the relational DB;
  - *Qdrant*: excellent but adds a service to deploy/operate — against Klask's
    "drop-in" value proposition.
- Schema: `chunk(id, file_id, project, version, path, extension, start_line, end_line,
  kind, vector)` — metadata columns mirror Tantivy facets so filters work in both
  engines.

### 3.3 Chunking: tree-sitter with line-window fallback

- Parse with tree-sitter grammars (start with: Rust, TypeScript/JS, Java, Python, Go);
  one chunk per **function / method / class**, prefixed with a context line
  (`// repo > path > parent symbol`) which measurably improves code embeddings.
- Unsupported languages / huge functions → fixed window of ~60 lines with 15-line
  overlap.
- Chunks > model context → split, same overlap rule.

### 3.4 Fusion: Reciprocal Rank Fusion (RRF)

- Run BM25 and ANN in parallel, fuse with `score = Σ 1/(60 + rank_i)`.
- Rank-based (no score normalization problem between BM25 and cosine), proven default
  in hybrid search literature, one tunable constant.
- Search modes exposed: `keyword` (default, unchanged), `semantic` (ANN only),
  `hybrid` (RRF).

---

## 4. Indexing Pipeline Changes

1. **Hook point**: the crawler's file-processing path (where `upsert_file` is called)
   additionally pushes `(file_id, content, metadata)` to an **embedding queue**
   (bounded `tokio::sync::mpsc`).
2. A dedicated **embedding worker** batches chunks (e.g. 32/batch), embeds, writes to
   LanceDB. Decoupled so crawl speed is unaffected; queue backpressure degrades to
   "semantic index lags behind" rather than slowing the crawl.
3. **Deletions/updates**: same lifecycle as Tantivy — delete chunks by `file_id` on
   upsert, by `project` on repository deletion (mirror `delete_project_documents`).
4. **Backfill job**: admin endpoint + button ("Build semantic index") iterating the
   existing Tantivy docs, with progress reporting via the existing `ProgressTracker`.
5. Feature-flagged: `semantic_search.enabled = false` by default in config; when
   disabled, zero overhead and no model download.

## 5. Query Path Changes

- `SearchQuery` gains `mode: SearchMode` (`Keyword | Semantic | Hybrid`).
- API: `GET /api/search?mode=hybrid&...` — backward compatible (absent = `keyword`).
- Facet filters (project/version/extension/size) are applied as LanceDB metadata
  predicates so both engines see the same filtered universe.
- Snippet for semantic hits = the chunk's line range (we know `start_line..end_line`),
  highlighted client-side as today.

## 6. Frontend Changes

- Search mode toggle in `SearchPageV3` (`Keyword | Hybrid | Semantic`), persisted in
  the existing search-state store; tooltip explaining the modes.
- Badge on results indicating the match origin in hybrid mode (keyword / semantic /
  both) — cheap and great for trust/debugging.
- Admin dashboard: semantic index card (chunk count, size, model name, backfill
  progress, rebuild button).

## 7. MCP Synergy

Once this lands, the MCP `search_code` tool gains a `mode` parameter (default
`hybrid` for agents — they ask natural-language questions). This combination
(agents + semantic cross-repo search) is the end-state killer feature; see
[MCP_SERVER_PLAN.md](MCP_SERVER_PLAN.md).

## 8. Rollout Phases

| Phase | Content | Status |
|---|---|---|
| **1** | `EmbeddingProvider` (fastembed behind the `semantic-search` cargo feature) + chunker + RRF fusion utility + unit tests + model benchmark; config/startup plumbing | ✅ done (PR #120) |
| **2** | LanceDB store + embedding worker + crawl integration + delete/update lifecycle | ✅ done (this PR) |
| **3** | Backfill admin job + progress UI | ✅ done (this PR) |
| **4** | Query path: `mode` param, RRF fusion wiring, API + tests | ✅ done (this PR) |
| **5** | Frontend toggle + result badges + admin card | planned |
| **6** | MCP `mode` param; eval pass (latency P95, recall@10 vs keyword) and tuning | planned |

**Phase 1 measurements** (debug build, CPU, `Xenova/bge-small-en-v1.5`, 384 dims):
embedding throughput ≈ 7.6 chunks/s on ~6-line-function chunks; semantically
related code snippets score cosine ≈ 0.82 vs ≈ 0.35–0.45 for unrelated ones.
Reproduce with:
`cargo test --features semantic-search --test semantic_embedding_test -- --ignored --nocapture`

**Phase 2 notes:**
- Vector store is **LanceDB** (`lancedb` 0.30, embedded), table `chunks` with the
  metadata columns from §3.2 + a `FixedSizeList<Float32, dim>` vector column.
  Persists under `SEMANTIC_SEARCH_VECTOR_DIR` (default `./vector-index`).
- The embedding worker is a single `tokio` task fed by a **bounded** queue;
  **the crawl blocks when the queue is full** (strict backpressure — chunks are
  never silently dropped, keeping the vector index consistent with the crawl).
- Lifecycle mirrors Tantivy: delete-then-insert per `file_id` on upsert,
  delete-by-`repository` on re-crawl and repository deletion. Re-opening the
  store with a different embedding dimension (model change) is refused with a
  clear error to prevent silent corruption.
- **Build dependency:** lancedb→lance pulls `prost`, which needs `protoc`
  (Protocol Buffers compiler) at build time. Building with
  `--features semantic-search` requires `protobuf-compiler` installed; this must
  be added to the Dockerfile / CI when the feature is enabled in deployment.
- Verify the full write path against a real model + real LanceDB index with:
  `cargo test --features semantic-search --test semantic_indexing_test -- --ignored --nocapture`

**Phase 3 notes:**
- **Backfill source is Tantivy, not the git clones.** `SearchService::iter_documents`
  streams every live stored document (content is `STORED`) back into the Phase 2
  `VectorIndexer`. This is the source of truth for *what is searchable* (the
  crawler already applied its extension/size/branch filtering), so the rebuilt
  vector index stays consistent with the keyword index — and it needs no
  re-crawl, no network, and survives pod restarts (unlike the ephemeral
  `CRAWLER_TEMP_DIR` clones).
- **Single-flight + cancellable.** `BackfillController` runs one rebuild at a
  time; a concurrent request is rejected so the API returns **409 Conflict**.
  The job clears the vector store first (so a rebuild drops chunks of files that
  no longer exist), then streams documents through the bounded indexer queue
  (strict backpressure — the backfill can't outrun the embedding worker). A
  blocking Tantivy reader bridges to the async enqueue loop via a small bounded
  channel; cancellation stops at the next document boundary.
- **Admin API (admin-only):** `POST /api/admin/semantic/backfill` (202 / 409 /
  503-when-disabled), `GET /api/admin/semantic/status`
  (`{enabled, running, processed, total, chunks_indexed, model, dimension,
  error, cancelled, started_at, finished_at}`), `POST /api/admin/semantic/cancel`.
  All compile in both feature modes; without the feature they report
  `enabled: false` / 503.
- **UI:** a "Semantic Index" card on the admin Index Management page shows the
  model/dimension and chunk count, with a Build/Rebuild button and a
  poll-driven progress bar (polls `status` every ~1.5 s while running). The card
  renders nothing when semantic search is disabled on the server.

**Phase 4 notes:**
- **`mode` param, backward compatible.** `GET /api/search?mode=keyword|semantic|hybrid`;
  absent ⇒ `keyword`, so existing clients are unchanged. `SearchMode` lives on
  `SearchQuery`; the keyword path (`SearchService::search`) is untouched.
- **Degrade, never break.** When `semantic`/`hybrid` is requested but the
  backend is unavailable (feature off, `SEMANTIC_SEARCH_ENABLED=false`, or the
  model failed to load) the API silently falls back to keyword search. The
  decision is centralized in the API layer (`run_search`); the semantic query
  module is only reached when the backend is present.
- **Vector search.** `VectorStore::search(query_vec, k, filters)` does cosine KNN
  over LanceDB (`vector_search().distance_type(Cosine).only_if(predicate)`).
  Facet filters (repo/project/version/extension) are applied as escaped `IN(...)`
  predicates so both engines see the same universe (same `sql_quote` injection
  guard as the delete path). **Brute-force KNN** for now — an IVF_PQ ANN index is
  deferred to Phase 6 with the eval/tuning pass (correct results need no index).
- **Fusion.** Hybrid runs keyword + vector, fuses by `file_id` with the Phase 1
  RRF utility (rank-based, so incomparable BM25/cosine scores never need
  normalizing). Both engines over-fetch a bounded candidate set
  (`5×page_end`, capped at 500) before paging. Results are hydrated back to full
  `SearchResult`s from Tantivy; semantic hits anchor their snippet on the
  matched chunk's `start_line`.
- **Facets** in hybrid/semantic come from the keyword path only (they describe
  the keyword universe; consistent with current behaviour).
- **Latent bug fixed.** `SearchService::get_file_by_id` matched nothing for real
  UUIDs because `file_id` is tokenized `TEXT` (split on hyphens); the new query
  path is its first full-UUID consumer. Added `file_id_query()` (hyphen-aware
  `PhraseQuery`) so hydration matches the indexed form.
- **Frontend:** API plumbing only (optional `mode` in `SearchQuery` /
  `useMultiSelectSearch`, sent only when non-default). The mode **toggle UI**,
  result badges and snippet-range rendering are Phase 5.

## 9. Risks & Mitigations

- **Index size** (768 floats/chunk ≈ 3 KB; ~10M chunks ≈ 30 GB) → start with the
  384-dim model if needed; scalar quantization (int8) in LanceDB cuts 4×; make
  semantic indexing opt-in per repository if necessary.
- **Initial backfill cost** (CPU embedding of millions of chunks) → batched worker,
  progress UI, runs in background; document expected throughput; optional GPU via
  ONNX Runtime providers later.
- **Model download at startup** breaks air-gapped installs → support pre-provisioned
  model directory (config path) + document baking it into the image.
- **Quality disappointment** → Phase 1 includes a small golden eval set (20 NL
  queries → expected files on a known repo) so we measure before we ship; hybrid mode
  means BM25 keeps a quality floor.
- **Memory** → ONNX session is the main cost (~500 MB for the base model); document
  new resource requests in the Helm chart.
