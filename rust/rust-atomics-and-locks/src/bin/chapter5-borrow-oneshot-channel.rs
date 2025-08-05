use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

struct Sender<'a, T> {
    channel: &'a Oneshot<T>,
}
struct Receiver<'a, T> {
    channel: &'a Oneshot<T>,
}

struct Oneshot<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Oneshot<T> where T: Sync {}

impl<T> Oneshot<T> {
    fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }

    fn split<'a>(&'a mut self) -> (Sender<'a, T>, Receiver<'a, T>) {
        *self = Self::new();
        (Sender { channel: self }, Receiver { channel: self })
    }
}

impl<'a, T> Sender<'a, T> {
    fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Ordering::Release);
    }
}

impl<'a, T> Receiver<'a, T> {
    fn is_ready(&self) -> bool {
        self.channel.ready.load(Ordering::Relaxed)
    }

    fn receive(&self) -> T {
        if !self.channel.ready.swap(false, Ordering::Acquire) {
            panic!("no message available!");
        }
        unsafe { (*self.channel.message.get()).assume_init_read() }
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
    let mut oneshot = Oneshot::new();
    let (tx, rx) = oneshot.split();
    let t = thread::current();

    thread::scope(|s| {
        s.spawn(|| {
            tx.send("Hello");
            t.unpark();
        });

        while !rx.is_ready() {
            thread::park()
        }
        println!("{}", rx.receive());
    });
}
