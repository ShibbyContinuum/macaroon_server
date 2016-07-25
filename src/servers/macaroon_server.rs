
use super::key::Key;
use super::api::Api;
use std::net::TcpListener;
use std::io::BufReader;
use std::io::Read;
use std::net::TcpStream;
use std::thread;

use macaroons::token::Token;
use macaroons::verifier::Verifier;

use rand::{ Rng, SeedableRng };
use rand::os::OsRng;
use rand::chacha::ChaChaRng;

use hex::*;

pub struct MacaroonServer {
    server: TcpListener,
    interface: MacaroonAuth,
}

impl MacaroonServer {
    pub fn new() -> MacaroonServer {
        MacaroonServer {
            server: TcpListener::bind("127.0.0.1:12345").expect("Unable to bind"),
            interface: MacaroonAuth::new(),
        }
    }

    pub fn listen(&self, key: [u8; 512]) {
        println!("Listening");
        for stream in self.server.incoming() {
            match stream {
                Ok(stream) => {
// TODO: Implement ThreadPool::new()
                    thread::spawn(move|| {
                        match MacaroonServer::handle_connection(stream) {
                            Ok(received) => match received.verify(&key) {
                                true => {
                                    let mut api = Api::new();
                                    api.set_macaroon(received.identifier.to_vec());
                                    api.set_id(received.caveats[0].caveat_id.to_vec());
                                    api.set_video_request(received.caveats[1].caveat_id.to_vec());
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

pub struct MacaroonAuth {
    pub minter: MacaroonMinter,
    challenge: Option<Token>,
    new_token: Option<Token>,
    verifier: Option<Verifier>,
}

pub struct MacaroonMinter {
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

    pub fn mint_token(&mut self, key: &Key) -> Token {
        let id = self.get_identity().as_mut().to_hex();
        let token = Token::new( &key.key[0..], id.into_bytes() , None);
//  Write the ID to the database!  Remove the ID from the database if access has been revoked, the user
//  may request new macaroons if their access has been revoked incorrectly!
        token
    }
}
