//! URL validation to prevent SSRF and dangerous Git transports.
//!
//! Two policies are exposed:
//! - the strict policy (`validate_external_url`): HTTPS only, no loopback,
//!   link-local, private or `.local`/`.internal` hosts;
//! - the configurable policy (`validate_outbound_url`, `validate_git_remote`):
//!   the strict policy by default, relaxed when the operator sets
//!   `ALLOW_PRIVATE_NETWORK_URLS=true` (self-hosted GitLab on a LAN).
//!
//! Limits: hostnames are checked syntactically; DNS names that resolve to a
//! private address are not detected here.

use std::net::IpAddr;

/// Validates a URL the server is about to call on behalf of a user.
///
/// With `allow_private_networks == false` this is the strict policy. With
/// `true`, `http://` is accepted and private/loopback hosts are allowed, which
/// is what a self-hosted GitLab reachable only on the LAN requires.
pub fn validate_outbound_url(url: &str, allow_private_networks: bool) -> Result<(), String> {
    let (scheme, rest) = url.split_once("://").ok_or_else(|| "URL must contain a scheme".to_string())?;
    let allowed_scheme = if allow_private_networks { "http" } else { "https" };
    let scheme_ok =
        scheme.eq_ignore_ascii_case("https") || (allow_private_networks && scheme.eq_ignore_ascii_case(allowed_scheme));
    if !scheme_ok {
        return Err(if allow_private_networks {
            "URL must use the http or https scheme".to_string()
        } else {
            "URL must use HTTPS scheme".to_string()
        });
    }

    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    // Drop optional userinfo, then the port
    let host_port = authority.rsplit_once('@').map(|(_, h)| h).unwrap_or(authority);
    let hostname = extract_hostname(host_port);
    check_hostname(hostname, allow_private_networks)
}

/// Validates a Git remote supplied for a Git, GitLab or GitHub repository.
///
/// Accepted forms: `https://`, `ssh://` and scp-like `user@host:path`
/// (`http://` only when private networks are allowed). Rejected outright:
/// leading `-` (would be read as a git option), `file://`, and git's `ext::`
/// transport, which runs arbitrary commands.
pub fn validate_git_remote(url: &str, allow_private_networks: bool) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("Repository URL cannot be empty".to_string());
    }
    if trimmed.starts_with('-') {
        return Err("Repository URL cannot start with '-'".to_string());
    }
    if trimmed.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("Repository URL cannot contain whitespace or control characters".to_string());
    }
    if trimmed.to_ascii_lowercase().starts_with("ext::") {
        return Err("The git 'ext::' transport is not allowed".to_string());
    }

    if let Some((scheme, rest)) = trimmed.split_once("://") {
        return match scheme.to_ascii_lowercase().as_str() {
            "https" => validate_outbound_url(trimmed, allow_private_networks),
            "http" if allow_private_networks => validate_outbound_url(trimmed, true),
            "ssh" => {
                let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
                let host_port = authority.rsplit_once('@').map(|(_, h)| h).unwrap_or(authority);
                check_hostname(extract_hostname(host_port), allow_private_networks)
            }
            other => Err(format!("URL scheme '{}' is not allowed for Git repositories", other)),
        };
    }

    // scp-like syntax: [user@]host:path
    let host_part = trimmed.split_once(':').map(|(h, _)| h).unwrap_or("");
    let host = host_part.rsplit_once('@').map(|(_, h)| h).unwrap_or(host_part);
    if host.is_empty() {
        return Err("Repository URL must contain a host".to_string());
    }
    check_hostname(host, allow_private_networks)
}

/// Strip the port from a `host[:port]` or `[ipv6]:port` authority.
fn extract_hostname(host_port: &str) -> &str {
    if let Some(stripped) = host_port.strip_prefix('[') {
        // [ipv6]:port
        return stripped.split(']').next().unwrap_or("");
    }
    // Only strip a port when there is exactly one colon (bare IPv6 has several)
    if host_port.matches(':').count() == 1 {
        return host_port.split(':').next().unwrap_or("");
    }
    host_port
}

fn check_hostname(hostname: &str, allow_private_networks: bool) -> Result<(), String> {
    if hostname.is_empty() {
        return Err("URL must contain a valid hostname".to_string());
    }
    if allow_private_networks {
        return Ok(());
    }

    // Reject .local, .internal, .localhost domains
    let lower = hostname.to_ascii_lowercase();
    if lower.ends_with(".local")
        || lower.ends_with(".internal")
        || lower.ends_with(".localhost")
        || lower == "localhost"
    {
        return Err("URL hostname cannot be a local/internal domain".to_string());
    }

    if let Ok(ip) = hostname.parse::<IpAddr>()
        && is_private_ip(ip)
    {
        return Err("URL hostname cannot be a private or loopback IP address".to_string());
    }
    Ok(())
}

/// Checks if an IP address is in a private range
fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            // 10.0.0.0/8
            octets[0] == 10
                // 172.16.0.0/12
                || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                // 192.168.0.0/16
                || (octets[0] == 192 && octets[1] == 168)
                // 169.254.0.0/16 (link-local)
                || (octets[0] == 169 && octets[1] == 254)
                // 127.0.0.0/8 (loopback)
                || octets[0] == 127
                // 0.0.0.0/8
                || octets[0] == 0
        }
        IpAddr::V6(v6) => {
            // ::1 (loopback)
            v6.is_loopback()
                // fc00::/7 (unique local)
                || v6.is_unique_local()
                // fe80::/10 (link-local)
                || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_https_url() {
        assert!(validate_outbound_url("https://github.com", false).is_ok());
        assert!(validate_outbound_url("https://gitlab.com/api/v4", false).is_ok());
        assert!(validate_outbound_url("https://example.com:8080/path", false).is_ok());
    }

    #[test]
    fn test_http_rejected() {
        assert!(validate_outbound_url("http://github.com", false).is_err());
    }

    #[test]
    fn test_localhost_rejected() {
        assert!(validate_outbound_url("https://localhost", false).is_err());
        assert!(validate_outbound_url("https://127.0.0.1", false).is_err());
        assert!(validate_outbound_url("https://[::1]:8443/", false).is_err());
        assert!(validate_outbound_url("https://0.0.0.0", false).is_err());
    }

    #[test]
    fn test_local_domains_rejected() {
        assert!(validate_outbound_url("https://example.local", false).is_err());
        assert!(validate_outbound_url("https://server.internal", false).is_err());
        assert!(validate_outbound_url("https://host.localhost", false).is_err());
    }

    #[test]
    fn test_private_ips_rejected() {
        assert!(validate_outbound_url("https://10.0.0.1", false).is_err());
        assert!(validate_outbound_url("https://172.16.0.1", false).is_err());
        assert!(validate_outbound_url("https://172.31.255.255", false).is_err());
        assert!(validate_outbound_url("https://192.168.1.1", false).is_err());
        assert!(validate_outbound_url("https://169.254.1.1", false).is_err());
    }

    #[test]
    fn test_userinfo_does_not_hide_host() {
        assert!(validate_outbound_url("https://user:pass@192.168.1.1/x", false).is_err());
        assert!(validate_outbound_url("https://user@example.com/x", false).is_ok());
    }

    #[test]
    fn test_invalid_url_format() {
        assert!(validate_outbound_url("https://", false).is_err());
    }

    #[test]
    fn test_private_networks_allowed_when_configured() {
        assert!(validate_outbound_url("http://gitlab.lan/group", true).is_ok());
        assert!(validate_outbound_url("https://192.168.1.10:8443", true).is_ok());
        assert!(validate_outbound_url("ftp://gitlab.lan", true).is_err());
    }

    #[test]
    fn test_git_remote_rejects_dangerous_forms() {
        assert!(validate_git_remote("--upload-pack=touch /tmp/x", false).is_err());
        assert!(validate_git_remote("file:///etc", false).is_err());
        assert!(validate_git_remote("ext::sh -c id", false).is_err());
        assert!(validate_git_remote("https://github.com/a/b.git", false).is_ok());
        assert!(validate_git_remote("git@github.com:a/b.git", false).is_ok());
        assert!(validate_git_remote("ssh://git@github.com/a/b.git", false).is_ok());
        assert!(validate_git_remote("http://github.com/a/b.git", false).is_err());
        assert!(validate_git_remote("git@127.0.0.1:a/b.git", false).is_err());
        assert!(validate_git_remote("git@gitlab.lan:a/b.git", true).is_ok());
    }
}
