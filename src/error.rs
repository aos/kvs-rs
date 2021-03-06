use serde_json;
use std::io;
use std::net::AddrParseError;
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Time(#[from] SystemTimeError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("Key not found")]
    KeyNotFound,
    #[error(transparent)]
    AddrParse(#[from] AddrParseError),
    #[error("Invalid engine")]
    InvalidEngine,
    #[error("Unspecified")]
    Unspecified,
}

/// The Result type encapsulates standard result
pub type Result<T> = std::result::Result<T, Error>;
