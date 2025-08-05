use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
    thread,
};

struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

impl<T> Channel<T> {
    fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new(),
        }
    }

    fn send(&self, message: T) {
        self.queue.lock().unwrap().push_back(message);
        self.item_ready.notify_one();
    }

    fn receive(&self) -> T {
        let mut q = self.queue.lock().unwrap();
        loop {
            match q.pop_front() {
                Some(message) => return message,
                None => {
                    q = self.item_ready.wait(q).unwrap();
                }
            }
        }
    }
}

fn main() {
    let c = Channel::new();

    thread::scope(|s| {
        s.spawn(|| {
            for i in 0..10 {
                c.send(i);
            }
        });

        for _ in 0..10 {
            println!("{}", c.receive());
        }
    });
}
