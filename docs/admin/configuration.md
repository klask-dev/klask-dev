# Configuration reference

The backend is configured with environment variables. For local development it
also reads a `.env` file in its working directory (see
[`klask-rs/.env.example`](../../klask-rs/.env.example)); for Docker Compose
use the `.env` file at the repository root (see [`.env.example`](../../.env.example)).

Defaults below are the ones of the current code. Boolean and numeric values that
cannot be parsed silently fall back to their default.

## Required

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string, e.g. `postgres://user:password@host:5432/klask`. The backend exits if it is missing or empty. Database migrations are applied automatically at startup. |
| `JWT_SECRET` | Secret used to sign login tokens. At least **32 characters**, otherwise the backend refuses to start. Generate one with `openssl rand -hex 32`. Changing it logs everybody out. |
| `ENCRYPTION_KEY` | Key used to encrypt the repository access tokens (GitLab, GitHub) stored in the database. At least **16 characters**; exactly 32 avoids a warning (other lengths are hashed with SHA-256). `openssl rand -hex 16` gives a 32-character key. At startup the backend checks that the key can decrypt the tokens already stored: **if you lose or change the key, stored tokens become unreadable** and the backend refuses to start until the right key is provided. |

## Server and database

| Variable | Default | Description |
|---|---|---|
| `HOST` | `0.0.0.0` | Address the backend binds to. |
| `PORT` | `3000` | Port the backend listens on. |
| `DATABASE_MAX_CONNECTIONS` | `10` | Size of the PostgreSQL connection pool. |
| `RUST_LOG` | `klask_rs=debug,tantivy=info,sqlx=warn` (the Docker image sets `info`) | Log filter, in [`tracing` syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html). |
| `ENVIRONMENT` | `development` | Label shown in the admin system statistics. No other effect. |

## Authentication and CORS

| Variable | Default | Description |
|---|---|---|
| `JWT_EXPIRES_IN` | `24h` | Lifetime of a login token: a number followed by `s`, `m`, `h` or `d` (a bare number means hours). |
| `ALLOW_REGISTRATION` | `true` | Whether visitors can create their own account from the login page. The very first account is created through the initial setup page and becomes the administrator; set this to `false` to prevent self-registration afterwards. |
| `ALLOWED_ORIGINS` | `http://localhost:5173,http://localhost:8080` | Comma-separated origins allowed by CORS. It must contain the URL of the frontend as seen by the browser. `docker-compose.yml` sets it to `http://localhost:8080` unless overridden. |

## Search index and crawler

| Variable | Default | Description |
|---|---|---|
| `SEARCH_INDEX_DIR` | `./index` | Directory of the Tantivy index (`/app/index` in the Docker image, backed by the `klask_index` volume). Keep it on persistent storage. |
| `SEARCH_MAX_RESULTS` | `10000` | Parsed into the configuration but not used by the search API at the moment. |
| `CRAWLER_TEMP_DIR` | `<system temp dir>/klask-crawler` | Where repositories are cloned while they are crawled. |
| `KLASK_TANTIVY_MEMORY_MB` | `200` | Memory budget of the index writer, in MB. |
| `KLASK_TANTIVY_NUM_THREADS` | unset | Number of indexing threads; when unset or invalid Tantivy chooses. |
| `KLASK_SKIP_RESUME_CRAWLS` | `false` | Set to `true` to not resume interrupted crawls at startup (useful after rebuilding the index). |
| `KLASK_GITLAB_ACCEPT_INVALID_CERTS` | `false` | `true` disables TLS certificate verification for GitLab API calls and clones (self-signed servers). Insecure. |
| `KLASK_GITHUB_ACCEPT_INVALID_CERTS` | `false` | Same for GitHub (for example GitHub Enterprise Server with a private CA). Insecure. |
| `KLASK_GITLAB_EXCLUDED_PROJECTS` | empty | Comma-separated `group/project` paths never imported from GitLab. |
| `KLASK_GITLAB_EXCLUDED_PATTERNS` | empty | Comma-separated patterns (`*` is a wildcard) matched against the `group/project` path. |
| `KLASK_GITHUB_EXCLUDED_REPOSITORIES` | empty | Comma-separated `owner/repo` names never imported from GitHub. |
| `KLASK_GITHUB_EXCLUDED_PATTERNS` | empty | Comma-separated patterns (`*` is a wildcard) matched against `owner/repo`. |

## Semantic search

Only effective with a backend built with `--features semantic-search`; see
[Semantic search](semantic-search.md).

| Variable | Default | Description |
|---|---|---|
| `SEMANTIC_SEARCH_ENABLED` | `false` | Turns semantic indexing and search on. |
| `SEMANTIC_SEARCH_MODEL` | `jinaai/jina-embeddings-v2-base-code` | Embedding model, downloaded on first start. |
| `SEMANTIC_SEARCH_CACHE_DIR` | `./models` | Where the model files are cached. |
| `SEMANTIC_SEARCH_VECTOR_DIR` | `./vector-index` | Where the vector store is persisted. |
| `SEMANTIC_SEARCH_CHUNK_MAX_LINES` | `45` | Maximum lines per embedded chunk. |
| `SEMANTIC_SEARCH_CHUNK_OVERLAP_LINES` | `10` | Lines shared by consecutive chunks. |
| `SEMANTIC_SEARCH_BATCH_SIZE` | `32` | Embedding batch size. |
| `SEMANTIC_SEARCH_QUEUE_CAPACITY` | `1000` | Size of the queue between the crawler and the embedder; the crawl waits when it is full. |

## Frontend (container)

| Variable | Default | Description |
|---|---|---|
| `BACKEND_BASE_URL` | `http://localhost:3000` | URL of the backend API **as seen by the browser**. Read when the container starts (written to `runtime-config.js`), so changing it does not require rebuilding the image. |

When running the frontend with `npm run dev`, the API URL comes from
`VITE_API_BASE_URL` (default `http://localhost:3000`).

## Docker Compose only

| Variable | Default | Description |
|---|---|---|
| `POSTGRES_PASSWORD` | none, required | Password of the bundled PostgreSQL (user `klask_user`, database `klask_dev`). Also used to build `DATABASE_URL`, so use hex or alphanumeric characters. |
| `KLASK_IMAGE_TAG` | `latest` | Tag of the `ghcr.io/klask-dev/klask-backend` and `klask-frontend` images. |

## Build time

`KLASK_VERSION`, `GIT_COMMIT_HASH` and `BUILD_TIMESTAMP` (backend) and
`VITE_APP_VERSION`, `VITE_APP_COMMIT` and `VITE_APP_BUILD_TIME` (frontend) are
build arguments set by the CI to display the version in the interface.
