const U8_BYTE_SIZE: u32 = 8;

#[derive(Clone, Debug)]
pub struct Peer {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub bitfield: Vec<u8>,
    pub choked_me: bool,
    pub interested_in_me: bool,
    pub is_choked: bool,
}

#[derive(Clone)]
pub struct IncomingPeer {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub bitfield: Vec<u8>,
    pub choked_me: bool,
    pub interested_in_me: bool,
    pub is_choked: bool,
}

impl Peer {
    pub fn new(id: String, ip: String, port: u16) -> Peer {
        Peer {
            id,
            ip,
            port,
            is_choked: true,
            choked_me: true,
            interested_in_me: false,
            bitfield: vec![],
        }
    }
}

impl IncomingPeer {
    pub fn new(id: String, ip: String, port: u16) -> IncomingPeer {
        IncomingPeer {
            id,
            ip,
            port,
            is_choked: true,
            choked_me: true,
            interested_in_me: false,
            bitfield: vec![],
        }
    }
}
pub trait PeerInterface {
    fn get_id(&self) -> String;
    fn get_ip(&self) -> String;
    fn get_port(&self) -> u16;
    fn get_bitfield(&self) -> Vec<u8>;
    fn get_choked_me(&self) -> bool;
    fn get_is_choked(&self) -> bool;
    fn get_interested_in_me(&self) -> bool;
    fn set_id(&mut self, id: String);
    fn set_interested_in_me(&mut self, val: bool);
    fn set_choked_me(&mut self, val: bool);
    fn set_is_choked(&mut self, val: bool);
    fn set_bitfield(&mut self, val: Vec<u8>);
    fn has_piece(&self, index: u32) -> bool;
    fn add_piece(&mut self, index: u32);
}
impl PeerInterface for Peer {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_ip(&self) -> String {
        self.ip.clone()
    }

    fn get_port(&self) -> u16 {
        self.port
    }

    fn get_bitfield(&self) -> Vec<u8> {
        self.bitfield.clone()
    }

    fn get_choked_me(&self) -> bool {
        self.choked_me
    }

    fn get_is_choked(&self) -> bool {
        self.is_choked
    }

    fn get_interested_in_me(&self) -> bool {
        false
    }
    fn set_interested_in_me(&mut self, val: bool) {
        self.interested_in_me = val;
    }

    fn set_choked_me(&mut self, val: bool) {
        self.choked_me = val;
    }

    fn set_is_choked(&mut self, val: bool) {
        self.is_choked = val;
    }

    fn set_bitfield(&mut self, val: Vec<u8>) {
        self.bitfield = val;
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }
    fn has_piece(&self, index: u32) -> bool {
        let byte_index = index / U8_BYTE_SIZE;
        let offset = index % U8_BYTE_SIZE;
        if byte_index >= self.bitfield.len() as u32 {
            return false;
        }
        self.bitfield[byte_index as usize] >> (U8_BYTE_SIZE - 1 - offset) & 1 != 0
    }

    fn add_piece(&mut self, index: u32) {
        add_piece_to_bitfield(&mut self.bitfield, index);
    }
}

impl PeerInterface for IncomingPeer {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_ip(&self) -> String {
        self.ip.clone()
    }

    fn get_port(&self) -> u16 {
        self.port
    }

    fn get_bitfield(&self) -> Vec<u8> {
        self.bitfield.clone()
    }

    fn get_choked_me(&self) -> bool {
        self.choked_me
    }

    fn get_is_choked(&self) -> bool {
        self.is_choked
    }

    fn get_interested_in_me(&self) -> bool {
        self.interested_in_me
    }
    fn set_interested_in_me(&mut self, val: bool) {
        self.interested_in_me = val;
    }
    fn set_choked_me(&mut self, val: bool) {
        self.choked_me = val;
    }

    fn set_is_choked(&mut self, val: bool) {
        self.is_choked = val;
    }

    fn set_bitfield(&mut self, val: Vec<u8>) {
        self.bitfield = val;
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn has_piece(&self, index: u32) -> bool {
        let byte_index = index / U8_BYTE_SIZE;
        let offset = index % U8_BYTE_SIZE;

        self.bitfield[byte_index as usize] >> (U8_BYTE_SIZE - 1 - offset) & 1 != 0
    }

    fn add_piece(&mut self, index: u32) {
        add_piece_to_bitfield(&mut self.bitfield, index);
    }
}

pub(crate) fn add_piece_to_bitfield(bitfield: &mut [u8], index: u32) {
    let byte_index = index / U8_BYTE_SIZE;
    let offset = index % U8_BYTE_SIZE;

    bitfield[byte_index as usize] |= 1 << (U8_BYTE_SIZE - 1 - offset);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::create_id;

    #[test]
    fn test_has_piece() {
        let mut peer = Peer::new(create_id(), "127.0.0.1".to_string(), 443);

        let bitfield = [255, 255].to_vec(); // file composed by 16 pieces

        peer.bitfield = bitfield;
        assert!(peer.has_piece(1));
    }

    #[test]
    fn test_hasnt_piece() {
        let mut peer = Peer::new(create_id(), "127.0.0.1".to_string(), 443);

        let bitfield = [255, 247].to_vec(); // file composed by 16 pieces but only 15 are available

        peer.bitfield = bitfield;
        assert!(!peer.has_piece(12));
    }
}
