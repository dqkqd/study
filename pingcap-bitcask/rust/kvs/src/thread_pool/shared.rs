use super::ThreadPool;
use crate::Result;

pub struct SharedQueueThreadPool {}

impl ThreadPool for SharedQueueThreadPool {
    fn new(_threads: u32) -> Result<SharedQueueThreadPool> {
        todo!()
    }

    fn spawn<F>(&self, _job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        todo!()
    }
}
