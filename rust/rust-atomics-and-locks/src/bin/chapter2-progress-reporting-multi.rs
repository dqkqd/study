use std::{
    sync::atomic::{AtomicU64, AtomicUsize},
    thread,
    time::{Duration, Instant},
};

use rand::{Rng, SeedableRng, rngs::StdRng};

fn process_item(t: usize) {
    thread::sleep(Duration::from_millis(
        StdRng::seed_from_u64(t as u64).random_range(0..200),
    ));
}

fn main() {
    let num_done = &AtomicUsize::new(0);
    let total_time = &AtomicU64::new(0);
    let max_time = &AtomicU64::new(0);

    thread::scope(|s| {
        for t in 0..4 {
            s.spawn(move || {
                for i in 0..25 {
                    let start = Instant::now();
                    process_item(t * 25 + i);
                    let time_taken = start.elapsed().as_micros() as u64;
                    total_time.fetch_add(time_taken, std::sync::atomic::Ordering::Relaxed);
                    max_time.fetch_max(time_taken, std::sync::atomic::Ordering::Relaxed);
                    num_done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            });
        }

        loop {
            let total_time =
                Duration::from_micros(total_time.load(std::sync::atomic::Ordering::Relaxed));
            let max_time =
                Duration::from_micros(max_time.load(std::sync::atomic::Ordering::Relaxed));
            let n = num_done.load(std::sync::atomic::Ordering::Relaxed);
            if n == 100 {
                break;
            }
            if n == 0 {
                println!("Working.. no thing done yet")
            } else {
                println!(
                    "Working.. {n} / 100 done, {:?} average, {:?} peak",
                    total_time / n as u32,
                    max_time
                );
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    println!("Done");
}
