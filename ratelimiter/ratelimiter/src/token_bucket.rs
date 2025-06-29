use anyhow::{Result, bail};
use uuid::Uuid;

use crate::Ratelimit;

#[derive(Clone)]
pub struct TokenBucket {
    capacity: u32,
    rate: u64,
    client: redis::Client,
    key: Uuid,
}

impl TokenBucket {
    pub fn new(capacity: u32, rate: u64) -> TokenBucket {
        let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
        let key = Uuid::new_v4();
        TokenBucket {
            capacity,
            rate,
            client,
            key,
        }
    }
}

impl Ratelimit for TokenBucket {
    fn try_accept(&self) -> Result<()> {
        let mut conn = self.client.get_connection().unwrap();
        let script = redis::Script::new(
            r#"
local capacity = tonumber(ARGV[1])
local expire = tonumber(ARGV[2])
local key = ARGV[3]
local counter = redis.call('GET', key)
if counter == false then
    redis.call('SET', key, 0, 'EX', expire)
end
counter = redis.call('INCR', key)
return counter <= capacity
"#,
        );
        let ok: bool = script
            .arg(self.capacity)
            .arg(self.rate)
            .arg(self.key.to_string())
            .invoke(&mut conn)
            .unwrap();
        if !ok {
            bail!("not enough tokens")
        }
        Ok(())
    }
}
