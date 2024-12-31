use clap::crate_version;
use tracing::info;

use crate::{
    command::{Command, CommandLocations},
    log::{finder, LogId, LogRead, LogReader, LogReaderWriter, LogWrite},
    merger::Merger,
    KvError, KvOption, Result,
};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    path::{Path, PathBuf},
};

use super::engine::KvsEngine;

const DATA_FOLDER: &str = "kvstore";

/// An on-disk key value store.
#[derive(Debug)]
pub(crate) struct KvStore {
    /// Path to the store.
    path: PathBuf,

    /// Append writer recoding the incoming commands.
    writer: LogReaderWriter<File>,
    /// Immutable readers.
    readers: BTreeSet<LogId>,

    /// In memory map pointing to located commands on disk.
    locations: CommandLocations,

    /// Database options.
    options: KvOption,

    /// Merger controls the merging process.
    merger: Merger,
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
        let _ = fs::create_dir(&path);

        let mut locations = CommandLocations::new();

        // Transfer all previous remaining written log.
        for id in finder::writer_log_ids(&path)? {
            let writer = LogReaderWriter::open(&path, id)?;
            writer.transfer()?;
        }

        // Read all commands from read-only log files.
        let readers: BTreeSet<LogId> = finder::immutable_log_ids(&path)?.into_iter().collect();
        for id in &readers {
            let reader = LogReader::open(&path, *id)?;
            for (command, location) in reader.into_commands()? {
                locations.merge(command.key(), location);
            }
        }

        let writer = LogReaderWriter::open(&path, finder::next_log_id(&path))?;

        let merger = Merger::new(&path);

        let store = KvStore {
            path: path.as_ref().to_path_buf(),
            writer,
            readers,
            locations,
            options,
            merger,
        };

        info!(version = crate_version!(), database_path = %store.path.display(), "opened kvs database:");

        Ok(store)
    }

    /// Determine whether merge process should be performing.
    fn should_merge(&self) -> bool {
        !self.merger.running() && self.readers.len() >= self.options.num_readers
    }

    /// Merging process.
    fn merge(&mut self) -> Result<()> {
        self.gather_merged_result()?;
        if self.should_merge() {
            let readers: Vec<LogId> = self.readers.iter().cloned().collect();
            self.merger.merge(readers);
        }
        Ok(())
    }

    /// Gather merged result and modify existing key locations, directory.
    fn gather_merged_result(&mut self) -> Result<()> {
        if let Some(Ok(merge_info)) = self.merger.result() {
            // transfer new key
            for (key, location) in merge_info.locations.data {
                self.locations.merge(key, location)
            }

            // remove old file ids
            for id in &merge_info.reader_ids {
                let reader_path = finder::reader_path(&self.path, id);
                fs::remove_file(reader_path)?;
                self.readers.remove(id);
            }
        }

        Ok(())
    }

    fn should_rollover(&self) -> bool {
        self.writer.offset >= self.options.writer_size
    }

    fn rollover(&mut self) -> Result<()> {
        if self.should_rollover() {
            let writer_log_id = finder::next_log_id(&self.path);
            let old_writer_log_id = self.writer.id;

            let mut writer = LogReaderWriter::open(&self.path, writer_log_id)?;
            std::mem::swap(&mut writer, &mut self.writer);

            writer.transfer()?;
            self.readers.insert(old_writer_log_id);
        }
        Ok(())
    }
}

impl KvsEngine for KvStore {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.rollover()?;
        self.merge()?;

        let command = Command::set(key.clone(), value);
        let location = self.writer.write(&command)?;
        self.locations.merge(key, location);

        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.locations.data.get(&key) {
            Some(location) => {
                let command = if location.id == self.writer.id {
                    self.writer.read(location)?
                } else {
                    LogReader::open(&self.path, location.id)?.read(location)?
                };

                Ok(command.value())
            }
            None => Ok(None),
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if self.locations.data.remove(&key).is_none() {
            return Err(KvError::KeyNotFound(key));
        }
        self.rollover()?;
        self.writer.write(&Command::remove(key))?;

        Ok(())
    }
}
