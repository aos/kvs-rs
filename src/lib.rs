//! This library houses a key-value store

mod client;
pub mod command;
mod engines;
mod error;
pub mod server;

pub use client::KvsClient;
pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use error::Result;
