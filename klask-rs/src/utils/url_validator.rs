//! URL validation to prevent SSRF attacks

use std::net::IpAddr;

/// Validates an external URL to prevent SSRF attacks
///
/// This function:
/// 1. Requires HTTPS scheme
/// 2. Rejects localhost, 127.0.0.1, ::1, and 0.0.0.0
/// 3. Rejects private IP ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 169.254.0.0/16)
/// 4. Rejects domains ending with .local, .internal, or .localhost
pub fn validate_external_url(url: &str) -> Result<(), String> {
    // Must use HTTPS
    if !url.starts_with("https://") {
        return Err("URL must use HTTPS scheme".to_string());
    }

    // Extract hostname from URL
    // Remove the https:// prefix and get everything before the first / or :
    let url_after_scheme = &url[8..]; // Skip "https://"
    let hostname = url_after_scheme.split('/').next().unwrap_or("").split(':').next().unwrap_or("");

    if hostname.is_empty() {
        return Err("URL must contain a valid hostname".to_string());
    }

    // Reject .local, .internal, .localhost domains
    if hostname.ends_with(".local")
        || hostname.ends_with(".internal")
        || hostname.ends_with(".localhost")
        || hostname == "localhost"
    {
        return Err("URL hostname cannot be a local/internal domain".to_string());
    }

    // Reject loopback IPs and special cases
    if hostname == "127.0.0.1" || hostname == "::1" || hostname == "0.0.0.0" || hostname == "localhost" {
        return Err("URL hostname cannot be a loopback address".to_string());
    }

    // Try to parse as IP address to check for private ranges
    if let Ok(ip) = hostname.parse::<IpAddr>()
        && is_private_ip(ip)
    {
        return Err("URL hostname cannot be a private IP address".to_string());
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
        assert!(validate_external_url("https://github.com").is_ok());
        assert!(validate_external_url("https://gitlab.com/api/v4").is_ok());
        assert!(validate_external_url("https://example.com:8080/path").is_ok());
    }

    #[test]
    fn test_http_rejected() {
        assert!(validate_external_url("http://github.com").is_err());
    }

    #[test]
    fn test_localhost_rejected() {
        assert!(validate_external_url("https://localhost").is_err());
        assert!(validate_external_url("https://127.0.0.1").is_err());
        assert!(validate_external_url("https://::1").is_err());
        assert!(validate_external_url("https://0.0.0.0").is_err());
    }

    #[test]
    fn test_local_domains_rejected() {
        assert!(validate_external_url("https://example.local").is_err());
        assert!(validate_external_url("https://server.internal").is_err());
        assert!(validate_external_url("https://host.localhost").is_err());
    }

    #[test]
    fn test_private_ips_rejected() {
        assert!(validate_external_url("https://10.0.0.1").is_err());
        assert!(validate_external_url("https://172.16.0.1").is_err());
        assert!(validate_external_url("https://172.31.255.255").is_err());
        assert!(validate_external_url("https://192.168.1.1").is_err());
        assert!(validate_external_url("https://169.254.1.1").is_err());
    }

    #[test]
    fn test_invalid_url_format() {
        assert!(validate_external_url("https://").is_err());
    }
}
