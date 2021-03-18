use crate::command;
use crate::engines::{KvStore, KvsEngine, SledKvsEngine};
use crate::Result;
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
        engine: String,
        logger: &slog::Logger,
    ) -> Result<KvsServer> {
        let path = path.into();
        match engine.as_str() {
            "kvs" => Ok(KvsServer {
                connection: TcpListener::bind(addr)?,
                engine: Box::new(KvStore::open(path)?),
                logger: logger.new(o!("kvs" => "new kvs")),
            }),
            "sled" => Ok(KvsServer {
                connection: TcpListener::bind(addr)?,
                engine: Box::new(SledKvsEngine::new(sled::open(path)?)),
                logger: logger.new(o!("sled" => "new sled")),
            }),
            _ => unreachable!(),
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
