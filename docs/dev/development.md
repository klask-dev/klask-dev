# Development setup

This page gets a development environment running. For the project layout see
[Architecture](architecture.md), for tests see [Testing](testing.md).

## Prerequisites

- **Rust**, a recent stable toolchain (1.88 or newer: the code uses `let` chains
  with the 2024 edition). Install with [rustup](https://rustup.rs/).
- **Node.js 22** and npm (the version used by the CI).
- **Docker** with the Compose plugin, for PostgreSQL.
- Optional: `cargo install sqlx-cli` if you want to run migrations by hand.

## 1. Database

`docker-compose.dev.yml` starts PostgreSQL (and, if you want them, the whole
stack built from source). To start the database only:

```bash
docker compose -f docker-compose.dev.yml up -d postgres
```

It is published on `127.0.0.1:5433` with user `klask_user`, password
`klask_password`, database `klask_dev`. These credentials are for development
only.

## 2. Backend

```bash
cd klask-rs
cp .env.example .env      # already points at the database above
cargo run --bin klask-rs
```

The API listens on <http://localhost:3000> (health check at `/health`). Database
migrations are applied automatically at startup. On a fresh database, the first
account is created from the web interface (setup page) and becomes the
administrator.

## 3. Frontend

```bash
cd klask-react
npm install
npm run dev
```

The dev server runs on <http://localhost:5173> and talks to
`http://localhost:3000` unless `VITE_API_BASE_URL` says otherwise (this is the
default of `ALLOWED_ORIGINS` on the backend, so no CORS setup is needed).

## Useful commands

| Task | Command |
|---|---|
| Format Rust | `cd klask-rs && cargo fmt` |
| Lint Rust (as in CI) | `cd klask-rs && cargo clippy -- -D warnings` |
| Lint frontend | `cd klask-react && npm run lint` |
| Type-check frontend | `cd klask-react && npm run typecheck` |
| Build frontend | `cd klask-react && npm run build` |
| Whole stack from source | `docker compose -f docker-compose.dev.yml up -d --build` (frontend on 8081, API on 3001) |

The `Makefile` wraps some of these (`make help`).

## Git hooks

`make install-hooks` points `core.hooksPath` at the `git-hooks/` directory. The
pre-commit hook runs `cargo fmt` and ESLint on the files you changed; the
pre-push hook runs the heavier checks. Bypass them with `--no-verify` if needed;
the CI runs the same checks anyway.

## Commit messages

Releases are automated with semantic-release from
[Conventional Commits](https://www.conventionalcommits.org/): see
[CONTRIBUTING.md](../../CONTRIBUTING.md).
