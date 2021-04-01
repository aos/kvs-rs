use crate::Result;

use super::ThreadPool;
use std::panic;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

type Thunk = Box<dyn FnOnce() + Send + 'static>;

struct Worker(Arc<Mutex<Receiver<Thunk>>>);

impl Drop for Worker {
    fn drop(&mut self) {
        if thread::panicking() {
            println!("Thread panicking... dropping");
            spawn_in_pool(Worker(self.0.clone()));
        }
    }
}

fn spawn_in_pool(job_receiver: Worker) {
    thread::spawn(move || loop {
        // Only lock for the duration required to receive a job
        // Not for also executing a job
        let message = {
            let lock = job_receiver
                .0
                .lock()
                .expect("Worker thread unable to lock job_receiver");

            lock.recv()
        };

        match message {
            Ok(job) => {
                println!("Thread got a job, executing...");
                job();
            }
            Err(..) => break,
        };
    });
}

pub struct SharedQueueThreadPool {
    job_sender: Sender<Thunk>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<SharedQueueThreadPool> {
        assert!(threads > 0);

        let (tx, rx) = channel::<Thunk>();
        let locked_receiver = Arc::new(Mutex::new(rx));

        for _ in 0..threads {
            let job_receiver = Arc::clone(&locked_receiver);
            spawn_in_pool(Worker(job_receiver));
        }

        Ok(SharedQueueThreadPool { job_sender: tx })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.job_sender
            .send(Box::new(job))
            .expect("ThreadPool::spawn unable to send jobs into queue");
    }
}
