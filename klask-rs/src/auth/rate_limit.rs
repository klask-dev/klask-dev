//! Bounded fixed-window rate limiting for authentication endpoints.
//!
//! Memory is capped: expired windows are purged, and when the table is still
//! full the oldest window is evicted. An attacker rotating usernames or
//! addresses therefore cannot grow the table without limit.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::HeaderMap;
use axum::http::request::Parts;
use std::convert::Infallible;

/// Peer address of the TCP connection, when the server provides it
/// (`into_make_service_with_connect_info`). Always succeeds: a missing address
/// falls back to a shared bucket in `client_key`.
pub struct PeerAddr(pub Option<SocketAddr>);

impl<S: Send + Sync> FromRequestParts<S> for PeerAddr {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(PeerAddr(
            parts.extensions.get::<ConnectInfo<SocketAddr>>().map(|info| info.0),
        ))
    }
}

/// Fixed-window counter keyed by an arbitrary string.
pub struct FixedWindowLimiter {
    windows: Mutex<HashMap<String, Window>>,
    max_hits: u32,
    window: Duration,
    max_entries: usize,
}

#[derive(Clone, Copy)]
struct Window {
    count: u32,
    started: Instant,
}

impl FixedWindowLimiter {
    pub fn new(max_hits: u32, window: Duration, max_entries: usize) -> Self {
        Self { windows: Mutex::new(HashMap::new()), max_hits, window, max_entries: max_entries.max(1) }
    }

    /// Returns `Err(retry_after_secs)` when `key` has used its budget for the
    /// current window. Does not count anything.
    pub fn check(&self, key: &str) -> Result<(), u64> {
        let now = Instant::now();
        let windows = self.lock();
        match windows.get(key) {
            Some(w) if now.duration_since(w.started) < self.window && w.count >= self.max_hits => {
                Err(self.retry_after(w, now))
            }
            _ => Ok(()),
        }
    }

    /// Counts one hit against `key`.
    pub fn record(&self, key: &str) {
        let now = Instant::now();
        let mut windows = self.lock();

        if !windows.contains_key(key) && windows.len() >= self.max_entries {
            // Drop expired windows first, then the oldest one if still full.
            windows.retain(|_, w| now.duration_since(w.started) < self.window);
            if windows.len() >= self.max_entries
                && let Some(oldest) = windows.iter().min_by_key(|(_, w)| w.started).map(|(k, _)| k.clone())
            {
                windows.remove(&oldest);
            }
        }

        let entry = windows.entry(key.to_string()).or_insert(Window { count: 0, started: now });
        if now.duration_since(entry.started) >= self.window {
            entry.count = 0;
            entry.started = now;
        }
        entry.count = entry.count.saturating_add(1);
    }

    /// Slow-down delay for `key`, without refusing anything: zero while the
    /// budget is not exhausted, then 250 ms per hit beyond it, capped at 5 s.
    /// Used for limits shared by many legitimate users, so that a misjudged
    /// key can slow traffic down but never lock everyone out.
    pub fn penalty(&self, key: &str) -> Duration {
        const STEP: Duration = Duration::from_millis(250);
        const CAP: Duration = Duration::from_secs(5);
        let now = Instant::now();
        let windows = self.lock();
        match windows.get(key) {
            Some(w) if now.duration_since(w.started) < self.window && w.count >= self.max_hits => {
                let excess = w.count - self.max_hits + 1;
                STEP.saturating_mul(excess).min(CAP)
            }
            _ => Duration::ZERO,
        }
    }

    /// Forgets `key` (e.g. after a successful login).
    pub fn reset(&self, key: &str) {
        self.lock().remove(key);
    }

    /// Number of tracked keys; exposed for tests.
    #[cfg(test)]
    pub fn tracked_keys(&self) -> usize {
        self.lock().len()
    }

    fn retry_after(&self, w: &Window, now: Instant) -> u64 {
        self.window.saturating_sub(now.duration_since(w.started)).as_secs().max(1)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Window>> {
        // A poisoned lock only means another thread panicked mid-update; the
        // map is still usable, so keep serving rather than failing every login.
        self.windows.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Limits applied to the unauthenticated auth endpoints.
pub struct AuthRateLimits {
    /// Failed logins for one (client, username) pair. Keyed by client too, so
    /// an attacker cannot lock a victim's account from another address.
    pub login_per_client_and_user: FixedWindowLimiter,
    /// Failed logins for one client across all usernames. Slow-down only: a
    /// shared address (NAT, misconfigured proxy) must never lock out everyone.
    pub login_per_client: FixedWindowLimiter,
    /// Registration and first-run setup attempts per client. Slow-down only.
    pub registration_per_client: FixedWindowLimiter,
}

const MAX_TRACKED_KEYS: usize = 100_000;

impl Default for AuthRateLimits {
    fn default() -> Self {
        Self {
            login_per_client_and_user: FixedWindowLimiter::new(10, Duration::from_secs(900), MAX_TRACKED_KEYS),
            login_per_client: FixedWindowLimiter::new(50, Duration::from_secs(900), MAX_TRACKED_KEYS),
            registration_per_client: FixedWindowLimiter::new(10, Duration::from_secs(3600), MAX_TRACKED_KEYS),
        }
    }
}

/// Client address used for rate limiting.
///
/// `trusted_hops` is the number of reverse proxies between the internet and
/// the backend that append to `X-Forwarded-For` (`TRUSTED_PROXY_HOPS`):
/// - `0`: the header is ignored, the peer address is used;
/// - `N`: the N-th entry counted from the right is the client as seen by the
///   outermost trusted proxy. Entries further left are supplied by the client
///   and are never trusted.
///
/// Falls back to the peer address when the header has fewer than N entries or
/// the selected entry is not an IP address. Without a peer address either, all
/// such requests share one bucket.
pub fn client_key(headers: &HeaderMap, peer: Option<SocketAddr>, trusted_hops: usize) -> String {
    if let Some(ip) = forwarded_client(headers, trusted_hops) {
        return ip.to_string();
    }
    match peer {
        Some(addr) => addr.ip().to_string(),
        None => "unknown".to_string(),
    }
}

/// The client address selected by `trusted_hops` from all `X-Forwarded-For` values.
fn forwarded_client(headers: &HeaderMap, trusted_hops: usize) -> Option<IpAddr> {
    if trusted_hops == 0 {
        return None;
    }
    let joined =
        headers.get_all("x-forwarded-for").iter().filter_map(|v| v.to_str().ok()).collect::<Vec<_>>().join(",");
    let entries: Vec<&str> = joined.split(',').map(str::trim).collect();
    if entries.len() < trusted_hops {
        return None;
    }
    entries[entries.len() - trusted_hops].parse::<IpAddr>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn blocks_after_budget_and_resets() {
        let limiter = FixedWindowLimiter::new(3, Duration::from_secs(60), 10);
        for _ in 0..3 {
            assert!(limiter.check("a").is_ok());
            limiter.record("a");
        }
        let retry = limiter.check("a").unwrap_err();
        assert!((1..=60).contains(&retry));
        // Other keys are unaffected
        assert!(limiter.check("b").is_ok());
        limiter.reset("a");
        assert!(limiter.check("a").is_ok());
    }

    #[test]
    fn expired_window_allows_again() {
        let limiter = FixedWindowLimiter::new(1, Duration::from_millis(20), 10);
        limiter.record("a");
        assert!(limiter.check("a").is_err());
        std::thread::sleep(Duration::from_millis(40));
        assert!(limiter.check("a").is_ok());
    }

    #[test]
    fn table_is_bounded_under_key_rotation() {
        let limiter = FixedWindowLimiter::new(5, Duration::from_secs(600), 100);
        for i in 0..10_000 {
            limiter.record(&format!("user-{i}"));
        }
        assert!(limiter.tracked_keys() <= 100);
    }

    fn xff(value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static(value));
        headers
    }

    fn peer() -> Option<SocketAddr> {
        Some("10.0.0.5:4444".parse().expect("valid socket address"))
    }

    #[test]
    fn hops_zero_ignores_forwarded_for() {
        let headers = xff("203.0.113.9, 198.51.100.7");
        assert_eq!(client_key(&headers, peer(), 0), "10.0.0.5");
    }

    #[test]
    fn hops_one_takes_the_rightmost_entry() {
        // nginx appends the address it received from the client
        let headers = xff("203.0.113.9, 198.51.100.7");
        assert_eq!(client_key(&headers, peer(), 1), "198.51.100.7");
    }

    #[test]
    fn hops_two_skips_the_outer_proxy() {
        // ingress appends its own address after nginx appended the client's
        let headers = xff("203.0.113.9, 198.51.100.7");
        assert_eq!(client_key(&headers, peer(), 2), "203.0.113.9");
    }

    #[test]
    fn short_header_falls_back_to_peer() {
        let headers = xff("203.0.113.9");
        assert_eq!(client_key(&headers, peer(), 2), "10.0.0.5");
        assert_eq!(client_key(&HeaderMap::new(), peer(), 1), "10.0.0.5");
    }

    #[test]
    fn invalid_selected_entry_falls_back_to_peer() {
        let headers = xff("203.0.113.9, not-an-ip");
        assert_eq!(client_key(&headers, peer(), 1), "10.0.0.5");
    }

    #[test]
    fn forged_leftmost_entries_are_not_used() {
        // A client sends a fake value; the proxy appends the real address.
        let headers = xff("1.1.1.1, 2.2.2.2, 198.51.100.7");
        assert_eq!(client_key(&headers, peer(), 1), "198.51.100.7");
        assert_eq!(client_key(&headers, peer(), 3), "1.1.1.1");
    }

    #[test]
    fn multiple_header_lines_are_joined() {
        let mut headers = HeaderMap::new();
        headers.append("x-forwarded-for", HeaderValue::from_static("203.0.113.9"));
        headers.append("x-forwarded-for", HeaderValue::from_static("198.51.100.7"));
        assert_eq!(client_key(&headers, peer(), 1), "198.51.100.7");
    }

    #[test]
    fn no_peer_and_no_usable_header_share_one_bucket() {
        assert_eq!(client_key(&HeaderMap::new(), None, 1), "unknown");
    }

    #[test]
    fn penalty_is_zero_under_budget_then_grows_and_caps() {
        let limiter = FixedWindowLimiter::new(2, Duration::from_secs(600), 10);
        assert_eq!(limiter.penalty("a"), Duration::ZERO);
        limiter.record("a");
        limiter.record("a");
        // Exactly at the budget: first delayed step
        assert_eq!(limiter.penalty("a"), Duration::from_millis(250));
        for _ in 0..100 {
            limiter.record("a");
        }
        assert_eq!(limiter.penalty("a"), Duration::from_secs(5));
        // Penalties never refuse: check() still reflects the budget only
        assert!(limiter.check("a").is_err());
    }
}
