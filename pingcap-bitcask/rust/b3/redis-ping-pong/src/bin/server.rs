use core::str;
use std::{
    error::Error,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream).unwrap();
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = [0u8; 1024];

    let pong = |s: &str| format!("${}\r\n{}\r\n", s.len(), s);
    loop {
        let n = stream.read(&mut buf)?;
        if n > 0 {
            // perform action?
            let s = str::from_utf8(&buf[..n]).unwrap().to_lowercase();
            let trimed = s.trim();

            let resp = match trimed.starts_with("ping") {
                true => {
                    if trimed == "ping" {
                        "PONG".to_owned()
                    } else {
                        let parts: Vec<&str> = trimed.split_whitespace().skip(1).collect();
                        parts.join(" ")
                    }
                }
                false => "INVALID COMMAND".to_owned(),
            };

            stream.write_all(pong(&resp).as_bytes())?;
        }
    }
}
