use crate::error;
use crate::Result;
use sled::{Db, Tree};

pub trait KvsEngine {
    fn get(&mut self, key: String) -> Result<Option<String>>;

    fn set(&mut self, key: String, value: String) -> Result<()>;

    fn remove(&mut self, key: String) -> Result<()>;
}

impl KvsEngine for Db {
    fn get(&mut self, key: String) -> Result<Option<String>> {
        <Tree>::get(self, key)
            .map(|opt| {
                if let Some(some_opt) = opt {
                    match std::str::from_utf8(&*some_opt) {
                        Ok(v) => Some(v),
                        Err(_) => None,
                    }
                    .map(|r| r.to_owned())
                } else {
                    None
                }
            })
            .map_err(error::Error::Sled)
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        <Tree>::insert(self, key, value.as_str())
            .map(|_| ())
            .map_err(error::Error::Sled)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        <Tree>::remove(self, key)
            .map(|_| ())
            .map_err(error::Error::Sled)
    }
}
