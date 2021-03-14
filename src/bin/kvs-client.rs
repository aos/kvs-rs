use kvs::{command::Request, KvsClient, Result};
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
    match ClientOpts::from_args() {
        ClientOpts::Get { key, addr } => {
            // Write request over the wire
            if let Some(found) = KvsClient::send(Request::Get(key), addr)? {
                println!("{}", found);
            } else {
                println!("Key not found");
            }
            exit(0);
        }
        ClientOpts::Set { key, value, addr } => {
            KvsClient::send(Request::Set(key, value), addr)?;
            exit(0);
        }
        ClientOpts::Rm { key, addr } => {
            if let Err(e) = KvsClient::send(Request::Rm(key), addr) {
                eprintln!("{}", e);
                exit(1);
            }
            exit(0);
        }
    }
}
