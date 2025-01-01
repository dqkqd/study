use clap::crate_version;
use tracing::info;

use crate::{
    command::{Command, CommandLocations},
    log::{finder, LogId, LogRead, LogReader, LogWrite, LogWriter},
    merger::Merger,
    KvError, KvOption, Result,
};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use super::engine::KvsEngine;

const DATA_FOLDER: &str = "kvstore";

/// An on-disk key value store.
#[derive(Debug, Clone)]
pub(crate) struct KvStore {
    /// Path to the store.
    path: PathBuf,

    /// Append writer recoding the incoming commands.
    writer: SharedRw<LogWriter<File>>,
    /// Immutable readers.
    readers: SharedRw<BTreeSet<LogId>>,

    /// In memory map pointing to located commands on disk.
    locations: SharedRw<CommandLocations>,

    /// Database options.
    options: KvOption,

    /// Merger controls the merging process.
    merger: SharedRw<Merger>,
}

impl KvStore {
    /// Path to kvstore database.
    pub fn dbpath<P: AsRef<Path>>(path: P) -> PathBuf {
        path.as_ref().join(DATA_FOLDER)
    }

    /// Open database with default options.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<KvStore> {
        let dbpath = KvStore::dbpath(&path);
        KvStore::open_with_options(&dbpath, KvOption::default())
    }

    /// Open database with provided options.
    pub fn open_with_options<P: AsRef<Path>>(path: P, options: KvOption) -> Result<KvStore> {
        info!(dbpath = %path.as_ref().display(), "open database:");
        let _ = fs::create_dir_all(&path);

        let mut locations = CommandLocations::new();

        // Read all commands from previous log files.
        let readers: BTreeSet<LogId> = finder::all_log_ids(&path)?.into_iter().collect();
        for id in &readers {
            let reader = LogReader::open(&path, *id)?;
            for (command, location) in reader.into_commands()? {
                locations.merge(command.key(), location);
            }
        }

        // Create new writer.
        let writer = LogWriter::open(&path, finder::next_log_id(&path))?;

        let merger = Merger::new(&path);

        let store = KvStore {
            path: path.as_ref().to_path_buf(),
            writer: SharedRw::new(writer),
            readers: SharedRw::new(readers),
            locations: SharedRw::new(locations),
            merger: SharedRw::new(merger),
            options,
        };

        info!(version = crate_version!(), database_path = %store.path.display(), "opened kvs database:");

        Ok(store)
    }

    /// Merging process.
    fn merge(&self) -> Result<()> {
        self.gather_merged_result()?;

        let mut merger = self.merger.wlock()?;
        let readers = self.readers.rlock()?;
        let should_merge = !merger.running() && readers.len() >= self.options.num_readers;

        if should_merge {
            let readers: Vec<LogId> = {
                let writer = self.writer.rlock()?;
                let readers = readers
                    .iter()
                    .filter(|reader_id| reader_id != &&writer.id)
                    .cloned()
                    .collect();
                readers
            };

            merger.merge(readers);
        }
        Ok(())
    }

    /// Gather merged result and modify existing key locations, directory.
    fn gather_merged_result(&self) -> Result<()> {
        let mut merger = self.merger.wlock()?;

        if let Some(Ok(merge_info)) = merger.result() {
            // transfer new key
            {
                let mut locations = self.locations.wlock()?;
                for (key, location) in merge_info.locations.data {
                    locations.merge(key, location)
                }
            }

            // remove old file ids
            let mut readers = self.readers.wlock()?;
            for id in &merge_info.reader_ids {
                let reader_path = finder::log_path(&self.path, id);
                fs::remove_file(reader_path)?;
                readers.remove(id);
            }
        }

        Ok(())
    }

    fn rollover(&self) -> Result<()> {
        let mut writer = self.writer.wlock()?;
        if writer.offset >= self.options.writer_size {
            let new_writer_id = finder::next_log_id(&self.path);
            *writer = LogWriter::open(&self.path, new_writer_id)?;
            let mut readers = self.readers.wlock()?;
            readers.insert(writer.id);
        }
        Ok(())
    }
}

impl KvsEngine for KvStore {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.rollover()?;
        self.merge()?;

        let mut writer = self.writer.wlock()?;

        let command = Command::set(key.clone(), value);
        let location = writer.write(&command)?;

        let mut locations = self.locations.wlock()?;
        locations.merge(key, location);

        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let locations = self.locations.rlock()?;
        match locations.data.get(&key) {
            Some(location) => {
                let command = LogReader::open(&self.path, location.id)?.read(location)?;
                Ok(command.value())
            }
            None => Ok(None),
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        let mut locations = self.locations.wlock()?;
        if locations.data.remove(&key).is_none() {
            return Err(KvError::KeyNotFound(key));
        }
        self.rollover()?;

        let mut writer = self.writer.wlock()?;
        writer.write(&Command::remove(key))?;

        Ok(())
    }
}

#[derive(Debug)]
struct SharedRw<T>
where
    T: Send + Sync,
{
    inner: Arc<RwLock<T>>,
}

impl<T> SharedRw<T>
where
    T: Send + Sync,
{
    fn new(value: T) -> SharedRw<T> {
        SharedRw {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    fn rlock(&self) -> Result<RwLockReadGuard<T>> {
        self.inner
            .read()
            .map_err(|e| KvError::SharedRead(e.to_string()))
    }

    fn wlock(&self) -> Result<RwLockWriteGuard<T>> {
        self.inner
            .write()
            .map_err(|e| KvError::SharedWrite(e.to_string()))
    }
}

impl<T> Clone for SharedRw<T>
where
    T: Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
