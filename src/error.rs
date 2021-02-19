use std::io;
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Time(#[from] SystemTimeError),
}

/// The Result type encapsulates standard result
pub type Result<T> = std::result::Result<T, Error>;