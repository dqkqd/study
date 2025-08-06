mod arc {
    use std::{
        cell::UnsafeCell,
        ops::Deref,
        ptr::NonNull,
        sync::atomic::{AtomicUsize, Ordering, fence},
    };

    struct ArcData<T> {
        data_ref_count: AtomicUsize,
        alloc_ref_count: AtomicUsize,
        data: UnsafeCell<Option<T>>,
    }

    pub struct Arc<T> {
        weak: Weak<T>,
    }

    pub struct Weak<T> {
        ptr: NonNull<ArcData<T>>,
    }

    unsafe impl<T> Sync for Weak<T> where T: Send + Sync {}
    unsafe impl<T> Send for Weak<T> where T: Send + Sync {}

    impl<T> Weak<T> {
        fn data(&self) -> &ArcData<T> {
            unsafe { self.ptr.as_ref() }
        }

        fn upgrade(&self) -> Option<Arc<T>> {
            let mut n = self.data().data_ref_count.load(Ordering::Relaxed);
            loop {
                if n == 0 {
                    return None;
                }
                assert!(n <= usize::MAX / 2);
                if let Err(e) = self.data().data_ref_count.compare_exchange_weak(
                    n,
                    n + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    n = e;
                    continue;
                }
                return Some(Arc { weak: self.clone() });
            }
        }
    }

    impl<T> Clone for Weak<T> {
        fn clone(&self) -> Self {
            if self.data().alloc_ref_count.fetch_add(1, Ordering::Relaxed) > usize::MAX / 2 {
                std::process::abort();
            }
            Self { ptr: self.ptr }
        }
    }

    impl<T> Drop for Weak<T> {
        fn drop(&mut self) {
            if self.data().alloc_ref_count.fetch_sub(1, Ordering::Release) == 1 {
                fence(Ordering::Acquire);
                unsafe { drop(Box::from_raw(self.ptr.as_ptr())) }
            }
        }
    }

    impl<T> Arc<T> {
        pub fn new(value: T) -> Self {
            let data = ArcData {
                data_ref_count: AtomicUsize::new(1),
                alloc_ref_count: AtomicUsize::new(1),
                data: UnsafeCell::new(Some(value)),
            };
            Self {
                weak: Weak {
                    ptr: NonNull::from(Box::leak(Box::new(data))),
                },
            }
        }

        pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
            if arc.weak.data().alloc_ref_count.load(Ordering::Relaxed) == 1 {
                fence(Ordering::Acquire);
                let arcdata = unsafe { arc.weak.ptr.as_mut() };
                let option = arcdata.data.get_mut();
                let data = option.as_mut().unwrap();
                Some(data)
            } else {
                None
            }
        }

        fn downgrade(arc: &Self) -> Weak<T> {
            arc.weak.clone()
        }
    }

    impl<T> Deref for Arc<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            let ptr = self.weak.data().data.get();
            unsafe { (*ptr).as_ref().unwrap() }
        }
    }

    impl<T> Clone for Arc<T> {
        fn clone(&self) -> Self {
            if self
                .weak
                .data()
                .data_ref_count
                .fetch_add(1, Ordering::Relaxed)
                > usize::MAX / 2
            {
                std::process::abort();
            }
            Self {
                weak: self.weak.clone(),
            }
        }
    }

    impl<T> Drop for Arc<T> {
        fn drop(&mut self) {
            if self
                .weak
                .data()
                .data_ref_count
                .fetch_sub(1, Ordering::Release)
                == 1
            {
                fence(Ordering::Acquire);
                let ptr = self.weak.data().data.get();
                unsafe { (*ptr) = None };
            }
        }
    }

    #[test]
    fn test() {
        static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);
        struct DetectDrop;
        impl Drop for DetectDrop {
            fn drop(&mut self) {
                NUM_DROPS.fetch_add(1, Ordering::Relaxed);
            }
        }

        let x = Arc::new(("hello", DetectDrop));
        let y = x.clone();
        let t = std::thread::spawn(move || {
            assert_eq!(x.0, "hello");
        });

        assert_eq!(y.0, "hello");

        t.join().unwrap();

        assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 0);

        drop(y);
        assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_mut() {
        let x = Arc::new("hello");
        let mut y = x.clone();
        let t = std::thread::spawn(move || {
            assert_eq!(*x, "hello");
        });

        assert_eq!(*y, "hello");
        assert_eq!(Arc::get_mut(&mut y), None);

        t.join().unwrap();

        assert!(Arc::get_mut(&mut y).is_some());
        assert_eq!(*Arc::get_mut(&mut y).unwrap(), "hello");
    }

    #[test]
    fn test_weak() {
        let x = Arc::new("hello");
        let y = Arc::downgrade(&x);
        let z = Arc::downgrade(&x);
        let t = std::thread::spawn(move || {
            let y = y.upgrade().unwrap();
            assert_eq!(*y, "hello");
        });

        assert_eq!(*x, "hello");
        t.join().unwrap();
        assert!(z.upgrade().is_some());

        drop(x);
        assert!(z.upgrade().is_none());
    }
}

fn main() {}
