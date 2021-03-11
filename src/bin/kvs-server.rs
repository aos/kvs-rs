use slog::*;
use slog_async;
use slog_term;
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
    let decorator = slog_term::PlainDecorator::new(std::io::stderr());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!("version" => env!("CARGO_PKG_VERSION")));
    let opts = ServerOpts::from_args();

    let logger = log.new(o!("addr" => opts.addr.to_string(), "engine" => "kvs"));

    info!(logger, "starting");
    info!(logger, "listening");
    debug!(logger, "connected");

    Ok(())
}
