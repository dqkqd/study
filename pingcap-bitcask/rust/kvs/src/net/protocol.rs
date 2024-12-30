use serde::{Deserialize, Serialize};

use crate::parser::ByteParser;
use crate::KvError;

#[doc(hidden)]
#[derive(Debug, Serialize, Deserialize)]
pub enum KvsRequest {
    Set { key: String, value: String },
    Get { key: String },
    Remove { key: String },
}

#[doc(hidden)]
#[derive(Debug, Serialize, Deserialize)]
pub enum KvsResponse {
    Ok(Option<String>),
    KeyNotFound(String),
    InvalidCommand(String),
    ServerError,
}

impl ByteParser for KvsRequest {}
impl ByteParser for KvsResponse {}

impl From<KvError> for KvsResponse {
    fn from(value: KvError) -> Self {
        match value {
            KvError::KeyNotFound(key) => KvsResponse::KeyNotFound(key),
            _ => KvsResponse::ServerError,
        }
    }
}
