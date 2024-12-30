use std::{env, net::SocketAddr};

use clap::{Parser, ValueEnum};
use kvs::Result;

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
    let cli = Cli::parse();

    let current_dir = env::current_dir().expect("get current working directory");
    let _ = kvs::KvStore::open(current_dir)?;

    dbg!(cli);
    Ok(())
}
