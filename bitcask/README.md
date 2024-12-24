# Bitcask

Go implementation of [bitcask](https://riak.com/assets/bitcask-intro.pdf)

## How to run

```bash
go run main.go
```

## Result

```bash
>>> set one two
>>> get one
two
>>> delete one
>>> get one
>>>
```
