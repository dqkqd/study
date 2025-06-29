use anyhow::Result;

mod fixed_window_counter;
mod sliding_window_log;
mod token_bucket;

use fixed_window_counter::FixedWindowCounter;
use sliding_window_log::SlidingWindowLog;
use token_bucket::TokenBucket;

pub trait Ratelimit {
    fn try_accept(&self) -> Result<()>;
}

#[derive(Clone)]
pub enum Ratelimiter {
    TokenBucket(TokenBucket),
    FixedWindowCounter(FixedWindowCounter),
    SlidingWindowLog(SlidingWindowLog),
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

    pub fn sliding_window_log(config: &appconfig::RatelimiterConfig) -> Ratelimiter {
        Ratelimiter::SlidingWindowLog(SlidingWindowLog::new(
            config.sliding_window_log.capacity,
            config.sliding_window_log.window_size,
        ))
    }
}
impl Ratelimit for Ratelimiter {
    fn try_accept(&self) -> Result<()> {
        match self {
            Ratelimiter::TokenBucket(r) => r.try_accept(),
            Ratelimiter::FixedWindowCounter(r) => r.try_accept(),
            Ratelimiter::SlidingWindowLog(r) => r.try_accept(),
        }
    }
}
