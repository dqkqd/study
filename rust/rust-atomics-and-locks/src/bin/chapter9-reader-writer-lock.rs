use std::thread;

mod rwlock {
    use std::{
        cell::UnsafeCell,
        ops::{Deref, DerefMut},
        sync::atomic::{AtomicU32, Ordering},
    };

    pub struct RwLock<T> {
        /// The number of readers times 2,
        /// +1 if there is a waiting writer.
        ///
        /// even: all are readers
        /// odd: has one waiting writer.
        ///
        /// or u32::MAX if write-locked
        ///
        state: AtomicU32,
        /// Increment to wake up writers.
        writer_wake_counter: AtomicU32,
        value: UnsafeCell<T>,
    }

    unsafe impl<T> Sync for RwLock<T> where T: Send + Sync {}

    impl<T> RwLock<T> {
        pub const fn new(value: T) -> Self {
            Self {
                state: AtomicU32::new(0),
                writer_wake_counter: AtomicU32::new(0),
                value: UnsafeCell::new(value),
            }
        }

        pub fn read(&self) -> ReadGuard<T> {
            let mut s = self.state.load(Ordering::Relaxed);
            loop {
                if s % 2 == 0 {
                    assert!(s < u32::MAX - 1, "too many readers");
                    match self.state.compare_exchange(
                        s,
                        s + 2,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => return ReadGuard { rwlock: self },
                        Err(e) => s = e,
                    }
                }

                if s % 2 == 1 {
                    atomic_wait::wait(&self.state, s);
                    s = self.state.load(Ordering::Relaxed);
                }
            }
        }

        pub fn write(&self) -> WriteGuard<T> {
            let mut s = self.state.load(Ordering::Relaxed);
            loop {
                // read lock is unlocked
                if s <= 1 {
                    match self.state.compare_exchange(
                        s,
                        u32::MAX,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => return WriteGuard { rwlock: self },
                        Err(e) => {
                            s = e;
                            continue;
                        }
                    }
                }

                // block new reader
                if s % 2 == 0 {
                    match self.state.compare_exchange(
                        s,
                        s + 1,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            s = e;
                            continue;
                        }
                    }
                }

                let w = self.writer_wake_counter.load(Ordering::Acquire);
                s = self.state.load(Ordering::Relaxed);
                if s >= 2 {
                    atomic_wait::wait(&self.writer_wake_counter, w);
                    s = self.state.load(Ordering::Relaxed);
                }
            }
        }
    }

    pub struct ReadGuard<'a, T> {
        rwlock: &'a RwLock<T>,
    }

    pub struct WriteGuard<'a, T> {
        rwlock: &'a RwLock<T>,
    }

    impl<T> Deref for ReadGuard<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            let ptr = self.rwlock.value.get();
            unsafe { &*ptr }
        }
    }

    impl<T> Deref for WriteGuard<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            let ptr = self.rwlock.value.get();
            unsafe { &*ptr }
        }
    }

    impl<T> DerefMut for WriteGuard<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            let ptr = self.rwlock.value.get();
            unsafe { &mut *ptr }
        }
    }

    impl<T> Drop for ReadGuard<'_, T> {
        fn drop(&mut self) {
            if self.rwlock.state.fetch_sub(2, Ordering::Release) == 3 {
                self.rwlock
                    .writer_wake_counter
                    .fetch_add(1, Ordering::Release);
                atomic_wait::wake_one(&self.rwlock.writer_wake_counter);
            }
        }
    }

    impl<T> Drop for WriteGuard<'_, T> {
        fn drop(&mut self) {
            self.rwlock.state.store(0, Ordering::Release);
            self.rwlock
                .writer_wake_counter
                .fetch_add(1, Ordering::Release);
            atomic_wait::wake_one(&self.rwlock.writer_wake_counter);
            atomic_wait::wake_all(&self.rwlock.state);
        }
    }
}

fn main() {
    let rw = rwlock::RwLock::new(0);
    thread::scope(|s| {
        for _ in 0..100 {
            s.spawn(|| {
                for _ in 0..1_000 {
                    std::hint::black_box(&*rw.read());
                }
            });
        }

        for _ in 0..100 {
            s.spawn(|| {
                for _ in 0..1_000 {
                    *rw.write() += 1;
                }
            });
        }
    });

    assert_eq!(*rw.read(), 100_000);
}
