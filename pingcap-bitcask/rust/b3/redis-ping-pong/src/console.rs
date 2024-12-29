use std::io::{self, BufRead, Write};

pub struct Console<'a> {
    reader: io::StdinLock<'a>,
    writer: io::StdoutLock<'a>,
}

impl<'a> Default for Console<'a> {
    fn default() -> Console<'a> {
        Console {
            reader: io::stdin().lock(),
            writer: io::stdout().lock(),
        }
    }
}

impl<'a> Console<'a> {
    pub fn new() -> Console<'a> {
        Console::default()
    }

    pub fn prompt(&mut self) -> std::io::Result<()> {
        self.write(">>> ")
    }

    pub fn read(&mut self) -> std::io::Result<String> {
        let mut buffer = String::new();
        self.reader.read_line(&mut buffer)?;
        Ok(buffer)
    }

    pub fn write(&mut self, s: &str) -> std::io::Result<()> {
        self.writer.write_all(s.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }
}
