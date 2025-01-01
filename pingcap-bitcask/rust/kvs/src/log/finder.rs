use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::Result;

use super::LogId;

const LOG_PREFIX: &str = "KVLOG";
const LOG_EXT: &str = "wal";

/// Path for log reading.
pub(crate) fn log_path<P: AsRef<Path>>(folder: P, id: &LogId) -> PathBuf {
    folder
        .as_ref()
        .join(format!("{}_{:0>10}.{}", LOG_PREFIX, id.0, LOG_EXT))
}

/// Iterate over all existing log ids in the folder, attempt to create a new log reader file,
/// if it succeeds, then the log file should be usable.
pub(crate) fn next_log_id<P: AsRef<Path>>(folder: P) -> LogId {
    let mut id = LogId(0);
    loop {
        let path = log_path(&folder, &id);
        if fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .is_ok()
        {
            break;
        }
        id.0 += 1;
    }
    id
}

/// Get all logs ids.
pub(crate) fn all_log_ids<P: AsRef<Path>>(folder: P) -> Result<Vec<LogId>> {
    let pattern = format!("{}/{}_*.{}", folder.as_ref().display(), LOG_PREFIX, LOG_EXT);

    let path_to_file_stem = |path: PathBuf| -> Option<String> {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.to_string())
    };

    let file_stem_to_log_id = |stem: &str| -> Option<LogId> {
        stem.split("_")
            .last()
            .and_then(|name| name.parse::<u64>().ok())
            .map(LogId)
    };

    let log_ids: Vec<LogId> = glob::glob(&pattern)?
        .filter_map(|p| p.ok().and_then(path_to_file_stem))
        .filter_map(|stem| file_stem_to_log_id(&stem))
        .collect();

    Ok(log_ids)
}
