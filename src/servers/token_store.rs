use redis;

pub struct TokenStore {
    pub connection: redis::Client,
}

impl TokenStore {
    pub fn new(client: redis::Client) -> TokenStore {
        TokenStore {
            connection: client,
        }
    }
}


