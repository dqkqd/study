use std::io;

use thiserror::Error;

#[doc(hidden)]
#[derive(Error, Debug)]
pub enum KvError {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("bson serialize error")]
    BsonSer(#[from] bson::ser::Error),
    #[error("bson deserialize error")]
    BsonDe(#[from] bson::de::Error),
    #[error("invalid pattern `{0}`")]
    GlobPattern(#[from] glob::PatternError),

    #[error("file id `{0}` does not exist")]
    FileIdDoesNotExist(u64),
    #[error("key `{0}` does not exist")]
    KeyDoesNotExist(String),
    #[error("cannot write bytes length `{0}`")]
    CannotWriteLen(usize),
    #[error("cannot transfer active log file, err: `{0}`")]
    CannotTransferActiveLog(String),

    #[error("merge result not available")]
    MergeResultNotAvailable,
}

/// Alias result to avoid duplication
pub type Result<T> = std::result::Result<T, KvError>;
