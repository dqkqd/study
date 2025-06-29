use anyhow::Result;

mod fixed_window_counter;
mod token_bucket_naive;
mod token_bucket_valkey;

use fixed_window_counter::FixedWindowCounter;
use token_bucket_naive::TokenBucketNaive;
use token_bucket_valkey::TokenBucketValkey;

pub trait Ratelimit {
    fn try_accept(&self) -> Result<()>;
}

#[derive(Clone)]
pub enum Ratelimiter {
    TokenBucketNaive(TokenBucketNaive),
    TokenBucketValkey(TokenBucketValkey),
    FixedWindowCounter(FixedWindowCounter),
}
impl Ratelimiter {
    pub fn token_bucket_naive(config: &appconfig::RatelimiterConfig) -> Ratelimiter {
        Ratelimiter::TokenBucketNaive(TokenBucketNaive::new(
            config.token_bucket.capacity,
            config.token_bucket.rate,
        ))
    }

    pub fn token_bucket_valkey(config: &appconfig::RatelimiterConfig) -> Ratelimiter {
        Ratelimiter::TokenBucketValkey(TokenBucketValkey::new(
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
            Ratelimiter::TokenBucketNaive(r) => r.try_accept(),
            Ratelimiter::TokenBucketValkey(r) => r.try_accept(),
            Ratelimiter::FixedWindowCounter(r) => r.try_accept(),
        }
    }
}
