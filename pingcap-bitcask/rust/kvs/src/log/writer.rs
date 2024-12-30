use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Seek, Write},
    path::{Path, PathBuf},
};

use crate::{
    command::{Command, CommandLocation},
    parser::ByteParser,
    KvError, Result,
};

use super::{finder, read_command, IntoCommands, LogId, LogRead, LogWrite};

#[derive(Debug)]
pub(crate) struct LogReaderWriter<Rw>
where
    Rw: Read + Write + Seek,
{
    /// Writer log id.
    pub id: LogId,
    // Current writing offset.
    pub offset: usize,

    folder: PathBuf,
    writer: BufWriter<Rw>,
    reader: BufReader<Rw>,
}

impl<Rw> LogRead<Rw> for LogReaderWriter<Rw>
where
    Rw: Read + Write + Seek,
{
    fn read(&mut self, location: &CommandLocation) -> Result<Command> {
        self.writer.flush()?;
        read_command(&mut self.reader, location)
    }

    fn into_commands(mut self) -> Result<IntoCommands<Rw>> {
        self.writer.flush()?;
        Ok(IntoCommands::new(self.id, self.reader))
    }
}

impl<Rw> LogWrite for LogReaderWriter<Rw>
where
    Rw: Read + Write + Seek,
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

impl LogReaderWriter<File> {
    pub(crate) fn open<P>(folder: P, id: LogId) -> Result<LogReaderWriter<File>>
    where
        P: AsRef<Path>,
    {
        let path = finder::writer_path(&folder, &id);
        let mut file = fs::OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&path)?;

        let offset = file.seek(std::io::SeekFrom::End(0))? as usize;

        let reader = BufReader::new(file.try_clone()?);
        let writer = BufWriter::new(file);

        Ok(LogReaderWriter {
            id,
            folder: folder.as_ref().to_path_buf(),
            reader,
            writer,
            offset,
        })
    }

    /// Transfer all written commands to reader file.
    pub(crate) fn transfer(mut self) -> Result<()> {
        // Flush remaining write
        self.writer.flush()?;

        // Make sure no invalid commands are written
        let file = self
            .writer
            .into_inner()
            .map_err(|e| KvError::CannotTransferActiveLog(e.to_string()))?;
        file.set_len(self.offset as u64)?;

        let writer_path = finder::writer_path(&self.folder, &self.id);
        let reader_path = finder::reader_path(&self.folder, &self.id);

        fs::copy(&writer_path, &reader_path)
            .and_then(|_| fs::remove_file(&writer_path))
            .map_err(|e| KvError::CannotTransferActiveLog(e.to_string()))?;

        Ok(())
    }
}
