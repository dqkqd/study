mod token_bucket;
use hyper::Request;
use redis::Connection;
use token_bucket::TokenBucket;

use crate::Result;

pub(crate) enum Ratelimiter {
    TokenBucket(TokenBucket),
}

impl Ratelimiter {
    pub(crate) fn new() -> Result<Ratelimiter> {
        let rate_limiter = Ratelimiter::TokenBucket(TokenBucket::new()?);
        Ok(rate_limiter)
    }
    pub(crate) fn accepted<R>(&mut self, req: &Request<R>) -> bool {
        match self {
            Ratelimiter::TokenBucket(token_bucket) => token_bucket.accepted(req),
        }
    }
}

pub(super) fn get_env<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    std::env::var(key)
        .ok()
        .and_then(|value| std::str::FromStr::from_str(&value).ok())
        .unwrap_or(default)
}

pub(super) fn get_redis_connection() -> Result<Connection> {
    let redis_client = redis::Client::open("redis://cache:6379")?;
    let connection = redis_client.get_connection()?;
    Ok(connection)
}
