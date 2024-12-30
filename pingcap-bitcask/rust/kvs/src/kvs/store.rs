use crate::{KvError, Result};
use std::path::Path;

use super::{kv::KvStore, sled::SledKvsEngine, KvsEngine};

/// TODO: docs
pub enum Store {
    /// TODO: docs
    Kvs(KvStore),
    /// TODO: docs
    Sled(SledKvsEngine),
}

impl Store {
    /// TODO: docs
    pub fn open_as_kvs<P: AsRef<Path>>(path: P) -> Result<Store> {
        if SledKvsEngine::dbpath(&path).exists() {
            return Err(KvError::MismatchEngine);
        }
        let inner = KvStore::open(&path)?;
        Ok(Store::Kvs(inner))
    }

    /// TODO: docs
    pub fn open_as_sled<P: AsRef<Path>>(path: P) -> Result<Store> {
        if KvStore::dbpath(&path).exists() {
            return Err(KvError::MismatchEngine);
        }
        let inner = SledKvsEngine::open(&path)?;
        Ok(Store::Sled(inner))
    }
}

impl KvsEngine for Store {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        match self {
            Store::Kvs(store) => store.set(key, value),
            Store::Sled(store) => store.set(key, value),
        }
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        match self {
            Store::Kvs(store) => store.get(key),
            Store::Sled(store) => store.get(key),
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        match self {
            Store::Kvs(store) => store.remove(key),
            Store::Sled(store) => store.remove(key),
        }
    }
}
