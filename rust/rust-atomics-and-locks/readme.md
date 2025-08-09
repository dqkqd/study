# Rust Atomics and Locks

Notes and exercises from [Rust Atomics and Locks](https://marabos.nl/atomics/).
(_however, I only note what I did not know_).

## Chapter 1: Basics

### `t.join()`

`t.join.()` does not block (thus does not guarantee order). Before
reading this chapter, I thought this was a block operation, and the main process
would wait for `t` to complete before executing.

```rust
fn main() {
    let t1 = thread::spawn(f);
    let t2 = thread::spawn(f);
    println!("Hello from the main thread");
    t1.join().unwrap();
    t2.join().unwrap();
}
```

### `thread:scope`

[`thread::scope`](https://doc.rust-lang.org/stable/std/thread/fn.scope.html)
allows us to define thread that cannot outlive a certain scope. All threads
will be joined at the end of the scope.

```rust
 use std::thread;

 let mut a = vec![1, 2, 3];
 let mut x = 0;

 thread::scope(|s| {
     s.spawn(|| {
         println!("hello from the first scoped thread");
         // We can borrow `a` here.
         dbg!(&a);
     });
     s.spawn(|| {
         println!("hello from the second scoped thread");
         // We can even mutably borrow `x` here,
         // because no other threads are using it.
         x += a[0] + a[2];
     });
     println!("hello from the main thread");
 });

 // After the scope, we can modify and access our variables again:
 a.push(4);
 assert_eq!(x, a.len());
```

### `Box::leak`

[`Box::leak`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak)
releases ownership of `Box`, making it lives until the end of the program. Thus,
allowing it to be borrowed and used by any thread.

```rust
 let x: &'static = [i32; 3] = Box::leak(Box::new([1, 2, 3]));
 thread::spawn(move || dbg!(x));
 thread::spawn(move || dbg!(x));
```

### Undefined behavior can "travel back in time"

Suppose we have these snippet:

```rust
 match index {
     0 => x(),
     1 => y(),
     _ => z(index)
 }
 // ...
 let a = [123, 456, 789];
 let b = unsafe { a.get_unchecked(index) };
```

The compiler will optimize the above snippet into:

```rust
 match index {
     0 => x(),
     1 => y(),
     _ => z(2) // 2. because `index` can only be 0,1,2; `z(index)` became `z(2)`.
 }
 // suppose index=3, the `a.get_unchecked(index)` would panic. However
 // since `z(index)` - which should be `z(3)` - became `z(2)`, the program
 // becomes undefined even before reaching the `a.get_unchecked(index)`.
 // making it hard to reason about.
 // ...
 let a = [123, 456, 789];
 let b = unsafe { a.get_unchecked(index) }; // 1. `index` can only be 0,1,2
```

### `Cell`

[`Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html) cannot
modify the underlying value, but [`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html) can.

Using `Cell`, we need to take the value out, modify it, and put the value back.

```rust
 fn f(v: &Cell<Vec<i32>>) {
     let mut v2 = v.take(); // take the value out
     v2.push(1); // modify the value
     v.set(v2); // put the value back to the cell
 }
```

Using `RefCell`, we can just modify the value inplace.

```rust
 fn f(v: &RefCell<Vec<i32>>) {
     v.borrow_mut().push(1);
 }
```

All interior mutability rely on [`UnsafeCell`](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html)

### Send and Sync

A type is `Send` if it can be sent to another thread.

- `Rc<i32>` is not `Send` because its reference counter is not thread safe.
  which could lead to many threads `drop` it at the same time.
- `Arc<i32>` is `Send` because its reference counter is `Atomic` (and thus, thread safe).
- `&i32` is `Send` because it reference to something immutable.
- `Cell<i32>` is `Send` even though its value can be mutate, because after `Send`,
  only one thread owns it. However `&Cell<i32>` is not `Send` because another thread
  can hold `&Cell<i32>` and mutate its value.

A type is `Sync` if it can be shared with another thread.
`T` is `Sync` iff `&T` is sent.

- `i32` is `Sync` because `&i32` is `Send`
- `Cell<i32>` is not `Sync` because `&Cell<i32>` is not `Send`.

### PhantomData

We can use [Phantom type parameter](https://doc.rust-lang.org/rust-by-example/generics/phantom.html)
to disable `Send` or `Sync` to the struct that has all of its field implement `Send` or `Sync`.

```rust
 use std::marker::PhantomData;
 struct X {
     handle: i32,
     _not_sync: PhantomData<Cell<()>>, // Cell<()> is not Sync
 }
```

### Mutex

A [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html) can be
[`poisoned`](https://doc.rust-lang.org/std/sync/struct.PoisonError.html)
if a thread panics when holding the lock. Calling `lock()` on a `Mutex` returns
a `Result` to determine whether its thread is poisoned.

`MutexGuard` returned by `lock()` will be dropped at the end of the statement.

```rust
// case 1
// this keep the guard until the end of the if block
if let Some(item) = list.lock().unwrap().pop() {
    process_item(item);
} // guard dropped here

// case 2
let item = list.lock().unwrap().pop();
// guard dropped here
if let Some(item) = item {
    process_item(item);
}
```

### park and unpark

Thread can [`park`](https://doc.rust-lang.org/std/thread/fn.park.html) itself
and go into sleep, while other thread can [`unpark`](https://doc.rust-lang.org/std/thread/struct.Thread.html#method.unpark)
the parked thread.

```rust
use std::{collections::VecDeque, sync::Mutex, thread, time::Duration};

fn main() {
    let queue = Mutex::new(VecDeque::new());
    thread::scope(|s| {
        let t = s.spawn(|| {
            loop {
                let v = queue.lock().unwrap().pop_front();
                if let Some(v) = v {
                    dbg!(v);
                } else {
                    thread::park();
                }
            }
        });

        for i in 0.. {
            queue.lock().unwrap().push_back(i);
            t.thread().unpark();
            thread::sleep(Duration::from_millis(500));
        }
    });
}
```

### `Condition Variables`

[`Condvar`](https://doc.rust-lang.org/std/sync/struct.Condvar.html)
can be used to wait and notify threads. Threads can wait on a condition variable
and be waked up later when another thread notifies the same condition variable.

```rust
use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
    thread,
    time::Duration,
};

fn main() {
    let queue = Mutex::new(VecDeque::new());
    let not_empty = Condvar::new();
    thread::scope(|s| {
        s.spawn(|| {
            loop {
                let mut guard = not_empty
                    .wait_while(queue.lock().unwrap(), |q| q.is_empty())
                    .unwrap();
                dbg!(guard.pop_front().unwrap());
            }
        });

        for i in 0.. {
            queue.lock().unwrap().push_back(i);
            not_empty.notify_one();
            thread::sleep(Duration::from_millis(500));
        }
    });
}
```

## Chapter 2: Atomics

### Compare exchange

`compare_exchange` are compare-and-swap operation in Rust.
`compare_exchange_week` is similar to `compare_exchange` but it can fail.
`compare_exchange_week` should be prefer to `compare_exchange` if the consequence
of the failure is insignificant.

```rust
 // allocation without overflow
 fn allocation_new_id() -> u32 {
     static NEXT_ID = AtomicU32::new(0);
     let mut id = NEXT_ID.load(Relaxed);
     loop {
         assert!(id < 1000, "too many IDs!");
         match NEXT_ID.compare_exchange_week(id, id + 1, Relaxed, Relaxed) {
             Ok(_) => return id,
             Err(v) = id = v;
         }
     }
 }
```

We can replace the loop above with a convenience method
[`fetch_update`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicU32.html#method.fetch_update)

### Lazy initialization

Simple implementation of Lazy initialization can create thundering herd.

```rust
 fn lazy_init() -> i32 {
     static VALUE = AtomicI32::new(0);
     let mut value = VALUE.load(Relaxed); // <-- lots of threads execute this and return 0 at the same time
     if value == 0 { // <-- this check is true for *lots of threads*
         value = compute(); // <-- *lots of threads* execute this
         VALUE.store(value, Relaxed)
     }
 }
```

This can be avoid with `compare_exchange` operation

```rust
 static UNINITIALIZED = 0;
 static INITIALIZING = 1;

 fn lazy_init() -> i32 {
     static VALUE = AtomicI32::new(UNINITIALIZED);
     if VALUE.compare_exchange(UNINITIALIZED, INITIALIZING, Relaxed, Relaxed) == UNINITIALIZED { // <-- only one thread can win this
         let value = compute();
         VALUE.store(value, Relaxed);
     }
 }
```

## Chapter 3: Memory Ordering

### Relaxed

Using `Relaxed` results in no memory ordering.
The compiler free to reorder instructions in more performant ways.

Suppose `a()` and `b()` are executed by different threads.

```rust
 static X: AtomicI32 = AtomicI32::new(0);
 static Y: AtomicI32 = AtomicI32::new(0);

 fn a() {
     X.store(10, Relaxed); // 1
     Y.store(20, Relaxed); // 2
 }

 fn b() {
     let y = Y.load(Relaxed); // 3
     let x = X.load(Relaxed); // 4
     println!("{x} {y}");
 }
```

The output can be (easy):

- 10 20: 1 -> 2 -> 3 -> 4
- 0 0: 3 -> 4 -> 1 -> 2
- 10 0: 1 -> 3 -> 4 -> 2

However, it can also be 20 0. Because the compiler can reorder instructions.

- 3 does not depend on 2, so output of 3 can be 0 or 20.
- 4 does not depend on 1, so output of 4 can be 0 or 10.

### Spawning and Joining

Spawning and Joining threads result in happens-before relationship.

Given the following example, it can never fail.

```rust
 static X = AtomicI32::new(0);

 fn main() {
     X.store(1, Relaxed);
     let t = thread::spawn(f);
     X.store(2, Relaxed);
     t.join().unwrap();
     X.store(3, Relaxed);
 }

 fn f() {
     let x = X.load(Relaxed);
     assert!(x == 1 || x == 2);
 }
```

- The spawn of `t` happens after the `X.store(1, Relaxed)`,
  which means `X` can never be 0 in `f()`.
- The join of `t` happens after the `X.store(2, Relaxed)`,
  which means `X` can be `2` or `1`, but never be `3`.

![image](https://marabos.nl/atomics/images/raal_0302.png)

### Release and Acquire

Release and Acquire can create happens-before relationship between threads.

From the [nomicon book](https://doc.rust-lang.org/nomicon/atomics.html#acquire-release)

- operations occur after an acquire stay after it.
- operations occur before a release stay before it.

```rust
 use std::sync::Arc;
 use std::sync::atomic::{AtomicBool, Ordering};
 use std::thread;

 fn main() {
     let lock = Arc::new(AtomicBool::new(false)); // value answers "am I locked?"

     // ... distribute lock to threads somehow ...

     // Try to acquire the lock by setting it to true
     while lock.compare_and_swap(false, true, Ordering::Acquire) { }
     // broke out of the loop, so we successfully acquired the lock!

     // ... scary data accesses ...

     // ok we're done, release the lock
     lock.store(false, Ordering::Release);
 }
```

From the book:

```rust
 use std::sync::atomic::Ordering::{Acquire, Release};

 static DATA: AtomicU64 = AtomicU64::new(0);
 static READY: AtomicBool = AtomicBool::new(false);

 fn main() {
     thread::spawn(|| {
         DATA.store(123, Relaxed);
         READY.store(true, Release); // Everything from before this store ..
     });
     while !READY.load(Acquire) { // .. is visible after this loads `true`.
         thread::sleep(Duration::from_millis(100));
         println!("waiting...");
     }
     println!("{}", DATA.load(Relaxed));
 }
```

![image](https://marabos.nl/atomics/images/raal_0303.png)

Let's see an example from [`swap`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicBool.html#method.swap)

```
Stores a value into the bool, returning the previous value.

swap takes an Ordering argument which describes the memory ordering of this operation.
All ordering modes are possible. Note that using Acquire makes the store part of this
operation Relaxed, and using Release makes the load part Relaxed.

Note: This method is only available on platforms that support atomic operations on u8.
```

Suppose we are using `Acquire` for `swap`, the operations become:

```rust
let old = self.load(Acquire);
self.store(new, Relaxed);
return old
```

Since `Acquire` makes all operations after it stay after it. We can never have
cases that `store` stores `new` into `self` before `load`.

Similar to `Release`, we can never have cases that `load` loads `new` from
`self`.

```rust
let old = self.load(Relaxed);
self.store(new, Release);
return old
```

### Consumer ordering

Consumer ordering is lightweight synchronization depends on the loaded value.
Where consume-load of a value happens after release-store for that value

### SeqCst

[`SeqCst`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html#variant.SeqCst) (sequential ordering)
is the strongest memory ordering it can replace both `Acquire` and `Release`.
_In practice, we never need to use sequential ordering_.

### fence

[`fence`](https://doc.rust-lang.org/std/sync/atomic/fn.fence.html)
allows separating memory ordering from atomic operation.

- a release-store `x.store(Release)` can be broken down to a `fence(Release)` and a `x.store(Relaxed)`.
- a acquire-load `x.load(Acquire)` can be broken down to a `x.load(Relaxed)` and a `fence(Acquire)`.

Separating `PTR.load(Acquire)` to use `Relaxed` with `fence(Acquire)` in the following example
allowing to only use acquire release relationship when `p` is not null.

```rust
let p = PTR.load(Relaxed);
if p.is_null() {
    println!("no data");
} else {
    fence(Acquire);
    println!("data = {}", unsafe { *p });
}
```

A more complicated example:

```rust
use std::sync::atomic::fence;

static mut DATA: [u64; 10] = [0; 10];

const ATOMIC_FALSE: AtomicBool = AtomicBool::new(false);
static READY: [AtomicBool; 10] = [ATOMIC_FALSE; 10];

fn main() {
    for i in 0..10 {
        thread::spawn(move || {
            let data = some_calculation(i);
            unsafe { DATA[i] = data };
            READY[i].store(true, Release);
        });
    }
    thread::sleep(Duration::from_millis(500));
    let ready: [bool; 10] = std::array::from_fn(|i| READY[i].load(Relaxed));
    if ready.contains(&true) {
        fence(Acquire); // <-- only use acquire memory ordering if one of the data is ready.
        for i in 0..10 {
            if ready[i] {
                println!("data{i} = {}", unsafe { DATA[i] });
            }
        }
    }
}
```

## Chapter 4: SpinLock

[Safe guard SpinLock](./src/bin/chapter4-safe-guard-spin-lock.rs)

## Chapter 5: Channels

[Simple Mutex](./src/bin/chapter5-mutex-channel.rs)
[Oneshot Channel](./src/bin/chapter5-oneshot-channel.rs)
[Arc Oneshot Channel](./src/bin/chapter5-arc-oneshot-channel.rs)
[Borrowed Oneshot Channel](./src/bin/chapter5-borrow-oneshot-channel.rs)
[Blocking Oneshot Channel](./src/bin/chapter5-blocking-oneshot-channel.rs)

## Chapter 6: Arc

From the drop implementation in [Basic Reference Counting](https://marabos.nl/atomics/building-arc.html#basic-reference-counting),
why we choose `Release` for `fetch_sub` and call `fence(Acquire)` later?

```rust
    impl<T> Drop for Arc<T> {
        fn drop(&mut self) {
            if self.data().ref_count.fetch_sub(1, Ordering::Release) == 1 {
                fence(Ordering::Acquire);
                unsafe { drop(Box::from_raw(self.ptr.as_ptr())) }
            }
        }
    }
```

Quote back from the [nomicon book](https://doc.rust-lang.org/nomicon/atomics.html#acquire-release)

- Acquire: operations occur after stay after.
- Release: operations occur before stay before.

Using `Release` ensures each thread's drop happens before the final.
Using `fence(Acquire)` ensures the final thread see all the changes from the `Release` drop.
Otherwise, no happens-before relationship can be establish.

## Chapter 7: Hardware

### x86-64 lock prefix

x86 use `lock` prefix for `fetch_add`.

```rust
pub fn a(x: &AtomicI32) {
    x.fetch_add(10, Relaxed);
}
```

Is compiled to:

```asm
 a:
     lock add dword ptr [rdi], 10
     ret
```

x86 use `lock` with label for `fetch_or`.

```rust
pub fn a(x: &AtomicI32) -> i32 {
    x.fetch_or(10, Ordering::Relaxed)
}
```

It takes the value from `eax` at every iteration (the `mov ecx, eax` line) -
this means if other threads execute the same instruction, they also write
to the same `eax` register.
Then it performs `or` operation and compare and exchange the value.

```asm
a:
        mov     eax, dword ptr [rdi]
.LBB0_1:
        mov     ecx, eax
        or      ecx, 10
        lock    cmpxchg dword ptr [rdi], ecx
        jne     .LBB0_1
        ret
```

### ARM lock use Store-Conditional and Lock-Linked

Store-Conditional instruction refuses to store memory if any other thread
has overwritten that memory since the load-linked instruction.

- only one memory address per core can be tracked.
- store-conditional has false negative, it can fail even the memory hasn't
  changed.

Here is a `fetch_add` example.

```rust
pub fn a(x: &AtomicI32) {
    x.fetch_add(10, Relaxed);
}
```

```asm
 a:
 .L1:
     ldxr w8, [x0]       ; load x0 to w8
     add w9, w8, #10     ; add 10 to w8 and store to w9
     stxr w10, w9, [x0]  ; store-conditional w9 to x0, return success/fail to w10
     cbnz w10, .L1       ; loopback if w10 failed, otherwise break
     ret
```

Here is a `compare_exchange_weak` example (the exchange can fail)

```rust
 pub fn a(x: &AtomicI32) {
     x.compare_exchange_weak(5, 6, Relaxed, Relaxed);
 }
```

```asm
 a:
     ldxr w8, [x0]       ; load x0 to w8
     cmp w8, #5          ; compare w8 with 5
     b.ne .L1            ; if it is not equal, jump to L1 (which is clear the result and return)
     mov w8, #6          ; move 6 to w8
     stxr w9, w8, [x0]   ; store-conditional w8 back to x0, return success/fail to w9
     ret                 ; just return, do not care about w9 result
 .L1:
     clrex
     ret
```

### Caching protocols

write-through protocol does not cache writes, but sends those writes to the
next layer - which has a shared communication channel - so that they can be
observed by other threads.

The MESI protocol allows caches from a thread to communicate with other
threads at the same cache level. If a cache miss occurs, it asks other
threads in the same cache level whether they have the cached values,
if all of them say no, it checks the next cache layer.

#### black_box

[`black_box(x)`](https://doc.rust-lang.org/std/hint/fn.black_box.html)
tells the compiler that `x` is being used. This function is useful for
benchmarking, avoiding the compiler to optimize the operations that we are
trying to benchmark.

```rust
 use std::hint::black_box;

 static A: AtomicU64 = AtomicU64::new(0);

 fn main() {
     black_box(&A); // New!
     let start = Instant::now();
     for _ in 0..1_000_000_000 {
         black_box(A.load(Relaxed)); // New!
     }
     println!("{:?}", start.elapsed());
 }
```

#### Cache perline

Caching happens per _cache line_, which is usually 64 byte.

The main thread only loads `A[1]`, the background thread store `A[0]` and `A[2]`.
Even though they are separated elements, they share the same cache line.
Thus both storing `A[0], A[2]` and loading `A[1]` need to lock the same cache line.

```rust
static A: [AtomicU64; 3] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

fn main() {
    black_box(&A);
    thread::spawn(|| {
        loop {
            A[0].store(0, Relaxed);
            A[2].store(0, Relaxed);
        }
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A[1].load(Relaxed));
    }
    println!("{:?}", start.elapsed());
}
```

Having known the cache line is 64 bytes, we can align each element with 64 bytes.
This allows `A[0], A[2]` and `A[1]` to be executed separately without locking.

```rust
 #[repr(align(64))] // This struct must be 64-byte aligned.
 struct Aligned(AtomicU64);

 static A: [Aligned; 3] = [
     Aligned(AtomicU64::new(0)),
     Aligned(AtomicU64::new(0)),
     Aligned(AtomicU64::new(0)),
 ];

 fn main() {
     black_box(&A);
     thread::spawn(|| {
         loop {
             A[0].0.store(1, Relaxed);
             A[2].0.store(1, Relaxed);
         }
     });
     let start = Instant::now();
     for _ in 0..1_000_000_000 {
         black_box(A[1].0.load(Relaxed));
     }
     println!("{:?}", start.elapsed());
}
```

When multiple atomic variables are related, putting them close to others (to the
same cache line) can avoid multiple lockings.

### Ordering instructions at lowlevel

Instructions can happen out of order, here are some examples:

- Buffered writes: write can be buffered, while processors can execute the
  next instructions without waiting.
- Invalidation caches: caches can be invalidated, however the invalidations
  are asynchronously executed to avoid blocking, results in slightly staled
  cache.
- Pipelining: instructions can be executed in parallel making they being
  reordered.

x86-64 is strongly ordered, most its operations are not reordered. Relaxed
operations cost as "expensive" as Acquire / Release operations.

ARM64 is weakly ordered, their Relexed operations is not the same as
Acquire / Release.

## Chapter 8: Operating System

### pthread

`pthread` provides several synchronizations using `pthread_mutex_t`,
`pthread_rwlock_t`, `pthread_cond_t`, etc. Each of them has their own `init()`
and `destroy()` methods.

Calling `pthread` can be done using [`libc`](https://docs.rs/crate/libc/latest)
crate in Rust.
However, wrapping `pthread` syscall using `libc` in Rust is not straightforward
because Rust's ownership model _moves_ values around.
For example, in C, lock and unlock look at a specific atomic variable address,
this can't be done in Rust because when unlock is called, the atomic variable's
address may have changed.

### Futex

Futex (fast user-space mutex) is a user-space locking technique with minimal
kernel involvement.

It has two main syscalls

- wait: puts thread to sleep, it takes a 32-bit integer and compares it with the
  internal 32-bit atomic value to decide whether sleeping should occur.
- wake: wakes the waiting threads, then increments the internal atomic value.
  Two important notes:

- A wake can be lost if it is called _right before_ wait. Therefore, we allow
  wakes to apply to the next wait as well.
- A wake has two operations: increment the internal atomic value, wake the
  waiting thread. The increment must complete before waking threads; otherwise,
  a thread might wait _after_ being woken but _before_ seeing the updated
  value. Consider the following examples with the initial atomic value is 0.

  Incorrect implementation

  | thread 1            | thread 2                                      |
  | ------------------- | --------------------------------------------- |
  | wake                |                                               |
  |                     | value == 0 => true => wait (already woken up) |
  | increment value = 1 |                                               |

  Correct implementation

  | thread 1            | thread 2                                            |
  | ------------------- | --------------------------------------------------- |
  |                     | case 1: value == 0 => true => wait (woken up later) |
  | increment value = 1 |                                                     |
  |                     | case 2: value == 0 => false => not waiting          |
  | wake                |                                                     |

### Futex Rust wrapper

[Here is the brief wrapper around futex, using `libc`](./src/bin/chapter8-futex.rs)

Using futexes allows the program to specify the waiting _state_. For example, if we
set `a = 1` initially, we can skip waiting entirely.
This provides more control and flexibility for optimization.

### API for futex syscall

- FUTEX_WAIT:
  - internal 32-bit atomic integer
  - operation name: FUTEX_WAIT
  - expected value: value to check whether to sleep.
  - max time to wait
- FUTEX_WAKE:
  - internal 32-bit atomic integer
  - operation name: FUTEX_WAKE
  - number of threads to wake
- FUTEX_WAIT_BITSET:
  - internal 32-bit atomic integer
  - operation name: FUTEX_WAIT_BITSET
  - expected value: value to check whether to sleep.
  - max time to wait
  - ignored pointer
  - bitset: set the bitset so that only FUTEX_WAKE_BITSET with overlapping
    bitset can wake this thread up.
- FUTEX_WAKE_BITSET:
  - internal 32-bit atomic integer
  - operation name: FUTEX_WAKE_BITSET
  - number of threads to wake
  - ignored pointer
  - ignored pointer
  - bitset: only wakes waiting thread with overlapping bitset set by
    FUTEX_WAIT_BITSET.
- FUTEX_REQUEUE:
  - internal 32-bit atomic integer
  - operation name: FUTEX_REQUEUE
  - number of threads to wake
  - number of threads to requeue: after waking up some number of waiting
    threads, this requeues some of remaining threads to wait on the second
    atomic variable.
  - address of secondary atomic variable
- FUTEX_CMP_REQUEUE
  - internal 32-bit atomic integer
  - operation name: FUTEX_CMP_REQUEUE
  - number of threads to wake
  - number of threads to requeue
  - address of secondary atomic variable
  - expected value of the second atomic variable: similar to FUTEX_REQUEUE, but
    requires expected atomic value to decide whether it should requeue.
- FUTEX_WAKE_OP:

  - internal 32-bit atomic integer
  - operation name: FUTEX_WAKE_OP
  - number of threads to wake on primary atomic variable
  - number of threads to wake on secondary atomic variable
  - address of secondary atomic variable
  - comparison operations and arguments: can be ==, !=, <, <=, >, and >=, this
    wakes threads waiting on the first variable, and _conditionally_ on
    the second variable.

    ```rust
    let old = atomic2.fetch_update(Relaxed, Relaxed, some_operation);
    wake(atomic1, N);
    if some_condition(old) {
        wake(atomic2, M);
    }
    ```
