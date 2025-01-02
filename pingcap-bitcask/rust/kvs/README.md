# Bitcask

A rust implementation of bitcask from [this awesome course](https://github.com/pingcap/talent-plan/tree/master/courses/rust)

![demo](./images/demo.gif)

## How to run

First, start a server at specific address.

```bash
cargo run --bin kvs-server -- --engine kvs --addr 127.0.0.1:8080
```

Then we can query to that address.

```bash
cargo run --bin kvs-client set key1 value1 --addr 127.0.0.1:8080
cargo run --bin kvs-client get key1 --addr 127.0.0.1:8080
```

To run tests

```bash
cargo test
```

## Implementation details

Below are some notes when implementing this. (In case I forgot this after months).

- Database engine can be provide using `--engine` flag when initializing server.

  - The default database is [bitcask](https://riak.com/assets/bitcask-intro.pdf).
  - [sled](https://github.com/spacejam/sled) can also be
    configured using `--engine sled`.

- Database files:

  - Binary database file is encoded / decoded using [bson](https://github.com/mongodb/bson-rust)
    instead of following the structure provided in [bitcask paper](https://riak.com/assets/bitcask-intro.pdf).
  - Commands are saved directly in the binary file instead of key, value
    (to avoid thinking about `TOMBSTONE` string for deleted values).

- Concurrency: The database is thread-safe, can serve more than 1000 concurrent requests
  (See more in [benches_pool.rs](./benches/benches_pool.rs))

  - A thread-pool is used for serving connection,
    its implementation is taken from [threadpool](https://docs.rs/threadpool/latest/threadpool/).

  - [rayon threadpool](https://docs.rs/rayon/latest/rayon/struct.ThreadPool.html)
    is also added for benchmarking.
    (it cannot be specified using command line argument though)

  - The database internally using lock-free [hashmap](https://docs.rs/dashmap/latest/dashmap/struct.DashMap.html)
    and [hashset](https://docs.rs/dashmap/latest/dashmap/struct.DashSet.html)
    to serve read requests.
    It is inspired by [the course](https://github.com/pingcap/talent-plan/blob/master/courses/rust/projects/project-4/README.md#part-8-lock-free-readers).
