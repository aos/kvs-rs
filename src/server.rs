use crate::command;
use crate::engines::KvsEngine;
use crate::Result;
use crate::ThreadPool;
use slog::{error, info};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    engine: E,
    //logger: &'static slog::Logger,
    thread_pool: P,
}

impl<E: KvsEngine, P: ThreadPool> KvsServer<E, P> {
    pub fn new(
        engine: E,
        //logger: &'static slog::Logger,
        thread_pool: P,
    ) -> KvsServer<E, P> {
        KvsServer {
            engine,
            thread_pool, /*logger*/
        }
    }

    pub fn start<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let connection = TcpListener::bind(addr)?;
        //info!(self.logger, "starting server...");

        for stream in connection.incoming() {
            let engine = self.engine.clone();
            self.thread_pool.spawn(move || match stream {
                Ok(stream) => {
                    serve(engine, stream /*&self.logger*/);
                }
                Err(stream_err) => {
                    //error!(self.logger, "ERROR connecting to stream: {}", stream_err)
                }
            });
        }
        Ok(())
    }
}

fn serve<E: KvsEngine>(engine: E, stream: TcpStream /*logger: &slog::Logger*/) {
    //info!(logger, "accepting incoming connection...");

    match bincode::deserialize_from(&stream) {
        Ok(command::Request::Get(key)) => {
            //info!(logger, "GET request"; "key" => key.as_str());
            match engine.get(key) {
                Ok(value) => {
                    if let Some(v) = value {
                        match bincode::serialize_into(stream, &command::Response::OK(v)) {
                            Ok(_) => {}
                            Err(e) => {
                                //error!(logger, "ERROR serialzing response: {}", e)
                            }
                        }
                    } else {
                        match bincode::serialize_into(stream, &command::Response::NotFound) {
                            Ok(_) => {}
                            Err(e) => {
                                //error!(logger, "ERROR serialzing response: {}", e)
                            }
                        }
                    };
                }
                Err(e) => {
                    //error!(logger, "ERROR requesting key: {}", e);
                    match bincode::serialize_into(
                        stream,
                        &command::Response::Error("Error GET key".to_string()),
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            //error!(logger, "ERROR serialzing response: {}", e)
                        }
                    };
                }
            }
        }
        Ok(command::Request::Set(key, value)) => {
            //info!(logger, "SET request"; "key" => key.as_str(), "value" => value.as_str());
            match engine.set(key, value) {
                Ok(_) => {
                    match bincode::serialize_into(stream, &command::Response::OK("".to_string())) {
                        Ok(_) => {}
                        Err(e) => {
                            //error!(logger, "ERROR serialzing response: {}", e)
                        }
                    }
                }
                Err(e) => {
                    //error!(logger, "ERROR requesting value: {}", e);
                    match bincode::serialize_into(
                        stream,
                        &command::Response::Error("Error SET key".to_string()),
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            //error!(logger, "ERROR serialzing response: {}", e)
                        }
                    };
                }
            }
        }
        Ok(command::Request::Rm(key)) => {
            //info!(logger, "RM request"; "key" => key.as_str());
            match engine.remove(key) {
                Ok(_) => {
                    match bincode::serialize_into(stream, &command::Response::OK("".to_string())) {
                        Ok(_) => {}
                        Err(e) => {
                            //error!(logger, "ERROR serialzing response: {}", e)
                        }
                    }
                }
                Err(e) => {
                    //error!(logger, "ERROR removing key: {}", e);
                    match bincode::serialize_into(stream, &command::Response::NotFound) {
                        Ok(_) => {}
                        Err(e) => {
                            //error!(logger, "ERROR serialzing response: {}", e)
                        }
                    };
                }
            }
        }
        Err(e) => {} //error!(logger, "ERROR deserializing request: {}", e),
    }
}
