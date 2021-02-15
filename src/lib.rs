//! This library houses a key-value store

mod error;
mod command;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::{self, File, OpenOptions};
use std::time::{Instant, Duration};
pub use error::Result;
use command::Command;

const EXT: &str = ".kvstore"; // {timestamp}.kvstore

/// KvStore holds an in-memory HashMap of <String, String>
pub struct KvStore {
    map: HashMap<String, String>,
    dir: PathBuf,
}

impl KvStore {
    /// Returns a newly instantiated KvStore
    /// ```rust
    /// use kvs::KvStore;
    /// let k = KvStore::new();
    /// ```
    pub fn new(dir: PathBuf) -> Self {
        KvStore {
            map: HashMap::new(),
            dir,
        }
    }

    /// Gets an item from the KvStore
    /// ```
    /// use kvs::KvStore;
    /// let mut k = KvStore::new();
    ///
    /// k.set("hi".to_owned(), "bye".to_owned());
    /// assert_eq!(k.get("hi".to_owned()), Some("bye".to_owned()));
    /// assert_eq!(k.get("no".to_owned()), None);
    /// ```
    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.map.get(&key).cloned())
    }

    /// Inserts an item or updates an existing item in the store
    /// ```
    /// use kvs::KvStore;
    /// let mut k = KvStore::new();
    ///
    /// k.set("hi".to_owned(), "bye".to_owned());
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.map.insert(key, value);
        // writes should be write-through:
        // update the in-memory map + the file on disk
        // worst-case scenario we can buffer it?
        Ok(())
    }

    /// Removes an item from the store
    /// ```
    /// use kvs::KvStore;
    /// let mut k = KvStore::new();
    ///
    /// k.set("hi".to_owned(), "bye".to_owned());
    /// k.remove("hi".to_owned());
    /// ```
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.map.remove(&key);
        Ok(())
    }

    /// Opens the KvStore at a given path. Return the KvStore
    pub fn open(dir: impl Into<PathBuf>) -> Result<KvStore> {
        // 1. check to see if an existing file is available here
        // 2. if available, open it and slurp into memory (for now we will not buffer)
        //      - otherwise, create a new one
        let current_dir = fs::read_dir(dir.into())?;

        Ok(())
    }
}
