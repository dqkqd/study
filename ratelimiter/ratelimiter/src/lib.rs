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
pub(crate) type IncomingRequest = Request<hyper::body::Incoming>;
pub(crate) type FullResponse = Response<Full<Bytes>>;

pub async fn service(req: IncomingRequest) -> Result<FullResponse> {
    let mut ratelimiter = Ratelimiter::new()?;
    if ratelimiter.accepted(&req) {
        ratelimiter.accept_request(req).await
    } else {
        ratelimiter.drop_request().await
    }
}

async fn forward_request_to_server(req: IncomingRequest) -> Result<FullResponse> {
    eprintln!("{:#?}", &req);
    let body = req.into_body();
    let bytes = body.collect().await.map(|collected| collected.to_bytes())?;

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
