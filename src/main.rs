#![feature(type_ascription)]

extern crate rand;
extern crate macaroons;

use std::io::prelude::*;
use std::io::{ BufWriter, BufReader };
use std::prelude::*;
use std::net::{ TcpStream, TcpListener };
use std::thread;
use std::str;

use rand::{ Rng, SeedableRng };
use rand::os::OsRng;
use rand::chacha::ChaChaRng;

use macaroons::token::Token;
use macaroons::caveat::Caveat;
use macaroons::verifier::Verifier;

macro_rules! add_caveats {
    ($token:expr, $($caveat:expr),*) => {
        $token
        $(
            .add_caveat($caveat.into())
        )*
    }
}

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
                        match Server::handle_connection(stream) {
                            Ok(received) => { println!("{}", received); },
                            Err(e) => { panic!("Bad Data, bleh"); },
                        }
                    });
                }
                Err(e) => { println!("Connection Failed: {}", e); }
            }
        }
    }

    fn handle_connection(stream: TcpStream) -> Result<String, std::string::FromUtf8Error> {
        let mut bufreader = BufReader::new(stream);
        let mut vec: Vec<u8> = Vec::new();
        bufreader.read_to_end(&mut vec);
        let str = String::from_utf8(vec);
        str
    }
}

struct Client {
    connection: BufWriter<TcpStream>,
}

impl Client {
    fn new() -> Client {
        Client {
            connection: BufWriter::new(TcpStream::connect("127.0.0.1:12345").expect("Unable to Connect")),
        }
    }

    fn write(&mut self, buf: &[u8]) {
        match self.connection.write_all(buf) {
            Ok(o) => {},
            Err(e) => println!("Write failed: {}", e),
        }
    }
}

//  WARNING: HERE BE DRAGONS, THE KEY STRUCTURE IS USED TO GENERATE A SERVER SECRET FOR MACAROONS.
//  MESSING THIS UP WILL MAKE YOUR MACAROONS WORTHLESS, SHARING THIS WILL MAKE YOUR MACAROONS WORTHLESS.
//  THIS IS NOT GAURANTEED TO NOT BE MESSED UP, IF CONSIDERING THIS LIB FOR PRODUCTION USAGE TURN BACK NOW.
//  WARNING: THIS IS AN UNVETTED IMPLEMENTATION.  REALLY DO NOT USE THIS IMPLEMENTATION. (as of July 12, 2016)

struct Key {
    key: [u8; 512],
}

impl Key {
    pub fn new() -> Key {
        Key {
            key: [0; 512],
        }
    }

    fn genkey(&mut self) {
        let mut osrng = OsRng::new().expect("Failed to start OsRng during Key::genkey");
        let mut word: [u32; 8] = osrng.gen();
        let mut chacha = ChaChaRng::from_seed(&word);
        chacha.fill_bytes(&mut self.key[0..]);
    }
}
    
//  The Api Structure should hold all the modules this server would like to utilize.

struct Api {
    auth: MacaroonAuth,
    is_auth: bool,
}

impl Api {
    pub fn new() -> Api {
        Api {
            auth: MacaroonAuth::new(),
            is_auth: false,
        }
    }
}

struct MacaroonAuth {
    minter: MacaroonMinter,
    challenge: Option<Token>,
    new_token: Option<Token>,
    verifier: Option<Verifier>,
}

struct MacaroonMinter {
    id_rng: ChaChaRng,
}

impl MacaroonAuth {
    pub fn new() -> MacaroonAuth {
        MacaroonAuth {
            minter: MacaroonMinter::new(),
            challenge: None,
            new_token: None,
            verifier: None,
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
        chacha
    }

    pub fn get_identity(&mut self) -> [u8; 512] {
        let mut id: [u8; 512] = [0; 512];
        self.id_rng.fill_bytes(&mut id);
        id
    }

    fn mint_token(&mut self, key: &Key) -> Token {
        let id = self.get_identity();
        let token = Token::new( &key.key[0..], id.to_vec() , None);
        token
    }
}

fn main() {
    let mut key = Key::new();
    key.genkey();
    let mut api = Api::new();
    let service_token = api.auth.minter.mint_token(&key);
    println!("Starting Server..");
    let server = thread::spawn(move || { Server::new().listen(); });
    let s_token = add_caveats!(service_token, 
        Caveat::first_party(b"interface = portal".to_vec())
    );
    println!("Starting Client!");
    let client = thread::spawn(move || {
        let client = Client::new().write(&s_token.serialize().into_bytes());
    });
    client.join();
    server.join();
}
