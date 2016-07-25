use std::io::BufWriter;
use std::net::TcpStream;
use std::io::prelude::*;
use std::prelude::*;

pub struct User {
    connection: BufWriter<TcpStream>,
}

impl User {
    pub fn new() -> User {
        User {
            connection: BufWriter::new(TcpStream::connect("127.0.0.1:12345").expect("Unable to Connect")),
        }
    }

    pub fn write(&mut self, buf: &[u8]) {
        match self.connection.write_all(buf) {
            Ok(o) => {},
            Err(e) => println!("Write failed: {}", e),
        }
    }
}
