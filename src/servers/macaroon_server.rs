
use super::key::Key;
use super::api::Api;
use std::net::TcpListener;
use std::net::Ipv4Addr;
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


pub struct MacaroonServerBuilder {
    key: Key,
    addr: Ipv4Addr,
    port: u16,
    interface: MacaroonAuth,
}

impl MacaroonServerBuilder {
    pub fn new() -> MacaroonServerBuilder {
        MacaroonServerBuilder {
            key: Key::new(),
            addr: Ipv4Addr::new(127,0,0,1),
            port: 12345,
            interface: MacaroonAuth::new(),
        }
    }
}

pub struct MacaroonServer {}

impl MacaroonServer {

    pub fn listen(&self, server_builder: MacaroonServerBuilder) {
        let server = TcpListener::bind((server_builder.addr, server_builder.port)).expect("Unable to bind");
        println!("Listening");
        for stream in server.incoming() {
            match stream {
                Ok(stream) => {
// TODO: Implement ThreadPool::new()
                    thread::spawn(move|| {
                        match MacaroonServer::handle_connection(stream) {
                            Ok(received) => match received.verify_integrity(&server_builder.key.key.clone()) {
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
}


pub struct MacaroonMinter {
    id_rng: ChaChaRng,
}

impl MacaroonAuth {
    pub fn new() -> MacaroonAuth {
        MacaroonAuth {
            minter: MacaroonMinter::new(),
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
