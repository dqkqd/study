use crate::{
    command::{Command, CommandLocation},
    Result,
};
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read, Seek},
    path::Path,
};

use super::{finder, read_command, IntoCommands, LogId, LogRead};

pub(crate) struct LogReader<R>
where
    R: Read + Seek,
{
    id: LogId,
    reader: BufReader<R>,
}

impl<R> LogRead<R> for LogReader<R>
where
    R: Read + Seek,
{
    fn read(&mut self, location: &CommandLocation) -> Result<Command> {
        read_command(&mut self.reader, location)
    }

    fn into_commands(self) -> Result<IntoCommands<R>> {
        Ok(IntoCommands::new(self.id, self.reader))
    }
}

impl LogReader<File> {
    pub(crate) fn open<P>(folder: P, id: LogId) -> Result<LogReader<File>>
    where
        P: AsRef<Path>,
    {
        let path = finder::reader_path(&folder, &id);
        let file = OpenOptions::new().read(true).open(path)?;

        Ok(LogReader {
            id,
            reader: BufReader::new(file),
        })
    }
}
