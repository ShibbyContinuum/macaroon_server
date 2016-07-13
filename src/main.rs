#![feature(type_ascription)]
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
    interface: Api,
}

impl Server {
    pub fn new() -> Server {
        Server {
            server: TcpListener::bind("127.0.0.1:12345").expect("Unable to bind"),
            interface: Api::new(),
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
        let mut word: [u32; 8] = osrng.gen();
        println!("{:?}", &word);
        let mut chacha = ChaChaRng::from_seed(&word);
        chacha.fill_bytes( unsafe { &mut self.key.as_mut_slice() });
        self.key.set_protection(Protection::Read);
    }
}
    

struct Api {
    auth: Auth,
}

impl Api {
    pub fn new() -> Api {
        Api {
            auth: Auth::new(),
        }
    }
}
    

struct Auth {
    auth_type: MacaroonAuth,
}

impl Auth {
    pub fn new() -> Auth {
        Auth {
            auth_type: MacaroonAuth::new(),
        }
    }
}

struct MacaroonAuth {
    minter: MacaroonMinter,
    challenge: Option<Token>,
    authed: bool,
    new_token: Option<Token>,
}

struct MacaroonMinter {
    id_rng: ChaChaRng,
}

impl MacaroonAuth {
    pub fn new() -> MacaroonAuth {
        MacaroonAuth {
            minter: MacaroonMinter::new(),
            challenge: None,
            authed: false,
            new_token: None,
        }
    }
}

impl MacaroonMinter {
    pub fn new() -> MacaroonMinter {
        MacaroonMinter {
            id_rng: MacaroonMinter::get_identity_rng(),
        }
    }

    pub fn get_identity_rng() -> ChaChaRng {
        let mut osrng = OsRng::new().expect("Failed to generate identity");
        let mut word: [u32; 8] = osrng.gen();
        let mut chacha = ChaChaRng::from_seed(&mut word);
    }

    pub fn get_identity(&self) -> [u8; 512] {
        let mut id: [u8; 512] = [0; 512];
        self.id_rng.fill_bytes(&mut id);
        id
    }

///  This array MUST be verified [0..] 
///  array[0] must always contain the service that is being accessed
///  array[1] must contain the user identification
///-----------------------------------------------------------------
///  || array[0] and array[1] can be thought of as service caveats
///-----------------------------------------------------------------
///  array[2] might contain which interface the user is accessing
///    perhaps "interface = admin" || "interface = user" || "interface = moderator"
///  array[3] might be which api function we want to access
///  array[4..] might be the user supplied data, functions, the sky is the limit.

    fn service_caveats() -> [Caveat; 3] {
        let service: Caveat = Caveat::first_party(b"service = testservice_please_ignore".to_vec());
        let mut array: [Caveat; 3] = [service; 3];
//      array[0] is service caveat, it must be the first to be verified.
        array[1] = Caveat::first_party(b"id = test_id_please_remove".to_vec());
        array[2] = Caveat::first_party(b"interface = visitor".to_vec());
    }

    fn mint_token(self, caveat: [Caveat], key: Key) -> Token {
        Token::new( unsafe { key.key.as_slice(); }, { self.id_rng.get_identity(); }, None)
              .add_caveat(caveat[0])
              .add_caveat(caveat[1])
              .add_caveat(caveat[2]);
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
    let mut api_auth = Auth::new();
    let macaroon_interface = MacaroonAuth::new();
    api_auth(&macaroon_interface);
    let service_caveats = MacaroonMinter::service_caveats();
    let service_token = macaroon_interface.mint_token(service_caveats);
    println!("Printing Service Token:{:?}", service_token);
    client.join();
    server.join();
}
