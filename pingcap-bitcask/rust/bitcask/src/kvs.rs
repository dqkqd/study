use crate::{
    command::Command,
    datafile::TimedLocation,
    directory::Directory,
    error::{KvError, Result},
    merger::{background_merge, merge_location},
    KvOption,
};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    thread,
    time::Duration,
};

pub(crate) type KeyLocations = BTreeMap<String, TimedLocation>;

/// An on-disk key value store.
#[derive(Debug)]
pub struct KvStore {
    directory: Arc<RwLock<Directory>>,
    key_locations: Arc<RwLock<KeyLocations>>,
    options: KvOption,
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
            directory: Arc::new(RwLock::new(directory)),
            key_locations: Arc::new(RwLock::new(key_locations)),
            options,
        };

        store.start_background_merge_thread();

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
        let command = Command::set(key.clone(), value);

        let location = {
            let mut directory = write_lock(&self.directory)?;
            if directory.active_datafile.location.offset >= self.options.active_datafile_size {
                directory.rollover_active_datafile();
            }
            let location = directory
                .active_datafile
                .location
                .timed_location(command.timestamp());
            directory.active_datafile.write(&command)?;

            location
        };

        let mut key_locations = write_lock(&self.key_locations)?;
        merge_location(&mut key_locations, key, location);

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
        let location = {
            let key_locations = read_lock(&self.key_locations)?;
            match key_locations.get(&key) {
                Some(location) => location.loc,
                None => {
                    return Ok(None);
                }
            }
        };

        let mut directory = write_lock(&self.directory)?;
        let command = if location.id == directory.active_datafile.id {
            directory.active_datafile.read(&location)?
        } else {
            let datafile = directory
                .readonly_datafiles
                .get(&location.id)
                .expect("readonly datafile must exist");
            datafile.read(&location)?
        };

        Ok(command.value())
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
        let mut key_locations = write_lock(&self.key_locations)?;
        if key_locations.remove(&key).is_none() {
            return Err(KvError::KeyDoesNotExist(key));
        }

        let mut directory = write_lock(&self.directory)?;
        directory.active_datafile.write(&Command::remove(key))?;

        Ok(())
    }

    fn start_background_merge_thread(&self) {
        let directory = Arc::clone(&self.directory);
        let key_locations = Arc::clone(&self.key_locations);
        let options = self.options.clone();
        thread::spawn(move || loop {
            let _ = background_merge(&directory, &key_locations, options.clone());
            thread::sleep(Duration::from_millis(500));
        });
    }
}

pub(crate) fn read_lock<T>(value: &Arc<RwLock<T>>) -> Result<RwLockReadGuard<'_, T>> {
    value
        .read()
        .map_err(|_| KvError::Lock("cannot lock for read".to_owned()))
}
pub(crate) fn write_lock<T>(value: &Arc<RwLock<T>>) -> Result<RwLockWriteGuard<'_, T>> {
    value
        .write()
        .map_err(|_| KvError::Lock("cannot lock for write".to_owned()))
}
