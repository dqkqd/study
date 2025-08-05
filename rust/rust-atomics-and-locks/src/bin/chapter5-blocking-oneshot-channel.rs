use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

struct Sender<'a, T> {
    channel: &'a Oneshot<T>,
    receiving_thread: thread::Thread,
}
struct Receiver<'a, T> {
    channel: &'a Oneshot<T>,
    _no_send: PhantomData<*const ()>,
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
        (
            Sender {
                channel: self,
                receiving_thread: thread::current(),
            },
            Receiver {
                channel: self,
                _no_send: PhantomData,
            },
        )
    }
}

impl<'a, T> Sender<'a, T> {
    fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Ordering::Release);
        self.receiving_thread.unpark();
    }
}

impl<'a, T> Receiver<'a, T> {
    fn receive(&self) -> T {
        while !self.channel.ready.swap(false, Ordering::Acquire) {
            thread::park();
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
    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_millis(500));
            tx.send("Hello");
        });
        println!("{}", rx.receive());
    });
}
