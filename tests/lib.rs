#![feature(type_ascription)]

extern crate rand;
extern crate macaroons;
extern crate hex;
extern crate redis;
extern crate tiny_keccak;
extern crate serde_json;
extern crate byteorder;
extern crate macaroon_server;

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

use macaroon_server::{ Server, Key, User, AuthRedis, Api };


#[test]
fn mint_token() {
    let mut key = Key::new();
    key.genkey();
    let mut api = Api::new();
    api.auth.minter.mint_token(&key);
}

#[test]
fn verify_true_token() {
    let mut key = Key::new();
    key.genkey();
    let mut api = Api::new();
    let token = api.auth.minter.mint_token(&key);
    let bool = token.verify(&key.key);
    assert!(bool)
}

#[test]
#[should_panic(expected = "failed")]
fn verify_false_token() {
    let mut key = Key::new();
    key.genkey();
    let mut key2 = Key::new();
    key2.genkey();
    let mut api = Api::new();
    let token = api.auth.minter.mint_token(&key);
    let bool =  token.verify(&key2.key);
    assert!(bool)
}
