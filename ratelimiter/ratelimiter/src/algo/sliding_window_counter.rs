use std::time::SystemTime;

use hyper::Request;

use super::{get_env, get_redis_connection, host, too_many_request};
use crate::{forward_request_to_server, FullResponse, IncomingRequest, Result};

pub struct SlidingWindowCounter {
    limit: u64,
    redis_connection: redis::Connection,
}

impl SlidingWindowCounter {
    pub(super) fn new() -> Result<SlidingWindowCounter> {
        let redis_connection = get_redis_connection()?;
        let limit: u64 = get_env("LIMIT", 3);
        Ok(SlidingWindowCounter {
            limit,
            redis_connection,
        })
    }

    pub(super) async fn try_accept_request(
        &mut self,
        req: IncomingRequest,
    ) -> Result<FullResponse> {
        if self.accepted(&req) {
            forward_request_to_server(req).await
        } else {
            too_many_request().await
        }
    }

    pub(super) fn accepted<R>(&mut self, req: &Request<R>) -> bool {
        let timestamp_key = host(req) + ".sliding_window_counter";

        let script = r#"
local timestamp_key = KEYS[1]
local limit = tonumber(ARGV[1])
local now = tonumber(ARGV[2])
local now_as_sec = tonumber(ARGV[3])
local previous_portion = tonumber(ARGV[4])

redis.call('ZREMRANGEBYSCORE', timestamp_key, '-inf', now_as_sec - 2)
local previous = tonumber(redis.call('ZCOUNT', timestamp_key, now_as_sec - 1, now_as_sec))
local current = tonumber(redis.call('ZCOUNT', timestamp_key, now_as_sec, '+inf'))
local size = previous * previous_portion + current

if size < limit then
    redis.call('ZADD', timestamp_key, now, now)
    return true
else
    return false
end
"#;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let now_as_sec = now.round();
        let current_portion = now - now_as_sec;
        let previous_portion = 1.0 - current_portion;

        redis::Script::new(script)
            .key(&timestamp_key)
            .arg(self.limit)
            .arg(now)
            .arg(now_as_sec)
            .arg(previous_portion)
            .invoke(&mut self.redis_connection)
            .unwrap_or_default()
    }
}
