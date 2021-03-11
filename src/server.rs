use crate::error::Error;
use crate::{KvStore, KvsEngine, Result, BUCKET_EXT};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;

struct KvsServer {
    connection: TcpListener,
    engine: Box<dyn KvsEngine>,
}

impl KvsServer {
    pub fn new(
        addr: SocketAddr,
        path: impl Into<PathBuf>,
        engine: Option<String>,
    ) -> Result<KvsServer> {
        let path = path.into();
        let dir_is_empty = path.clone().read_dir()?.next().is_none();
        if let Some(engine) = engine {
            match (engine.as_str(), dir_is_empty) {
                ("kvs", true) => Ok(KvsServer {
                    connection: TcpListener::bind(addr)?,
                    engine: Box::new(KvStore::open(path)?),
                }),
                ("sled", true) => Ok(KvsServer {
                    connection: TcpListener::bind(addr)?,
                    engine: Box::new(sled::open(path)?),
                }),
                (e, false) => {
                    let kvs_exists = std::fs::read_dir(&path)?
                        .filter_map(|f| f.ok())
                        .any(|f| f.path().ends_with(BUCKET_EXT));

                    match (e, kvs_exists) {
                        ("kvs", true) => Ok(KvsServer {
                            connection: TcpListener::bind(addr)?,
                            engine: Box::new(KvStore::open(path)?),
                        }),
                        ("sled", true) => Err(Error::InvalidEngine),
                        ("kvs", false) => Err(Error::InvalidEngine),
                        ("sled", false) => Ok(KvsServer {
                            connection: TcpListener::bind(addr)?,
                            engine: Box::new(sled::open(path)?),
                        }),
                        (_, _) => unreachable!()
                    }
                }
                (_, true) => unreachable!(),
            }
        } else {
            Ok(KvsServer {
                connection: TcpListener::bind(addr)?,
                engine: Box::new(KvStore::open(path)?),
            })
        }
    }

    pub fn start(&mut self) -> Result<()> {
        for stream in self.connection.incoming() {
            stream?;
        }
        Ok(())
    }
}
