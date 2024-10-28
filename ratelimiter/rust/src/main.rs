use std::net::SocketAddr;

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
    listen_from_client().await?;
    Ok(())
}

async fn listen_from_client() -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = TcpListener::bind(addr).await?;

    loop {
        // TODO: check whether we accept connection
        // using ratelimiter
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(forward))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
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
    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    let mut buf = Vec::new();
    let mut buffer = io::BufWriter::new(&mut buf);

    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            buffer.write_all(chunk).await?;
        }
    }
    buffer.flush().await?;

    Ok(Response::new(Full::new(Bytes::from(buf))))
}
