use kvs::Result;
use kvs::server::KvsServer;
use slog::{o, warn, Drain};
use slog_async;
use slog_term;
use std::env;
use std::fs;
use std::net::SocketAddr;
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
    let decorator = slog_term::PlainDecorator::new(std::io::stderr());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!("version" => env!("CARGO_PKG_VERSION")));
    let opts = ServerOpts::from_args();

    let logger = log.new(o!("addr" => opts.addr.to_string(), "engine" => opts.engine.to_owned()));

    let curr_engine = current_engine()?.and_then(|e| {
        if true {}
    });

    let mut server = KvsServer::new(env::current_dir()?, opts.addr, opts.engine, &logger)?;

    server.start()?;

    Ok(())
}

fn current_engine() -> Result<Option<String>> {
    let engine_file = env::current_dir()?.join("engine)");
    if !engine_file.exists() {
        return Ok(None)
    }

    match fs::read_to_string(engine_file)?.as_str() {
        "Kvs" => Ok(Some("Kvs".to_owned())),
        "Sled" => Ok(Some("Sled".to_owned())),
        _ => Ok(None)
    }
}
