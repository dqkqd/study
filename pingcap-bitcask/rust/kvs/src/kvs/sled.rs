use std::path::{Path, PathBuf};

use clap::crate_version;
use tracing::info;

use super::KvsEngine;
use crate::{KvError, Result};

const DATA_FOLDER: &str = "sledstore";

/// Wrapper for sled db engine.
#[derive(Debug, Clone)]
pub(crate) struct SledKvsEngine {
    db: sled::Db,
}

impl SledKvsEngine {
    /// Path to sled database.
    pub fn dbpath<P: AsRef<Path>>(path: P) -> PathBuf {
        path.as_ref().join(DATA_FOLDER)
    }

    /// Open sled db at specific path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<SledKvsEngine> {
        let dbpath = SledKvsEngine::dbpath(path);
        let db = sled::open(&dbpath)?;
        let store = SledKvsEngine { db };

        info!(version = crate_version!(), database_path = %dbpath.display(), "opened sled database:");

        Ok(store)
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.db.insert(&key, value.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let value = self
            .db
            .get(&key)?
            .and_then(|v| String::from_utf8(v.to_vec()).ok());
        Ok(value)
    }

    fn remove(&self, key: String) -> Result<()> {
        if self.db.remove(&key)?.is_none() {
            return Err(KvError::KeyNotFound(key));
        }
        self.db.flush()?;
        Ok(())
    }
}
