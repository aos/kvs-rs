use crate::Result;

use std::panic;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use super::ThreadPool;

type Thunk = Box<dyn FnOnce() + Send + 'static>;

struct NotifyParent(u32);
struct Worker {
    id: u32,
    thread: thread::JoinHandle<()>,
    parent: Sender<NotifyParent>,
}

impl Worker {
    fn new(id: u32, job_receiver: Arc<Mutex<Receiver<Thunk>>>, parent_sender: Sender<NotifyParent>) -> Self {
        let thread = thread::spawn(move || loop {
            // Only lock for the duration required to receive a job
            // Not for also executing a job
            let message = {
                let lock = job_receiver
                    .lock()
                    .expect("Worker thread unable to lock job_receiver");

                lock.recv()
            };

            match message {
                Ok(job) => {
                    println!("Thread {} got a job, executing...", id);
                    job();
                },
                Err(..) => break,
            };
        });
        Worker { id, thread, parent: parent_sender }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if thread::panicking() {
            println!("Thread panicking... dropping");
            self.parent.send(NotifyParent(self.id)).expect("Worker unable to notify parent");
        }
    }
}

pub struct SharedQueueThreadPool {
    job_sender: Sender<Thunk>,
    notify_panic: thread::JoinHandle<()>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<SharedQueueThreadPool> {
        assert!(threads > 0);

        let (tx, rx) = channel::<Thunk>();
        let locked_receiver = Arc::new(Mutex::new(rx));

        let (notify_tx, notify_rx) = channel::<NotifyParent>();

        for id in 0..threads {
            Worker::new(id, Arc::clone(&locked_receiver), notify_tx.clone());
        }

        Ok(SharedQueueThreadPool {
            job_sender: tx,
            notify_panic: thread::spawn(move || loop {
                match notify_rx.recv() {
                    Ok(job) => {
                        println!("Worker thread {} panicked, spawning new thread...", job.0);
                        Worker::new(job.0 + 1, Arc::clone(&locked_receiver), notify_tx.clone());
                    },
                    Err(..) => break,
                };

            })
        })
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
