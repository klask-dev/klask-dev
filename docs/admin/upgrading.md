# Upgrade notes

Changes that need attention when you upgrade an existing instance. Releases not
listed here need nothing more than the usual image update.

## Next release

### Defaults that changed

- **Registration is closed by default.** `ALLOW_REGISTRATION` now defaults to
  `false`. Set `ALLOW_REGISTRATION=true` if you relied on open sign-up, or
  create accounts from the admin panel.
- **API token scopes are enforced.** A token with the `read-only` scope (the
  only scope created until now) is refused on every write request and on every
  admin endpoint; MCP calls still work. Create a `read-write` token for clients
  that modify data.
- **FileSystem repositories need an allowed root.** With the empty default of
  `FILESYSTEM_ALLOWED_ROOTS`, no FileSystem repository can be created or
  edited. Existing ones keep being crawled; set the variable to the directories
  they use before editing them.
- **`file://` URLs are refused** for Git repositories, as are the `ext::`
  transport and URLs starting with `-`. Git URLs are checked at creation and
  update.
- **Files that usually hold secrets are no longer indexed** (`.env`, private
  keys, keystores, registry and cloud credentials). Documents already in the
  index disappear at the next crawl of each repository. See
  [Sensitive files](configuration.md#sensitive-files).
- **Branch and project filters with a suffix now work.** A pattern such as
  `*-archive` never matched before. If you had configured one, the matching
  branches or projects are now really excluded.

### Other behaviour changes

- Passwords set at registration, setup and password change need 8 characters
  with an uppercase letter, a lowercase letter and a digit. Existing accounts
  can still log in with their current password.
- Changing a password revokes the API tokens of the account.
- Logins are rate limited per client address and username (see the
  [configuration reference](configuration.md#authentication-and-cors)).
- `GET /health` returns `503` when the database is unreachable (it returned
  `200`). The chart's readiness probe uses it.
- An invalid `JWT_EXPIRES_IN` or an invalid boolean for a security setting stops
  the backend at startup instead of falling back silently.
- Search: `page=0` and `limit=0` are rejected with `400`, and results beyond the
  10 000th are not reachable by paging. A double-quoted query is a phrase.

### Docker

- The backend image runs as UID 1000 instead of starting as root. Volumes
  created by earlier versions are already owned by that user.
- `docker-compose.yml` requires `POSTGRES_PASSWORD`, `JWT_SECRET` and
  `ENCRYPTION_KEY` in `.env` and no longer publishes PostgreSQL on the host.
- The PostgreSQL volume is mounted on `/var/lib/postgresql`, where the
  `postgres:18` image keeps its data. See the installation guide before
  updating an instance created with an earlier compose file.
