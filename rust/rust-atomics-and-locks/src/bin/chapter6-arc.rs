mod arc {
    use std::{
        ops::Deref,
        ptr::NonNull,
        sync::atomic::{AtomicUsize, Ordering, fence},
    };
    struct ArcData<T> {
        ref_count: AtomicUsize,
        data: T,
    }

    pub struct Arc<T> {
        ptr: NonNull<ArcData<T>>,
    }

    unsafe impl<T> Sync for Arc<T> where T: Send + Sync {}
    unsafe impl<T> Send for Arc<T> where T: Send + Sync {}

    impl<T> Arc<T> {
        pub fn new(value: T) -> Self {
            let data = ArcData {
                ref_count: AtomicUsize::new(1),
                data: value,
            };
            Self {
                ptr: NonNull::from(Box::leak(Box::new(data))),
            }
        }

        pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
            if arc.data().ref_count.load(Ordering::Relaxed) == 1 {
                fence(Ordering::Acquire);
                unsafe { Some(&mut arc.ptr.as_mut().data) }
            } else {
                None
            }
        }

        fn data(&self) -> &ArcData<T> {
            unsafe { self.ptr.as_ref() }
        }
    }

    impl<T> Deref for Arc<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.data().data
        }
    }

    impl<T> Clone for Arc<T> {
        fn clone(&self) -> Self {
            if self.data().ref_count.fetch_add(1, Ordering::Relaxed) > usize::MAX / 2 {
                std::process::abort();
            }
            Self { ptr: self.ptr }
        }
    }

    impl<T> Drop for Arc<T> {
        fn drop(&mut self) {
            if self.data().ref_count.fetch_sub(1, Ordering::Release) == 1 {
                fence(Ordering::Acquire);
                unsafe { drop(Box::from_raw(self.ptr.as_ptr())) }
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
}

fn main() {}
