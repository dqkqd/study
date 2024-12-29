use crate::{
    command::Command,
    datafile::TimedLocation,
    directory::{Directory, FileId},
    error::{KvError, Result},
    merger::{merge_location, Merger},
    KvOption,
};
use std::{collections::BTreeMap, path::Path};

pub(crate) type KeyLocations = BTreeMap<String, TimedLocation>;

/// An on-disk key value store.
#[derive(Debug)]
pub struct KvStore {
    directory: Directory,
    key_locations: KeyLocations,
    options: KvOption,
    merger: Merger,
}

impl KvStore {
    /// Open database with default options.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<KvStore> {
        KvStore::open_with_options(path, KvOption::default())
    }

    pub(crate) fn open_with_options<P: AsRef<Path>>(path: P, options: KvOption) -> Result<KvStore> {
        let mut key_locations: KeyLocations = BTreeMap::new();
        let directory = Directory::open(&path)?;

        for readonly_datafile in directory.readonly_datafiles.values() {
            for (command, location) in readonly_datafile.all_commands()? {
                let location = location.timed_location(command.timestamp());
                merge_location(&mut key_locations, command.key(), location);
            }
        }

        let store = KvStore {
            directory,
            key_locations,
            options,
            merger: Merger::new(),
        };

        Ok(store)
    }

    /// Set a key with value to the store.
    ///
    /// # Examples
    /// ```rust
    /// # use kvs::KvStore;
    /// # use kvs::Result;
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let mut kvs = KvStore::open(&directory)?;
    ///
    /// kvs.set("key1".to_string(), "value1".to_string())?;
    /// assert_eq!(kvs.get("key1".to_string())?, Some("value1".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        if self.should_rollover_active_datafile() {
            self.directory.rollover_active_datafile();
        }

        self.merge()?;

        let command = Command::set(key.clone(), value);
        let location = self
            .directory
            .active_datafile
            .location
            .timed_location(command.timestamp());
        self.key_locations.insert(key, location);
        self.directory.active_datafile.write(&command)?;

        Ok(())
    }

    /// Get value of a key from the store.
    ///
    /// # Examples
    /// ```rust
    /// # use kvs::KvStore;
    /// # use kvs::Result;
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let mut kvs = KvStore::open(&directory)?;
    ///
    /// assert_eq!(kvs.get("key1".to_string())?, None);
    ///
    /// kvs.set("key1".to_string(), "value1".to_string())?;
    /// assert_eq!(kvs.get("key1".to_string())?, Some("value1".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.key_locations.get(&key) {
            Some(location) => {
                let location = location.loc;

                let command = if location.id == self.directory.active_datafile.id {
                    self.directory.active_datafile.read(&location)?
                } else {
                    let datafile = self
                        .directory
                        .readonly_datafiles
                        .get(&location.id)
                        .expect("readonly datafile must exist");
                    datafile.read(&location)?
                };

                Ok(command.value())
            }
            None => Ok(None),
        }
    }

    /// Remove a key from the store.
    ///
    /// # Examples
    /// ```rust
    /// # use kvs::KvStore;
    /// # use kvs::Result;
    /// # use tempfile::TempDir;
    /// # fn main() -> Result<()> {
    /// # let directory = TempDir::new().expect("unable to create temporary working directory");
    /// let mut kvs = KvStore::open(&directory)?;
    ///
    /// kvs.set("key1".to_string(), "value1".to_string())?;
    /// assert_eq!(kvs.get("key1".to_string())?, Some("value1".to_string()));
    ///
    /// kvs.remove("key1".to_string())?;
    /// assert_eq!(kvs.get("key1".to_string())?, None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.key_locations.remove(&key).is_none() {
            return Err(KvError::KeyDoesNotExist(key));
        }

        if self.should_rollover_active_datafile() {
            self.directory.rollover_active_datafile();
        }

        self.directory
            .active_datafile
            .write(&Command::remove(key))?;

        Ok(())
    }

    fn merge(&mut self) -> Result<()> {
        self.gather_merge_result()?;
        if self.should_merge_readonly_datafiles() {
            self.start_merge_process()?;
        }
        Ok(())
    }

    /// Gather merged result and modify existing key locations, directory.
    fn gather_merge_result(&mut self) -> Result<()> {
        if let Ok(merge_info) = self.merger.result() {
            // transfer new key
            for (key, location) in merge_info.key_locations {
                merge_location(&mut self.key_locations, key, location)
            }

            // remove old file ids
            for file_id in merge_info.readonly_datafile_ids {
                self.directory.remove_readonly_datafile(&file_id)?;
            }

            // set the new readonly file id pointing to `new_datafile`
            self.directory.readonly_datafiles.insert(
                merge_info.new_readonly_datafile_id,
                self.directory
                    .readonly_datafile(&merge_info.new_readonly_datafile_id),
            );
        }

        Ok(())
    }

    /// Start a merge process in the background.
    fn start_merge_process(&mut self) -> Result<()> {
        if !self.merger.running() {
            // Prepare the merged file. If this file is failed to created,
            // the merge process will be postpone until next time.
            let new_datafile = self.directory.next_active_datafile()?;
            let readonly_datafile_ids: Vec<FileId> =
                self.directory.readonly_datafiles.keys().cloned().collect();

            self.merger.start(
                self.directory.path.clone(),
                new_datafile,
                readonly_datafile_ids,
            );
        }

        Ok(())
    }

    fn should_rollover_active_datafile(&self) -> bool {
        self.directory.active_datafile.location.offset >= self.options.active_datafile_size
    }

    fn should_merge_readonly_datafiles(&self) -> bool {
        self.directory.readonly_datafiles.len() >= self.options.num_readonly_datafiles
            && !self.merger.running()
    }
}
