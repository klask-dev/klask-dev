# Installing with Docker Compose

`docker-compose.yml` at the root of the repository runs Klask from the
pre-built images published on the GitHub Container Registry:

| Service | Image | Port |
|---|---|---|
| `klask-frontend` | `ghcr.io/klask-dev/klask-frontend` | 8080 |
| `klask-backend` | `ghcr.io/klask-dev/klask-backend` | 3000 |
| `postgres` | `postgres:18-alpine` | not published on the host |

Requirements: Docker with the Compose plugin, and enough disk for the search
index (it grows with the amount of code you index).

## Install

```bash
git clone https://github.com/klask-dev/klask-dev.git
cd klask-dev
cp .env.example .env
```

Fill in the three required values in `.env`:

```bash
echo "POSTGRES_PASSWORD=$(openssl rand -hex 24)"
echo "JWT_SECRET=$(openssl rand -hex 32)"
echo "ENCRYPTION_KEY=$(openssl rand -hex 16)"
```

Then start the stack:

```bash
docker compose up -d
```

Open <http://localhost:8080>. On a fresh database the first visit shows a setup
page that creates the administrator account.

Compose refuses to start while one of the three secrets is empty: there are no
built-in defaults. **Keep a copy of `ENCRYPTION_KEY`**: it protects the
repository access tokens stored in the database, and the backend cannot start
with another key once tokens exist.

## Add your code

1. Sign in as the administrator and open **Admin > Repositories**.
2. Add a repository (Git, GitLab, GitHub or a directory on the
   server's file system) or import several from a GitLab or GitHub account with
   an access token.
3. Start a crawl, and optionally set a cron schedule so the repository is
   re-indexed automatically. Schedules are cron expressions with **six fields,
   seconds first** (`seconds minutes hours day-of-month month day-of-week`; for
   example `0 0 2 * * *` runs every day at 02:00).

The configuration options are listed in [Configuration](configuration.md).

## Serving Klask on another host name

With the default settings the browser calls the backend on
`http://localhost:3000`, which only works on the machine that runs the stack.
To serve Klask to other machines, use a single origin: the frontend container
(port 8080) serves the interface and relays `/api` and `/mcp` to the backend.
In `.env`:

```bash
BACKEND_BASE_URL=                            # empty: the browser calls /api on the host serving the interface
ALLOWED_ORIGINS=https://klask.example.com    # public URL of Klask
COOKIE_SECURE=true                           # when Klask is served over HTTPS
TRUSTED_PROXY_HOPS=2                         # your reverse proxy + the frontend nginx (1 without a reverse proxy)
```

Put a reverse proxy with TLS in front of port 8080 only. The API is then at
`https://klask.example.com/api` and the MCP endpoint at
`https://klask.example.com/mcp`.

`TRUSTED_PROXY_HOPS` tells the backend how many proxies to trust when it reads
the client address for the login rate limit. With a value above 0, the backend
port (3000) must not be reachable by clients directly, since they could then
forge that address: remove the `ports:` entry of `klask-backend` in
`docker-compose.yml`, or block the port on the host firewall.

Serving the API from its own host name still works: set `BACKEND_BASE_URL` to
its public URL (for example `https://klask-api.example.com`), list the URL of
the interface in `ALLOWED_ORIGINS`, and proxy both ports.

## Data and backups

State lives in three Docker volumes:

| Volume | Content |
|---|---|
| `postgres_data` | Users, repositories, settings, API tokens (the part to back up) |
| `klask_index` | Tantivy search index (can be rebuilt by crawling again) |
| `klask_data` | Mounted at `/app/data` in the backend container |

Back up the database with:

```bash
docker compose exec -T postgres pg_dump -U klask_user klask_dev > klask-backup.sql
```

## Update

```bash
docker compose pull
docker compose up -d
```

Database migrations run when the backend starts. Set `KLASK_IMAGE_TAG` in `.env`
to a release number to pin the version instead of following `latest`. If the
search index format changes between versions, the admin interface offers to
rebuild it.

## Stop and remove

```bash
docker compose down        # stop, keep the data
docker compose down -v     # stop and delete the volumes (all data)
```

## Upgrading from an older compose file

Earlier versions of `docker-compose.yml` shipped with a default database
password (`klask_password`) and a weak default `JWT_SECRET`. An existing
`postgres_data` volume keeps the password it was created with: put that value in
`POSTGRES_PASSWORD` in `.env` (or change the password inside PostgreSQL).

The PostgreSQL volume is now mounted at `/var/lib/postgresql` (the location the
PostgreSQL 18 image expects) instead of `/var/lib/postgresql/data`. With the old
mount, the database files were written to an anonymous volume rather than to
`postgres_data`. Before updating a deployment created with an older file,
dump the database (`pg_dump`, see above) and restore it into the new stack.

## Other deployments

- Kubernetes: [Helm chart](helm.md).
- Development stack built from source: [`docker-compose.dev.yml`](../dev/development.md).
