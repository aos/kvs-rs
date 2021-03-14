use crate::command::{Request, Response};
use crate::Result;
use std::net::{SocketAddr, TcpStream};

pub struct KvsClient {}

impl KvsClient {
    pub fn send(request: Request, addr: SocketAddr) -> Result<Option<String>> {
        let stream = TcpStream::connect(addr)?;
        match bincode::serialize_into(&stream, &request) {
            Ok(_) => match bincode::deserialize_from(stream)? {
                Response::OK(v) => Ok(Some(v)),
                Response::NotFound => match request {
                    Request::Get(_) => Ok(None),
                    Request::Rm(_) => Err(crate::error::Error::KeyNotFound),
                    _ => unreachable!(),
                },
                Response::Error(v) => Err(crate::error::Error::Response(v)),
            },
            Err(e) => Err(e.into()),
        }
    }
}
