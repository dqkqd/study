mod naive;
mod rayon;
mod shared;

use crate::Result;

pub use naive::NaiveThreadPool;
pub use rayon::RayonThreadPool;
pub use shared::SharedQueueThreadPool;

pub trait ThreadPool
where
    Self: std::marker::Sized,
{
    fn new(threads: u32) -> Result<Self>;
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}
