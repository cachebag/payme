use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_window: u32,
    pub window_duration: Duration,
    pub cleanup_interval: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 100,
            window_duration: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(300),
        }
    }
}

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    max_tokens: f64,
    refill_rate: f64,
}

impl TokenBucket {
    fn new(max_tokens: u32, window: Duration) -> Self {
        let refill_rate = max_tokens as f64 / window.as_secs_f64();
        Self {
            tokens: max_tokens as f64,
            last_refill: Instant::now(),
            max_tokens: max_tokens as f64,
            refill_rate,
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
        self.last_refill = now;
    }

    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn available_tokens(&mut self) -> u32 {
        self.refill();
        self.tokens.floor() as u32
    }

    fn time_until_token(&self) -> Duration {
        if self.tokens >= 1.0 {
            return Duration::from_secs(0);
        }
        let tokens_needed = 1.0 - self.tokens;
        let seconds = tokens_needed / self.refill_rate;
        Duration::from_secs_f64(seconds)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RateLimitKey {
    Ip(IpAddr),
    User(i64),
    IpUser(IpAddr, i64),
}

struct RateLimitEntry {
    bucket: TokenBucket,
    last_access: Instant,
}

pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<RwLock<HashMap<RateLimitKey, RateLimitEntry>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let limiter = Self {
            config,
            buckets: Arc::new(RwLock::new(HashMap::new())),
        };

        limiter.start_cleanup_task();
        limiter
    }

    fn start_cleanup_task(&self) {
        let buckets = Arc::clone(&self.buckets);
        let cleanup_interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                let mut buckets = buckets.write().await;
                let now = Instant::now();
                buckets.retain(|_, entry| {
                    now.duration_since(entry.last_access) < Duration::from_secs(600)
                });
            }
        });
    }

    pub async fn check_rate_limit_ip(&self, ip: IpAddr) -> RateLimitResult {
        self.check_limit(RateLimitKey::Ip(ip)).await
    }

    pub async fn check_rate_limit_user(&self, user_id: i64) -> RateLimitResult {
        self.check_limit(RateLimitKey::User(user_id)).await
    }

    pub async fn check_rate_limit_combined(&self, ip: IpAddr, user_id: i64) -> RateLimitResult {
        let ip_result = self.check_rate_limit_ip(ip).await;
        if !ip_result.allowed {
            return ip_result;
        }

        let user_result = self.check_rate_limit_user(user_id).await;
        if !user_result.allowed {
            return user_result;
        }

        let combined_key = RateLimitKey::IpUser(ip, user_id);
        self.check_limit(combined_key).await
    }

    async fn check_limit(&self, key: RateLimitKey) -> RateLimitResult {
        let mut buckets = self.buckets.write().await;

        let entry = buckets.entry(key).or_insert_with(|| RateLimitEntry {
            bucket: TokenBucket::new(self.config.requests_per_window, self.config.window_duration),
            last_access: Instant::now(),
        });

        entry.last_access = Instant::now();

        let allowed = entry.bucket.try_consume();
        let remaining = entry.bucket.available_tokens();
        let retry_after = if allowed {
            None
        } else {
            Some(entry.bucket.time_until_token())
        };

        RateLimitResult {
            allowed,
            limit: self.config.requests_per_window,
            remaining,
            retry_after,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub retry_after: Option<Duration>,
}
