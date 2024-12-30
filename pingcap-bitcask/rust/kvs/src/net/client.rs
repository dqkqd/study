use tracing::info;

use crate::{parser::ByteParser, Result};
use std::{
    io::{BufReader, BufWriter, Write},
    net::{SocketAddr, TcpStream},
};

use super::protocol::{KvsRequest, KvsResponse};

/// TODO: docs
pub struct KvsClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    /// TODO: docs
    pub fn open(address: SocketAddr) -> Result<KvsClient> {
        let stream = TcpStream::connect(address)?;
        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream);

        let client = KvsClient { reader, writer };
        info!(server_address = %address, "client connected");

        Ok(client)
    }

    /// TODO: docs
    pub fn send(&mut self, request: KvsRequest) -> Result<()> {
        info!(request = ?request, "sent request");

        let bytes = request.to_bytes()?;
        self.writer.write_all(&bytes)?;
        self.writer.flush()?;

        Ok(())
    }

    /// TODO: docs
    pub fn recv(&mut self) -> Result<KvsResponse> {
        let response = KvsResponse::from_reader(&mut self.reader);
        info!(response = ?response, "received response");

        response
    }
}
