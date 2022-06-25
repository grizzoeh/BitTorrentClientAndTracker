const U8_BYTE_SIZE: u32 = 8;

#[derive(Clone)]
pub struct Peer {
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
            interested_in_me: false,
            choked_me: true,
            bitfield: vec![],
        }
    }

    // HasPiece tells if a bitfield has a particular index set
    pub fn has_piece(&self, index: u32) -> bool {
        let byte_index = index / U8_BYTE_SIZE;
        let offset = index % U8_BYTE_SIZE;

        self.bitfield[byte_index as usize] >> (U8_BYTE_SIZE - 1 - offset) & 1 != 0
    }

    pub fn add_piece(&mut self, index: u32) {
        let byte_index = index / U8_BYTE_SIZE;
        let offset = index % U8_BYTE_SIZE;

        self.bitfield[byte_index as usize] |= 1 << (U8_BYTE_SIZE - 1 - offset);
    }
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
