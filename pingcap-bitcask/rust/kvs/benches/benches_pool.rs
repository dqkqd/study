use std::collections::BTreeMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::process::Command;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, Criterion};
use criterion::{BenchmarkGroup, BenchmarkId};

use crossbeam_utils::sync::WaitGroup;
use rand::{distributions::Alphanumeric, prelude::*};
use rand_chacha::ChaCha20Rng;
use tempfile::TempDir;

use kvs::thread_pool::{RayonThreadPool, SharedQueueThreadPool, ThreadPool};
use kvs::{KvsEngine, KvsServer, Result, Store};

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

#[derive(Debug, Clone, Copy)]
enum StoreType {
    Kvs,
    Sled,
}

fn server_factory<P>(
    nthreads: u32,
    store_type: StoreType,
    path: &Path,
) -> Result<KvsServer<Store, P>>
where
    P: ThreadPool,
{
    let store = match store_type {
        StoreType::Kvs => Store::open_with_kvs(path)?,
        StoreType::Sled => Store::open_with_sled(path)?,
    };

    let pool = P::new(nthreads)?;
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000);
    let server = KvsServer::open(address, store, pool)?;
    Ok(server)
}

fn write_to_server<E, P1, P2, Pth>(
    path: Pth,
    server: KvsServer<E, P1>,
    client_pool: &P2,
    sample: Vec<(String, String)>,
) where
    Pth: AsRef<Path>,
    E: KvsEngine,
    P1: ThreadPool,
    P2: ThreadPool,
{
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

fn read_from_server<P, Pth>(
    path: Pth,
    client_pool: &P,
    sample: Vec<(String, String)>,
    server_address: SocketAddr,
) where
    Pth: AsRef<Path>,
    P: ThreadPool,
{
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

fn run_bench_write<P>(
    group: &mut BenchmarkGroup<WallTime>,
    bench_mark_name: &str,
    ncpu: u32,
    store_type: StoreType,
    sample: Vec<(String, String)>,
) where
    P: ThreadPool,
{
    let client_pool = SharedQueueThreadPool::new(NCLIENTS).unwrap();

    let bench_mark_name = match store_type {
        StoreType::Kvs => format!("{}-kvs-ncpus-{:02}", bench_mark_name, ncpu),
        StoreType::Sled => format!("{}-sled-ncpus-{:02}", bench_mark_name, ncpu),
    };

    group.bench_with_input(
        BenchmarkId::from_parameter(&bench_mark_name),
        &ncpu,
        |b, &ncpu| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let path = temp_dir.path().join(&bench_mark_name);
                fs::create_dir_all(&path).unwrap();
                let server = server_factory::<P>(ncpu, store_type, &path).unwrap();
                write_to_server(path, server, &client_pool, sample.clone());
            });
        },
    );
}

fn write_pool(c: &mut Criterion) {
    let sample = unique_sample(WRITE_SIZE);

    let cpus = num_cpus::get() as u32;
    let mut ncpu = 1;

    let mut group = c.benchmark_group("write_pool");
    while ncpu <= 2 * cpus {
        for store_type in [StoreType::Kvs, StoreType::Sled] {
            run_bench_write::<SharedQueueThreadPool>(
                &mut group,
                "queued",
                ncpu,
                store_type,
                sample.clone(),
            );
            run_bench_write::<RayonThreadPool>(
                &mut group,
                "rayon",
                ncpu,
                store_type,
                sample.clone(),
            );
        }
        ncpu *= 2;
    }

    group.finish()
}

fn run_bench_read<P>(
    group: &mut BenchmarkGroup<WallTime>,
    bench_mark_name: &str,
    ncpu: u32,
    store_type: StoreType,
    sample: Vec<(String, String)>,
) where
    P: ThreadPool,
{
    let client_pool = SharedQueueThreadPool::new(NCLIENTS).unwrap();

    let bench_mark_name = match store_type {
        StoreType::Kvs => format!("{}-kvs-ncpus-{:02}", bench_mark_name, ncpu),
        StoreType::Sled => format!("{}-sled-ncpus-{:02}", bench_mark_name, ncpu),
    };

    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join(format!("ncpus-{}", &ncpu));
    fs::create_dir_all(&path).unwrap();
    let server = server_factory::<P>(ncpu, store_type, &path).unwrap();
    write_to_server(&path, server, &client_pool, sample.clone());

    // reinitialize the same server
    let server = server_factory::<P>(ncpu, store_type, &path).unwrap();
    let server = server.serve();
    let address = server.address;

    group.bench_with_input(
        BenchmarkId::from_parameter(bench_mark_name),
        &ncpu,
        |b, _| {
            b.iter(|| read_from_server(&path, &client_pool, sample.clone(), address));
        },
    );

    server.shutdown();
}

fn read_pool(c: &mut Criterion) {
    let sample = unique_sample(WRITE_SIZE);

    let cpus = num_cpus::get() as u32;
    let mut ncpu = 1;

    let mut group = c.benchmark_group("read_pool");
    while ncpu <= 2 * cpus {
        for store_type in [StoreType::Kvs, StoreType::Sled] {
            run_bench_read::<SharedQueueThreadPool>(
                &mut group,
                "queued",
                ncpu,
                store_type,
                sample.clone(),
            );
            run_bench_read::<RayonThreadPool>(
                &mut group,
                "rayon",
                ncpu,
                store_type,
                sample.clone(),
            );
        }
        ncpu *= 2;
    }

    group.finish()
}

criterion_group!(benches_pool, write_pool, read_pool);
criterion_main!(benches_pool);
