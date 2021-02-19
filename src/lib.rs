//! This library houses a key-value store

mod command;
mod error;

use command::{Command, Operation};
pub use error::Result;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const BUCKET_EXT: &str = "kvstore"; // {timestamp}.kvstore

/// KvStore holds an in-memory HashMap of <String, String>
pub struct KvStore {
    map: HashMap<String, String>,
    dir: PathBuf,
    active_file: File,
}

impl KvStore {
    /// Returns a newly instantiated KvStore
    /// ```rust
    /// use kvs::KvStore;
    /// let k = KvStore::new();
    /// ```
    pub fn new(dir: PathBuf, file: File) -> Self {
        KvStore {
            map: HashMap::new(),
            dir,
            active_file: file,
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
        self.map.insert(key.clone(), value.clone());
        // writes should be write-through:
        // update the in-memory map + the file on disk at the same time (not atomic)
        // TODO: consider buffering
        serde_json::to_writer(&self.active_file, &Command(Operation::Set(key, value)))?;
        writeln!(self.active_file)?;
        // This is written (on multiple operations):
        // {"Set":["key1","value1"]}\n{"Set":["key2","value2"]}
        //let mut buf = String::new();
        //self.active_file.seek(SeekFrom::Start(0))?;
        //self.active_file.read_to_string(&mut buf)?;

        //println!("We read: {}", buf);
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
        // 1. check to see if an existing file is available in dir
        // 2. if available, open it and slurp into memory (for now we will not buffer)
        //      - otherwise, create a new one
        let current_dir: PathBuf = dir.into();
        let maybe_latest = fs::read_dir(&current_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().map_or(false, |ft| ft.is_file())
                    && e.path().extension().map_or(false, |e| e == BUCKET_EXT)
            })
            .max_by_key(|e| {
                e.file_name().to_str().map_or(0, |e| {
                    e.split(".")
                        .next()
                        .map_or(0, |v| v.parse::<usize>().unwrap_or(0))
                })
            });

        let mut file = OpenOptions::new();
        if let Some(found_latest) = maybe_latest {
            let file: File = file
                .read(true)
                .append(true)
                .create(true)
                .open(found_latest.path())?;
            let mut kv = KvStore::new(current_dir, file);
            let map = HashMap::new();
            // Slurp the serialized data from file into hashmap

            kv.map = map;
            Ok(kv)
        } else {
            let ts = SystemTime::now().duration_since(UNIX_EPOCH)?;
            let file = file.read(true).append(true).create(true).open(format!(
                "{}/{}.{}",
                current_dir.as_path().display(),
                ts.as_secs(),
                BUCKET_EXT
            ))?;
            Ok(KvStore::new(current_dir, file))
        }
    }
}
