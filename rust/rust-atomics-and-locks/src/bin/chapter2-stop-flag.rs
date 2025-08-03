use std::{sync::atomic::AtomicBool, thread, time::Duration};

fn main() {
    static STOP: AtomicBool = AtomicBool::new(false);
    let background_thread = thread::spawn(|| {
        while STOP.load(std::sync::atomic::Ordering::Relaxed) {
            println!("work");
            thread::sleep(Duration::from_millis(500));
        }
    });

    for line in std::io::stdin().lines() {
        match line.unwrap().as_str() {
            "help" => println!("command: help, stop"),
            "stop" => break,
            cmd => println!("unknown command: {cmd:?}"),
        }
    }
    STOP.store(true, std::sync::atomic::Ordering::Relaxed);
    background_thread.join().unwrap();
}
