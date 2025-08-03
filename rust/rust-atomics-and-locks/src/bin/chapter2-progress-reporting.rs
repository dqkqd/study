use std::{sync::atomic::AtomicUsize, thread, time::Duration};

fn process_item() {
    thread::sleep(Duration::from_millis(50));
}

fn main() {
    let num_done = AtomicUsize::new(0);
    let main_thread = thread::current();

    thread::scope(|s| {
        s.spawn(|| {
            for i in 0..100 {
                process_item();
                num_done.store(i + 1, std::sync::atomic::Ordering::Relaxed);
                main_thread.unpark();
            }
        });

        loop {
            let n = num_done.load(std::sync::atomic::Ordering::Relaxed);
            println!("Working.. {n} / 100 done");
            if n == 100 {
                break;
            }
            thread::park_timeout(Duration::from_millis(1000));
        }
    })
}
