use std::{
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};

use crate::{
    command::CommandLocations,
    log::{finder, LogId, LogRead, LogReader, LogWrite, LogWriter},
    Result,
};

#[derive(Debug)]
pub(crate) struct MergeInfo {
    pub reader_ids: Vec<LogId>,
    pub locations: CommandLocations,
}

type MergeResult = Result<MergeInfo>;

#[derive(Debug)]
pub(crate) struct Merger {
    path: PathBuf,
    job: Option<JoinHandle<MergeResult>>,
}

impl Merger {
    pub fn new<P: AsRef<Path>>(path: P) -> Merger {
        let path = path.as_ref().to_path_buf();
        Merger { path, job: None }
    }

    pub fn running(&self) -> bool {
        self.job.as_ref().is_some_and(|j| !j.is_finished())
    }

    pub fn merge(&mut self, reader_ids: Vec<LogId>) {
        if !self.running() {
            let path = self.path.clone();
            self.job = Some(thread::spawn(move || merge(&path, reader_ids)));
        }
    }

    pub fn result(&mut self) -> Option<MergeResult> {
        let finished = self.job.as_ref().is_some_and(|j| j.is_finished());

        if finished {
            let job = std::mem::take(&mut self.job);
            job.and_then(|j| j.join().ok())
        } else {
            None
        }
    }
}

fn merge<P: AsRef<Path>>(path: P, reader_ids: Vec<LogId>) -> MergeResult {
    let mut locations = CommandLocations::new();

    let mut writer = LogWriter::open(&path, finder::next_log_id(&path))?;

    for id in &reader_ids {
        let reader = LogReader::open(&path, *id)?;
        for (command, location) in reader.into_commands()? {
            locations.merge(command.key(), location);
        }
    }

    for location in locations.data.values_mut() {
        let command = LogReader::open(&path, location.id)?.read(location)?;
        *location = writer.write(&command)?;
    }

    Ok(MergeInfo {
        reader_ids,
        locations,
    })
}
