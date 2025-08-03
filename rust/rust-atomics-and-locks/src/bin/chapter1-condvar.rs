use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
    thread,
    time::Duration,
};

fn main() {
    let queue = Mutex::new(VecDeque::new());
    let not_empty = Condvar::new();
    thread::scope(|s| {
        s.spawn(|| {
            loop {
                let mut guard = not_empty
                    .wait_while(queue.lock().unwrap(), |q| q.is_empty())
                    .unwrap();
                dbg!(guard.pop_front().unwrap());
            }
        });

        for i in 0.. {
            queue.lock().unwrap().push_back(i);
            not_empty.notify_one();
            thread::sleep(Duration::from_millis(500));
        }
    });
}
