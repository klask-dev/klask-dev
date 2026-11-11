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
| **5** | Frontend toggle + result badges + admin card | ✅ done (this PR) |
| **6** | MCP `mode` param; eval pass (latency P95, recall@10 vs keyword) and tuning | ✅ done |

**Phase 1 measurements** (debug build, CPU, `Xenova/bge-small-en-v1.5`, 384 dims):
embedding throughput ≈ 7.6 chunks/s on ~6-line-function chunks; semantically
related code snippets score cosine ≈ 0.82 vs ≈ 0.35–0.45 for unrelated ones.
Reproduce with:
`cargo test --features semantic-search --test semantic_embedding_test -- --ignored --nocapture`

**Phase 6 measurements** (`cargo run --features semantic-search --bin semantic-recall -- --repo ..`,
this repository as corpus: 421 files, 3503 chunks, `jina-embeddings-v2-base-code`,
45-line chunks, ANN index built):

| query set | mode | recall@10 | MRR |
|---|---|---|---|
| 22 natural-language questions | keyword | 0.09 | 0.06 |
| | semantic | 0.95 | 0.71 |
| | hybrid | 0.95 | 0.71 |
| 10 short identifier queries | keyword | 0.90 | 0.68 |
| | semantic | 1.00 | 0.95 |
| | hybrid | 1.00 | 0.90 |

Reading:
- On natural-language questions BM25 collapses (2 hits out of 22) while the
  vector side answers 21 out of 22, usually at rank 1. This is the case the
  feature exists for, and it delivers.
- The second set exists to keep the comparison honest: scoring only NL
  questions would show the vector engine beating BM25 at a game BM25 never
  played. On the short queries users type today, keyword already works (0.90),
  and enabling the vector side does not degrade it — it improves it to 1.00.
- Hybrid matches semantic on this corpus and is marginally *behind* pure
  semantic on short-query MRR (0.90 vs 0.95): RRF dilutes a ranking the vector
  side already got right. Worth revisiting the RRF constant. Hybrid still keeps
  BM25 as a quality floor, which a 421-file corpus under-rewards.
- The one universal miss ("report how far along a long running indexing job
  is" -> `services/progress.rs`) is a genuine failure, not a narrow golden
  entry: the model returned `search.rs` and `search_metrics.rs`, conflating
  Tantivy indexing with crawl progress.

Limits of this measurement, which matter before generalizing:
- 421 files / 3503 chunks is orders of magnitude smaller than a real
  deployment. Recall degrades as distractors multiply.
- The golden set was written by the same author as the code under test, so the
  queries are likely cleaner than what real users type. Queries collected from
  actual usage would be more credible.
- Every query is in English. The model probe (`semantic-eval`) shows the query
  language, not the identifier language, is what breaks: with French queries
  the cosine margin drops from +0.217 to +0.046 on English-identifier code.
- Query latency, same corpus with the ANN index built, 110 timed runs per mode
  (22 queries x 5 repetitions, one untimed warm-up each):

| mode | p50 | p95 | max |
|---|---|---|---|
| keyword | 0 ms | 15 ms | 19 ms |
| semantic | 31 ms | 43 ms | 50 ms |
| hybrid | 33 ms | 46 ms | 54 ms |

  The ~30 ms the semantic modes add is almost entirely the forward pass that
  embeds the question; a short query costs far less than a 400-token chunk.
  That part is constant with corpus size, while ANN search grows sub-linearly,
  so latency is not the scaling risk here. Indexing throughput is.

**Phase 6 cost measurements** (same corpus, Intel Core Ultra 7 165U, 14 threads,
CPU only, ANN index built):

| model | params | dim | throughput | recall@10 (NL, hybrid) |
|---|---|---|---|---|
| jina-embeddings-v2-base-code | 137M | 768 | 2.26 chunks/s | 0.95 |
| bge-small-en-v1.5 | 33M | 384 | 9.22 chunks/s | 0.86 |

- **The build profile is not a lever**: release only beats debug 2.26 vs 1.89
  chunks/s (x1.2). The bottleneck is ONNX inference, not the surrounding Rust,
  so roughly 440 ms per chunk for the 137M model on this CPU.
- The model *is* a lever, and it scales with parameter count: the 4.1x speedup
  matches the 4.2x parameter ratio. It costs 9 points of recall@10 here, and
  that gap should be expected to *widen* with corpus size, since the weaker
  model starts with a much thinner cosine margin (+0.078 vs +0.217 on the model
  probe) and margins are what survive extra distractors.
- **Hybrid earns its keep exactly when the model is weaker**: with bge-small it
  beats pure semantic (0.86 vs 0.82 recall, 0.64 vs 0.58 MRR), while with jina
  the two are identical. The BM25 floor is insurance against model quality, so
  hybrid is the right default regardless of which model is configured.
- Sizing, from the measured ratio of one chunk per 32 lines of indexed text:

| corpus | chunks | jina | bge-small | vector bytes (jina) |
|---|---|---|---|---|
| 1M lines | 31k | 3.8 h | 0.9 h | 95 MB |
| 10M lines | 312k | 38 h | 9.4 h | 960 MB |
| 25M lines | 780k | 96 h | 23.5 h | 2.4 GB |

  These are full-index times. They are acceptable as a one-off backfill running
  in the background and unacceptable per crawl, which is what happens today:
  the crawler re-indexes every file every time (no `last_modified` check), so
  **incremental crawling is the precondition for semantic search at scale**, not
  a nice-to-have. It is worth 2 to 3 orders of magnitude on recurring crawls,
  far more than any model or runtime tuning, and it speeds up Tantivy indexing
  too.
- Untested cheap leads: `intra_threads` is left at ORT's default (every logical
  core), which on a hybrid P/E-core CPU can be slower than pinning to the
  performance cores; fastembed exposes it and Klask does not. GPU execution
  providers are exposed by fastembed and unused. Markdown alone accounts for
  20% of this repository's chunks, so a vector-index inclusion policy separate
  from the (much cheaper) Tantivy one is worth 20-40% on a real corpus.

**Phase 1 model comparison** (`cargo run --features semantic-search --bin semantic-eval`,
8 concepts, chance P@1 = 0.12, margin = cosine gap to the best distractor):

| model | EN code / EN query | EN code / FR query | FR code / FR query | FR code / EN query |
|---|---|---|---|---|
| jina-embeddings-v2-base-code (768) | 1.00 / +0.217 | 0.75 / +0.046 | 0.62 / +0.013 | 1.00 / +0.147 |
| bge-small-en-v1.5 (384) | 0.88 / +0.078 | 0.62 / -0.000 | 0.50 / -0.002 | 0.62 / +0.021 |
| multilingual-e5-small (384) | 0.62 / +0.010 | 0.50 / +0.002 | 0.50 / -0.002 | 0.50 / +0.004 |

The code-specialized model wins by a factor of 3 on margin, which settles the
default. The multilingual model is worst even on English, so fastembed offers
no model that is both multilingual and code-aware: non-English queries stay a
known weakness, mitigated only by hybrid mode's BM25 floor.

**Phase 2 notes:**
- Vector store is **LanceDB** (`lancedb` 0.30, embedded), table `chunks` with the
  metadata columns from §3.2 + a `FixedSizeList<Float32, dim>` vector column.
  Persists under `SEMANTIC_SEARCH_VECTOR_DIR` (default `./vector-index`).
- The embedding worker is a single `tokio` task fed by a **bounded** queue;
  **the crawl blocks when the queue is full** (strict backpressure — chunks are
  never silently dropped, keeping the vector index consistent with the crawl).
- Lifecycle mirrors Tantivy: delete-by-`repository` on re-crawl and repository
  deletion, delete-then-insert per `file_id` when a single file is re-indexed on
  its own. Re-opening the store with a different embedding dimension (model
  change) is refused with a clear error to prevent silent corruption.
- **Writes are batched, and only the bulk paths skip the delete probe.**
  `IndexJob::mode` (`WriteMode`) says whether a file's existing chunks must be
  removed first. A crawl and a backfill both purge up front (repository chunks /
  whole store) and produce each `file_id` once, so they use `Append`: the worker
  accumulates chunks across files and issues one LanceDB write per batch. This
  matters because every write commits a table version and a fragment, and there
  is no scalar index on `file_id`, so the previous per-file delete-then-insert
  scanned the table and committed twice *per file* — the cost grew with the
  table and dominated indexing time. `Replace` keeps the old behaviour for a
  one-off file re-index and is applied on its own, never batched.
- **Maintenance runs once per bulk write, not per file.** `VectorStore::optimize`
  compacts fragments, prunes superseded versions, builds the `file_id` scalar
  index (so a `Replace` delete is a lookup) and, above 10k chunks, the ANN index
  on `vector` (without it every search is a brute-force scan of all vectors).
  It is called after a backfill and, detached, once the embedding queue has
  drained after a crawl; concurrent calls are skipped rather than queued.
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

**Phase 5 notes:**
- **Engine toggle.** `SearchPageV3` gains a Keyword | Hybrid | Semantic segmented
  control, persisted in the URL as `mode=` (omitted when keyword, so plain
  keyword URLs stay clean and backward compatible) and wired into the existing
  `mode` param of `useMultiSelectSearch`. It is **orthogonal** to the keyword-engine
  toggles (fuzzy/regex/case) — those choose how the keyword engine matches; this
  chooses which engine answers.
- **Availability gating.** The admin `status` endpoint is admin-only, so the
  regular search page can't use it to decide whether to show the toggle. Added a
  lightweight authenticated **`GET /api/search/capabilities`** → `{ semantic_enabled }`
  (true only when the feature is built, the model loaded, and the store opened);
  the toggle renders only when true and degrades to hidden if the endpoint is
  unreachable. Capabilities are static for the process lifetime, so the hook
  caches them indefinitely.
- **Result badges (match provenance).** Phase 4 hydrated results without saying
  which engine matched. Phase 5 adds `MatchSource` (`keyword`/`semantic`/`both`)
  on `SearchResult`, populated in the query path — `hydrate_semantic_only` tags
  every hit `Semantic`; `hybrid_fuse` tags each fused file by membership in the
  keyword/vector rankings (`Both` when in both). It serializes with
  `skip_serializing_if = "Option::is_none"` so the keyword response is unchanged,
  and the UI shows a small badge per result in hybrid/semantic mode.
- **Admin card** (model/dimension, chunk count, rebuild + progress) already
  shipped in Phase 3; nothing further needed here.
- **Hydration fixes (post-Phase 4 field findings).** Semantic/hybrid results
  initially displayed the Tantivy hydration score (BM25 / 1.0) and the whole
  file as snippet. The query path now sets `score` to the cosine similarity
  (`1 - distance`, clamped to [0,1]) in semantic mode and the RRF fused score
  in hybrid mode, anchors `line_number` to the best-matching chunk and trims
  `content_snippet` to that chunk's line range.
- **Performance findings & fixes (measured on a dev machine, jina-v2-base-code
  on CPU):**
  - fastembed truncates every input at **512 tokens** (its default; now set
    explicitly as `MAX_SEQUENCE_TOKENS` in `semantic::embedder`). A 60-line
    code chunk is ~800 tokens, so chunk tails were silently *not embedded*,
    and per-chunk cost was a flat ~1 s regardless of chunk size. Chunk
    defaults are now **45 lines / 10 overlap** (stride 35) so every line falls
    inside the embedded window of at least one chunk.
  - Embedding cost is ~1 s/chunk with jina-v2-base-code and ~0.27 s/chunk with
    `Xenova/bge-small-en-v1.5` (3.7×). Model choice — not chunk size — is the
    indexing-throughput lever; switching models changes the dimension and
    requires a vector-store wipe + backfill (enforced at open).
  - The query path and the indexing worker used to share one ONNX session
    behind a mutex, so an interactive search could wait tens of seconds behind
    an indexing batch. `init_vector_indexer` now loads a **dedicated session
    for the worker** (falls back to sharing if the second load fails), keeping
    query embedding at ~35 ms even while indexing.
- **Embedding-progress visibility.** The backfill previously reported
  `running=false` once every document was *enqueued*, while the worker kept
  embedding invisibly for a long time. `VectorIndexer` now tracks a
  `pending` counter (queued + in-flight files), the backfill stays `running`
  until the queue drains, the status payload exposes `queue_depth`, and the
  admin card shows real embedding progress (`processed - queue_depth`) plus a
  notice when a *crawl* is feeding the semantic index outside a rebuild.

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
