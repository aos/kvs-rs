use super::ThreadPool;
use crate::Result;
use rayon;

pub struct RayonThreadPool(rayon::ThreadPool);

impl ThreadPool for RayonThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(RayonThreadPool(
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads as usize)
                .build()?,
        ))
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.0.install(job);
    }
}
