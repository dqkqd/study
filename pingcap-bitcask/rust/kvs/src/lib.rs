//! **An on-disk key value store**

#![deny(missing_docs)]

mod command;
mod error;
mod kvs;
mod log;
mod merger;
mod options;
pub use error::KvError;
pub use error::Result;
pub use kvs::KvStore;
pub use options::KvOption;
