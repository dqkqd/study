use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

struct Oneshot<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    in_use: AtomicBool,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Oneshot<T> where T: Sync {}

impl<T> Oneshot<T> {
    fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            in_use: AtomicBool::new(false),
            ready: AtomicBool::new(false),
        }
    }

    fn send(&self, message: T) {
        if self.in_use.swap(true, Ordering::Relaxed) {
            panic!("already in use")
        }
        unsafe { (*self.message.get()).write(message) };
        self.ready.store(true, Ordering::Release);
    }

    fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }

    fn receive(&self) -> T {
        if !self.ready.swap(false, Ordering::Acquire) {
            panic!("no message available!");
        }
        unsafe { (*self.message.get()).assume_init_read() }
    }
}

impl<T> Drop for Oneshot<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe {
                (*self.message.get()).assume_init_drop();
            }
        }
    }
}

fn main() {
    let c = Oneshot::new();
    let t = thread::current();

    thread::scope(|s| {
        s.spawn(|| {
            c.send("Hello");
            t.unpark();
        });

        while !c.is_ready() {
            thread::park()
        }
        println!("{}", c.receive());
    });
}
