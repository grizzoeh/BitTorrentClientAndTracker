pub struct Peer {
    pub id: String,
    pub ip: String,
    pub port: u16,
    //pub is_connected: bool, ??  FIXME
    pub is_choked: bool,
    pub is_interested: bool,
    pub choked_me: bool,
}

impl Peer {
    pub fn new(id: String, ip: String, port: u16) -> Peer {
        Peer {
            id,
            ip,
            port,
            is_choked: true,
            is_interested: false,
            choked_me: true,
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_ip(&self) -> &String {
        &self.ip
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_is_choked(&self) -> bool {
        self.is_choked
    }

    pub fn get_is_interested(&self) -> bool {
        self.is_interested
    }

    pub fn get_choked_me(&self) -> bool {
        self.choked_me
    }
}
