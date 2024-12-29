use std::{collections::BTreeMap, path::Path, sync::mpsc, thread};

use crate::{
    datafile::{ActiveDatafile, TimedLocation},
    directory::{readonly_datafile, FileId},
    kvs::KeyLocations,
    KvError, Result,
};

#[derive(Debug)]
enum Status {
    Free,
    Running,
}

#[derive(Debug)]
pub(crate) struct MergeInfo {
    pub new_readonly_datafile_id: FileId,
    pub readonly_datafile_ids: Vec<FileId>,
    pub key_locations: KeyLocations,
}

type MergeResult = Result<MergeInfo>;

#[derive(Debug)]
pub(crate) struct Merger {
    tx: mpsc::Sender<MergeResult>,
    rx: mpsc::Receiver<MergeResult>,
    status: Status,
}

impl Merger {
    pub fn new() -> Merger {
        let (tx, rx) = mpsc::channel();
        Merger {
            tx,
            rx,
            status: Status::Free,
        }
    }

    pub fn running(&self) -> bool {
        matches!(self.status, Status::Running)
    }

    pub fn start<P: AsRef<Path>>(
        &mut self,
        folder: P,
        new_datafile: ActiveDatafile,
        readonly_datafile_ids: Vec<FileId>,
    ) {
        if self.running() {
            return;
        }

        self.status = Status::Running;

        let tx = self.tx.clone();
        let path = folder.as_ref().to_path_buf();
        thread::spawn(move || {
            let res = merge_readonly_datafiles(path, new_datafile, readonly_datafile_ids);
            tx.send(res)
        });
    }

    pub fn result(&mut self) -> MergeResult {
        match self.rx.try_recv() {
            Ok(res) => {
                self.status = Status::Free;
                res
            }
            Err(_) => Err(KvError::MergeResultNotAvailable),
        }
    }
}

/// Perform merge readonly datafiles.
/// This merge should run in separated process,
/// so it should not write to `key_locations` directly.
pub(crate) fn merge_readonly_datafiles<P: AsRef<Path>>(
    folder: P,
    mut new_datafile: ActiveDatafile,
    readonly_datafile_ids: Vec<FileId>,
) -> MergeResult {
    // Read all keys' location from readonly files.
    let mut key_locations: BTreeMap<String, TimedLocation> = BTreeMap::new();
    for file_ids in &readonly_datafile_ids {
        let datafile = readonly_datafile(&folder, file_ids);
        for (command, location) in datafile.all_commands()? {
            let location = location.timed_location(command.timestamp());
            merge_location(&mut key_locations, command.key(), location);
        }
    }

    // Write all to new datafiles and update keys' location
    for timed_location in key_locations.values_mut() {
        let location = timed_location.loc;
        let datafile = readonly_datafile(&folder, &location.id);
        let command = datafile.read(&location)?;

        let new_location = new_datafile.location.timed_location(command.timestamp());
        new_datafile.write(&command)?;
        *timed_location = new_location;
    }

    Ok(MergeInfo {
        new_readonly_datafile_id: new_datafile.id,
        readonly_datafile_ids,
        key_locations,
    })
}

pub(crate) fn merge_location(
    key_locations: &mut BTreeMap<String, TimedLocation>,
    key: String,
    location: TimedLocation,
) {
    key_locations
        .entry(key)
        .and_modify(|old_location| {
            if old_location.timestamp < location.timestamp {
                *old_location = location;
            }
        })
        .or_insert(location);
}
