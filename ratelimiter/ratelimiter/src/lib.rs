use anyhow::Result;

mod fixed_window_counter;
mod token_bucket;

use fixed_window_counter::FixedWindowCounter;
use token_bucket::TokenBucket;

pub trait Ratelimit {
    fn try_accept(&self) -> Result<()>;
}

#[derive(Clone)]
pub enum Ratelimiter {
    TokenBucket(TokenBucket),
    FixedWindowCounter(FixedWindowCounter),
}
impl Ratelimiter {
    pub fn token_bucket(config: &appconfig::RatelimiterConfig) -> Ratelimiter {
        Ratelimiter::TokenBucket(TokenBucket::new(
            config.token_bucket.capacity,
            config.token_bucket.rate,
        ))
    }

    pub fn fixed_window_counter(config: &appconfig::RatelimiterConfig) -> Ratelimiter {
        Ratelimiter::FixedWindowCounter(FixedWindowCounter::new(
            config.fixed_window_counter.capacity,
            config.fixed_window_counter.window_size,
        ))
    }
}
impl Ratelimit for Ratelimiter {
    fn try_accept(&self) -> Result<()> {
        match self {
            Ratelimiter::TokenBucket(r) => r.try_accept(),
            Ratelimiter::FixedWindowCounter(r) => r.try_accept(),
        }
    }
}
