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

use tiny_keccak::Keccak;

pub mod servers;
pub mod user;
