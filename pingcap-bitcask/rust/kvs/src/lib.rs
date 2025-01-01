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

pub mod thread_pool;

pub use kvs::Store;
#[doc(hidden)]
pub use kvs::Store as KvStore;

pub use kvs::KvsEngine;

pub use options::KvOption;

#[doc(hidden)]
pub use error::{KvError, Result};

pub use net::client::KvsClient;
pub use net::server::KvsServer;

#[doc(hidden)]
pub use net::protocol::{KvsRequest, KvsResponse};
