//! Custom threadpool implementation.

mod naive;
mod rayon;
mod shared;

use crate::Result;

pub use naive::NaiveThreadPool;
pub use rayon::RayonThreadPool;
pub use shared::SharedQueueThreadPool;

/// Trait handles thread pool to avoiding re-creating threads.
pub trait ThreadPool
where
    Self: std::marker::Sized,
{
    /// Create new thread pool.
    fn new(threads: u32) -> Result<Self>;

    /// Spawn a new function running in thread pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}
