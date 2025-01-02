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
