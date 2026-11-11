# Klask Deployment Guide

This guide covers deploying Klask to various environments.

## 📦 Table of Contents

- [Local Development](#local-development)
- [Docker Compose](#docker-compose)
- [Kubernetes (Helm)](#kubernetes-helm)
- [Production Best Practices](#production-best-practices)

---

## 🏠 Local Development

### Quick Start

```bash
# 1. Start PostgreSQL
docker-compose -f docker-compose.dev.yml up -d

# 2. Start backend
cd klask-rs
cargo run --bin klask-rs

# 3. Start frontend (in another terminal)
cd klask-react
npm run dev
```

Access:
- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:3000

---

## 🐳 Docker Compose

### Development Environment

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# With optional tools (pgAdmin, Redis)
docker-compose --profile tools up -d
```

### Production Environment

```bash
# Use production compose file
docker-compose -f docker-compose.prod.yml up -d
```

### Configuration

Create a `.env` file for environment variables:

```env
# Database
POSTGRES_USER=klask
POSTGRES_PASSWORD=your-secure-password
POSTGRES_DB=klask

# Backend
JWT_SECRET=your-jwt-secret-change-in-production
ENCRYPTION_KEY=your-encryption-key-change-in-production
RUST_LOG=info

# Frontend
VITE_API_URL=http://localhost:3000
```

### Security settings

These variables change the security posture of the backend. Defaults are
safe for a new install; **existing instances are affected by the default changes
marked "Upgrade impact"**.

| Variable | Default | Purpose | Upgrade impact |
|----------|---------|---------|----------------|
| `ALLOW_REGISTRATION` | `false` | Self-service sign-up on `/api/auth/register`. | **Changed** (was `true`). Instances that relied on open sign-up must set `ALLOW_REGISTRATION=true` explicitly, or create accounts from the admin panel. |
| `COOKIE_SECURE` | `false` | Adds `Secure` to the `auth_token` cookie. | None. Set `true` when the site is served over HTTPS. |
| `TRUSTED_PROXY_HOPS` | `0` | Number of proxies that append to `X-Forwarded-For` in front of the backend. `0` ignores the header; `N` takes the N-th entry counted from the right. Falls back to the TCP peer when the header is too short or invalid. | None for direct access. The compose files use `0`; the chart sets `1` when the ingress or Istio is enabled. See the note below the table. |
| `SETUP_TOKEN` | unset | Secret required by `POST /api/auth/setup` while no user exists. | None. The setup page does not yet have a field for it: use the API (`setup_token` in the JSON body) or leave it unset. |
| `FILESYSTEM_ALLOWED_ROOTS` | empty | Comma-separated directories that `FileSystem` repositories may point into. Paths are canonicalised (symlinks resolved). | **Changed**: with the empty default, no FileSystem repository can be created or updated. Existing FileSystem repositories keep crawling, but set this variable to the directories they use before editing them. |
| `ALLOW_PRIVATE_NETWORK_URLS` | `true` | Allow `http://` and private/loopback hosts for Git, GitLab and GitHub repositories and for GitLab discovery. | Compared with the previous release: `file://` URLs are now refused for Git repositories, and Git URLs are checked at creation and update. Set `false` on exposed or shared instances. See "What the URL option protects" below. |
| `MAX_REQUEST_BODY_BYTES` | `4194304` (4 MiB) | Maximum request body; larger requests get `413`. | None for normal use. Raise it if you upload very large profile avatars. |
| `REQUEST_TIMEOUT_SECS` | `60` | Maximum duration of a request; `0` disables it. Long crawls run in the background and are not affected. | None. |
| `JWT_EXPIRES_IN` | `24h` | Session lifetime (`s`, `m`, `h`, `d` suffixes). Invalid values now stop the server at startup. | Invalid values used to fall back silently. |

Boolean variables accept `true/false`, `1/0`, `yes/no`, `on/off`; any other value stops
the server at startup.

**Client address behind proxies (`TRUSTED_PROXY_HOPS`).** Rate limits are per client address.
Count only the proxies that sit between the client and the backend and append to
`X-Forwarded-For`. In the compose stack the browser calls the backend directly on its published
port (`BACKEND_BASE_URL`), with no proxy in between, so the value is `0` and the header is
ignored. In Kubernetes the ingress (or Istio gateway) routes `/api` straight to the backend
service, which makes one hop: the chart sets `1` automatically when either is enabled. Add one
for each extra proxy or load balancer in front of that gateway.
Setting the value higher than the real number of proxies lets a client choose the address it is
counted under, by sending its own `X-Forwarded-For`, and so escape the per-account login limit.

**What the URL option protects (`ALLOW_PRIVATE_NETWORK_URLS`).** It only checks the *syntax*
of the URL: scheme, and with `false`, IP literals in private or loopback ranges and the names
`localhost`, `*.local`, `*.internal`, `*.localhost`. It does **not** resolve DNS names, so a name
that resolves to a private address is accepted, and an administrator can always point a repository
at an internal service. It is a guard against mistakes and casual SSRF, not a security boundary
against a malicious administrator. The following are refused whatever the setting:
`file://`, the git `ext::` transport, and URLs starting with `-`.

**Other behaviour changes in this release**

- API tokens: a token created with `scope: "read-only"` (the only scope created so far)
  is now enforced. It is refused on every write request and on every admin endpoint.
  Tokens can be created with `scope: "read-write"`. Clients that wrote data with a token
  created before this change must be given a new `read-write` token.
- Changing the password revokes all personal API tokens of the account.
- Registration and first-run setup passwords must have at least 8 characters, with an
  uppercase letter, a lowercase letter and a digit (the rule already used by password
  change). Login still accepts any password of 6 characters or more, so existing accounts are not locked out.
- Login: a (client address, username) pair that fails 10 times in 15 minutes is refused until the
  window ends. Failures from one client address across all usernames only slow its requests
  down (up to 5 s each), so a shared address cannot lock out the whole instance. Registration
  and setup attempts are slowed down the same way.
- `GET /health` returns `503` when the database is unreachable (it returned `200` before).
  The chart's backend readiness probe now uses `/health`.

---

## ☸️ Kubernetes (Helm)

### Prerequisites

- Kubernetes cluster (1.23+)
- Helm 3.x
- kubectl configured
- Storage class for persistent volumes
- Ingress controller (nginx, traefik, etc.)

### Quick Install

```bash
# Install with default values
helm install klask ./charts/klask

# Install with custom values
helm install klask ./charts/klask -f helm-example.yaml
```

### Basic Configuration

See [`helm-example.yaml`](helm-example.yaml) for a complete example. Here's a minimal setup:

```yaml
# minimal-values.yaml
backend:
  replicaCount: 1
  resources:
    requests:
      memory: "256Mi"
      cpu: "250m"

frontend:
  replicaCount: 1

postgresql:
  enabled: true
  auth:
    password: changeme

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: klask.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: klask-tls
      hosts:
        - klask.example.com
```

Install:

```bash
helm install klask ./charts/klask -f minimal-values.yaml
```

### Helm Commands

```bash
# Install
helm install klask ./charts/klask -f values.yaml

# Upgrade
helm upgrade klask ./charts/klask -f values.yaml

# Rollback
helm rollback klask

# Uninstall
helm uninstall klask

# View status
helm status klask

# Validation before upgrade
helm upgrade klask ./charts/klask -f values.yaml --dry-run --debug
```

See [charts/klask/SAFE_DEPLOYMENT.md](charts/klask/SAFE_DEPLOYMENT.md) for detailed deployment safety guide.

---

## 🔒 Production Best Practices

### Security

1. **Change default passwords**
2. **Use Kubernetes secrets for sensitive data**
3. **Enable TLS/SSL with cert-manager**
4. **Configure network policies**

### Resource Management

1. **Set appropriate resource limits**
2. **Enable horizontal pod autoscaling**
3. **Use appropriate storage classes**

### Monitoring

1. **Enable Prometheus metrics**
2. **Configure health checks**
3. **Set up alerting**

### High Availability

1. **Run multiple replicas**
2. **Configure pod disruption budgets**
3. **Use anti-affinity rules**

See full guide: [charts/klask/README.md](charts/klask/README.md)

---

## 📚 Additional Resources

- [Helm Chart Documentation](charts/klask/README.md)
- [Helm Safety Guide](charts/klask/SAFE_DEPLOYMENT.md)
- [Immutable Fields Reference](charts/klask/IMMUTABLE_FIELDS.md)
- [Development Guide](CLAUDE.md)
