use std::io::{ BufWriter, BufReader };
use std::net::{ TcpStream, TcpListener };

use std::io::prelude::*;
use std::prelude::*;

pub struct User {
    writer: BufWriter<TcpStream>,
    reader: BufReader<TcpListener>,
}

impl User {
    pub fn new() -> User {
        User {
            writer: BufWriter::new(TcpStream::connect("127.0.0.1:12345").expect("Unable to Connect")),
            reader: BufReader::new(TcpListener::bind("127.0.0.1:12346").expect("Unable to Bind")),
        }
    }

    pub fn write(&mut self, buf: &[u8]) {
        match self.writer.write_all(buf) {
            Ok(o) => {},
            Err(e) => println!("Write failed: {}", e),
        }
    }
}
