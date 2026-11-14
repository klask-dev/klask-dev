# klask-rs

The Klask backend: a Rust (Axum) HTTP server that crawls repositories, maintains
a [Tantivy](https://github.com/quickwit-oss/tantivy) full-text index, stores
application data in PostgreSQL and exposes the REST API (`/api`), the MCP
endpoint (`/mcp`), `/health` and `/version`.

See the [project README](../README.md) for an overview and the
[documentation](../docs/README.md) for everything else:

- [Development setup](../docs/dev/development.md): database, `.env`, `cargo run`
- [Architecture](../docs/dev/architecture.md): module overview
- [Testing](../docs/dev/testing.md)
- [Configuration reference](../docs/admin/configuration.md): environment variables

## Quick start

```bash
# from the repository root: PostgreSQL for development
docker compose -f docker-compose.dev.yml up -d postgres

cd klask-rs
cp .env.example .env
cargo run --bin klask-rs      # http://localhost:3000, migrations run at startup
```

## Layout

```
klask-rs/
├── src/
│   ├── api/           REST endpoints (auth, users, repositories, search, files, admin, scheduler)
│   ├── auth/          JWT, API tokens, extractors
│   ├── mcp/           MCP server
│   ├── models/        Database models
│   ├── repositories/  Database queries
│   ├── services/      Crawler, search, tokenizer, scheduler, encryption, semantic search
│   ├── config.rs      Environment configuration
│   └── main.rs        Entry point
├── migrations/        SQL migrations (applied at startup)
├── tests/             Integration tests
└── examples/          Small standalone programs (cron expression checks, regex on raw fields)
```

## Cargo features

| Feature | Effect |
|---|---|
| `semantic-search` | Local embeddings and vector store for semantic and hybrid search (see [Semantic search](../docs/admin/semantic-search.md)). Off by default. |
| `test-utils` | Test helpers. |
| `pdf-parser`, `docx-parser` | Declared in `Cargo.toml`. |

## License

GPL-3.0-only, see [LICENSE](../LICENSE).
