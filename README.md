# Monorepo for my study

## Database

- [Bitcask (go)](./bitcask/README.md): This is my first attempt with bitcask.
  It just a simple database with command line access.
  Though it is thread-safe, it holds a global lock for every access.

- [Bitcask (rust)](./pingcap-bitcask/rust/kvs/README.md):
  This is my second attempt using material from [ping-cap](https://github.com/pingcap/talent-plan/blob/master/courses/rust).
  The database is integrated with server / client cli.
  It is thread-safe, a bit more performant
  (can serve more than 1000 read / write requests).

- [Mini-lsm (rust)](https://github.com/dqkqd/mini-lsm/tree/study):
  A [beautiful course](https://skyzh.github.io/mini-lsm/) about LSM.
  It was soo fun and practical,
  though the difficulty is higher and requires lots of reading.

## Distributed systems

- [MIT Distributied systems course 6.5840-golabs-2025](https://github.com/dqkqd/6.5840-golabs-2025/)
