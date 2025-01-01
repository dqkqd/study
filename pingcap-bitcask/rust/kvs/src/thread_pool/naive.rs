use std::thread;

use super::ThreadPool;
use crate::Result;

/// Naive thread pool implementation without allocating threads ahead.
pub struct NaiveThreadPool {}

impl ThreadPool for NaiveThreadPool {
    fn new(_threads: u32) -> Result<NaiveThreadPool> {
        Ok(NaiveThreadPool {})
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(job);
    }
}
