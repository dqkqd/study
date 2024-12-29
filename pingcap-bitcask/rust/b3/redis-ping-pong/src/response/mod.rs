mod deserialize;
mod error;
mod serialize;

use serde::{Deserialize, Serialize};

pub use deserialize::from_bytes;
pub use error::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Response {
    SimpleString(String),
    BulkString(Vec<u8>),
}
