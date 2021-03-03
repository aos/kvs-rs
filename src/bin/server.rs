use kvs::{KvStore, Result};
use std::env;
use std::net::SocketAddr;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs-server")]
struct ServerOpts {
    #[structopt(default_value = "127.0.0.0:4000", long, help = "IP:PORT")]
    addr: SocketAddr,
    #[structopt(long, help = "ENGINE-NAME")]
    engine: Option<String>,
}

fn main() -> Result<()> {
    //let mut kvs = KvStore::open(env::current_dir()?)?;
    let opts = ServerOpts::from_args();

    println!("{:?}", opts);
    Ok(())
}
