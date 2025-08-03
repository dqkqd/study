use std::{collections::VecDeque, sync::Mutex, thread, time::Duration};

fn main() {
    let queue = Mutex::new(VecDeque::new());
    thread::scope(|s| {
        let t = s.spawn(|| {
            loop {
                let v = queue.lock().unwrap().pop_front();
                if let Some(v) = v {
                    dbg!(v);
                } else {
                    thread::park();
                }
            }
        });

        for i in 0.. {
            queue.lock().unwrap().push_back(i);
            t.thread().unpark();
            thread::sleep(Duration::from_millis(500));
        }
    });
}
