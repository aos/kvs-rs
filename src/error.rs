use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error)
}

/// The Result type encapsulates standard result
pub type Result<T> = std::result::Result<T, Error>;
