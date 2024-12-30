use std::{io, net::SocketAddr};

use clap::{Parser, Subcommand};
use kvs::{KvsClient, KvsRequest, Result};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,

    #[arg(long, global = true, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
}

#[derive(Subcommand, Debug)]
enum CliCommands {
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
    tracing_subscriber::fmt().with_writer(io::stderr).init();

    let cli = Cli::parse();
    let mut client = KvsClient::open(cli.addr)?;

    match cli.command {
        CliCommands::Set { key, value } => {
            client.send(KvsRequest::Set { key, value })?;
            let resp = client.recv()?;
            if !matches!(resp, kvs::KvsResponse::Ok(_)) {
                return Err(kvs::KvError::Unknown);
            }
        }
        CliCommands::Get { key } => {
            client.send(KvsRequest::Get { key })?;
            match client.recv()? {
                kvs::KvsResponse::Ok(Some(v)) => println!("{v}"),
                kvs::KvsResponse::Ok(None) => println!("Key not found"),
                _ => return Err(kvs::KvError::Unknown),
            }
        }
        CliCommands::Remove { key } => {
            client.send(KvsRequest::Remove { key })?;
            match client.recv()? {
                kvs::KvsResponse::Ok(_) => {}
                kvs::KvsResponse::KeyNotFound(key) => {
                    eprintln!("Key not found");
                    return Err(kvs::KvError::KeyNotFound(key));
                }
                _ => return Err(kvs::KvError::Unknown),
            }
        }
    }

    Ok(())
}
