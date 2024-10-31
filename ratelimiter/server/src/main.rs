use std::net::SocketAddr;

use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn service(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let body = req.into_body();
    let bytes = body.collect().await.map(|collected| collected.to_bytes())?;
    Ok(Response::new(Full::new(bytes)))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;

    loop {
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
