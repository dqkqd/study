use std::fmt;

use criterion::BenchmarkId;
use criterion::{criterion_group, criterion_main, Criterion};
use kvs::{KvsEngine, Store};
use rand::{distributions::Alphanumeric, prelude::*};
use rand_chacha::ChaCha20Rng;
use tempfile::TempDir;

const RANDOM_SEED: u64 = 42;

const WRITE_SIZE: usize = 100;
const READ_SIZE: usize = 1000;

const KEY_LENGTH: usize = 100000;
const VALUE_LENGTH: usize = 100000;

fn sample(size: usize) -> Vec<(String, String)> {
    let mut rng = ChaCha20Rng::seed_from_u64(RANDOM_SEED);
    let mut iter = rng.clone().sample_iter(&Alphanumeric);

    let mut get_string = |len: usize| -> String {
        let nbytes = rng.gen_range(1..len);
        let mut v = vec![];
        for c in iter.by_ref() {
            if v.len() >= nbytes {
                break;
            }
            v.push(c);
        }
        String::from_utf8(v).unwrap()
    };

    (0..size)
        .map(|_| (get_string(KEY_LENGTH), get_string(VALUE_LENGTH)))
        .collect()
}

#[derive(Clone, Copy)]
enum StoreType {
    Kvs,
    Sled,
}

impl StoreType {
    fn store(&self) -> (Store, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        let store = match self {
            StoreType::Kvs => Store::open_with_kvs(&temp_dir),
            StoreType::Sled => Store::open_with_sled(&temp_dir),
        }
        .unwrap();

        (store, temp_dir)
    }
}

impl fmt::Display for StoreType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreType::Kvs => f.write_str("kvs"),
            StoreType::Sled => f.write_str("sled"),
        }
    }
}

fn write(c: &mut Criterion) {
    let write_sample = sample(WRITE_SIZE);

    let mut group = c.benchmark_group("write");
    for store_type in [StoreType::Kvs, StoreType::Sled].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(store_type),
            store_type,
            |b, &store_type| {
                b.iter(|| {
                    // Store must be re-crated in different folder.
                    let (store, _temp_dir) = store_type.store();
                    for (k, v) in write_sample.clone() {
                        store.set(k, v).unwrap();
                    }
                });
            },
        );
    }

    group.finish()
}

fn read(c: &mut Criterion) {
    let write_sample = sample(WRITE_SIZE);
    let mut rng = ChaCha20Rng::seed_from_u64(RANDOM_SEED);
    let read_sample: Vec<(String, String)> = write_sample
        .choose_multiple(&mut rng, READ_SIZE)
        .cloned()
        .collect();

    let mut group = c.benchmark_group("read");

    for store_type in [StoreType::Kvs, StoreType::Sled].iter() {
        // The same store is used for reading.
        let (store, _temp_dir) = store_type.store();
        for (k, v) in write_sample.clone() {
            store.set(k, v).unwrap();
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(store_type),
            store_type,
            |b, _| {
                b.iter(|| {
                    for (key, value) in read_sample.clone() {
                        let value_from_store = store.get(key).unwrap();
                        assert_eq!(value_from_store, Some(value));
                    }
                });
            },
        );
    }

    group.finish()
}

criterion_group!(benches, write, read);
criterion_main!(benches);
