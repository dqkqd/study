use std::env;

use clap::{Parser, Subcommand};
use kvs::Result;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Set {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
    #[command(name = "rm")]
    Remove {
        key: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let current_dir = env::current_dir().expect("get current working directory");
    let mut kvs = kvs::KvStore::open(current_dir)?;

    match cli.command {
        Commands::Set { key, value } => {
            kvs.set(key, value)?;
        }
        Commands::Get { key } => {
            let value = kvs.get(key)?;
            match value {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }
        }
        Commands::Remove { key } => {
            if let Err(err) = kvs.remove(key) {
                if matches!(err, kvs::KvError::KeyDoesNotExist(_)) {
                    println!("Key not found");
                }
                return Err(err);
            }
        }
    }
    Ok(())
}
