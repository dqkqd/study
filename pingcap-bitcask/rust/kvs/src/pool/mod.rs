use crate::Result;

pub(crate) trait ThreadPool
where
    Self: std::marker::Sized,
{
    fn new(threads: u32) -> Result<Self>;
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}
