use std::time::SystemTime;

use hyper::Request;

use crate::forward_request_to_server;
use crate::FullResponse;
use crate::IncomingRequest;
use crate::Result;

use super::get_env;
use super::get_redis_connection;
use super::host;
use super::too_many_request;

pub struct FixedWindow {
    limit: u64,
    redis_connection: redis::Connection,
}

impl FixedWindow {
    pub(super) fn new() -> Result<FixedWindow> {
        let redis_connection = get_redis_connection()?;
        let limit: u64 = get_env("LIMIT", 10);
        Ok(FixedWindow {
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
        let counter_key = host(req);
        let update_at_key = format!("{}_update_at", &counter_key);

        let script = r#"
local counter_key = KEYS[1]
local update_at_key = KEYS[2]
local limit = tonumber(ARGV[1])
local now = tonumber(ARGV[2])

local counter = tonumber(redis.call('GET', counter_key) or limit)
local update_at = tonumber(redis.call('GET', update_at_key) or now)

if now > update_at then
    redis.call('SET', counter_key, limit)
    redis.call('SET', update_at_key, now)
    counter = limit
end

if counter > 0 then
    redis.call('DECR', counter_key)
    return true
else
    return false
end
"#;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        redis::Script::new(script)
            .key(&counter_key)
            .key(&update_at_key)
            .arg(self.limit)
            .arg(now)
            .invoke(&mut self.redis_connection)
            .unwrap_or_default()
    }
}
