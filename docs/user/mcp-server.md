# MCP server

Klask includes a [Model Context Protocol](https://modelcontextprotocol.io) server
so that AI agents and editors can search your indexed code. It is part of the
backend: there is nothing to enable or install.

## Endpoint and authentication

- **URL:** `POST /mcp` on the backend (for example `http://localhost:3000/mcp`).
  The frontend container and the Helm chart (ingress, HTTPRoute, Istio) also
  route `/mcp`, so on a deployed instance it is `https://<your-klask-host>/mcp`.
- **Transport:** Streamable HTTP, stateless: every JSON-RPC message gets a plain
  JSON response. There are no sessions and no server-sent events, so `GET` and
  `DELETE` return `405`.
- **Protocol versions:** `2025-06-18`, `2025-03-26` and `2024-11-05`. Only tools
  are exposed (`initialize`, `tools/list`, `tools/call`, `ping`).
- **Authentication:** `Authorization: Bearer <token>`, with either a login JWT
  or, for agents, a personal [API token](api-tokens.md).

Quick check:

```bash
curl -s http://localhost:3000/mcp \
  -H "Authorization: Bearer klask_pat_..." \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

Configure your MCP client with the endpoint URL and an `Authorization` header;
the exact syntax depends on the client.

## Tools

| Tool | Purpose |
|---|---|
| `search_code` | Search the indexed repositories. Arguments: `query` (required), `repositories`, `projects`, `versions` (branches or tags), `extensions` (without the dot), `regex`, `case_sensitive`, `mode` (`keyword`, `semantic` or `hybrid`), `limit` (default 20, max 100), `page`. Each result carries a snippet, the matching line number and a `doc_address`. |
| `get_file` | Read an indexed file by `doc_address` (from `search_code`) or by file UUID, optionally limited to a line range (`start_line`, `end_line`). |
| `list_repositories` | List indexed repositories with type, URL and last crawl time. `include_disabled`, `limit` (default 100, max 500), `page`. |
| `get_search_facets` | Discover repositories, projects, branches and extensions, with document counts, optionally narrowed by a query or filters. |

The query follows the same rules as the web search: see
[Search syntax](search-syntax.md). The default `mode` is `hybrid`; it falls back
to keyword search when the server has no [semantic index](../admin/semantic-search.md).

Design notes: [MCP_SERVER_PLAN.md](../MCP_SERVER_PLAN.md).
