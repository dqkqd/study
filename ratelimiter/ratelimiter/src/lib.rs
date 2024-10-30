mod algo;

pub use algo::Ratelimiter;
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub(crate) type IncomingRequest = Request<hyper::body::Incoming>;
pub(crate) type FullResponse = Response<Full<Bytes>>;

pub async fn service(req: IncomingRequest) -> Result<FullResponse> {
    Ratelimiter::new()?.try_accept_request(req).await
}

async fn forward_request_to_server(req: IncomingRequest) -> Result<FullResponse> {
    let bytes = convert_request_to_bytes(req).await?;
    let resp = forward_bytes_to_server(bytes).await?;
    Ok(resp)
}

async fn convert_request_to_bytes(req: IncomingRequest) -> Result<Bytes> {
    eprintln!("{:#?}", &req);
    let body = req.into_body();
    let bytes = body.collect().await.map(|collected| collected.to_bytes())?;
    Ok(bytes)
}

async fn forward_bytes_to_server(bytes: Bytes) -> Result<FullResponse> {
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

    let resp = Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from(buf)))?;

    Ok(resp)
}
