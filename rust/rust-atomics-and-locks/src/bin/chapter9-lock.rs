use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Instant,
};

struct Mutex<T> {
    /// 0: unlocked
    /// 1: locked without waiters
    /// 2: locked with waiters
    state: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}

struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

unsafe impl<T> Sync for MutexGuard<'_, T> where T: Sync {}
unsafe impl<T> Send for MutexGuard<'_, T> where T: Send {}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        if self.mutex.state.swap(0, Ordering::Release) == 2 {
            // only trigger the kernel call if there was a contention
            atomic_wait::wake_one(&self.mutex.state);
        }
    }
}

impl<T> Mutex<T> {
    const fn new(value: T) -> Mutex<T> {
        Mutex {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    fn lock(&self) -> MutexGuard<T> {
        // unlock -> locked
        if self
            .state
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // cannot change, someone is holding the lock
            // make the lock state become `locked with waiter`
            // if original state = 0: we have successfully grabbed the lock
            // if original state = 1 or 2: someone is holding the lock, we need to wait.
            while self.state.swap(2, Ordering::Acquire) != 0 {
                // keep waiting while state = 2
                atomic_wait::wait(&self.state, 2);
            }
        }

        MutexGuard { mutex: self }
    }
}

fn uncontended_case() {
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    for _ in 0..5_000_000 {
        *m.lock() += 1;
    }
    let duration = start.elapsed();
    println!(
        "Uncontended case: locked {} times in {:?}",
        *m.lock(),
        duration
    );
}

fn contended_case() {
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                for _ in 0..5_000_000 {
                    *m.lock() += 1;
                }
            });
        }
    });
    let duration = start.elapsed();
    println!(
        "Contended case: locked {} times in {:?}",
        *m.lock(),
        duration
    );
}

fn main() {
    uncontended_case();
    contended_case();
}
