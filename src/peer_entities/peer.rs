use crate::utilities::constants::U8_BYTE_SIZE;
use std::collections::HashSet;

/// This struct is used to store the information of a Peer and manage its bitfield.
#[derive(Clone, Debug)]
pub struct Peer {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub bitfield: HashSet<u32>,
    pub choked_me: bool,
    pub interested_in_me: bool,
    pub is_choked: bool,
}

/// This struct is used to represent a Peer that wants to connect with us.
#[derive(Clone)]
pub struct IncomingPeer {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub bitfield: HashSet<u32>,
    pub choked_me: bool,
    pub interested_in_me: bool,
    pub is_choked: bool,
}

impl Peer {
    /// Creates a new Peer instance.
    pub fn new(id: String, ip: String, port: u16) -> Peer {
        Peer {
            id,
            ip,
            port,
            is_choked: true,
            choked_me: true,
            interested_in_me: false,
            bitfield: HashSet::new(),
        }
    }
}

impl IncomingPeer {
    /// Creates a new IncomingPeer instance.
    pub fn new(id: String, ip: String, port: u16) -> IncomingPeer {
        IncomingPeer {
            id,
            ip,
            port,
            is_choked: true,
            choked_me: true,
            interested_in_me: false,
            bitfield: HashSet::new(),
        }
    }
}

pub trait PeerInterface {
    fn get_id(&self) -> String;
    fn get_ip(&self) -> String;
    fn get_port(&self) -> u16;
    fn get_bitfield(&self) -> &HashSet<u32>;
    fn get_choked_me(&self) -> bool;
    fn get_is_choked(&self) -> bool;
    fn get_interested_in_me(&self) -> bool;
    fn set_id(&mut self, id: String);
    fn set_interested_in_me(&mut self, val: bool);
    fn set_choked_me(&mut self, val: bool);
    fn set_is_choked(&mut self, val: bool);
    fn set_bitfield(&mut self, val: Vec<u8>);
    fn add_piece(&mut self, index: u32);
}

impl PeerInterface for Peer {
    /// Returns peer id.
    fn get_id(&self) -> String {
        self.id.clone()
    }

    /// Returns peer ip.
    fn get_ip(&self) -> String {
        self.ip.clone()
    }

    /// Returns peer port.
    fn get_port(&self) -> u16 {
        self.port
    }

    /// Returns peer bitfield.
    fn get_bitfield(&self) -> &HashSet<u32> {
        &self.bitfield
    }

    /// Returns if peer is has choked me.
    fn get_choked_me(&self) -> bool {
        self.choked_me
    }

    /// Returns if we choked the peer.
    fn get_is_choked(&self) -> bool {
        self.is_choked
    }

    /// Returns if peer is interested in me.
    fn get_interested_in_me(&self) -> bool {
        false
    }

    /// Sets peer interested in me to val.
    fn set_interested_in_me(&mut self, val: bool) {
        self.interested_in_me = val;
    }

    /// Sets if i am choked to val.
    fn set_choked_me(&mut self, val: bool) {
        self.choked_me = val;
    }

    /// Sets if the peer is choked to val.
    fn set_is_choked(&mut self, val: bool) {
        self.is_choked = val;
    }

    /// Sets bitfield to val.
    fn set_bitfield(&mut self, val: Vec<u8>) {
        self.bitfield = translate_bitfield(val);
    }

    /// Sets peer id to val.
    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    /// Adds a piece to the peer bitfield.
    fn add_piece(&mut self, index: u32) {
        self.bitfield.insert(index);
    }
}

impl PeerInterface for IncomingPeer {
    /// Returns peer id.
    fn get_id(&self) -> String {
        self.id.clone()
    }

    /// Returns peer ip.
    fn get_ip(&self) -> String {
        self.ip.clone()
    }

    /// Returns peer port.
    fn get_port(&self) -> u16 {
        self.port
    }

    /// Returns peer bitfield.
    fn get_bitfield(&self) -> &HashSet<u32> {
        &self.bitfield
    }

    /// Returns if peer is has choked me.
    fn get_choked_me(&self) -> bool {
        self.choked_me
    }

    /// Returns if we choked the peer.
    fn get_is_choked(&self) -> bool {
        self.is_choked
    }

    /// Returns if peer is interested in me.
    fn get_interested_in_me(&self) -> bool {
        self.interested_in_me
    }

    /// Sets peer interested in me to val.
    fn set_interested_in_me(&mut self, val: bool) {
        self.interested_in_me = val;
    }

    /// Sets if i am choked to val.
    fn set_choked_me(&mut self, val: bool) {
        self.choked_me = val;
    }

    /// Sets if the peer is choked to val.
    fn set_is_choked(&mut self, val: bool) {
        self.is_choked = val;
    }

    /// Sets bitfield to val.
    fn set_bitfield(&mut self, val: Vec<u8>) {
        self.bitfield = translate_bitfield(val);
    }

    /// Sets peer id to val.
    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    /// Adds a piece to the peer bitfield.
    fn add_piece(&mut self, index: u32) {
        self.bitfield.insert(index);
    }
}

/// Returns HashSet of u32 representing the bitfield given a vector of bytes.
fn translate_bitfield(src: Vec<u8>) -> HashSet<u32> {
    let mut bitfield = HashSet::new();
    for i in 0..(src.len() * 8) {
        if bitfield_has_piece(i as u32, &src) {
            bitfield.insert(i as u32);
        }
    }
    bitfield
}

/// Adds a piece to the bitfield.
pub(crate) fn add_piece_to_bitfield(bitfield: &mut [u8], index: u32) {
    let byte_index = index / U8_BYTE_SIZE;
    let offset = index % U8_BYTE_SIZE;

    bitfield[byte_index as usize] |= 1 << (U8_BYTE_SIZE - 1 - offset);
}

/// Returns if the bitfield has a piece at index.
fn bitfield_has_piece(index: u32, src: &Vec<u8>) -> bool {
    let byte_index = index / U8_BYTE_SIZE;
    let offset = index % U8_BYTE_SIZE;
    if byte_index >= src.len() as u32 {
        return false;
    }
    src[byte_index as usize] >> (U8_BYTE_SIZE - 1 - offset) & 1 != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utilities::utils::create_id;

    #[test]
    fn test_has_piece() {
        let mut peer = Peer::new(create_id(), "127.0.0.1".to_string(), 443);
        peer.set_bitfield(vec![255, 255]);

        assert!(peer.get_bitfield().contains(&1));
    }

    #[test]
    fn test_hasnt_piece() {
        let mut peer = Peer::new(create_id(), "127.0.0.1".to_string(), 443);
        peer.set_bitfield(vec![255, 247]); // file composed by 16 pieces but only 15 are available

        assert!(!peer.get_bitfield().contains(&12));
    }
}
