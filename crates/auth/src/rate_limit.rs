use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};

const WINDOW: Duration = Duration::from_secs(5 * 60);
const IP_LIMIT: u32 = 30;
const LOGIN_LIMIT: u32 = 10;
const MAX_BUCKETS: usize = 20_000;

#[derive(Clone, Eq, Hash, PartialEq)]
enum RateKey {
    Ip(IpAddr),
    Login([u8; 32]),
}

struct RateWindow {
    started_at: Instant,
    attempts: u32,
}

/// Bounded process-local login rate limiter.
///
/// Multi-instance deployments must use a shared limiter; these buckets
/// coordinate requests only within the current API process.
#[derive(Clone, Default)]
pub struct LoginRateLimiter {
    buckets: Arc<Mutex<HashMap<RateKey, RateWindow>>>,
}

impl LoginRateLimiter {
    /// Creates an empty limiter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks and records an attempt against the peer IP and normalized login.
    ///
    /// The login value is SHA-256 hashed before it is used as an in-memory key.
    pub fn check(&self, ip: IpAddr, normalized_login: &str) -> bool {
        self.check_at(Instant::now(), ip, normalized_login)
    }

    /// Removes the login-specific failure bucket after a successful password check.
    /// The IP bucket remains to limit broad credential-stuffing attempts.
    pub fn record_success(&self, normalized_login: &str) {
        let digest: [u8; 32] = Sha256::digest(normalized_login.as_bytes()).into();
        let mut buckets = self
            .buckets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        buckets.remove(&RateKey::Login(digest));
    }

    fn check_at(&self, now: Instant, ip: IpAddr, normalized_login: &str) -> bool {
        let digest: [u8; 32] = Sha256::digest(normalized_login.as_bytes()).into();
        let ip_key = RateKey::Ip(ip);
        let login_key = RateKey::Login(digest);
        let mut buckets = self
            .buckets
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        buckets.retain(|_, bucket| now.saturating_duration_since(bucket.started_at) < WINDOW);
        if is_blocked(&buckets, &ip_key, now, IP_LIMIT)
            || is_blocked(&buckets, &login_key, now, LOGIN_LIMIT)
        {
            return false;
        }

        let missing = usize::from(!buckets.contains_key(&ip_key))
            + usize::from(!buckets.contains_key(&login_key));
        if buckets.len().saturating_add(missing) > MAX_BUCKETS {
            return false;
        }

        increment(&mut buckets, ip_key, now);
        increment(&mut buckets, login_key, now);
        true
    }
}

fn is_blocked(
    buckets: &HashMap<RateKey, RateWindow>,
    key: &RateKey,
    now: Instant,
    limit: u32,
) -> bool {
    buckets.get(key).is_some_and(|bucket| {
        now.saturating_duration_since(bucket.started_at) < WINDOW && bucket.attempts >= limit
    })
}

fn increment(buckets: &mut HashMap<RateKey, RateWindow>, key: RateKey, now: Instant) {
    match buckets.get_mut(&key) {
        Some(bucket) if now.saturating_duration_since(bucket.started_at) < WINDOW => {
            bucket.attempts = bucket.attempts.saturating_add(1);
        }
        Some(bucket) => {
            bucket.started_at = now;
            bucket.attempts = 1;
        }
        None => {
            buckets.insert(
                key,
                RateWindow {
                    started_at: now,
                    attempts: 1,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::IpAddr,
        str::FromStr,
        time::{Duration, Instant},
    };

    use super::{LoginRateLimiter, LOGIN_LIMIT, WINDOW};

    fn test_ip() -> IpAddr {
        IpAddr::from_str("192.0.2.1")
            .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
    }

    #[test]
    fn login_bucket_blocks_after_limit_and_expires() {
        let limiter = LoginRateLimiter::new();
        let ip = test_ip();
        let start = Instant::now();

        for _ in 0..LOGIN_LIMIT {
            assert!(limiter.check_at(start, ip, "alice"));
        }
        assert!(!limiter.check_at(start, ip, "alice"));
        assert!(limiter.check_at(start + WINDOW + Duration::from_secs(1), ip, "alice"));
    }

    #[test]
    fn different_logins_have_independent_login_buckets() {
        let limiter = LoginRateLimiter::new();
        let ip = test_ip();
        let start = Instant::now();

        for index in 0..LOGIN_LIMIT {
            assert!(limiter.check_at(start, ip, &format!("user-{index}")));
        }
        assert!(limiter.check_at(start, ip, "another-user"));
    }

    #[test]
    fn successful_login_clears_only_its_login_bucket() {
        let limiter = LoginRateLimiter::new();
        let ip = test_ip();
        let start = Instant::now();

        for _ in 0..LOGIN_LIMIT {
            assert!(limiter.check_at(start, ip, "alice"));
        }
        limiter.record_success("alice");
        assert!(limiter.check_at(start, ip, "alice"));
    }
}
