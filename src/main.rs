extern crate memmap;
extern crate rand;
extern crate macaroons;

use std::io::prelude::*;
use std::io::{ BufWriter, BufReader };
use std::net::{ TcpStream, TcpListener };
use std::thread;

use memmap::*;
use rand::{ Rng, SeedableRng };
use rand::os::OsRng;
use rand::chacha::ChaChaRng;

use macaroons::token::Token;
use macaroons::caveat::Caveat;
use macaroons::verifier::Verifier;

struct Server {
    server: TcpListener,
}

impl Server {
    pub fn new() -> Server {
        Server {
            server: TcpListener::bind("127.0.0.1:12345").expect("Unable to bind"),
        }
    }

    fn listen(&self) {
        println!("Listening");
        for stream in self.server.incoming() {
            match stream {
                Ok(stream) => {
// TODO: Implement ThreadPool::new()
                    thread::spawn(move|| {
                        Server::handle_connection(stream)
                    });
                }
                Err(e) => { println!("Connection Failed: {}", e); }
            }
        }
    }

    fn handle_connection(stream: TcpStream) {
        let mut bufreader = BufReader::new(stream);
        let mut vec: Vec<u8> = Vec::new();
        bufreader.read_to_end(&mut vec);
        println!("{:?}", vec);
    }
}

//  struct Client
//  source: Read video from server
//  cursor: To stdin of vlc
//  bufwriter: From Cursor to file //temporary
//  path: Local copy of destination file

struct Client {
    source: BufWriter<TcpStream>,
}

impl Client {
    fn new() -> Client {
        Client {
            source: BufWriter::new(TcpStream::connect("127.0.0.1:12345").expect("Unable to Connect")),
        }
    }

    fn write(&mut self) {
        let mut string = "TEST".to_string();
        println!("Writing: TEST");
        self.source.write(string.as_bytes());
    }
}

//  WARNING: HERE BE DRAGONS, THE KEY STRUCTURE IS USED TO GENERATE A SERVER SECRET FOR MACAROONS.
//  MESSING THIS UP WILL MAKE YOUR MACAROONS WORTHLESS, SHARING THIS WILL MAKE YOUR MACAROONS WORTHLESS.
//  THIS IS NOT GAURANTEED TO NOT BE MESSED UP, IF CONSIDERING THIS LIB FOR PRODUCTION USAGE TURN BACK NOW.
//  WARNING: THIS IS AN UNVETTED IMPLEMENTATION.  REALLY DO NOT USE THIS IMPLEMENTATION. (as of July 12, 2016)

struct Key {
    key: Mmap,
}

impl Key {
    pub fn new() -> Key {
        Key {
            key: Mmap::anonymous(512, Protection::ReadWrite).expect("Unable to create memory map"),
        }
    }

    fn genkey(&mut self) {
        let mut osrng = OsRng::new().expect("Failed to start OsRng during Key::genkey");
        let mut word: [u32; 8] = [0; 8];
        word[0] = osrng.next_u32();
        word[1] = osrng.next_u32();
        word[2] = osrng.next_u32();
        word[3] = osrng.next_u32();
        word[4] = osrng.next_u32();
        word[5] = osrng.next_u32();
        word[6] = osrng.next_u32();
        word[7] = osrng.next_u32();
        println!("{:?}", &word);
        let mut chacha = ChaChaRng::from_seed(&word);
        chacha.fill_bytes( unsafe { &mut self.key.as_mut_slice() });
        self.key.set_protection(Protection::Read);
    }
}
    

struct GetVideo {
    path: Option<String>,
    identity: [u8; 512],
}

impl GetVideo {
    fn new() -> GetVideo {
        GetVideo {
            path: None,
            identity: GetVideo::get_identity(),
        }
    }

    pub fn get_identity() -> [u8; 512] {
        let mut id: [u8; 512] = [0; 512];
        let mut word: [u32; 8] = [0; 8];
        let mut osrng = OsRng::new().expect("Failed to generate identity");
        word[0] = osrng.next_u32();
        word[1] = osrng.next_u32();
        word[2] = osrng.next_u32();
        word[3] = osrng.next_u32();
        word[4] = osrng.next_u32();
        word[5] = osrng.next_u32();
        word[6] = osrng.next_u32();
        word[7] = osrng.next_u32();
        let mut chacha = ChaChaRng::from_seed(&mut word);
        chacha.fill_bytes(&mut id);
        id
    }

    fn gen_video_caveats(&self) {
        let video_caveat = Caveat::first_party(b"video = ME3331".to_vec());
        let time_caveat = Caveat::first_party(b"time < 2017-12-19T16:39:57-08:00".to_vec());
// Chrono::DateTime::parse_from_rfc2822
        let acct_caveat = Caveat::first_party(b"acc = test_account".to_vec());
    }

    
}

fn main() {
    println!("Starting Server..");
    let server = thread::spawn(move || { Server::new().listen(); });
    println!("Server Started!");
    println!("Starting Client..");
    let client = thread::spawn(move || { Client::new().write(); Client::new().write(); });
    println!("Client Started!");
    let mut key = Key::new();
    key.genkey();
    println!("{:?}", unsafe { key.key.as_slice() } );
    client.join();
    server.join();
}
