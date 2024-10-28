use std::{net::SocketAddr, time::SystemTime};

use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use redis::Commands;
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
    let mut token_bucket = TokenBucket::new(10, 3)?;
    if token_bucket.accepted(&req)? {
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

    fn refill(&mut self, req: &Request<hyper::body::Incoming>) -> Result<u64> {
        let tokens_key = Self::key(req);
        let update_at_key = format!("{}_update_at", &tokens_key);

        // get the latest modified key
        let tokens: u64 = redis::transaction(
            &mut self.redis_connection,
            &[&tokens_key, &update_at_key],
            |con, pipe| {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let ((tokens, update_at),): ((u64, u64),) = pipe
                    .set_nx(&tokens_key, self.limit)
                    .ignore()
                    .set_nx(&update_at_key, now)
                    .ignore()
                    .mget(&[&tokens_key, &update_at_key])
                    .query(con)?;

                if tokens >= self.limit {
                    return Ok(Some(tokens));
                }

                let duration = now - update_at;
                let add_tokens = duration * self.rate;
                let new_tokens = (tokens + add_tokens).min(self.limit);
                let tokens = if new_tokens > tokens {
                    () = pipe
                        .set(&tokens_key, new_tokens)
                        .ignore()
                        .set(&update_at_key, now)
                        .ignore()
                        .query(con)?;
                    new_tokens
                } else {
                    tokens
                };

                Ok(Some(tokens))
            },
        )?;

        Ok(tokens)
    }

    fn accepted(&mut self, req: &Request<hyper::body::Incoming>) -> Result<bool> {
        let current_tokens = self.refill(req)?;
        if current_tokens > 0 {
            () = self.redis_connection.decr(Self::key(req), 1)?;
            Ok(true)
        } else {
            Ok(false)
        }
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
