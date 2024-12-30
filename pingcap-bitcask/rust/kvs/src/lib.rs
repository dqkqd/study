//! **An on-disk key value store**

#![deny(missing_docs)]

mod command;
mod error;
mod kvs;
mod log;
mod merger;
mod net;
mod options;
mod parser;

pub use error::{KvError, Result};
pub use kvs::{KvStore, KvsEngine};
pub use options::KvOption;

pub use net::client::KvsClient;
pub use net::protocol::{KvsRequest, KvsResponse};
pub use net::server::KvsServer;
