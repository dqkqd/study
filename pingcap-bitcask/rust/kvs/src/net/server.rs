use tracing::info;

use crate::{parser::ByteParser, thread_pool::ThreadPool, KvsEngine, Result};
use std::{
    io::{BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
};

use super::protocol::{KvsRequest, KvsResponse};

#[derive(Debug)]
enum ServerMessage {
    Shutdown,
}

/// Server directly interacts with on-disk database to serve clients' requests.
///
/// Database engine must implement [`KvsEngine`].
pub struct KvsServer<E, P>
where
    E: KvsEngine,
    P: ThreadPool,
{
    /// The address at which the server is opened.
    pub address: SocketAddr,
    listener: TcpListener,
    store: E,
    pool: P,
}

impl<E, P> KvsServer<E, P>
where
    E: KvsEngine,
    P: ThreadPool,
{
    /// Open server at provided address.
    ///
    /// # Example
    /// ```rust
    /// # use kvs::Result;
    /// # use kvs::{KvsEngine, KvsServer, Store};
    /// # use kvs::thread_pool::{ThreadPool, SharedQueueThreadPool};
    /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let store = Store::open(&directory)?;
    /// let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000);
    /// let pool = SharedQueueThreadPool::new(8)?;
    /// let server = KvsServer::open(address, store, pool)?;
    /// # Ok(())
    /// # }
    pub fn open(address: SocketAddr, store: E, pool: P) -> Result<KvsServer<E, P>> {
        let listener = TcpListener::bind(address)?;
        let address = listener.local_addr()?;

        let server = KvsServer {
            address,
            listener,
            store,
            pool,
        };
        info!(addr = %address,  "server started");

        Ok(server)
    }

    /// Start listening for incoming requests.
    pub fn serve(self) -> RunningServer {
        info!("serving");

        let (sender, receiver) = mpsc::channel();

        let KvsServer {
            address,
            listener,
            store,
            pool,
        } = self;

        let active = Arc::new(AtomicBool::new(true));

        let waiting_server = RunningServer {
            address,
            active: active.clone(),
            receiver,
        };

        thread::spawn(move || loop {
            if !active.load(Ordering::SeqCst) {
                sender
                    .send(ServerMessage::Shutdown)
                    .expect("cannot notify shutdown message");
                break;
            }

            listener
                .set_nonblocking(true)
                .expect("cannot set listener as unblocking");

            if let Ok((stream, _)) = listener.accept() {
                let store = store.clone();

                let active = active.clone();

                pool.spawn(move || {
                    let _ = handle_connection(store, stream, active);
                })
            }
        });

        waiting_server
    }
}

/// Controller returned when serving, this helps shutting down server programmatically.
pub struct RunningServer {
    pub address: SocketAddr,
    active: Arc<AtomicBool>,
    receiver: Receiver<ServerMessage>,
}

impl RunningServer {
    pub fn shutdown(self) {
        info!(address = %self.address, "shutdown server");

        // Tell everyone to stop
        self.active.store(false, Ordering::SeqCst);

        // Waiting everyone to stop
        while let Ok(msg) = self.receiver.recv() {
            if matches!(msg, ServerMessage::Shutdown) {
                break;
            }
        }
    }
}

fn handle_connection<E: KvsEngine>(
    store: E,
    stream: TcpStream,
    active: Arc<AtomicBool>,
) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    info!(peer = %stream.peer_addr().unwrap(), "connection accepted:");

    while active.load(Ordering::SeqCst) {
        let request = KvsRequest::from_reader(&mut reader);
        info!(request = ?request, "request:");

        let response = match request {
            Ok(request) => handle_request(&store, request),
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

fn handle_request<E: KvsEngine>(store: &E, request: KvsRequest) -> KvsResponse {
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
