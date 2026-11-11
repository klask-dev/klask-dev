# Architecture

Klask is a Rust backend and a React single-page application, with PostgreSQL for
application data and an embedded Tantivy index for search.

```
Browser (React SPA, nginx)  ──HTTP/JSON──►  Backend (Axum)  ──►  PostgreSQL   users, repositories, tokens, schedules
AI agents / scripts  ──── /api, /mcp ───►        │          ──►  Tantivy index   files, full-text search
                                                 └─ crawler ──►  Git / GitLab / GitHub / local directories
                                                 └─ (optional) embeddings ─► vector store (LanceDB)
```

## Backend (`klask-rs/`)

| Path | Role |
|---|---|
| `src/main.rs`, `src/config.rs` | Startup, router assembly, environment configuration (see [Configuration](../admin/configuration.md)). |
| `src/api/` | REST endpoints under `/api`: `auth`, `users` (including API tokens), `repositories`, `search`, `files`, `scheduler`, `admin`. |
| `src/mcp/` | MCP server at `/mcp` (see [MCP server](../user/mcp-server.md)). |
| `src/auth/` | JWT sessions and personal API tokens (`klask_pat_…`), axum extractors for authenticated and admin users. |
| `src/models/`, `src/repositories/`, `src/database.rs` | Database models, SQLx queries, connection pool. Migrations are in `migrations/` and run at startup. |
| `src/services/search.rs`, `tokenizer.rs` | Tantivy index, schema, query building and the code tokenizer (see [Search syntax](../user/search-syntax.md)). |
| `src/services/crawler/` | Indexing pipeline: clones or opens a repository (Git through `gix`, GitLab, GitHub, file system), reads each branch's files from the Git tree, picks a parser by content type (plain text, PDF, DOCX, binary detection) and writes to the index. |
| `src/services/scheduler.rs` | Cron-based re-crawls (`croner`). |
| `src/services/progress.rs` | Crawl progress shown in the interface. |
| `src/services/encryption.rs` | AES-256-GCM encryption of the stored repository access tokens, keyed by `ENCRYPTION_KEY`. |
| `src/services/semantic/` | Optional semantic search (feature `semantic-search`): chunking, local embeddings (`fastembed`), vector store (LanceDB), rank fusion. See [Semantic search](../admin/semantic-search.md). |

Data flow of a crawl: an administrator (or the scheduler) starts a crawl; the
crawler lists the branches, walks their trees, and for each file
upserts a document in Tantivy (content, name, path, repository, project, branch
or tag, extension, size). PostgreSQL stores which repositories exist and the
crawl state.

Searches run in a blocking thread pool with a 30-second timeout. Results carry a
snippet generated from the matching content.

## Frontend (`klask-react/`)

React 19, TypeScript, Vite, Tailwind CSS 4, React Router, TanStack Query for
server state and Zustand for client state (`src/stores`).

| Path | Role |
|---|---|
| `src/features/` | Screens: `search`, `files`, `repositories`, `auth` (login, setup, profile, API tokens), `admin`. |
| `src/api/`, `src/hooks/` | API client and React Query hooks. |
| `src/components/` | Shared UI components. |
| `src/contexts/` | Theme and search-filter contexts. |
| `src/lib/config.ts` | Resolves the API URL: `runtime-config.js` (written by the container entrypoint from `BACKEND_BASE_URL`), then `VITE_API_BASE_URL`, then `http://localhost:3000`. |

The container serves the built files with nginx on port 8080.

## Deployment artifacts

- `klask-rs/Dockerfile`, `klask-react/Dockerfile`: production images.
- `docker-compose.yml`: pre-built images; `docker-compose.dev.yml`: build from source.
- `charts/klask`: Helm chart.

## Further reading

Design notes written while building features:

- [MCP_SERVER_PLAN.md](../MCP_SERVER_PLAN.md)
- [SEMANTIC_SEARCH_PLAN.md](../SEMANTIC_SEARCH_PLAN.md)
- [Security audit](security-audit.md)
