use std::{
    fs::{self, File},
    io::{self, BufWriter, Seek, Write},
    path::Path,
};

use crate::{
    command::{Command, CommandLocation},
    parser::ByteParser,
    KvError, Result,
};

use super::{finder, LogId, LogWrite};

#[derive(Debug)]
pub(crate) struct LogWriter<W>
where
    W: Write + Seek,
{
    /// Writer log id.
    pub id: LogId,
    // Current writing offset.
    pub offset: usize,

    writer: BufWriter<W>,
}

impl<R> LogWrite for LogWriter<R>
where
    R: Write + Seek,
{
    fn write(&mut self, command: &Command) -> Result<CommandLocation> {
        let bytes = command.to_bytes()?;
        let n = self.writer.write(&bytes)?;
        if n != bytes.len() {
            // fallback to the previous location
            self.writer.seek(io::SeekFrom::Current(-(n as i64)))?;
            return Err(KvError::CannotWriteLen(bytes.len()));
        }

        let location = CommandLocation {
            id: self.id,
            offset: self.offset,
            timestamp: command.timestamp(),
        };

        self.offset += n;
        self.writer.flush()?;

        Ok(location)
    }
}

impl LogWriter<File> {
    pub(crate) fn open<P>(folder: P, id: LogId) -> Result<LogWriter<File>>
    where
        P: AsRef<Path>,
    {
        let path = finder::log_path(&folder, &id);
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)?;

        let offset = file.seek(std::io::SeekFrom::End(0))? as usize;

        let writer = BufWriter::new(file);

        Ok(LogWriter { id, writer, offset })
    }
}
