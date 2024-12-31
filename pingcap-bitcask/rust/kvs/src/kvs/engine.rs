use crate::Result;

/// Trait for database engine.
///
/// Engine must implement this to talk with [KvsServer][`crate::KvsServer`].
pub trait KvsEngine: Clone + Send + 'static {
    /// Set a key with value to the store.
    fn set(&self, key: String, value: String) -> Result<()>;
    /// Get value of a key from the store.
    fn get(&self, key: String) -> Result<Option<String>>;
    /// Remove a key from the store.
    fn remove(&self, key: String) -> Result<()>;
}
