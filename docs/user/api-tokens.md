# API tokens

Scripts, CI jobs and AI agents can authenticate against the Klask API with a
personal API token instead of a login session. A token acts as the user who
created it. Tokens are stored hashed (SHA-256) and the full value is shown
**only once**, when it is created.

## Create and revoke a token

In the web interface, open **Settings > API tokens** (`/settings/api-tokens`).
Give the token a name (up to 255 characters) and an optional lifetime in days.
Copy the token immediately: it cannot be displayed again. A token can be revoked
from the same page, and only by its owner.

The same operations are available over the API (with a login session):

| Method and path | Purpose |
|---|---|
| `POST /api/users/tokens` | Create a token. Body: `{"name": "ci", "expires_in_days": 90}` (`expires_in_days` is optional). |
| `GET /api/users/tokens` | List your tokens (prefix, name, scope, status, dates; never the secret). |
| `DELETE /api/users/tokens/{id}` | Revoke a token. |

## Use a token

A token looks like `klask_pat_` followed by random characters (42 characters in
total). Send it as a bearer token:

```bash
curl -H "Authorization: Bearer klask_pat_..." \
     "http://localhost:3000/api/search?q=parser&limit=5"
```

Revoked, expired and disabled-user tokens are rejected with `401`.

Tokens are created with the `read-only` scope unless `scope: "read-write"` is
requested. A read-only token is refused on every request that modifies data and
on every admin endpoint; it still authenticates requests to the
[MCP server](mcp-server.md). Changing your password revokes all your tokens.
