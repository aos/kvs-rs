use crate::command;
use crate::engines::KvsEngine;
use crate::Result;
use crate::ThreadPool;
use slog::{error, info};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    engine: E,
    thread_pool: P,
    logger: slog::Logger,
}

impl<E: KvsEngine, P: ThreadPool> KvsServer<E, P> {
    pub fn new(engine: E, thread_pool: P, logger: slog::Logger) -> KvsServer<E, P> {
        KvsServer {
            engine,
            thread_pool,
            logger,
        }
    }

    pub fn start<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let connection = TcpListener::bind(addr)?;
        info!(self.logger, "starting server...");

        for stream in connection.incoming() {
            let engine = self.engine.clone();
            let logger = self.logger.clone();
            self.thread_pool.spawn(move || match stream {
                Ok(stream) => {
                    serve(engine, stream, logger);
                }
                Err(stream_err) => error!(logger, "ERROR connecting to stream: {}", stream_err),
            });
        }
        Ok(())
    }
}

fn serve<E: KvsEngine>(engine: E, stream: TcpStream, logger: slog::Logger) {
    info!(logger, "accepting incoming connection...");

    match bincode::deserialize_from(&stream) {
        Ok(command::Request::Get(key)) => {
            info!(logger, "GET request"; "key" => key.as_str());
            match engine.get(key) {
                Ok(value) => {
                    if let Some(v) = value {
                        if let Err(e) = bincode::serialize_into(stream, &command::Response::OK(v)) {
                            error!(logger, "ERROR serialzing response: {}", e)
                        }
                    } else {
                        if let Err(e) =
                            bincode::serialize_into(stream, &command::Response::NotFound)
                        {
                            error!(logger, "ERROR serialzing response: {}", e)
                        }
                    };
                }
                Err(e) => {
                    error!(logger, "ERROR requesting key: {}", e);
                    if let Err(e) = bincode::serialize_into(
                        stream,
                        &command::Response::Error("Error GET key".to_string()),
                    ) {
                        error!(logger, "ERROR serialzing response: {}", e)
                    };
                }
            }
        }
        Ok(command::Request::Set(key, value)) => {
            info!(logger, "SET request"; "key" => key.as_str(), "value" => value.as_str());
            match engine.set(key, value) {
                Ok(_) => {
                    if let Err(e) =
                        bincode::serialize_into(stream, &command::Response::OK("".to_string()))
                    {
                        error!(logger, "ERROR serialzing response: {}", e)
                    }
                }
                Err(e) => {
                    error!(logger, "ERROR requesting value: {}", e);
                    if let Err(e) = bincode::serialize_into(
                        stream,
                        &command::Response::Error("Error SET key".to_string()),
                    ) {
                        error!(logger, "ERROR serialzing response: {}", e)
                    }
                }
            }
        }
        Ok(command::Request::Rm(key)) => {
            info!(logger, "RM request"; "key" => key.as_str());
            match engine.remove(key) {
                Ok(_) => {
                    if let Err(e) =
                        bincode::serialize_into(stream, &command::Response::OK("".to_string()))
                    {
                        error!(logger, "ERROR serialzing response: {}", e)
                    }
                }
                Err(e) => {
                    error!(logger, "ERROR removing key: {}", e);
                    if let Err(e) = bincode::serialize_into(stream, &command::Response::NotFound) {
                        error!(logger, "ERROR serialzing response: {}", e)
                    };
                }
            }
        }
        Err(e) => {
            error!(logger, "ERROR deserializing request: {}", e);
        }
    }
}
