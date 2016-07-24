#![feature(type_ascription)]

extern crate rand;
extern crate macaroons;
extern crate hex;
extern crate redis;
extern crate tiny_keccak;
extern crate serde_json;
extern crate byteorder;

use std::io::prelude::*;
use std::io::{ BufWriter, BufReader };
use std::prelude::*;
use std::net::{ TcpStream, TcpListener };
use std::thread;
use std::str;
use std::collections::HashMap;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::Cursor;

use byteorder::{ BigEndian, ReadBytesExt, WriteBytesExt };

use rand::{ Rng, SeedableRng };
use rand::os::OsRng;
use rand::chacha::ChaChaRng;

use macaroons::token::Token;
use macaroons::caveat::Caveat;
use macaroons::verifier::Verifier;

use hex::*;

use redis::{ Commands, Connection, Client, ConnectionLike, Cmd };

use serde_json::Value;
use serde_json::builder;

use tiny_keccak::Keccak;

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

    fn listen(&self, key: [u8; 512]) {
        println!("Listening");
        for stream in self.server.incoming() {
            match stream {
                Ok(stream) => {
// TODO: Implement ThreadPool::new()
                    thread::spawn(move|| {
                        match Server::handle_connection(stream) {
                            Ok(received) => match received.verify(&key) {
                                true => {  
                                    let mut field = Fields::new();
                                    field.set_macaroon(received.identifier.to_vec());
                                    field.set_id(received.caveats[0].caveat_id.to_vec());
                                    field.set_video_request(received.caveats[1].caveat_id.to_vec()); 
                                },
                                false => { println!("false"); 
                                },
                            },
                            Err(e) => { panic!("Bad Data, bleh"); },
                        }
                    });
                }
                Err(e) => { println!("Connection Failed: {}", e); }
            }
        }
    }

    fn handle_connection(stream: TcpStream) -> Result<Token, &'static str> {
        let mut bufreader = BufReader::new(stream);
        let mut vec: Vec<u8> = Vec::new();
        bufreader.read_to_end(&mut vec);
        let token_result = Token::deserialize(&mut vec);
        token_result
    }
}

struct User {
    connection: BufWriter<TcpStream>,
}

impl User {
    fn new() -> User {
        User {
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
/*
    fn genkey_write_once(&mut self, path: & Path) {
        let mut osrng = OsRng::new().expect("Failed to start OsRng during Key::genkey_write_once");
        let mut word: [u32; 8] = osrng.gen();
        let mut file = OpenOptions::new()
                                   .write(true)
                                   .create(true)
                                   .open(&path).unwrap();

        file.write_all(&word[..]);
        let mut chacha = ChaChaRng::from_seed(&word);
        chacha.fill_bytes(&mut self.key[0..]);
    }
*/
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
        let id = self.get_identity().as_mut().to_hex();
        let token = Token::new( &key.key[0..], id.into_bytes() , None);
//  Write the ID to the database!  Remove the ID from the database if access has been revoked, the user
//  may request new macaroons if their access has been revoked incorrectly! 
        token
    }
}

struct StoreToken {
    connection: redis::Client,
}

impl StoreToken {
    pub fn new(client: redis::Client) -> StoreToken {
        StoreToken {
            connection: client,
        }
    }
}


struct Fields {
    id: Vec<u8>,
    macaroon_id: Vec<u8>,
    video_requested: Vec<u8>,
}

impl Fields {
    pub fn new() -> Fields {
        Fields {
            id: Vec::new(),
            macaroon_id: Vec::new(),
            video_requested: Vec::new(),
        }
    }

    fn set_id(&mut self, id: Vec<u8>) {
        self.id = id
    }

    fn set_macaroon(&mut self, macaroon_id: Vec<u8>) {
        self.macaroon_id = macaroon_id
    }

    fn set_video_request(&mut self, video_requested: Vec<u8>) {
        self.video_requested = video_requested
    }
}

struct AuthRedis {
    token_store: StoreToken,
    pii_key: Key,
    field: Fields,
}

impl AuthRedis {
    pub fn new(client: redis::Client) -> AuthRedis {
        AuthRedis {
            token_store: StoreToken::new(client),
            pii_key: AuthRedis::gen_pii_key(),
            field: Fields::new(),
        }
    }

    fn gen_pii_key() -> Key {
        let mut key = Key::new();
        key.genkey();
        key
    }

    fn hash(&mut self) -> String {
        let mut sha3 = Keccak::new_sha3_256();
        sha3.update(&self.field.id);
        sha3.update(&self.field.video_requested);
        sha3.update(&self.pii_key.key[..]);
        let mut res: [u8; 32] = [0; 32];
        sha3.finalize(&mut res);
        let str = String::from_utf8(res[..].to_vec()).unwrap();
        str
    }

    fn store_pair(&mut self) -> redis::RedisResult<()> {
        redis::cmd("SET").arg(self.hash()).arg(&self.field.macaroon_id[..]).query(&self.token_store.connection)
    }


    fn is_auth(&mut self) -> bool {
        match redis::cmd("EXISTS").arg(self.hash())
                                 .arg(self.field.macaroon_id.clone())
                                 .query(&self.token_store.connection) {
            Ok(()) => true,
            Err(e) => false,
        }
    }

    fn revoke(&mut self) -> redis::RedisResult<()> {
        redis::cmd("DEL").arg(self.hash())
                         .query(&self.token_store.connection)
    }
}

fn main() {

    let mut key = Key::new();

    key.genkey();

    let mut api = Api::new();

    let service_token = api.auth.minter.mint_token(&key);

    println!("Starting Server..");

    let auth_redis = thread::spawn(move || {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        AuthRedis::new(client);
    });
    let server = thread::spawn(move || {
        Server::new().listen(key.key); 
    });

    let s_token = add_caveats!(service_token, 
        Caveat::first_party(b"id = 00000000".to_vec()),
        Caveat::first_party(b"video = Fire".to_vec())
    );

    println!("Starting Client!");

    let client = thread::spawn(move || {
        let client = User::new().write(&s_token.serialize().into_bytes());
    });

    client.join();
    auth_redis.join();
    server.join();
}
