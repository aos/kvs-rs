use kvs::{KvStore, Result};
use std::env;
use std::net::SocketAddr;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs-client")]
enum ClientOpts {
    Set {
        #[structopt(index = 1, required = true)]
        key: String,
        #[structopt(index = 2, required = true)]
        value: String,
        #[structopt(default_value = "127.0.0.0:4000", long, help = "IP:PORT")]
        addr: SocketAddr,
    },
    Get {
        #[structopt(required = true)]
        key: String,
        #[structopt(default_value = "127.0.0.0:4000", long, help = "IP:PORT")]
        addr: SocketAddr,
    },
    Rm {
        #[structopt(required = true)]
        key: String,
        #[structopt(default_value = "127.0.0.0:4000", long, help = "IP:PORT")]
        addr: SocketAddr,
    },
}

fn main() -> Result<()> {
    // connect to a running server first
    let mut kvs = KvStore::open(env::current_dir()?)?;
    match ClientOpts::from_args() {
        ClientOpts::Get { key, addr } => {
            if let Some(found) = kvs.get(key)? {
                println!("{}", found);
            } else {
                println!("Key not found");
            }
            exit(0);
        }
        ClientOpts::Set { key, value, addr } => {
            kvs.set(key, value)?;
            exit(0);
        }
        ClientOpts::Rm { key, addr } => {
            if let Err(e) = kvs.remove(key) {
                println!("{}", e);
                exit(1);
            }
            exit(0);
        }
    }
}
