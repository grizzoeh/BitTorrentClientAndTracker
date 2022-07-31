use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Stores the data of the differents peers that sent us a request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Peer {
    pub port: String,
    pub id: String,
    pub ip: String,
    pub info_hash: Vec<u8>,
    pub uploaded: String,
    pub downloaded: String,
    pub left: String,
    pub connected: bool,
    pub completed: bool,
    pub numwant: u32,
    pub compact: u16,
    pub no_peer_id: u16,
    pub key: String,
    pub trackerid: String,
}

#[allow(clippy::too_many_arguments)]
impl Peer {
    pub fn new(
        port: String,
        id: String,
        ip: String,
        info_hash: Vec<u8>,
        uploaded: String,
        downloaded: String,
        left: String,
        connected: bool,
        completed: bool,
    ) -> Peer {
        Peer {
            port,
            id,
            ip,
            info_hash,
            uploaded,
            downloaded,
            left,
            connected,
            completed,
            numwant: DEFAULT_NUNWANT_VALUE,
            compact: DEFAULT_COMPACT_VALUE,
            no_peer_id: DEFAULT_NO_PEER_ID,
            key: DEFAULT_KEY.to_string(),
            trackerid: DEFAULT_TRACKERID.to_string(),
        }
    }

    pub fn set_numwant(&mut self, numwant: u32) {
        self.numwant = numwant;
    }

    pub fn set_compact(&mut self, compact: u16) {
        self.compact = compact;
    }

    pub fn set_no_peer_id(&mut self, no_peer_id: u16) {
        self.no_peer_id = no_peer_id;
    }

    pub fn set_key(&mut self, key: String) {
        self.key = key;
    }

    pub fn set_trackerid(&mut self, trackerid: String) {
        self.trackerid = trackerid;
    }
}
