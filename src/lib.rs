//! This library houses a key-value store

mod client;
pub mod command;
mod engine;
mod error;
pub mod server;
mod store;

pub use client::KvsClient;
pub use engine::KvsEngine;
pub use error::Result;
pub use store::{KvStore, BUCKET_EXT};
