# Security Policy

## Reporting a Vulnerability

**Please do NOT report security vulnerabilities in public GitHub issues.**

To report a security vulnerability:
1. Open a [GitHub Security Advisory](https://github.com/klask-dev/klask-dev/security/advisories/new)
2. Or open a confidential issue on the [issue tracker](https://github.com/klask-dev/klask-dev/issues/new) with the type "Security Issue"

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

## Known Limitations

See [docs/SECURITY_AUDIT.md](docs/SECURITY_AUDIT.md) for the full security audit with known issues and their remediation status.
