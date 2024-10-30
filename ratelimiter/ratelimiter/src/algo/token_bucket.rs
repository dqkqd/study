use std::time::SystemTime;

use hyper::Request;

use crate::{forward_request_to_server, FullResponse, IncomingRequest, Result};

use super::{get_env, get_redis_connection, host, too_many_request};

pub struct TokenBucket {
    limit: u64,
    rate: u64,
    redis_connection: redis::Connection,
}

impl TokenBucket {
    pub(super) fn new() -> Result<TokenBucket> {
        let redis_connection = get_redis_connection()?;
        let limit: u64 = get_env("LIMIT", 10);
        let rate: u64 = get_env("RATE", 3);
        Ok(TokenBucket {
            limit,
            rate,
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
        let tokens_key = host(req);
        let update_at_key = format!("{}_update_at", &tokens_key);

        let script = r#"
local tokens_key = KEYS[1]
local update_at_key = KEYS[2]
local limit = tonumber(ARGV[1])
local rate = tonumber(ARGV[2])

local tokens = tonumber(redis.call('GET', tokens_key) or limit)
local now = tonumber(ARGV[3])
local update_at = tonumber(redis.call('GET', update_at_key) or now)

local duration = now - update_at
local add_tokens = duration * rate
local new_tokens = tokens + add_tokens
if new_tokens > limit then
    new_tokens = limit
end

if new_tokens > 0 then
    redis.call('SET', tokens_key, new_tokens - 1)
    redis.call('SET', update_at_key, now)
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
            .key(&tokens_key)
            .key(&update_at_key)
            .arg(self.limit)
            .arg(self.rate)
            .arg(now)
            .invoke(&mut self.redis_connection)
            .unwrap_or_default()
    }
}
