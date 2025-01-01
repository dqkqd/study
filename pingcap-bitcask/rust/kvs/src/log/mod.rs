use std::io::{BufReader, Read};

use crate::{
    command::{Command, CommandLocation},
    parser::ByteParser,
    Result,
};

pub(crate) mod finder;
mod reader;
mod writer;

pub(crate) use reader::LogReader;
pub(crate) use writer::LogWriter;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LogId(pub u64);

pub(crate) trait LogRead<R>
where
    R: Read,
{
    /// Read a command at a specific location, consume itself.
    fn read(self, location: &CommandLocation) -> Result<Command>;

    /// Read all commands at once, consume itself
    fn into_commands(self) -> Result<IntoCommands<R>>;
}

pub(crate) trait LogWrite {
    /// Write a command and return written location.
    fn write(&mut self, command: &Command) -> Result<CommandLocation>;
}

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
