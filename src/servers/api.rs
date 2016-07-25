pub struct Api {
    id: Vec<u8>,
    macaroon_id: Vec<u8>,
    video_requested: Vec<u8>,
}

impl Api {
    pub fn new() -> Api {
        Api {
            id: Vec::new(),
            macaroon_id: Vec::new(),
            video_requested: Vec::new(),
        }
    }

    pub fn set_id(&mut self, id: Vec<u8>) {
        self.id = id
    }

    pub fn set_macaroon(&mut self, macaroon_id: Vec<u8>) {
        self.macaroon_id = macaroon_id
    }

    pub fn set_video_request(&mut self, video_requested: Vec<u8>) {
        self.video_requested = video_requested
    }
}
