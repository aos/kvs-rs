//! This library houses a key-value store

mod client;
pub mod command;
pub mod engines;
mod error;
pub mod server;
pub mod thread_pool;

pub use client::KvsClient;
pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use error::Result;
pub use thread_pool::{NaiveThreadPool, ThreadPool};
