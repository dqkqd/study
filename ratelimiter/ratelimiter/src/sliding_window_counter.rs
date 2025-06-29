use anyhow::{Result, bail};
use uuid::Uuid;

use crate::Ratelimit;

#[derive(Clone)]
pub struct SlidingWindowCounter {
    capacity: u32,
    window_size: u32, // window size in seconds
    batch_size: u64,  // batch size in microseconds
    client: redis::Client,
    key: Uuid,
}

impl SlidingWindowCounter {
    pub fn new(capacity: u32, window_size: u32, batch_ratio: f32) -> SlidingWindowCounter {
        let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
        let key = Uuid::new_v4();

        // batch size as microseconds
        let batch_size = (window_size as f32 * batch_ratio * 1_000_000.0) as u64;

        SlidingWindowCounter {
            capacity,
            window_size,
            batch_size,
            client,
            key,
        }
    }

    fn batch(&self) -> String {
        let ts = chrono::Local::now().timestamp_micros();
        let bs = self.batch_size as i64;
        // round down by fixed batch size
        let suffix = (ts / bs) * bs;
        format!("{}:{}", self.key, suffix)
    }

    fn window(&self) -> String {
        let ts = chrono::Local::now().timestamp();
        let ws = self.window_size as i64;
        // round down by fixed batch size
        let suffix = (ts / ws) * ws;
        format!("{}:{}", self.key, suffix)
    }
}

impl Ratelimit for SlidingWindowCounter {
    fn try_accept(&self) -> Result<()> {
        let mut conn = self.client.get_connection().unwrap();

        let window = self.window();
        let batch = self.batch();

        let (values,): (Vec<u32>,) = redis::pipe()
            // increment batch in the window
            .hincr(&window, &batch, 1)
            .ignore()
            // expire the whole window
            .cmd("EXPIRE")
            .arg(&window)
            .arg(self.window_size)
            .arg("NX")
            .ignore()
            // get all values in windows
            .hvals(&window)
            .query(&mut conn)
            .unwrap();

        let count: u32 = values.iter().sum();
        if count > self.capacity {
            bail!("too many requests")
        }
        Ok(())
    }
}
