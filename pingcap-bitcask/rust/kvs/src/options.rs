/// Provide database configuration.
#[derive(Debug, Clone)]
pub struct KvOption {
    /// Maximum number of readonly log files.
    pub(crate) num_readers: usize,

    /// Maximum writer size in bytes.
    pub(crate) writer_size: usize,
}

impl Default for KvOption {
    fn default() -> KvOption {
        KvOption {
            num_readers: 10,
            writer_size: 1024 * 1024, // 1 Mb
        }
    }
}

impl KvOption {
    /// Construct a new option with default value.
    pub fn new() -> KvOption {
        KvOption::default()
    }

    /// Set the maximum number of readonly log files.
    pub fn num_log_readers(&mut self, num_readonly_datafiles: usize) -> &mut KvOption {
        self.num_readers = num_readonly_datafiles;
        self
    }

    /// Set the maximum size of the writer.
    pub fn writer_size(&mut self, active_datafile_size: usize) -> &mut KvOption {
        self.writer_size = active_datafile_size;
        self
    }
}
