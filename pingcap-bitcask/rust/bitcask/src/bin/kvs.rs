use clap::{Parser, Subcommand};

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

fn main() {
    let cli = Cli::parse();
    let _ = kvs::KvStore::new();

    match cli.command {
        Commands::Set { key: _, value: _ } => {
            panic!("unimplemented")
        }
        Commands::Get { key: _ } => {
            panic!("unimplemented")
        }
        Commands::Remove { key: _ } => {
            panic!("unimplemented")
        }
    }
}
