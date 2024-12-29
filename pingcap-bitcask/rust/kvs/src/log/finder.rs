use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::Result;

use super::LogId;

const LOG_READER_PREFIX: &str = "LOG_READER";
const LOG_WRITER_PREFIX: &str = "LOG_WRITER";
const LOG_EXT: &str = "wal";

/// Path for log reading.
pub(crate) fn reader_path<P: AsRef<Path>>(folder: P, id: &LogId) -> PathBuf {
    folder
        .as_ref()
        .join(format!("{}_{:0>10}.{}", LOG_READER_PREFIX, id.0, LOG_EXT))
}

/// Path for log writing.
pub(crate) fn writer_path<P: AsRef<Path>>(folder: P, id: &LogId) -> PathBuf {
    folder
        .as_ref()
        .join(format!("{}_{:0>10}.{}", LOG_WRITER_PREFIX, id.0, LOG_EXT))
}

/// Iterate over all existing log ids in the folder, attempt to create a new log reader file,
/// if it succeeds, then the log file should be usable.
pub(crate) fn next_log_id<P: AsRef<Path>>(folder: P) -> LogId {
    let mut id = LogId(0);
    loop {
        let path = reader_path(&folder, &id);
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

/// Get immutable log ids.
/// These are read log ids that is not writing at the moment.
pub(crate) fn immutable_log_ids<P: AsRef<Path>>(folder: P) -> Result<Vec<LogId>> {
    let pattern = format!("{}/*.{}", folder.as_ref().display(), LOG_EXT);
    let file_stems: Vec<String> = glob::glob(&pattern)?
        .filter_map(|p| p.ok().and_then(path_to_file_stem))
        .collect();

    let writer_log_ids: BTreeSet<LogId> = file_stems
        .iter()
        .filter(|stem| stem.starts_with(LOG_WRITER_PREFIX))
        .filter_map(|stem| file_stem_to_log_id(stem))
        .collect();

    let immutable_log_ids: Vec<LogId> = file_stems
        .iter()
        .filter(|stem| stem.starts_with(LOG_READER_PREFIX))
        .filter_map(|stem| file_stem_to_log_id(stem))
        .filter(|log_id| !writer_log_ids.contains(log_id))
        .collect();

    Ok(immutable_log_ids)
}

/// Get writer ids.
pub(crate) fn writer_log_ids<P: AsRef<Path>>(folder: P) -> Result<Vec<LogId>> {
    let pattern = format!(
        "{}/{}_*.{}",
        folder.as_ref().display(),
        LOG_WRITER_PREFIX,
        LOG_EXT
    );

    let writer_log_ids: Vec<LogId> = glob::glob(&pattern)?
        .filter_map(|p| p.ok().and_then(path_to_file_stem))
        .filter_map(|stem| file_stem_to_log_id(&stem))
        .collect();

    Ok(writer_log_ids)
}

fn path_to_file_stem<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.to_string())
}

fn file_stem_to_log_id(stem: &str) -> Option<LogId> {
    stem.split("_")
        .last()
        .and_then(|name| name.parse::<u64>().ok())
        .map(LogId)
}
