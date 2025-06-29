use anyhow::{Result, bail};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::Ratelimit;

#[derive(Clone)]
pub struct TokenBucketNaive {
    capacity: u32,
    tokens: Arc<Mutex<u32>>,
    rate: u64,
}

impl TokenBucketNaive {
    pub fn new(capacity: u32, rate: u64) -> TokenBucketNaive {
        let tokens = Arc::new(Mutex::new(0));
        let bucket = TokenBucketNaive {
            capacity,
            tokens,
            rate,
        };
        bucket.run_refiller();
        bucket
    }

    fn run_refiller(&self) {
        let tokens = self.tokens.clone();
        let rate = self.rate;
        let capacity = self.capacity;
        thread::spawn(move || {
            loop {
                if let Ok(mut tokens) = tokens.lock() {
                    *tokens = capacity;
                }
                thread::sleep(Duration::from_secs(rate));
            }
        });
    }
}

impl Ratelimit for TokenBucketNaive {
    fn try_accept(&self) -> Result<()> {
        let mut tokens = match self.tokens.lock() {
            Ok(tokens) => tokens,
            Err(_) => bail!("failed to acquire tokens lock"),
        };

        if *tokens == 0 {
            bail!("not enough tokens")
        }

        *tokens -= 1;
        Ok(())
    }
}
