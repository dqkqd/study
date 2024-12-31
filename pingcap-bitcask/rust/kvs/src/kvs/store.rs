use crate::{KvError, Result};
use std::path::Path;

use super::{kv::KvStore, sled::SledKvsEngine, KvsEngine};

/// General store engine.
#[derive(Debug, Clone)]
pub struct Store(StoreInner);

#[derive(Debug, Clone)]
enum StoreInner {
    Kvs(KvStore),
    Sled(SledKvsEngine),
}

impl Store {
    /// Open database with kvs as internal engine.
    /// This function calls [`Store::open_with_kvs`] internally.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Store> {
        Store::open_with_kvs(path)
    }

    /// Open database with kvs as internal engine.
    pub fn open_with_kvs<P: AsRef<Path>>(path: P) -> Result<Store> {
        if SledKvsEngine::dbpath(&path).exists() {
            return Err(KvError::MismatchEngine);
        }
        let inner = KvStore::open(&path)?;
        Ok(Store(StoreInner::Kvs(inner)))
    }

    /// Open database with sled as internal engine.
    pub fn open_with_sled<P: AsRef<Path>>(path: P) -> Result<Store> {
        if KvStore::dbpath(&path).exists() {
            return Err(KvError::MismatchEngine);
        }
        let inner = SledKvsEngine::open(&path)?;
        Ok(Store(StoreInner::Sled(inner)))
    }
}

impl KvsEngine for Store {
    /// Set a key with value to the store.
    ///
    /// # Examples
    ///
    /// With kvs engine.
    /// ```rust
    /// # use kvs::KvsEngine;
    /// # use kvs::Store;
    /// # use kvs::Result;
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let store = Store::open_with_kvs(&directory)?;
    ///
    /// store.set("key1".to_string(), "value1".to_string())?;
    /// assert_eq!(store.get("key1".to_string())?, Some("value1".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// With sled engine.
    /// ```rust
    /// # use kvs::KvsEngine;
    /// # use kvs::Store;
    /// # use kvs::Result;
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let store = Store::open_with_sled(&directory)?;
    ///
    /// store.set("key1".to_string(), "value1".to_string())?;
    /// assert_eq!(store.get("key1".to_string())?, Some("value1".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    fn set(&self, key: String, value: String) -> Result<()> {
        match &self.0 {
            StoreInner::Kvs(store) => store.set(key, value),
            StoreInner::Sled(store) => store.set(key, value),
        }
    }

    /// Get value of a key from the store.
    ///
    /// # Examples
    ///
    /// With kvs engine.
    /// ```rust
    /// # use kvs::KvsEngine;
    /// # use kvs::Store;
    /// # use kvs::Result;
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let store = Store::open_with_kvs(&directory)?;
    ///
    /// assert_eq!(store.get("key1".to_string())?, None);
    ///
    /// store.set("key1".to_string(), "value1".to_string())?;
    /// assert_eq!(store.get("key1".to_string())?, Some("value1".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// With sled engine.
    /// ```rust
    /// # use kvs::KvsEngine;
    /// # use kvs::Store;
    /// # use kvs::Result;
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let store = Store::open_with_sled(&directory)?;
    ///
    /// assert_eq!(store.get("key1".to_string())?, None);
    ///
    /// store.set("key1".to_string(), "value1".to_string())?;
    /// assert_eq!(store.get("key1".to_string())?, Some("value1".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    fn get(&self, key: String) -> Result<Option<String>> {
        match &self.0 {
            StoreInner::Kvs(store) => store.get(key),
            StoreInner::Sled(store) => store.get(key),
        }
    }

    /// Remove a key from the store.
    ///
    /// # Examples
    ///
    /// With kvs engine.
    /// ```rust
    /// # use kvs::KvsEngine;
    /// # use kvs::Store;
    /// # use kvs::Result;
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let store = Store::open_with_kvs(&directory)?;
    ///
    /// store.set("key1".to_string(), "value1".to_string())?;
    /// assert_eq!(store.get("key1".to_string())?, Some("value1".to_string()));
    ///
    /// store.remove("key1".to_string())?;
    /// assert_eq!(store.get("key1".to_string())?, None);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// With sled engine.
    ///
    /// ```rust
    /// # use kvs::KvsEngine;
    /// # use kvs::Store;
    /// # use kvs::Result;
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let store = Store::open_with_sled(&directory)?;
    ///
    /// store.set("key1".to_string(), "value1".to_string())?;
    /// assert_eq!(store.get("key1".to_string())?, Some("value1".to_string()));
    ///
    /// store.remove("key1".to_string())?;
    /// assert_eq!(store.get("key1".to_string())?, None);
    /// # Ok(())
    /// # }
    /// ```
    fn remove(&self, key: String) -> Result<()> {
        match &self.0 {
            StoreInner::Kvs(store) => store.remove(key),
            StoreInner::Sled(store) => store.remove(key),
        }
    }
}
