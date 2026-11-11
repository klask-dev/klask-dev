# Configuration reference

The backend is configured with environment variables. For local development it
also reads a `.env` file in its working directory (see
[`klask-rs/.env.example`](../../klask-rs/.env.example)); for Docker Compose
use the `.env` file at the repository root (see [`.env.example`](../../.env.example)).

Defaults below are the ones of the current code. The security settings are
strict: an invalid value for `ALLOW_REGISTRATION`, `COOKIE_SECURE`,
`ALLOW_PRIVATE_NETWORK_URLS` (booleans: `true/false`, `1/0`, `yes/no`, `on/off`)
or `JWT_EXPIRES_IN` stops the backend at startup. Other numeric or boolean values
that cannot be parsed fall back to their default.

Settings whose default changed are listed in the [upgrade notes](upgrading.md).

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
| `ALLOW_REGISTRATION` | `false` | Whether visitors can create their own account from the login page. The very first account is created through the initial setup page and becomes the administrator; further accounts are created from the admin panel unless this is `true`. |
| `SETUP_TOKEN` | unset | Secret required by the first-run setup (`POST /api/auth/setup`, field `setup_token`) while no user exists. Without it, the setup is open to anyone who reaches the instance before you do. The setup page has no field for it yet: use the API, or leave it unset. |
| `COOKIE_SECURE` | `false` | Adds `Secure` to the session cookie. Set `true` whenever the site is served over HTTPS. |
| `ALLOWED_ORIGINS` | `http://localhost:5173,http://localhost:8080` | Comma-separated origins allowed by CORS. It must contain the URL of the frontend as seen by the browser. `docker-compose.yml` sets it to `http://localhost:8080` unless overridden. |
| `TRUSTED_PROXY_HOPS` | `0` | Number of proxies in front of the backend that append to `X-Forwarded-For`. `0` ignores the header; `N` takes the N-th entry counted from the right, and the TCP peer when the header is too short or invalid. See below. |

Passwords chosen at setup, registration and password change need at least 8
characters with an uppercase letter, a lowercase letter and a digit. Changing a
password revokes the API tokens of the account.

**Login rate limit.** A (client address, username) pair that fails 10 times in
15 minutes is refused until the window ends. Failures from one client address
across many usernames only slow that client down (up to 5 s per request), so a
shared address cannot lock out the whole instance.

**Client address behind proxies (`TRUSTED_PROXY_HOPS`).** The rate limit is per
client address, so the backend must know which proxies to trust. Count only the
proxies a request really goes through before reaching the backend. In the
compose stack the browser calls the backend on its published port, with no proxy
in between: the value is `0`. When requests go through the frontend nginx
(`BACKEND_BASE_URL` empty), that is one hop. In Kubernetes the ingress or gateway routes `/api`
straight to the backend, which is one hop: the chart sets `1` when the ingress,
the Gateway API route or Istio is enabled. Add one for each extra proxy or load
balancer in front. A value higher than the real number of proxies lets a client
choose the address it is counted under, by sending its own `X-Forwarded-For`,
and so escape the login limit.

## Repositories and limits

| Variable | Default | Description |
|---|---|---|
| `ALLOW_PRIVATE_NETWORK_URLS` | `true` | Allows `http://` and private or loopback hosts for Git, GitLab and GitHub repositories. Set `false` on an instance whose administrators are not all trusted. |
| `FILESYSTEM_ALLOWED_ROOTS` | empty | Comma-separated directories that `FileSystem` repositories may point into; paths are canonicalised (symlinks resolved). Empty means that no FileSystem repository can be created or edited. |
| `MAX_REQUEST_BODY_BYTES` | `4194304` (4 MiB) | Maximum request body; larger requests get `413`. |
| `REQUEST_TIMEOUT_SECS` | `60` | Maximum duration of a request; `0` disables it. Crawls run in the background and are not affected. |

`ALLOW_PRIVATE_NETWORK_URLS=false` only checks the *syntax* of the URL: the
scheme, IP literals in private or loopback ranges and the names `localhost`,
`*.local`, `*.internal`, `*.localhost`. It does not resolve DNS names, so a name
that resolves to a private address is accepted. It guards against mistakes and
casual SSRF; it is not a boundary against a malicious administrator. Whatever
the setting, `file://` URLs, the git `ext::` transport and URLs starting with
`-` are refused.

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

### Git operations

| Variable | Default | Description |
|---|---|---|
| `KLASK_GIT_FETCH_TIMEOUT_SECS` | `1800` | Maximum duration of a fetch into an existing clone. |
| `KLASK_GIT_CLONE_TIMEOUT_SECS` | `3600` | Maximum duration of a first clone. |

### Sensitive files

Files that usually hold secrets are not indexed, whatever the repository:
`.env` and `.env.*` (templates such as `.env.example` or `.env.sample` are
kept), `.npmrc`, `.pypirc`, `.netrc`, `.pgpass`, `.git-credentials`,
`.htpasswd`, SSH private keys (`id_rsa`, `id_ed25519`, ...; `.pub` files are
kept), `*.pem`, `*.key`, `*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.kdbx`,
`.docker/config.json`, `.aws/credentials` and `.kube/config`.

| Variable | Default | Description |
|---|---|---|
| `KLASK_EXCLUDED_FILE_PATTERNS` | empty | Extra comma-separated patterns to exclude (`*` is a wildcard): a file name, or a trailing path such as `.kube/config`. |
| `KLASK_INDEX_SENSITIVE_FILES` | `false` | `true` indexes the built-in list above too. Not recommended: anyone who can search can then read those files. |

This is a filter on file names, not a secret scanner: a password committed in a
source file is still indexed. A change applies at the next crawl of each
repository.

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
| `BACKEND_BASE_URL` | `http://localhost:3000` | URL of the backend **as seen by the browser**, without the `/api` suffix. Set but empty means *same origin*: the browser calls `/api` on the host that serves the interface. Read when the container starts (written to `runtime-config.js`), so changing it does not require rebuilding the image. |
| `BACKEND_UPSTREAM` | `klask-backend:3000` | `host:port` of the backend **as seen by the frontend container**. Its nginx relays `/api` and `/mcp` there, which is what serves same-origin requests when no ingress or gateway routes them. The name is resolved at request time without search domains: in Kubernetes use the full service name (the chart does). |
| `NGINX_RESOLVERS` | from `/etc/resolv.conf` | DNS servers used to resolve `BACKEND_UPSTREAM`, space-separated. |

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
