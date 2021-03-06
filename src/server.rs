use crate::{KvsEngine, KvStore, Result};
use crate::error::Error;
use std::path::PathBuf;
use std::net::SocketAddr;

struct KvsServer {
    addr: SocketAddr,
    engine: Box<dyn KvsEngine>,
}

impl KvsServer {
    pub fn new(addr: SocketAddr,
        path: impl Into<PathBuf>,
        engine: Option<String>
    ) -> Result<KvsServer> {
        let dir_is_empty = path.into().read_dir()?.next().is_none();
        if let Some(engine) = engine {
            match (engine.as_str(), dir_is_empty) {
                ("kvs", true) => {
                    Ok(KvsServer {
                        addr,
                        engine: Box::new(KvStore::open(path)?)
                    })
                },
                ("sled", true) => {

                },
                (e, false) => {
                    Err(Error::InvalidEngine)
                }
            }
        } else {
            "kvs"
        }
    }
}
