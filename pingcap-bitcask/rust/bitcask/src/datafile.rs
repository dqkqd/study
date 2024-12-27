use crate::{command::Command, directory::FileId, error::Result, KvError};
use std::{
    fs::{File, OpenOptions},
    io::{Seek, Write},
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Location {
    pub id: FileId,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TimedLocation {
    pub loc: Location,
    pub timestamp: Duration,
}

#[derive(Debug)]
pub(crate) struct ReadonlyDatafile {
    pub id: FileId,
    pub path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct ActiveDatafile {
    pub id: FileId,
    pub location: Location,
    file: File,
}

impl Location {
    pub fn timed_location(&self, timestamp: Duration) -> TimedLocation {
        TimedLocation {
            loc: *self,
            timestamp,
        }
    }
}

impl ReadonlyDatafile {
    pub fn new<P: AsRef<Path>>(id: FileId, path: P) -> ReadonlyDatafile {
        ReadonlyDatafile {
            id,
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn read(&self, location: &Location) -> Result<Command> {
        let mut file = OpenOptions::new().read(true).open(&self.path)?;
        read_command(&mut file, location)
    }

    pub fn all_commands(&self) -> Result<CommandIter> {
        let file = OpenOptions::new().read(true).open(&self.path)?;

        Ok(CommandIter {
            file,
            location: Location {
                id: self.id,
                offset: 0,
            },
        })
    }
}

impl ActiveDatafile {
    /// Open a new active file, raise error if the file has already existed.
    pub fn open<P: AsRef<Path>>(id: FileId, path: P) -> Result<ActiveDatafile> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)?;
        let offset = file.seek(std::io::SeekFrom::End(0))?;

        Ok(ActiveDatafile {
            id,
            file,
            location: Location {
                id,
                offset: offset as usize,
            },
        })
    }

    pub fn read(&mut self, location: &Location) -> Result<Command> {
        self.sync()?;
        read_command(&mut self.file, location)
    }

    pub fn write(&mut self, command: &Command) -> Result<()> {
        let bytes = command.to_bytes()?;
        let n = self.file.write(&bytes)?;
        if n != bytes.len() {
            // fallback
            self.file.set_len(self.location.offset as u64)?;
            return Err(KvError::CannotWriteLen(bytes.len()));
        }
        self.location.offset += n;

        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        self.file.sync_data()?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct CommandIter {
    file: File,
    location: Location,
}

impl Iterator for CommandIter {
    type Item = (Command, Location);

    fn next(&mut self) -> Option<Self::Item> {
        let command = Command::from_reader(&mut self.file).ok()?;
        let bytes = command.to_bytes().ok()?;

        let item = (command, self.location);
        self.location.offset += bytes.len();

        Some(item)
    }
}

fn read_command(file: &mut File, location: &Location) -> Result<Command> {
    file.seek(std::io::SeekFrom::Start(location.offset as u64))?;
    Command::from_reader(file)
}
