use crate::error::Error;
use crate::{KvStore, KvsEngine, Result, BUCKET_EXT};
use slog::{error, info, o};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;

pub struct KvsServer {
    connection: TcpListener,
    engine: Box<dyn KvsEngine>,
    logger: slog::Logger,
}

impl KvsServer {
    pub fn new(
        path: impl Into<PathBuf>,
        addr: SocketAddr,
        engine: Option<String>,
        logger: &slog::Logger,
    ) -> Result<KvsServer> {
        let path = path.into();
        let dir_is_empty = path.clone().read_dir()?.next().is_none();
        if let Some(engine) = engine {
            match (engine.as_str(), dir_is_empty) {
                ("kvs", true) => Ok(KvsServer {
                    connection: TcpListener::bind(addr)?,
                    engine: Box::new(KvStore::open(path)?),
                    logger: logger.new(o!("kvs" => "new kvs")),
                }),
                ("sled", true) => Ok(KvsServer {
                    connection: TcpListener::bind(addr)?,
                    engine: Box::new(sled::open(path)?),
                    logger: logger.new(o!("sled" => "new sled")),
                }),
                (e, false) => {
                    let kvs_exists = std::fs::read_dir(&path)?
                        .filter_map(|f| f.ok())
                        .any(|f| f.path().extension() == Some(BUCKET_EXT.as_ref()));

                    match (e, kvs_exists) {
                        ("kvs", true) => Ok(KvsServer {
                            connection: TcpListener::bind(addr)?,
                            engine: Box::new(KvStore::open(path)?),
                            logger: logger.new(o!("kvs" => "existing kvs")),
                        }),
                        ("sled", true) => {
                            error!(logger, "chosen sled, but kvs already exists in directory");
                            Err(Error::InvalidEngine)
                        }
                        ("kvs", false) => {
                            error!(logger, "chosen kvs, but sled already exists in directory");
                            Err(Error::InvalidEngine)
                        }
                        ("sled", false) => Ok(KvsServer {
                            connection: TcpListener::bind(addr)?,
                            engine: Box::new(sled::open(path)?),
                            logger: logger.new(o!("sled" => "existing sled")),
                        }),
                        (_, _) => unreachable!(),
                    }
                }
                (_, true) => unreachable!(),
            }
        } else {
            Ok(KvsServer {
                connection: TcpListener::bind(addr)?,
                engine: Box::new(KvStore::open(path)?),
                logger: logger.new(o!("kvs" => "default new kvs")),
            })
        }
    }

    pub fn start(&self) -> Result<()> {
        info!(self.logger, "starting server...");
        for stream in self.connection.incoming() {
            info!(self.logger, "accepting incoming connection...");
            stream?;
        }
        Ok(())
    }
}
