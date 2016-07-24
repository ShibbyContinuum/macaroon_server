extern crate macaroon_server;
extern crate redis;
extern crate macaroons;

use macaroon_server::{ Server, Key, User, AuthRedis, Api };
use std::thread;
use redis::*;

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


fn main() {

    let mut key = Key::new();

    key.genkey();

    let mut api = Api::new();

    let service_token = api.auth.minter.mint_token(&key);

    println!("Starting Server..");

    let auth_redis = thread::spawn(move || {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let mut auth = AuthRedis::new(client);
        println!("{}", auth.is_auth());

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

