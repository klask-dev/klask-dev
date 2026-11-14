# Security Policy

## Reporting a Vulnerability

**Please do NOT report security vulnerabilities in public GitHub issues,
discussions or pull requests.**

Report them privately with a
[GitHub Security Advisory](https://github.com/klask-dev/klask-dev/security/advisories/new).
Include the affected version, a description of the problem and, if possible, steps
to reproduce it.

**Response timeline:**
- Acknowledgement: within 48 hours
- Initial assessment: within 7 days
- Patch release: within 30 days for critical/high severity issues

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest  | ✓         |

## Scope

**In scope:**
- Authentication and authorization flaws
- Server-side request forgery (SSRF)
- Cross-site scripting (XSS)
- SQL injection
- Remote code execution
- Sensitive data exposure

**Out of scope:**
- Denial of service attacks
- Issues requiring physical access
- Social engineering
- Missing security headers (tracked separately)

## Deploying securely

- Set strong, unique values for `JWT_SECRET` and `ENCRYPTION_KEY` (see
  [Configuration](docs/admin/configuration.md)) and keep `ENCRYPTION_KEY` safe:
  it protects the repository access tokens stored in the database.
- Serve Klask over HTTPS behind a reverse proxy, and do not expose PostgreSQL.
- Set `ALLOW_REGISTRATION=false` if only invited people should have an account.

## Known Limitations

See [docs/dev/security-audit.md](docs/dev/security-audit.md) for the security
audit of April 2026 and the status of its findings.
