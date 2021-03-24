use super::KvsEngine;
use crate::error;
use crate::Result;
use sled::{Db, Tree};

#[derive(Clone)]
pub struct SledKvsEngine(Db);

impl SledKvsEngine {
    pub fn new(db: Db) -> Self {
        SledKvsEngine(db)
    }
}

impl KvsEngine for SledKvsEngine {
    fn get(&mut self, key: String) -> Result<Option<String>> {
        let tree: &Tree = &self.0;
        Ok(tree
            .get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        let tree: &Tree = &self.0;
        tree.insert(key, value.into_bytes()).map(|_| ())?;
        tree.flush()?;
        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let tree: &Tree = &self.0;
        tree.remove(key)?.ok_or(error::Error::KeyNotFound)?;
        tree.flush()?;
        Ok(())
    }
}
