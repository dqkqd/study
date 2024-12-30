use std::{env, io, net::SocketAddr};

use clap::{crate_version, Parser, ValueEnum};
use kvs::Result;
use tracing::info;

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
    let _ = kvs::KvStore::open(current_dir)?;
    let version = crate_version!();

    info!(version = version, addr = %cli.addr, engine= ?cli.engine,  "opened database");

    Ok(())
}
