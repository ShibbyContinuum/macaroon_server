use rand::{ Rng, SeedableRng };
use rand::os::OsRng;
use rand::chacha::ChaChaRng;

//  WARNING: HERE BE DRAGONS, THE KEY STRUCTURE IS USED TO GENERATE A SERVER SECRET FOR MACAROONS.
//  MESSING THIS UP WILL MAKE YOUR MACAROONS WORTHLESS, SHARING THIS WILL MAKE YOUR MACAROONS WORTHLESS.
//  THIS IS NOT GAURANTEED TO NOT BE MESSED UP, IF CONSIDERING THIS LIB FOR PRODUCTION USAGE TURN BACK NOW.
//  WARNING: THIS IS AN UNVETTED IMPLEMENTATION.  REALLY DO NOT USE THIS IMPLEMENTATION. (as of July 12, 2016)

pub struct Key {
    pub key: [u8; 512],
}

impl Key {
    pub fn new() -> Key {
        Key {
            key: [0; 512],
        }
    }

    pub fn genkey(&mut self) {
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
