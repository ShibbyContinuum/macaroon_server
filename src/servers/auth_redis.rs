pub struct AuthRedis {
    token_store: StoreToken,
    pii_key: Key,
    field: Api,
}

impl AuthRedis {
    pub fn new(client: redis::Client) -> AuthRedis {
        AuthRedis {
            token_store: StoreToken::new(client),
            pii_key: AuthRedis::gen_pii_key(),
            field: Api::new(),
        }
    }

    fn gen_pii_key() -> Key {
        let mut key = Key::new();
        key.genkey();
        key
    }

    fn hash(&self) -> String {
        let mut sha3 = Keccak::new_sha3_256();
        sha3.update(&self.field.id);
        sha3.update(&self.field.video_requested);
        sha3.update(&self.pii_key.key[..]);
        let mut res: [u8; 32] = [0; 32];
        sha3.finalize(&mut res);
        let str = res[..].as_ref().to_hex();
        str
    }

    fn store_pair(&mut self) -> redis::RedisResult<()> {
        redis::cmd("SET").arg(self.hash()).arg(&self.field.macaroon_id[..]).query(&self.token_store.connection)
    }


    pub fn is_auth(&mut self) -> bool {
        match redis::cmd("EXISTS").arg(self.hash())
                                 .arg(self.field.macaroon_id.clone())
                                 .query(&self.token_store.connection) {
            Ok(()) => true,
            Err(e) => false,
        }
    }

    fn revoke(&self) -> redis::RedisResult<()> {
        redis::cmd("DEL").arg(&self.hash())
                         .query(&self.token_store.connection)
    }
}

