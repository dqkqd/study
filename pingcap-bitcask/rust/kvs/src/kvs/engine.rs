use crate::Result;

/// Trait for database engine.
///
/// Engine must implement this to talk with [KvsServer][`crate::KvsServer`].
pub trait KvsEngine {
    /// Set a key with value to the store.
    fn set(&mut self, key: String, value: String) -> Result<()>;
    /// Get value of a key from the store.
    fn get(&mut self, key: String) -> Result<Option<String>>;
    /// Remove a key from the store.
    fn remove(&mut self, key: String) -> Result<()>;
}
