use crate::command;
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
        let dir_is_empty = path.read_dir()?.next().is_none();
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

    pub fn start(&mut self) -> Result<()> {
        info!(self.logger, "starting server...");

        for stream in self.connection.incoming() {
            info!(self.logger, "accepting incoming connection...");

            match stream {
                Ok(stream) => match bincode::deserialize_from(&stream) {
                    Ok(command::Request::Get(key)) => {
                        info!(self.logger, "GET request"; "key" => key.as_str());
                        match self.engine.get(key) {
                            Ok(value) => {
                                if let Some(v) = value {
                                    match bincode::serialize_into(stream, &command::Response::OK(v))
                                    {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!(self.logger, "ERROR serialzing response: {}", e)
                                        }
                                    }
                                } else {
                                    match bincode::serialize_into(
                                        stream,
                                        &command::Response::NotFound,
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!(self.logger, "ERROR serialzing response: {}", e)
                                        }
                                    }
                                };
                            }
                            Err(e) => {
                                error!(self.logger, "ERROR requesting key: {}", e);
                                match bincode::serialize_into(
                                    stream,
                                    &command::Response::Error("Error GET key".to_string()),
                                ) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!(self.logger, "ERROR serialzing response: {}", e)
                                    }
                                };
                            }
                        }
                    }
                    Ok(command::Request::Set(key, value)) => {
                        info!(self.logger, "SET request"; "key" => key.as_str(), "value" => value.as_str());
                        match self.engine.set(key, value) {
                            Ok(_) => {
                                match bincode::serialize_into(
                                    stream,
                                    &command::Response::OK("".to_string()),
                                ) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!(self.logger, "ERROR serialzing response: {}", e)
                                    }
                                }
                            }
                            Err(e) => {
                                error!(self.logger, "ERROR requesting value: {}", e);
                                match bincode::serialize_into(
                                    stream,
                                    &command::Response::Error("Error SET key".to_string()),
                                ) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!(self.logger, "ERROR serialzing response: {}", e)
                                    }
                                };
                            }
                        }
                    }
                    Ok(command::Request::Rm(key)) => {
                        info!(self.logger, "RM request"; "key" => key.as_str());
                        match self.engine.remove(key) {
                            Ok(_) => {
                                match bincode::serialize_into(
                                    stream,
                                    &command::Response::OK("".to_string()),
                                ) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!(self.logger, "ERROR serialzing response: {}", e)
                                    }
                                }
                            }
                            Err(e) => {
                                error!(self.logger, "ERROR removing key: {}", e);
                                match bincode::serialize_into(stream, &command::Response::NotFound)
                                {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!(self.logger, "ERROR serialzing response: {}", e)
                                    }
                                };
                            }
                        }
                    }
                    Err(e) => error!(self.logger, "ERROR deserializing request: {}", e),
                },
                Err(stream_err) => {
                    error!(self.logger, "ERROR connecting to stream: {}", stream_err)
                }
            }
        }
        Ok(())
    }
}
