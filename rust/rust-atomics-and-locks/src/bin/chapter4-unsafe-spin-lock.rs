use std::{cell::UnsafeCell, sync::atomic::AtomicBool, thread, time::Duration};

struct UnsafeSpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for UnsafeSpinLock<T> where T: Send {}

impl<T> UnsafeSpinLock<T> {
    const fn new(value: T) -> UnsafeSpinLock<T> {
        UnsafeSpinLock {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    #[allow(clippy::mut_from_ref)]
    fn lock(&self) -> &mut T {
        while self
            .locked
            .compare_exchange_weak(
                false,
                true,
                // must use `Acquire` to ensure when we read-modify-write, we see
                // all the changes `after` the `Release` in `unlock`
                std::sync::atomic::Ordering::Acquire,
                // it is ok to use relaxed, since we don't care its returned value
                std::sync::atomic::Ordering::Relaxed,
                // See more: https://marabos.nl/atomics/building-spinlock.html#happens-before-diagram-spinlock
            )
            .is_err()
        {
            std::hint::spin_loop();
        }

        unsafe { &mut *self.value.get() }
    }

    unsafe fn unlock(&self) {
        // Use `Release` so others' Acquire can see it -> happens-before relationship
        self.locked
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

fn main() {
    let v = UnsafeSpinLock::new(10);
    thread::scope(|s| {
        for _ in 0..100 {
            s.spawn(|| {
                *v.lock() += 1;
                unsafe { v.unlock() };
            });
        }
        thread::sleep(Duration::from_millis(100));
        println!("{}", v.lock());
    });
}
