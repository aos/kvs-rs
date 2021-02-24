//! This library houses a key-value store

mod command;
mod error;

use command::Command;
pub use error::{Error, Result};
use std::collections::HashMap;
use std::fs::{self, DirEntry, File, OpenOptions};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::PathBuf;

const BUCKET_EXT: &str = "kvstore"; // {current_generation}.kvstore
const COMPACTION_THRESHOLD: u64 = 1 * 1024 * 1024; // 1 MB

/// KvStore holds an in-memory HashMap of <String, String>
pub struct KvStore {
    keydir: HashMap<String, KeyDirEntry>,
    dir: PathBuf,
    active_file: ActiveFile,
    current_gen: u64,
    uncompacted: u64,
}

impl KvStore {
    /// Opens the KvStore at a given path. Return the KvStore
    pub fn open(dir: impl Into<PathBuf>) -> Result<KvStore> {
        let current_dir: PathBuf = dir.into();
        fs::create_dir_all(&current_dir)?;

        let files = Self::get_sorted_files(current_dir.clone())?;
        let mut keydir = HashMap::new();
        // Slurp the serialized data from each file into hashmap
        for entry in &files {
            let fd = File::open(entry.path())?;
            let mut it = serde_json::Deserializer::from_reader(&fd).into_iter::<Command>();
            let mut offset = it.byte_offset() as u64;
            while let Some(item) = it.next() {
                match item? {
                    Command::Set(k, _) => {
                        keydir.insert(
                            k,
                            KeyDirEntry {
                                file_id: entry.path(),
                                offset,
                            },
                        );
                    }
                    Command::Rm(k) => {
                        keydir.remove(&k);
                    }
                }
                offset = it.byte_offset() as u64;
            }
        }

        if let Some(latest) = files.last() {
            let gen = latest.file_name().to_str().map_or(0, |e| {
                e.split(".")
                    .next()
                    .map_or(0, |v| v.parse::<u64>().unwrap_or(0))
            });
            let file_path = PathBuf::from(format!(
                "{}/{}.{}",
                current_dir.as_path().display(),
                gen,
                BUCKET_EXT
            ));
            let mut fd = OpenOptions::new()
                .read(true)
                .append(true)
                .open(&file_path)?;
            fd.seek(SeekFrom::End(0))?;
            let mut kv = KvStore {
                keydir: HashMap::new(),
                dir: current_dir,
                active_file: ActiveFile {
                    fd,
                    path: file_path,
                },
                current_gen: gen,
                uncompacted: 0,
            };
            kv.keydir = keydir;

            Ok(kv)
        } else {
            let active_file = ActiveFile::new(current_dir.clone(), 0)?;
            Ok(KvStore {
                keydir: HashMap::new(),
                dir: current_dir,
                active_file,
                current_gen: 0,
                uncompacted: 0,
            })
        }
    }

    /// Gets an item from the KvStore
    pub fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(entry) = self.keydir.get(&key) {
            let mut file = File::open(&entry.file_id)?;
            file.seek(SeekFrom::Start(entry.offset))?;
            let mut it = serde_json::Deserializer::from_reader(&file).into_iter::<Command>();
            if let Some(item) = it.next() {
                match item? {
                    Command::Set(_, v) => Ok(Some(v.to_owned())),
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
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // writes should be write-through:
        // update the in-memory map + the file on disk at the same time (not atomic)
        let offset = self.active_file.fd.seek(SeekFrom::Current(0))?;
        self.keydir.insert(
            key.clone(),
            KeyDirEntry {
                file_id: self.active_file.path.clone(),
                offset,
            },
        );
        serde_json::to_writer(&self.active_file.fd, &Command::Set(key, value))?;
        writeln!(self.active_file.fd)?;
        self.uncompacted += offset;

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    /// Removes an item from the store
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.keydir.remove(&key).ok_or(Error::KeyNotFound)?;
        serde_json::to_writer(&self.active_file.fd, &Command::Rm(key))?;
        writeln!(self.active_file.fd)?;

        Ok(())
    }

    fn compact(&mut self) -> Result<()> {
        // Naive solution:
        // 1. Mark all current files for deletion
        // 2. Iterate through keydir and write all values to new file(s) with new offsets
        // 3. If step 2 succeeds, delete all marked files
        // 4. Create new active file
        let files_to_delete = Self::get_sorted_files(self.dir.clone())?;
        let new_gen = self.current_gen + 2;
        let mut new_keydir = HashMap::new();
        let mut new_active_file = ActiveFile::new(self.dir.clone(), new_gen)?;
        for (key, _) in self.keydir.iter() {
            if let Some(value) = self.get(key.to_owned())? {
                let offset = new_active_file.fd.seek(SeekFrom::Current(0))?;
                new_keydir.insert(
                    key.clone(),
                    KeyDirEntry {
                        file_id: new_active_file.path.clone(),
                        offset,
                    },
                );
                serde_json::to_writer(&new_active_file.fd, &Command::Set(key.clone(), value))?;
                writeln!(new_active_file.fd)?;
            }
        }

        self.keydir = new_keydir;
        self.active_file = new_active_file;
        self.current_gen = new_gen;

        for f in files_to_delete {
            fs::remove_file(f.path())?;
        }
        self.uncompacted = 0;

        Ok(())
    }

    fn get_sorted_files(current_dir: PathBuf) -> Result<Vec<DirEntry>> {
        let mut files: Vec<_> = fs::read_dir(&current_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().map_or(false, |ft| ft.is_file())
                    && e.path().extension() == Some(BUCKET_EXT.as_ref())
            })
            .collect();
        files.sort_by_cached_key(|e| {
            e.file_name().to_str().map_or(0, |e| {
                e.split(".")
                    .next()
                    .map_or(0, |v| v.parse::<u64>().unwrap_or(0))
            })
        });
        Ok(files)
    }
}

struct KeyDirEntry {
    file_id: PathBuf,
    offset: u64,
}

pub struct ActiveFile {
    fd: File,
    path: PathBuf,
}

impl ActiveFile {
    fn new(dir: impl Into<PathBuf>, gen: u64) -> Result<Self> {
        let dir = dir.into();
        let path = PathBuf::from(format!(
            "{}/{}.{}",
            dir.as_path().display(),
            gen,
            BUCKET_EXT
        ));
        let mut fd = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&path)?;
        fd.seek(SeekFrom::End(0))?;

        Ok(ActiveFile { fd, path })
    }
}
