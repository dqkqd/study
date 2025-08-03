use std::{sync::atomic::AtomicPtr, thread, time::Duration};

#[derive(Debug)]
struct Data {}
impl Data {
    fn init() -> Data {
        thread::sleep(Duration::from_secs(1));
        println!("expensive computation");
        Data {}
    }
}

fn get_data() -> &'static Data {
    static PTR: AtomicPtr<Data> = AtomicPtr::new(std::ptr::null_mut());

    let mut ptr = PTR.load(std::sync::atomic::Ordering::Acquire);
    if ptr.is_null() {
        ptr = Box::into_raw(Box::new(Data::init()));
        if let Err(e) = PTR.compare_exchange(
            std::ptr::null_mut(),
            ptr,
            std::sync::atomic::Ordering::Release,
            std::sync::atomic::Ordering::Acquire,
        ) {
            unsafe { drop(Box::from_raw(ptr)) };
            ptr = e;
        }
    }

    unsafe { &*ptr }
}

fn main() {
    get_data();
    get_data();
    get_data();
    get_data();
}
