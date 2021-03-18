use kvs::Result;
use kvs::server::KvsServer;
use slog::{o, error, warn, Drain};
use slog_async;
use slog_term;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::process;
use structopt::StructOpt;

const DEFAULT_ENGINE: &str = "kvs";

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
    let mut opts = ServerOpts::from_args();
    let logger = log.new(o!("addr" => opts.addr.to_string(), "engine" => opts.engine.to_owned()));

    let res = current_engine(&logger).and_then(|e| {
        if opts.engine.is_none() {
            opts.engine = e.to_owned();
        }
        if e.is_some() && opts.engine != e {
            error!(&logger, "Invalid engine!");
            process::exit(1);
        }

        run(opts, &logger)
    });

    if let Err(e) = res {
        error!(logger, "{}", e);
        process::exit(1);
    }

    Ok(())
}

fn run(opts: ServerOpts, logger: &slog::Logger) -> Result<()> {
    let engine = opts.engine.unwrap_or(DEFAULT_ENGINE.to_owned());
    let mut server = KvsServer::new(env::current_dir()?, opts.addr, engine.clone(), &logger)?;

    fs::write(env::current_dir()?.join("engine"), format!("{}", engine))?;

    server.start()
}

fn current_engine(logger: &slog::Logger) -> Result<Option<String>> {
    let engine_file = env::current_dir()?.join("engine");
    if !engine_file.exists() {
        return Ok(None)
    }

    match fs::read_to_string(engine_file)?.as_str() {
        "kvs" => Ok(Some("kvs".to_owned())),
        "sled" => Ok(Some("sled".to_owned())),
        e => {
            warn!(logger, "Invalid engine: {}", e);
            Ok(None)
        }
    }
}
