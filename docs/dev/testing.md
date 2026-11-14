# Testing

## Backend (Rust)

```bash
cd klask-rs
cargo test                  # unit and integration tests that need no external service
cargo test search           # only tests whose name contains "search"
cargo test --test mcp_test  # a single integration test file in klask-rs/tests/
cargo clippy -- -D warnings # lint, treated as an error in CI
cargo fmt -- --check        # formatting check (config in rustfmt.toml)
```

### Tests that need PostgreSQL

Some integration tests are marked `#[ignore = "Requires PostgreSQL database"]`
and skipped by a plain `cargo test`. They read `DATABASE_URL` and **truncate
the users and repositories tables**, so give them a dedicated empty database,
never your development one. With the development container running
(see [Development setup](development.md)):

```bash
docker compose -f docker-compose.dev.yml exec postgres \
  psql -U klask_user -d klask_dev -c "CREATE DATABASE klask_test;"

cd klask-rs
DATABASE_URL="postgres://klask_user:klask_password@localhost:5433/klask_test" \
  cargo test --test mcp_test -- --ignored
```

Leave the database empty: the tests apply the migrations themselves. The CI
runs `mcp_test` and `search_service_test` this way (PostgreSQL service
container, database `klask_test`).

### Semantic search tests

The tests that download an embedding model are ignored and need the cargo
feature:

```bash
cargo test --features semantic-search --test semantic_embedding_test -- --ignored
```

## Frontend (React)

```bash
cd klask-react
npm test            # Vitest in watch mode
npm run test:ci     # single run, as in CI
npm run test:coverage
npm run typecheck   # tsc
npm run lint        # ESLint
```

Tests use Vitest, Testing Library and jsdom. Shared helpers (render with
providers, React Query test client, mocks) are in `klask-react/src/test/`; see
[its README](../../klask-react/src/test/README.md).

## Helm chart

`scripts/test-helm.sh` and `make -f Makefile.helm` run `helm lint` and
`helm template` against several value sets (the Makefile also has targets that
install on a cluster; read it before running them). They need the `helm` binary.

## What the CI runs

The `Tests` workflow runs only the part that changed (`klask-rs/**` or
`klask-react/**`):

- frontend: `npm ci`, `npm run typecheck`, `npm test`, `npm run build`;
- backend: `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test`,
  then the PostgreSQL-backed suites above.

See [CI/CD](ci-cd.md).
