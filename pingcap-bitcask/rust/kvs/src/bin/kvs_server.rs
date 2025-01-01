use std::{env, io, net::SocketAddr, sync::mpsc};

use clap::{Parser, ValueEnum};
use kvs::{thread_pool, thread_pool::ThreadPool, KvsServer, Result, Store};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
    #[arg(long)]
    engine: Engine,
}

#[derive(Debug, Clone, ValueEnum)]
enum Engine {
    Kvs,
    Sled,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_writer(io::stderr).init();

    let cli = Cli::parse();

    let current_dir = env::current_dir().expect("get current working directory");
    let store = match cli.engine {
        Engine::Kvs => Store::open_with_kvs(&current_dir)?,
        Engine::Sled => Store::open_with_sled(&current_dir)?,
    };

    let pool = thread_pool::NaiveThreadPool::new(1)?;
    let server = KvsServer::open(cli.addr, store, pool)?;

    server.serve();

    // handling shutdown
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || tx.send(()).expect("could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");
    rx.recv().expect("Could not receive from channel.");

    Ok(())
}
