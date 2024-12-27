use std::path::Path;

use crate::KvStore;
use crate::Result;

/// Provide database configuration.
#[derive(Debug, Clone)]
pub struct KvOption {
    /// Number of readonly files before starting to merge
    pub(crate) num_readonly_datafiles: usize,

    /// Datafile size in bytes
    pub(crate) active_datafile_size: usize,
}

impl Default for KvOption {
    fn default() -> KvOption {
        KvOption {
            num_readonly_datafiles: 10,
            active_datafile_size: 1024 * 1024, // 1 Mb
        }
    }
}

impl KvOption {
    /// Construct a new option with default value.
    pub fn new() -> KvOption {
        KvOption::default()
    }

    /// Set number of readonly datafiles allowed.
    pub fn num_readonly_datafiles(&mut self, num_readonly_datafiles: usize) -> &mut KvOption {
        self.num_readonly_datafiles = num_readonly_datafiles;
        self
    }

    /// Set number of readonly datafiles allowed.
    pub fn active_datafile_size(&mut self, active_datafile_size: usize) -> &mut KvOption {
        self.active_datafile_size = active_datafile_size;
        self
    }

    /// Open database with specific options.
    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<KvStore> {
        KvStore::open_with_options(path, self.clone())
    }
}
