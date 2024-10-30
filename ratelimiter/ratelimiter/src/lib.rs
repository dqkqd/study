mod algo;

use algo::Ratelimiter;
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn service(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>> {
    let mut ratelimiter = Ratelimiter::new()?;
    if ratelimiter.accepted(&req) {
        forward_req(req).await
    } else {
        drop_req(req).await
    }
}

async fn forward_req(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>> {
    eprintln!("{:#?}", &req);
    let body = req.into_body();
    let bytes = body.collect().await.map(|collected| collected.to_bytes())?;
    let response = forward_bytes(bytes).await?;
    Ok(response)
}

async fn drop_req(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>> {
    let resp = Response::builder()
        .status(429)
        .body(Full::new(Bytes::from("Too many request\n")))?;
    Ok(resp)
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
