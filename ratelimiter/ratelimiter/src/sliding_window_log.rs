use anyhow::{Result, bail};
use uuid::Uuid;

use crate::Ratelimit;

#[derive(Clone)]
pub struct SlidingWindowLog {
    capacity: u32,
    window_size: u32, // window size in seconds
    client: redis::Client,
    key: Uuid,
}

impl SlidingWindowLog {
    pub fn new(capacity: u32, window_size: u32) -> SlidingWindowLog {
        let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
        let key = Uuid::new_v4();
        SlidingWindowLog {
            capacity,
            window_size,
            client,
            key,
        }
    }
}

impl Ratelimit for SlidingWindowLog {
    fn try_accept(&self) -> Result<()> {
        let mut conn = self.client.get_connection().unwrap();
        let now = chrono::Local::now().timestamp_micros();

        let key = self.key.to_string();

        let (count,): (u32,) = redis::pipe()
            // remove stale data
            .zrembyscore(&key, 0, now - (self.window_size * 1_000_000) as i64)
            .ignore()
            // add new data
            .zadd(&key, now, now)
            .ignore()
            .zcard(&key)
            .query(&mut conn)
            .unwrap();

        if count > self.capacity {
            bail!("too many requests")
        }
        Ok(())
    }
}
