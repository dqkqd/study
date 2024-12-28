use core::str;
use std::{
    error::Error,
    io::{self, BufRead, BufReader, Read, StdinLock, StdoutLock, Write},
    net::TcpStream,
};

struct Console<'a> {
    reader: StdinLock<'a>,
    writer: StdoutLock<'a>,
}

impl<'a> Console<'a> {
    fn new() -> Console<'a> {
        Console {
            reader: io::stdin().lock(),
            writer: io::stdout().lock(),
        }
    }

    fn prompt(&mut self) -> std::io::Result<()> {
        self.write(">>> ")
    }

    fn read(&mut self) -> std::io::Result<String> {
        let mut buffer = String::new();
        self.reader.read_line(&mut buffer)?;
        Ok(buffer)
    }

    fn write(&mut self, s: &str) -> std::io::Result<()> {
        self.writer.write_all(s.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }
}

fn parse_simple_string<R: Read>(reader: R) -> Result<Option<String>, Box<dyn Error>> {
    // +<data>\r\n
    let mut reader = BufReader::new(reader);

    let mut buf = Vec::with_capacity(1024);
    let n = reader.read_until(b'\n', &mut buf)?;

    // not start with '$'
    if n < 2 || buf[0] != b'+' {
        return Ok(None);
    }

    let s = str::from_utf8(&buf[1..n])?;
    Ok(Some(s.to_string()))
}

fn parse_bulk_string<R: Read>(reader: R) -> Result<Option<String>, Box<dyn Error>> {
    // $<length>\r\n<data>\r\n

    let mut reader = BufReader::new(reader);

    let mut buf = Vec::with_capacity(1024);
    let n = reader.read_until(b'\n', &mut buf)?;

    // not start with '$'
    if n < 2 || buf[0] != b'$' {
        return Ok(None);
    }

    // not end with '\r\n'
    if buf[n - 1] != b'\n' || buf[n - 2] != b'\r' {
        return Ok(None);
    }

    let length = str::from_utf8(&buf[1..n - 2])?.parse::<usize>()?;

    buf.clear();
    let n = reader.read_until(b'\n', &mut buf)?;
    if n != length + 2 {
        return Ok(None);
    }

    // not end with '\r\n'
    if buf[n - 1] != b'\n' || buf[n - 2] != b'\r' {
        return Ok(None);
    }

    let s = str::from_utf8(&buf[..n])?;
    Ok(Some(s.to_string()))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:6379")?;
    stream.set_read_timeout(None)?;
    stream.set_write_timeout(None)?;

    let mut console = Console::new();
    console.write("Connected to the server\n")?;

    let mut buf = [0u8; 1024];

    loop {
        // prompt
        console.prompt()?;

        let input = console.read()?;

        stream.write_all(input.as_bytes())?;
        stream.flush()?;

        let n = stream.read(&mut buf)?;

        let output = match parse_bulk_string(&buf[..n])? {
            Some(output) => output,
            None => parse_simple_string(&buf[..n])?.unwrap_or_default(),
        };
        console.write(&output)?;
    }
}
