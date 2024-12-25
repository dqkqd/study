use std::collections::BTreeMap;

/// A key value store that keeps data in an in-memory hash table.
#[derive(Default)]
pub struct KvStore {
    in_memory_data: BTreeMap<String, String>,
}

impl KvStore {
    /// Creates an empty store.
    pub fn new() -> KvStore {
        KvStore::default()
    }

    /// Set a value by key to the store.
    ///
    /// # Examples
    /// ```rust
    /// # use kvs::KvStore;
    /// let mut kvs = KvStore::new();
    ///
    /// kvs.set("one".to_string(), "two".to_string());
    /// assert_eq!(kvs.get("one".to_string()), Some("two".to_string()));
    /// ```
    pub fn set(&mut self, key: String, value: String) {
        self.in_memory_data.insert(key, value);
    }

    /// Get a key from the store.
    ///
    /// # Examples
    /// ```rust
    /// # use kvs::KvStore;
    /// let mut kvs = KvStore::new();
    ///
    /// kvs.set("one".to_string(), "two".to_string());
    /// assert_eq!(kvs.get("one".to_string()), Some("two".to_string()));
    /// assert_eq!(kvs.get("other one".to_string()), None);
    /// ```
    pub fn get(&self, key: String) -> Option<String> {
        self.in_memory_data.get(&key).cloned()
    }

    /// Remove a key from the store.
    ///
    /// # Examples
    /// ```rust
    /// # use kvs::KvStore;
    /// let mut kvs = KvStore::new();
    ///
    /// kvs.set("one".to_string(), "two".to_string());
    /// assert_eq!(kvs.get("one".to_string()), Some("two".to_string()));
    ///
    /// kvs.remove("one".to_string());
    /// assert_eq!(kvs.get("one".to_string()), None);
    /// ```
    pub fn remove(&mut self, key: String) {
        self.in_memory_data.remove(&key);
    }
}
