# Semantic search (optional)

Besides keyword search, Klask can embed code chunks locally and search them by
meaning ("where do we validate auth tokens?"). It is optional, disabled by
default, and needs a backend compiled with a cargo feature.

## What you need

1. **A backend built with the `semantic-search` feature:**

   ```bash
   cd klask-rs
   cargo build --release --features semantic-search
   ```

   The feature pulls in the ONNX runtime and a vector store (LanceDB). The
   `klask-rs/Dockerfile` runs a plain `cargo build --release`, so the standard
   Docker images do not include it unless you change the build command.

2. **Configuration:** set `SEMANTIC_SEARCH_ENABLED=true`. On first start the
   backend downloads the embedding model (default
   `jinaai/jina-embeddings-v2-base-code`) into `SEMANTIC_SEARCH_CACHE_DIR`; the
   vectors are stored in `SEMANTIC_SEARCH_VECTOR_DIR`. Both directories should be
   on persistent storage. All options are listed in
   [Configuration](configuration.md#semantic-search).

When the feature is not compiled in, or the model or the vector store fails to
load, the backend keeps working and silently answers every query with keyword
search. `GET /api/search/capabilities` reports `semantic_enabled`, and the web
interface only shows the mode selector when it is `true`.

## Using it

- **Search modes:** `keyword` (default), `semantic` and `hybrid` (keyword and
  semantic rankings fused with reciprocal-rank fusion), selected with the `mode`
  query parameter of `GET /api/search`, the mode selector of the search page, or
  the `mode` argument of the MCP `search_code` tool
  (see [MCP server](../user/mcp-server.md)).
- **Indexing:** files crawled while semantic search is enabled are embedded in
  the background. For repositories that were already indexed, an administrator
  can build the semantic index from **Admin > Index**; the same page shows its
  status.

## Design notes

[SEMANTIC_SEARCH_PLAN.md](../SEMANTIC_SEARCH_PLAN.md) describes the architecture
and the reasons for the choices made.
