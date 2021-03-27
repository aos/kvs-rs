use crate::Result;

pub trait KvsEngine {
    fn get(&self, key: String) -> Result<Option<String>>;

    fn set(&self, key: String, value: String) -> Result<()>;

    fn remove(&self, key: String) -> Result<()>;
}

mod kvs;
mod sled;

pub use self::kvs::{KvStore, BUCKET_EXT};
pub use self::sled::SledKvsEngine;
