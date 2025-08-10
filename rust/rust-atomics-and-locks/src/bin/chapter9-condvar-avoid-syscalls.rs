mod condvar {
    use std::{
        cell::UnsafeCell,
        ops::{Deref, DerefMut},
        sync::atomic::{AtomicU32, AtomicUsize, Ordering},
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
                lock_contended(&self.state);
            }

            MutexGuard { mutex: self }
        }
    }

    fn lock_contended(state: &AtomicU32) {
        // let spin a bit before making the syscall
        let mut spin_count = 0;
        // only wait in uncontended case, if someone is already wait for the lock (state = 2)
        // we should break out immediately and make the syscall
        while state.load(Ordering::Relaxed) == 1 && spin_count < 100 {
            spin_count += 1;
            std::hint::spin_loop();
        }

        // unlock -> locked
        if state
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            return;
        }

        // make the lock state become `locked with waiter`
        // if original state = 0: we have successfully grabbed the lock
        // if original state = 1 or 2: someone is holding the lock, we need to wait.
        while state.swap(2, Ordering::Acquire) != 0 {
            // keep waiting while state = 2
            atomic_wait::wait(state, 2);
        }
    }

    pub struct Condvar {
        counter: AtomicU32,
        num_waiters: AtomicUsize,
    }

    impl Condvar {
        pub const fn new() -> Condvar {
            Condvar {
                counter: AtomicU32::new(0),
                num_waiters: AtomicUsize::new(0),
            }
        }

        pub fn notify_one(&self) {
            if self.num_waiters.load(Ordering::Relaxed) > 0 {
                self.counter.fetch_add(1, Ordering::Relaxed);
                atomic_wait::wake_one(&self.counter);
            }
        }

        pub fn notify_all(&self) {
            if self.num_waiters.load(Ordering::Relaxed) > 0 {
                self.counter.fetch_add(1, Ordering::Relaxed);
                atomic_wait::wake_all(&self.counter);
            }
        }

        pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
            self.num_waiters.fetch_add(1, Ordering::Relaxed);
            let counter_value = self.counter.load(Ordering::Relaxed);
            let mutex = guard.mutex;
            drop(guard);

            // wait while the counter_value hasn'd changed since
            atomic_wait::wait(&self.counter, counter_value);
            self.num_waiters.fetch_sub(1, Ordering::Relaxed);
            mutex.lock()
        }
    }

    #[test]
    fn test_condvar() {
        let mutex = Mutex::new(0);
        let condvar = Condvar::new();
        let mut wakeups = 0;
        std::thread::scope(|s| {
            s.spawn(|| {
                std::thread::sleep(std::time::Duration::from_secs(1));
                *mutex.lock() = 123;
                condvar.notify_one();
            });

            let mut m = mutex.lock();
            while *m < 100 {
                m = condvar.wait(m);
                wakeups += 1;
            }
            assert_eq!(*m, 123);
        });

        // Check that the main thread actually did wait (not busy-loop),
        // while still allowing for a few spurious wake ups.
        assert!(wakeups < 10);
    }
}

fn main() {
    println!("Hello world");
}
