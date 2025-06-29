use anyhow::{Result, bail};
use uuid::Uuid;

use crate::Ratelimit;

#[derive(Clone)]
pub struct FixedWindowCounter {
    capacity: u32,
    window_size: u32, // window size in seconds
    client: redis::Client,
    key: Uuid,
}

impl FixedWindowCounter {
    pub fn new(capacity: u32, window_size: u32) -> FixedWindowCounter {
        let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
        let key = Uuid::new_v4();
        FixedWindowCounter {
            capacity,
            window_size,
            client,
            key,
        }
    }

    fn window(&self) -> String {
        let ts = chrono::Local::now().timestamp();
        let ws = self.window_size as i64;
        // round down by fixed window size
        let suffix = (ts / ws) * ws;
        format!("{}_{}", self.key, suffix)
    }
}

impl Ratelimit for FixedWindowCounter {
    fn try_accept(&self) -> Result<()> {
        let mut conn = self.client.get_connection().unwrap();
        let key = self.window();
        let (count,): (u32,) = redis::pipe()
            .incr(&key, 1)
            .expire(key, self.window_size as i64)
            .ignore()
            .query(&mut conn)
            .unwrap();
        if count > self.capacity {
            bail!("too many requests")
        }
        Ok(())
    }
}
