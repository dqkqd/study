use std::{env, io, net::SocketAddr};

use clap::{Parser, ValueEnum};
use kvs::{KvStore, KvsServer, Result};

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
    let store = KvStore::open(current_dir)?;
    let server = KvsServer::open(cli.addr, store)?;

    server.serve()?;

    Ok(())
}
