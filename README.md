<p align="center">
  <a href="https://github.com/klask-dev/klask-dev"><img src="resources/images/logo.svg" alt="Klask" width="360"></a>
</p>

<p align="center">
  <a href="https://github.com/klask-dev/klask-dev/actions/workflows/1-tests.yml"><img src="https://github.com/klask-dev/klask-dev/actions/workflows/1-tests.yml/badge.svg" alt="Tests"></a>
  <a href="https://github.com/klask-dev/klask-dev/releases"><img src="https://img.shields.io/github/v/release/klask-dev/klask-dev" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue" alt="License: GPL-3.0"></a>
</p>

**Klask** is a self-hosted search engine for source code. It crawls your Git,
GitLab and GitHub repositories, indexes every branch, and lets you search them
from a web interface or from an API. The backend is written in Rust (Axum,
Tantivy, PostgreSQL) and the interface in React.

Live demo: <https://app.klask.dev>

## Features

- **Repositories**: Git, GitLab, GitHub (including bulk import from a GitLab or
  GitHub account) and local directories; branches are indexed and can be used as
  a filter.
- **Search made for code**: camelCase words are split into their parts, `snake_case`
  identifiers stay whole, with case-sensitive, fuzzy and regular-expression options.
  See the [search syntax](docs/user/search-syntax.md).
- **Filters** by repository, project, branch, file extension and size, with
  counts for each value.
- **File view** with syntax highlighting, light and dark themes.
- **Scheduled crawls** with cron expressions, crawl progress in the interface.
- **Users and roles**: JWT sessions, administrator and user roles, and
  [personal API tokens](docs/user/api-tokens.md). Repository access tokens are
  encrypted in the database.
- **[MCP server](docs/user/mcp-server.md)** (`/mcp`) so that AI agents and
  editors can search your code.
- **[Semantic search](docs/admin/semantic-search.md)** (optional, local
  embeddings, no external service) alongside keyword search.
- **Deployment**: Docker Compose and a Helm chart.

## Quick start

You need Docker with the Compose plugin and `openssl`.

```bash
git clone https://github.com/klask-dev/klask-dev.git && cd klask-dev
printf 'POSTGRES_PASSWORD=%s\nJWT_SECRET=%s\nENCRYPTION_KEY=%s\n' \
  "$(openssl rand -hex 24)" "$(openssl rand -hex 32)" "$(openssl rand -hex 16)" > .env
docker compose up -d
```

Open <http://localhost:8080>: the first visit asks you to create the
administrator account. Then add a repository from **Admin > Repositories** and
start a crawl.

The secrets are generated for you and kept in `.env`: back that file up, in
particular `ENCRYPTION_KEY`, without which the stored repository tokens cannot
be decrypted. More in the [installation guide](docs/admin/installation.md).
For Kubernetes see the [Helm guide](docs/admin/helm.md).

## Documentation

The [documentation index](docs/README.md) lists everything. Starting points:

- Using Klask: [search syntax](docs/user/search-syntax.md),
  [API tokens](docs/user/api-tokens.md), [MCP server](docs/user/mcp-server.md)
- Running Klask: [installation](docs/admin/installation.md),
  [configuration reference](docs/admin/configuration.md),
  [Helm](docs/admin/helm.md)
- Developing Klask: [development setup](docs/dev/development.md),
  [architecture](docs/dev/architecture.md), [testing](docs/dev/testing.md),
  [CI/CD and releases](docs/dev/ci-cd.md)

## Contributing

Bug reports, ideas and pull requests are welcome. Read
[CONTRIBUTING.md](CONTRIBUTING.md) first, and follow the
[code of conduct](CODE_OF_CONDUCT.md). To report a security vulnerability, see
[SECURITY.md](SECURITY.md): do not open a public issue.

## License

Klask is released under the [GNU General Public License v3.0](LICENSE).
Copyright (C) 2016 Jérémie and the Klask contributors; see [NOTICE](NOTICE).
