use crate::Result;

/// TODO: docs
pub trait KvsEngine {
    /// TODO: docs
    fn set(&mut self, key: String, value: String) -> Result<()>;
    /// TODO: docs
    fn get(&mut self, key: String) -> Result<Option<String>>;
    /// TODO: docs
    fn remove(&mut self, key: String) -> Result<()>;
}
