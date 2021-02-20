use kvs::{KvStore, Result};
use std::env;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs")]
enum KvsOpts {
    Set {
        #[structopt(index = 1, required = true)]
        key: String,
        #[structopt(index = 2, required = true)]
        value: String,
    },
    Get {
        #[structopt(required = true)]
        key: String,
    },
    Rm {
        #[structopt(required = true)]
        key: String,
    },
}

fn main() -> Result<()> {
    let mut kvs = KvStore::open(env::current_dir()?)?;
    match KvsOpts::from_args() {
        KvsOpts::Get { key } => {
            if let Some(found) = kvs.get(key)? {
                println!("{}", found);
            } else {
                println!("Key not found");
            }
            std::process::exit(0);
        }
        KvsOpts::Set { key, value } => {
            kvs.set(key, value)?;
            std::process::exit(0);
        }
        KvsOpts::Rm { key } => {
            if let Err(e) = kvs.remove(key) {
                println!("{}", e);
                std::process::exit(1);
            }
            std::process::exit(0);
        }
    }
}
