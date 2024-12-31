use super::ThreadPool;
use crate::Result;

pub struct RayonThreadPool {}

impl ThreadPool for RayonThreadPool {
    fn new(_threads: u32) -> Result<RayonThreadPool> {
        todo!()
    }

    fn spawn<F>(&self, _job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        todo!()
    }
}
