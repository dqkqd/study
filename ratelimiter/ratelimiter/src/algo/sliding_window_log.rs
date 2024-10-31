use std::time::SystemTime;

use hyper::Request;

use super::{get_env, get_redis_connection, host, too_many_request};
use crate::{forward_request_to_server, FullResponse, IncomingRequest, Result};

pub struct SlidingWindowLog {
    limit: u64,
    redis_connection: redis::Connection,
}

impl SlidingWindowLog {
    pub(super) fn new() -> Result<SlidingWindowLog> {
        let redis_connection = get_redis_connection()?;
        let limit: u64 = get_env("LIMIT", 3);
        Ok(SlidingWindowLog {
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
        let timestamp_key = host(req) + ".sliding_window_log";

        let script = r#"
local timestamp_key = KEYS[1]
local limit = tonumber(ARGV[1])
local now = tonumber(ARGV[2])

redis.call('ZREMRANGEBYSCORE', timestamp_key, '-inf', now - 1)
local size = tonumber(redis.call('ZCARD', timestamp_key))

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

        redis::Script::new(script)
            .key(&timestamp_key)
            .arg(self.limit)
            .arg(now)
            .invoke(&mut self.redis_connection)
            .unwrap_or_default()
    }
}
