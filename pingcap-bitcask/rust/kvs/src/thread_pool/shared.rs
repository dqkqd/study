use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use tracing::{info, warn};

use super::ThreadPool;
use crate::Result;

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct WorkerId(u32);

enum ThreadPoolMessage {
    Run(Job),
}

#[derive(Debug)]
struct SharedData {
    receiver: Mutex<Receiver<ThreadPoolMessage>>,
}

pub struct SharedQueueThreadPool {
    /// Sender sends message to workers, for spawning workers.
    sender: Sender<ThreadPoolMessage>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<SharedQueueThreadPool> {
        let (sender, receiver) = mpsc::channel::<ThreadPoolMessage>();

        info!(num_threads = threads, "starting thread pool");

        let pool = SharedQueueThreadPool { sender };

        let shared_data = Arc::new(SharedData {
            receiver: Mutex::new(receiver),
        });

        for id in 0..threads {
            let worker_id = WorkerId(id);
            spawn_worker(worker_id, shared_data.clone());
        }

        Ok(pool)
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .send(ThreadPoolMessage::Run(Box::new(job)))
            .expect("cannot spawn job");
    }
}

/// Copy from <https://docs.rs/threadpool/latest/src/threadpool/lib.rs.html#101>
struct Sentinel<'a> {
    shared_data: &'a Arc<SharedData>,
    worker_id: WorkerId,
    active: bool,
}

impl<'a> Sentinel<'a> {
    fn new(worker_id: WorkerId, shared_data: &'a Arc<SharedData>) -> Sentinel<'a> {
        Sentinel {
            shared_data,
            worker_id,
            active: true,
        }
    }
    fn cancel(mut self) {
        self.active = false;
    }
}

impl<'a> Drop for Sentinel<'a> {
    fn drop(&mut self) {
        if self.active {
            if thread::panicking() {
                warn!(worker_id = self.worker_id.0, "worker panicked:");
            }
            spawn_worker(self.worker_id, self.shared_data.clone());
        }
    }
}

fn spawn_worker(worker_id: WorkerId, shared_data: Arc<SharedData>) {
    thread::spawn(move || {
        let sentinel = Sentinel::new(worker_id, &shared_data);

        loop {
            let msg = {
                let tx = shared_data
                    .receiver
                    .lock()
                    .unwrap_or_else(|_| panic!("cannot acquire lock for worker {}", worker_id.0));
                tx.recv()
            };

            match msg {
                Ok(ThreadPoolMessage::Run(job)) => job(),
                Err(_) => break,
            }
        }

        sentinel.cancel();
    });

    info!(worker_id = worker_id.0, "worker started:");
}
