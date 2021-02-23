//! This library houses a key-value store

mod command;
mod error;

use command::Command;
pub use error::{Error, Result};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const BUCKET_EXT: &str = "kvstore"; // {timestamp}.kvstore
const MAX_BUCKET_SIZE: u64 = 5 * 1024 * 1024; // 5 MB

struct KeyDirEntry {
    file_id: PathBuf,
    offset: u64,
    tstamp: u64,
}

pub struct ActiveFile {
    fd: File,
    path: PathBuf,
}

/// KvStore holds an in-memory HashMap of <String, String>
pub struct KvStore {
    keydir: HashMap<String, KeyDirEntry>,
    dir: PathBuf,
    active_file: ActiveFile,
}

impl KvStore {
    /// Returns a newly instantiated KvStore
    /// ```rust
    /// use kvs::KvStore;
    /// let k = KvStore::new();
    /// ```
    pub fn new(dir: PathBuf, file: ActiveFile) -> Self {
        KvStore {
            keydir: HashMap::new(),
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
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        // TODO: Get rid of these "if-let" chains somehow
        if let Some(entry) = self.keydir.get(&key) {
            let mut file = File::open(&entry.file_id)?;
            file.seek(SeekFrom::Start(entry.offset))?;
            let mut it = serde_json::Deserializer::from_reader(&file).into_iter::<Command>();
            if let Some(item) = it.next() {
                match item? {
                    Command::Set(_, v, _) => Ok(Some(v.to_owned())),
                    _ => Ok(None),
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Inserts an item or updates an existing item in the store
    /// ```
    /// use kvs::KvStore;
    /// let mut k = KvStore::new();
    ///
    /// k.set("hi".to_owned(), "bye".to_owned());
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let metadata = self.active_file.fd.metadata()?;
        let ts = Self::timestamp_sec()?;

        // Use new active file if we exceed bucket size
        if metadata.len() > MAX_BUCKET_SIZE {
            self.active_file.path = PathBuf::from(format!(
                "{}/{}.{}",
                self.dir.as_path().display(),
                ts,
                BUCKET_EXT
            ));
            self.active_file.fd = OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(&self.active_file.path)?;
        }

        let offset = self.active_file.fd.seek(SeekFrom::Current(0))?;
        self.keydir.insert(
            key.clone(),
            KeyDirEntry {
                file_id: self.active_file.path.clone(),
                offset,
                tstamp: ts,
            },
        );
        // writes should be write-through:
        // update the in-memory map + the file on disk at the same time (not atomic)
        serde_json::to_writer(&self.active_file.fd, &Command::Set(key, value, ts))?;
        writeln!(self.active_file.fd)?;
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
        self.keydir.remove(&key).ok_or(Error::KeyNotFound)?;
        serde_json::to_writer(&self.active_file.fd, &Command::Rm(key))?;
        writeln!(self.active_file.fd)?;

        Ok(())
    }

    /// Opens the KvStore at a given path. Return the KvStore
    pub fn open(dir: impl Into<PathBuf>) -> Result<KvStore> {
        let current_dir: PathBuf = dir.into();
        let mut files: Vec<_> = fs::read_dir(&current_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().map_or(false, |ft| ft.is_file())
                    && e.path().extension().map_or(false, |e| e == BUCKET_EXT)
            })
            .collect();
        files.sort_by_cached_key(|e| {
            e.file_name().to_str().map_or(0, |e| {
                e.split(".")
                    .next()
                    .map_or(0, |v| v.parse::<usize>().unwrap_or(0))
            })
        });
        let mut keydir = HashMap::new();
        let ts = Self::timestamp_sec()?;
        // Slurp the serialized data from each file into hashmap
        for entry in &files {
            let fd = File::open(entry.path())?;
            let de = serde_json::Deserializer::from_reader(&fd);
            let mut it = de.into_iter::<Command>();
            loop {
                let offset = it.byte_offset() as u64;
                if let Some(item) = it.next() {
                    match item? {
                        Command::Set(k, _, _) => {
                            keydir.insert(
                                k,
                                KeyDirEntry {
                                    file_id: entry.path(),
                                    offset,
                                    tstamp: ts,
                                },
                            );
                        }
                        Command::Rm(k) => {
                            keydir.remove(&k);
                        }
                        Command::Get(_) => {}
                    }
                } else {
                    break;
                }
            }
        }

        let mut fd = OpenOptions::new();
        if let Some(latest) = files.last() {
            let mut fd: File = fd
                .read(true)
                .append(true)
                .create(true)
                .open(latest.path())?;
            // We opened this file previously to add entries, seek to end so
            // that we can start to append new items correctly
            fd.seek(SeekFrom::End(0))?;
            let mut kv = KvStore::new(
                current_dir,
                ActiveFile {
                    fd,
                    path: latest.path(),
                },
            );
            kv.keydir = keydir;
            Ok(kv)
        } else {
            let file_path = PathBuf::from(format!(
                "{}/{}.{}",
                current_dir.as_path().display(),
                ts,
                BUCKET_EXT
            ));
            let fd = fd.read(true).append(true).create(true).open(&file_path)?;
            Ok(KvStore::new(
                current_dir,
                ActiveFile {
                    fd,
                    path: file_path,
                },
            ))
        }
    }

    fn timestamp_sec() -> Result<u64> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|t| t.as_secs())
            .map_err(std::convert::From::from)
    }
}
