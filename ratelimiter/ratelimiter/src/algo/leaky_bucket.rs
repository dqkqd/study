use std::{num::NonZeroUsize, time::Duration};

use hyper::body::Bytes;
use redis::Commands;

use super::{accepted, get_env, get_redis_connection};
use crate::{
    algo::too_many_request, convert_request_to_bytes, forward_bytes_to_server, FullResponse,
    IncomingRequest, Result,
};

pub struct LeakyBucket {
    limit: u64,
    outflow_rate: u64,
    redis_connection: redis::Connection,
}

impl LeakyBucket {
    pub(super) fn new() -> Result<LeakyBucket> {
        let redis_connection = get_redis_connection()?;
        let limit: u64 = get_env("LIMIT", 10);
        let outflow_rate: u64 = get_env("RATE", 3);
        Ok(LeakyBucket {
            limit,
            outflow_rate,
            redis_connection,
        })
    }

    pub(super) fn background_task(self) -> Result<()> {
        tokio::task::spawn(async move {
            let rate = self.outflow_rate as usize;
            let mut connection = self.redis_connection;
            loop {
                if let Err(e) = Self::leak(&mut connection, rate).await {
                    dbg!(e);
                }
                println!("Wait");
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        });
        Ok(())
    }

    async fn leak(connection: &mut redis::Connection, count: usize) -> Result<()> {
        let bucket_key = Self::key();
        let bytes: Vec<String> = connection.rpop(&bucket_key, NonZeroUsize::new(count))?;
        for byte in bytes {
            forward_bytes_to_server(Bytes::from(byte)).await?;
        }
        Ok(())
    }

    fn key() -> String {
        ".leaky_bucket".into()
    }

    pub(super) async fn try_accept_request(
        &mut self,
        req: IncomingRequest,
    ) -> Result<FullResponse> {
        let bucket_key = Self::key();
        let bytes = convert_request_to_bytes(req).await?;

        let script = r#"
local bucket_key = KEYS[1]
local limit = tonumber(ARGV[1])
local bytes = ARGV[2]

local size = tonumber(redis.call('LLEN', bucket_key))

if size < limit then
    redis.call('LPUSH', bucket_key, bytes)
    return true
else
    return false
end
"#;

        let accept: bool = redis::Script::new(script)
            .key(&bucket_key)
            .arg(self.limit)
            .arg(String::from_utf8(bytes.to_vec())?)
            .invoke(&mut self.redis_connection)
            .unwrap_or_default();

        if accept {
            accepted().await
        } else {
            too_many_request().await
        }
    }
}
