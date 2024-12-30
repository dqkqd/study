use tracing::info;

use crate::{parser::ByteParser, Result};
use std::{
    io::{BufReader, BufWriter, Write},
    net::{SocketAddr, TcpStream},
};

use super::protocol::{KvsRequest, KvsResponse};

/// Client that can talk with sever through internal network protocol.
pub struct KvsClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    /// Connect to server at specific address.
    ///
    /// # Example
    /// ```no_run
    /// # use kvs::Result;
    /// # use kvs::{KvsClient, KvsEngine, KvsServer, Store};
    /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// # let store = Store::open(&directory)?;
    /// # let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    /// let server = KvsServer::open(address, store)?;
    /// server.serve()?;
    /// let client = KvsClient::connect(address)?;
    /// # Ok(())
    /// # }
    pub fn connect(address: SocketAddr) -> Result<KvsClient> {
        let stream = TcpStream::connect(address)?;
        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream);

        let client = KvsClient { reader, writer };
        info!(server_address = %address, "client connected");

        Ok(client)
    }

    /// Send a request to server.
    pub fn send(&mut self, request: KvsRequest) -> Result<()> {
        info!(request = ?request, "sent request");

        let bytes = request.to_bytes()?;
        self.writer.write_all(&bytes)?;
        self.writer.flush()?;

        Ok(())
    }

    /// Get a response from server.
    pub fn recv(&mut self) -> Result<KvsResponse> {
        let response = KvsResponse::from_reader(&mut self.reader);
        info!(response = ?response, "received response");

        response
    }
}
