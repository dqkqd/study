use std::io::{BufReader, Read, Seek};

use crate::{
    command::{Command, CommandLocation},
    Result,
};

pub(crate) mod finder;
mod reader;
mod writer;

pub(crate) use reader::LogReader;
pub(crate) use writer::LogReaderWriter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LogId(pub u64);

pub(crate) trait LogRead<R>
where
    R: Read,
{
    /// Read a command at a specific location.
    /// We need `&mut self` since this require a file seek.
    fn read(&mut self, location: &CommandLocation) -> Result<Command>;

    /// Read all commands at once, consume itself
    fn into_commands(self) -> Result<IntoCommands<R>>;
}

pub(crate) trait LogWrite {
    /// Write a command and return written location.
    fn write(&mut self, command: &Command) -> Result<CommandLocation>;
}

#[derive(Debug)]
pub(crate) struct IntoCommands<R>
where
    R: Read,
{
    id: LogId,
    reader: BufReader<R>,
    offset: usize,
}

impl<R> IntoCommands<R>
where
    R: Read,
{
    pub(super) fn new(id: LogId, reader: BufReader<R>) -> IntoCommands<R> {
        IntoCommands {
            id,
            reader,
            offset: 0,
        }
    }
}

impl<R> Iterator for IntoCommands<R>
where
    R: Read,
{
    type Item = (Command, CommandLocation);

    fn next(&mut self) -> Option<Self::Item> {
        let command = Command::from_reader(&mut self.reader).ok()?;
        let bytes = command.to_bytes().ok()?;

        let location = CommandLocation {
            id: self.id,
            offset: self.offset,
            timestamp: command.timestamp(),
        };

        self.offset += bytes.len();

        Some((command, location))
    }
}

fn read_command<R: Read + Seek>(reader: &mut R, location: &CommandLocation) -> Result<Command> {
    reader.seek(std::io::SeekFrom::Start(location.offset as u64))?;
    Command::from_reader(reader)
}
