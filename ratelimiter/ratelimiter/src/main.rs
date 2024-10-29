use std::{net::SocketAddr, time::SystemTime};

use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = TcpListener::bind(addr).await?;

    loop {
        // using ratelimiter
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(service))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn service(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>> {
    let get_env_as_int = |key, default| {
        std::env::var(key)
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(default)
    };
    let limit = get_env_as_int("LIMIT", 10);
    let rate = get_env_as_int("RATE", 3);

    let mut token_bucket = TokenBucket::new(limit, rate)?;
    if token_bucket.accepted(&req) {
        forward(req).await
    } else {
        let resp = Response::builder()
            .status(429)
            .body(Full::new(Bytes::from("Too many request\n")))?;
        Ok(resp)
    }
}

struct TokenBucket {
    limit: u64,
    rate: u64,
    redis_connection: redis::Connection,
}
impl TokenBucket {
    fn new(limit: u64, rate: u64) -> Result<TokenBucket> {
        let redis_client = redis::Client::open("redis://cache:6379")?;
        Ok(TokenBucket {
            limit,
            rate,
            redis_connection: redis_client.get_connection()?,
        })
    }

    fn key(req: &Request<hyper::body::Incoming>) -> String {
        let host = req
            .headers()
            .get(hyper::header::HOST)
            .and_then(|host| host.to_str().ok())
            .unwrap_or_default()
            .to_string();
        host
    }

    fn accepted(&mut self, req: &Request<hyper::body::Incoming>) -> bool {
        let tokens_key = Self::key(req);
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

async fn forward(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>> {
    eprintln!("{:#?}", &req);
    let body = req.into_body();
    let bytes = body.collect().await.map(|collected| collected.to_bytes())?;
    let response = forward_bytes(bytes).await?;
    Ok(response)
}

async fn forward_bytes(bytes: Bytes) -> Result<Response<Full<Bytes>>> {
    let stream = TcpStream::connect("server:3000").await?;

    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            eprintln!("Connection failed: {:?}", err);
        }
    });

    let req = Request::builder().body(Full::<Bytes>::new(bytes))?;
    let mut res = sender.send_request(req).await?;

    let mut buf = Vec::new();
    let mut buffer = io::BufWriter::new(&mut buf);

    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            buffer.write_all(chunk).await?;
        }
    }
    buffer.flush().await?;
    buf.push(b'\n');

    Ok(Response::new(Full::new(Bytes::from(buf))))
}
