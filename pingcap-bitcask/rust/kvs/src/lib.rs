//! **An on-disk key value store**

#![deny(missing_docs)]

mod command;
mod datafile;
mod directory;
mod error;
mod kvs;
mod merger;
mod options;
pub use error::KvError;
pub use error::Result;
pub use kvs::KvStore;
pub use options::KvOption;
