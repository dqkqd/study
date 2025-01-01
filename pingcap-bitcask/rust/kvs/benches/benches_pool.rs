use std::collections::BTreeMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::process::Command;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use criterion::BenchmarkId;
use criterion::{criterion_group, criterion_main, Criterion};

use crossbeam_utils::sync::WaitGroup;
use rand::{distributions::Alphanumeric, prelude::*};
use rand_chacha::ChaCha20Rng;
use tempfile::TempDir;

use kvs::thread_pool::{SharedQueueThreadPool, ThreadPool};
use kvs::{KvsServer, Result, Store};

const RANDOM_SEED: u64 = 42;

const NCLIENTS: u32 = 1000;

const WRITE_SIZE: usize = 100;
const KEY_VALUE_LENGTH: usize = 1000;

// Set up size unique keys of the same length with the same values
fn unique_sample(size: usize) -> Vec<(String, String)> {
    let rng = ChaCha20Rng::seed_from_u64(RANDOM_SEED);
    let mut iter = rng.sample_iter(&Alphanumeric);
    let mut get_string = |len: usize| -> String {
        let mut v = vec![];
        for c in iter.by_ref() {
            if v.len() >= len {
                break;
            }
            v.push(c);
        }
        String::from_utf8(v).unwrap()
    };

    let value = get_string(KEY_VALUE_LENGTH);
    let mut unique_strings = BTreeMap::new();
    while unique_strings.len() < size {
        let key = get_string(KEY_VALUE_LENGTH);
        unique_strings.insert(key, value.clone());
    }

    unique_strings.into_iter().collect()
}

fn server_factory<P: AsRef<Path>>(
    nthreads: u32,
    path: P,
) -> Result<KvsServer<Store, SharedQueueThreadPool>> {
    let pool = SharedQueueThreadPool::new(nthreads)?;
    let store = Store::open_with_kvs(&path)?;
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000);
    let server = KvsServer::open(address, store, pool)?;
    Ok(server)
}

fn write_to_server<P: AsRef<Path>>(
    path: P,
    client_pool: &SharedQueueThreadPool,
    ncpu: u32,
    sample: Vec<(String, String)>,
) {
    let server = server_factory(ncpu, &path).unwrap();
    let server = server.serve();
    let address = server.address;

    let wg = WaitGroup::new();
    for (key, value) in sample {
        let path = path.as_ref().to_path_buf().clone();
        let wg = wg.clone();
        client_pool.spawn(move || {
            Command::cargo_bin("kvs-client")
                .unwrap()
                .args(["set", &key, &value, "--addr", &address.to_string()])
                .current_dir(path)
                .assert()
                .success();
            drop(wg);
        });
    }
    wg.wait();
    server.shutdown();
}

fn read_from_server<P: AsRef<Path>>(
    path: P,
    client_pool: &SharedQueueThreadPool,
    sample: Vec<(String, String)>,
    server_address: SocketAddr,
) {
    let wg = WaitGroup::new();
    for (key, value) in sample.clone() {
        let path = path.as_ref().to_path_buf().clone();
        let wg = wg.clone();
        client_pool.spawn(move || {
            Command::cargo_bin("kvs-client")
                .unwrap()
                .args(["get", &key, "--addr", &server_address.to_string()])
                .current_dir(path)
                .assert()
                .success()
                .stdout(format!("{}\n", value));
            drop(wg);
        });
    }
    wg.wait();
}

fn write_pool(c: &mut Criterion) {
    let sample = unique_sample(WRITE_SIZE);

    let pool = SharedQueueThreadPool::new(NCLIENTS).unwrap();
    let cpus = num_cpus::get() as u32;
    let mut ncpu = 1;

    let mut group = c.benchmark_group("write_pool");

    while ncpu <= 2 * cpus {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("ncpus={:02}", &ncpu)),
            &ncpu,
            |b, &ncpu| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let path = temp_dir.path().join(format!("ncpus-{}", &ncpu));
                    fs::create_dir_all(&path).unwrap();
                    write_to_server(path, &pool, ncpu, sample.clone());
                });
            },
        );
        ncpu *= 2;
    }

    group.finish()
}

fn read_pool(c: &mut Criterion) {
    let sample = unique_sample(WRITE_SIZE);

    let pool = SharedQueueThreadPool::new(NCLIENTS).unwrap();
    let cpus = num_cpus::get() as u32;
    let mut ncpu = 1;

    let mut group = c.benchmark_group("read_pool");

    while ncpu <= 2 * cpus {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join(format!("ncpus-{}", &ncpu));
        fs::create_dir_all(&path).unwrap();

        write_to_server(&path, &pool, ncpu, sample.clone());

        // reinitialize the same server
        let server = server_factory(ncpu, &path).unwrap();
        let server = server.serve();
        let address = server.address;

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("ncpus={:02}", &ncpu)),
            &ncpu,
            |b, _| {
                b.iter(|| read_from_server(&path, &pool, sample.clone(), address));
            },
        );

        server.shutdown();

        ncpu *= 2;
    }

    group.finish()
}

criterion_group!(benches_pool, write_pool, read_pool);
criterion_main!(benches_pool);
