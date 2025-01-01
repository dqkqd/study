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
    #[error("sled error `{0}`")]
    Sled(#[from] sled::Error),
    #[error("rayon threadpool build `{0}`")]
    RayonThreadPoolBuild(#[from] rayon::ThreadPoolBuildError),

    #[error("file id `{0}` does not exist")]
    FileIdDoesNotExist(u64),
    #[error("Key not found `{0}`")]
    KeyNotFound(String),
    #[error("cannot write bytes length `{0}`")]
    CannotWriteLen(usize),
    #[error("cannot transfer active log file, err: `{0}`")]
    CannotTransferActiveLog(String),

    #[error("unknown error")]
    Unknown,

    #[error("mismatch engine")]
    MismatchEngine,

    #[error("cannot read shared data `{0}`")]
    SharedRead(String),
    #[error("cannot write shared data `{0}`")]
    SharedWrite(String),
}

/// Alias result to avoid duplication
pub type Result<T> = std::result::Result<T, KvError>;
