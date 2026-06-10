# 🔌 MCP Server Plan — Klask as a Code-Search Tool for AI Agents

## 1. Goal & Positioning

Expose Klask as an **MCP (Model Context Protocol) server** so that AI coding agents
(Claude Code, Cursor, Copilot, custom agents) can use Klask as their **cross-repository
code-search backend**.

**Why this is the killer feature:**
- Coding agents are blind outside the repo they are launched in. Klask already indexes
  *all* the organization's repositories and branches.
- Self-hosted + Rust = private, fast, zero data leaves the infra. This is exactly what
  enterprises want for their AI agents.
- The search API already exists — the MCP server is a thin protocol adapter, not a new
  engine.

**Target UX** (Claude Code example):

```bash
claude mcp add --transport http klask http://klask.internal:3000/mcp \
  --header "Authorization: Bearer <token>"
```

Then the agent can answer: *"How do the other services call the billing API?"* by
searching the entire indexed codebase.

---

## 2. Technical Decisions

### 2.1 Transport: Streamable HTTP (stateless), mounted in the existing Axum server

- Single endpoint **`POST /mcp`** on the existing backend (port 3000). No separate
  process, no separate deployment, Helm chart unchanged.
- **Stateless JSON responses**: the MCP Streamable HTTP spec allows the server to
  answer each POST with a plain `application/json` JSON-RPC response (no SSE needed
  for a tools-only server). No session state to manage.
- `GET /mcp` → `405 Method Not Allowed` (no server-initiated streams in v1).
- stdio transport is **not needed**: Klask is a long-running server; HTTP is the
  natural transport and is supported by all major MCP clients.

### 2.2 Protocol implementation: hand-rolled (no new dependency)

A tools-only stateless MCP server needs exactly 5 JSON-RPC methods:
`initialize`, `notifications/initialized`, `tools/list`, `tools/call`, `ping`.

We implement them directly (~300 lines with types) instead of pulling the `rmcp` SDK:
- Consistent with the project's lean-dependency philosophy (pure Rust, no OpenSSL, gix
  instead of git2).
- No coupling to a fast-moving SDK and its axum version requirements.
- Full control over auth integration (reuse the existing `AuthenticatedUser` extractor).

Protocol version: negotiate `2025-06-18` (fall back to echoing older client versions —
the subset we use is identical).

### 2.3 Authentication: reuse JWT Bearer (v1), API keys later (v2)

- The existing `AuthenticatedUser` extractor already accepts
  `Authorization: Bearer <JWT>` — MCP clients send custom headers, so **v1 works with a
  standard user token** and zero new code.
- **v2 follow-up**: long-lived personal API tokens (`klask_pat_…`, hashed in DB, scoped
  read-only, revocable from the user profile page) because JWTs expire
  (`jwt_expires_in`, default 24h) which is annoying for an agent config. This is a
  separate, independent PR.

---

## 3. Tools Exposed (v1)

| Tool | Description | Backed by |
|---|---|---|
| `search_code` | Full-text/regex search across all indexed repos with filters | `SearchService::search()` |
| `get_file` | Retrieve full file content (with optional line range) | `get_file_by_doc_address()` / `get_file_by_id()` |
| `list_repositories` | List indexed repositories (name, url, type, last crawl) | `RepositoryRepository::list_repositories()` |
| `get_search_facets` | Discover available projects / branches / extensions (to scope searches) | `SearchService::search()` with `include_facets` |

### 3.1 `search_code`

Input schema (all filters optional):
```json
{
  "query":        "string (required) — terms, phrase, or regex",
  "projects":     ["array of project names"],
  "versions":     ["array of branches/tags"],
  "extensions":   ["array, e.g. rs, ts, java"],
  "regex":        "boolean (default false)",
  "case_sensitive": "boolean (default false)",
  "limit":        "integer 1-100 (default 20)",
  "page":         "integer (default 1)"
}
```
Output: JSON text content — `total` + array of `{project, version, path, line_number,
score, snippet, doc_address, file_id}`. `doc_address` is the handle for `get_file`.

Design notes:
- Filters are **arrays** in the schema (agent-friendly), joined to the comma-separated
  strings the service expects.
- `limit` capped at 100 (agents don't paginate like humans; protect their context
  window).
- Reuse `regex_validator::validate_regex_pattern` when `regex: true`.

### 3.2 `get_file`

Input: `doc_address` **or** `file_id`, plus optional `start_line` / `end_line`.
Output: file metadata + content. Content is truncated beyond a max line count
(default 2000) with an explicit `truncated: true` marker so the agent knows to request
a line range.

### 3.3 `list_repositories`

No input. Output: array of `{name, url, repository_type, enabled, last_crawled}`.
Read-only projection — **no tokens, no crawl config** ever exposed.

### 3.4 `get_search_facets`

Optional `query` + same filters as `search_code`. Output: facet counts for
repositories, projects, versions, extensions. Lets an agent discover *what exists*
before searching ("which repos have Kotlin files?").

---

## 4. File Layout & Integration

```
klask-rs/src/
├── mcp/
│   ├── mod.rs        # create_router(), POST handler, method dispatch
│   ├── protocol.rs   # JSON-RPC 2.0 + MCP types (requests, responses, errors)
│   └── tools.rs      # Tool definitions (JSON Schemas) + handlers calling services
└── main.rs           # .route("/mcp", ...) at root level (outside /api)
```

- Mounted at **`/mcp`** (root), not under `/api`: it is a protocol endpoint, not a REST
  resource, and this matches MCP conventions.
- Handlers receive `State<AppState>` + `AuthenticatedUser` exactly like REST endpoints;
  tool logic calls `SearchService` directly (no internal HTTP hop).
- JSON-RPC errors: `-32700` parse error, `-32601` method not found, `-32602` invalid
  params; tool execution failures use `tools/call` result with `isError: true`
  (per MCP spec), not JSON-RPC errors.

---

## 5. Testing Strategy

1. **Unit tests** (`src/mcp/`): JSON-RPC parsing, schema serialization of each tool,
   filter array → comma-string conversion, line-range/truncation logic.
2. **Integration test** (`tests/mcp_test.rs`, `axum-test` + `TestDatabase` + temp
   Tantivy index, same pattern as existing API tests):
   - full handshake: `initialize` → `notifications/initialized` (202) → `tools/list`;
   - `tools/call search_code` after indexing fixture files — verify hits and snippets;
   - `tools/call get_file` round-trip via `doc_address` from search results;
   - auth: request without Bearer → 401;
   - unknown method → `-32601`; malformed JSON → `-32700`.
3. **Manual smoke test**: register the server in Claude Code and run a real search.

---

## 6. Rollout Phases

| Phase | Content | Status |
|---|---|---|
| **1** | MCP endpoint + 4 tools + tests + this doc | 🚧 this PR |
| **2** | Personal API tokens (long-lived, revocable, read-only scope) | planned |
| **3** | `semantic_search` tool / `mode` param once hybrid search lands (see [SEMANTIC_SEARCH_PLAN.md](SEMANTIC_SEARCH_PLAN.md)) | planned |
| **4** | Listing on MCP server directories + README section + demo GIF | planned |

## 7. Risks & Mitigations

- **Token-hungry responses** → snippets only in search results, capped limits,
  truncation markers in `get_file`.
- **Search exposes all repos to any authenticated user** → matches current Klask
  semantics (the web UI already does); revisit if per-repo ACLs ever land.
- **MCP spec evolution** → surface is tiny and versioned; we negotiate the protocol
  version at `initialize`.
