use kvs::Result;
use std::net::{SocketAddr, TcpStream};
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
            let mut stream = TcpStream::connect(addr)?;
            // Write request over the wire
            if let Some(found) = kvs.get(key)? {
                println!("{}", found);
            } else {
                println!("Key not found");
            }
            exit(0);
        }
        ClientOpts::Set { key, value, addr } => {
            let mut stream = TcpStream::connect(addr)?;
            kvs.set(key, value)?;
            exit(0);
        }
        ClientOpts::Rm { key, addr } => {
            let mut stream = TcpStream::connect(addr)?;
            if let Err(e) = kvs.remove(key) {
                println!("{}", e);
                exit(1);
            }
            exit(0);
        }
    }
}
