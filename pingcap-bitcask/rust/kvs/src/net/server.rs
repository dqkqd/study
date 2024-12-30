use tracing::info;

use crate::{parser::ByteParser, KvsEngine, Result};
use std::{
    cell::RefCell,
    io::{BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use super::protocol::{KvsRequest, KvsResponse};

/// Server directly interacts with on-disk database to serve clients' requests.
///
/// Database engine must implement [`KvsEngine`].
pub struct KvsServer<E>
where
    E: KvsEngine,
{
    listener: TcpListener,
    store: RefCell<E>,
}

impl<E> KvsServer<E>
where
    E: KvsEngine,
{
    /// Open server at provided address.
    ///
    /// # Example
    /// ```rust
    /// # use kvs::Result;
    /// # use kvs::{KvsEngine, KvsServer, Store};
    /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let store = Store::open(&directory)?;
    /// let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    /// let server = KvsServer::open(address, store)?;
    /// # Ok(())
    /// # }
    pub fn open(address: SocketAddr, store: E) -> Result<KvsServer<E>> {
        let listener = TcpListener::bind(address)?;

        let server = KvsServer {
            listener,
            store: RefCell::new(store),
        };
        info!(addr = %address,  "server started");

        Ok(server)
    }

    /// Start listening for incoming requests.
    pub fn serve(&self) -> Result<()> {
        info!("server serving");

        for stream in self.listener.incoming() {
            let stream = stream?;
            self.handle_connection(stream)?;
        }

        Ok(())
    }

    fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = BufWriter::new(stream.try_clone()?);

        info!(peer = %stream.peer_addr().unwrap(), "connection accepted:");

        loop {
            let request = KvsRequest::from_reader(&mut reader);
            info!(request = ?request, "request:");

            let response = match request {
                Ok(request) => self.handle_request(request),
                Err(e) => KvsResponse::InvalidCommand(e.to_string()),
            };

            let bytes = response.to_bytes().unwrap_or_else(|_| {
                KvsResponse::ServerError
                    .to_bytes()
                    .expect("parser simple response")
            });

            // check connection before writing
            if stream.peer_addr().is_err() {
                break;
            }
            writer.write_all(&bytes)?;
            writer.flush()?;
        }

        Ok(())
    }

    fn handle_request(&self, request: KvsRequest) -> KvsResponse {
        let mut store = self.store.borrow_mut();

        let res = match request {
            KvsRequest::Get { key } => store
                .get(key.clone())
                .map(KvsResponse::Ok)
                .map_err(KvsResponse::from),

            KvsRequest::Set { key, value } => store
                .set(key, value)
                .map(|_| KvsResponse::Ok(None))
                .map_err(KvsResponse::from),

            KvsRequest::Remove { key } => store
                .remove(key)
                .map(|_| KvsResponse::Ok(None))
                .map_err(KvsResponse::from),
        };

        let res = match res {
            Ok(res) => res,
            Err(res) => res,
        };
        info!(res = ?res, "response:");

        res
    }
}
