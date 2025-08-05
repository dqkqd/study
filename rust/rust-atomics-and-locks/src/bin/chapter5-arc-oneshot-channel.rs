use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let a = Arc::new(Oneshot {
        message: UnsafeCell::new(MaybeUninit::uninit()),
        ready: AtomicBool::new(false),
    });
    (Sender { channel: a.clone() }, Receiver { channel: a })
}
struct Sender<T> {
    channel: Arc<Oneshot<T>>,
}
struct Receiver<T> {
    channel: Arc<Oneshot<T>>,
}

struct Oneshot<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Oneshot<T> where T: Sync {}

impl<T> Sender<T> {
    fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Ordering::Release);
    }
}

impl<T> Receiver<T> {
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
    let (tx, rx) = channel();
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
