use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

use crate::{
    command::CommandLocations,
    log::{finder, LogId, LogRead, LogReader, LogReaderWriter, LogWrite},
    KvError, Result,
};

#[derive(Debug)]
enum Status {
    Free,
    Running,
}

#[derive(Debug)]
pub(crate) struct MergeInfo {
    pub reader_ids: Vec<LogId>,
    pub locations: CommandLocations,
}

type MergeResult = Result<MergeInfo>;

#[derive(Debug)]
pub(crate) struct Merger {
    path: PathBuf,
    sender: mpsc::Sender<MergeResult>,
    receiver: mpsc::Receiver<MergeResult>,
    status: Status,
}

impl Merger {
    pub fn new<P: AsRef<Path>>(path: P) -> Merger {
        let path = path.as_ref().to_path_buf();
        let (sender, receiver) = mpsc::channel();

        Merger {
            path,
            sender,
            receiver,
            status: Status::Free,
        }
    }

    pub fn running(&self) -> bool {
        matches!(self.status, Status::Running)
    }
    pub fn start(&mut self, reader_ids: Vec<LogId>) {
        if self.running() {
            return;
        }
        self.status = Status::Running;

        let sender = self.sender.clone();
        let path = self.path.clone();
        thread::spawn(move || {
            let res = merge(&path, reader_ids);
            let _ = sender.send(res);
        });
    }

    pub fn result(&mut self) -> MergeResult {
        let res = self
            .receiver
            .try_recv()
            .map_err(|_| KvError::MergeResultNotAvailable)?;
        self.status = Status::Free;
        res
    }
}

fn merge<P: AsRef<Path>>(path: P, reader_ids: Vec<LogId>) -> MergeResult {
    let mut locations = CommandLocations::new();

    let mut writer = LogReaderWriter::open(&path, finder::next_log_id(&path))?;

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

    writer.transfer()?;

    Ok(MergeInfo {
        reader_ids,
        locations,
    })
}
