use anyhow::Result;

mod token_bucket_naive;
mod token_bucket_valkey;

use token_bucket_naive::TokenBucketNaive;
use token_bucket_valkey::TokenBucketValkey;

pub trait Ratelimit {
    fn try_accept(&self) -> Result<()>;
}

#[derive(Clone)]
pub enum Ratelimiter {
    TokenBucketNaive(TokenBucketNaive),
    TokenBucketValkey(TokenBucketValkey),
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
}
impl Ratelimit for Ratelimiter {
    fn try_accept(&self) -> Result<()> {
        match self {
            Ratelimiter::TokenBucketNaive(r) => r.try_accept(),
            Ratelimiter::TokenBucketValkey(r) => r.try_accept(),
        }
    }
}
