use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicBool,
    thread,
};

struct SafeSpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for SafeSpinLock<T> where T: Send {}

struct Guard<'a, T> {
    lock: &'a SafeSpinLock<T>,
}

unsafe impl<T> Sync for Guard<'_, T> where T: Sync {}
unsafe impl<T> Send for Guard<'_, T> where T: Send {}

impl<T> SafeSpinLock<T> {
    const fn new(value: T) -> SafeSpinLock<T> {
        SafeSpinLock {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    fn lock(&self) -> Guard<T> {
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

        Guard { lock: self }
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock
            .locked
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

fn main() {
    let x = SafeSpinLock::new(Vec::new());
    thread::scope(|s| {
        s.spawn(|| x.lock().push(1));
        s.spawn(|| {
            let mut g = x.lock();
            g.push(2);
            g.push(2);
        });
    });
    let g = x.lock();
    assert!(g.as_slice() == [1, 2, 2] || g.as_slice() == [2, 2, 1]);
}
