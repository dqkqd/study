mod leaky_bucket;
mod token_bucket;
use http_body_util::Full;
use hyper::{body::Bytes, Request, Response, StatusCode};
use leaky_bucket::LeakyBucket;
use redis::Connection;
use token_bucket::TokenBucket;

use crate::{FullResponse, IncomingRequest, Result};

pub enum Ratelimiter {
    TokenBucket(TokenBucket),
    LeakyBucket(LeakyBucket),
}

impl Ratelimiter {
    pub fn new() -> Result<Ratelimiter> {
        let algo = get_env("ALGO", "token_bucket".to_string());
        let ratelimiter = match algo.as_str() {
            "token_bucket" => Ratelimiter::TokenBucket(TokenBucket::new()?),
            "leaky_bucket" => Ratelimiter::LeakyBucket(LeakyBucket::new()?),
            _ => unimplemented!("{}", algo),
        };
        Ok(ratelimiter)
    }

    pub(crate) async fn try_accept_request(
        &mut self,
        req: IncomingRequest,
    ) -> Result<FullResponse> {
        match self {
            Ratelimiter::TokenBucket(token_bucket) => token_bucket.try_accept_request(req).await,
            Ratelimiter::LeakyBucket(leaky_bucket) => leaky_bucket.try_accept_request(req).await,
        }
    }

    pub async fn background_task(self) -> Result<()> {
        if let Ratelimiter::LeakyBucket(leaky_bucket) = self {
            leaky_bucket.background_task()?
        }
        Ok(())
    }
}

pub(super) fn get_env<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    std::env::var(key)
        .ok()
        .and_then(|value| std::str::FromStr::from_str(&value).ok())
        .unwrap_or(default)
}

pub(super) fn get_redis_connection() -> Result<Connection> {
    let redis_client = redis::Client::open("redis://cache:6379")?;
    let connection = redis_client.get_connection()?;
    Ok(connection)
}

pub(super) fn host<R>(req: &Request<R>) -> String {
    let host = req
        .headers()
        .get(hyper::header::HOST)
        .and_then(|host| host.to_str().ok())
        .unwrap_or_default()
        .to_string();
    host
}

pub(crate) async fn accepted() -> Result<FullResponse> {
    let resp = Response::builder()
        .status(StatusCode::ACCEPTED)
        .body(Full::new(Bytes::from("Accepted\n")))?;
    Ok(resp)
}

pub(crate) async fn too_many_request() -> Result<FullResponse> {
    let resp = Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .body(Full::new(Bytes::from("Too many request\n")))?;
    Ok(resp)
}
