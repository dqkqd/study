use std::{
    io::{Read, Write},
    net::TcpStream,
};

use redis_ping_pong::{from_bytes, Console};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
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
        let output: String = from_bytes(&buf[..n]).unwrap_or_default();

        console.write(&output)?;
        console.write("\n")?;
    }
}
