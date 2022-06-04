use std::io::prelude::*;
use std::net::TcpStream;

use crate::utils::vecu8_to_u32;

const PSTR: &str = "BitTorrent protocol";
const _CHOKE_MESSAGE: &[u8] = &[0, 0, 0, 1, 0]; //len = 1, id = 0
const UNCHOKE_MESSAGE: &[u8] = &[0, 0, 0, 1, 1]; //len = 1, id = 1
const INTERESTED_MESSAGE: &[u8] = &[0, 0, 0, 1, 2]; //len = 1, id = 2
const _NOT_INTERESTED_MESSAGE: &[u8] = &[0, 0, 0, 1, 3]; //len = 1, id = 3
const INFO_HASH_LEN: usize = 20;
const PEER_ID_LEN: usize = 20;
const RESERVED_SPACE_LEN: u8 = 8;
const BITFIELD_LEN_LEN: usize = 4;
const BITFIELD_ID_LEN: usize = 1;
const UNCHOKE_MESSAGE_LEN: usize = 5;
const REQUEST_MESSAGE: &[u8] = &[0, 0, 0, 13, 6];
const BITFIELD_ID: u8 = 5;
const PIECE_ID: u8 = 7;
const PIECE_ID_LEN: usize = 1;
const PIECE_INDEX_LEN: usize = 4;
const PIECE_OFFSET_LEN: usize = 4;
const CHUNK_LEN_LEN: usize = 4;
const CHUNK_INITIAL_LEN: u32 = 9;
const PSTR_LEN_LEN: usize = 1;
const U8_BYTE_SIZE: u32 = 8;

#[derive(Debug)]
pub enum PeerError {
    ConnectionError(String),
    HandshakeWritingError(String),
    HandshakeReadingError(String),
    InterestedError(String),
    PeerDidNotRespond(String),
    UnchokeError(String),
    RequestError(String),
    PeerDidNotUnchokeUs(String),
}

#[derive(Clone)]
pub struct Peer {
    pub id: String,
    pub ip: String,
    pub port: u16,
    // pub is_connected: bool,
    pub bitfield: Vec<u8>,
    pub choked_me: bool,
    // pub interested_in_me: bool,
    pub is_choked: bool,
}

pub struct PeerConnection {
    pub peer: Peer,
    pub stream: TcpStream,
}

impl PeerConnection {
    pub fn new(
        peer: Peer,
        info_hash: Vec<u8>,
        client_id: String,
    ) -> Result<PeerConnection, PeerError> {
        let stream = match TcpStream::connect(format!("{}:{}", peer.ip, peer.port)) {
            Ok(stream) => stream,
            Err(e) => return Err(PeerError::ConnectionError(e.to_string())),
        };
        let mut connection = PeerConnection { peer, stream };
        let con = match connection.handshake(info_hash, client_id) {
            Ok(_) => connection,
            Err(e) => return Err(e),
        };
        Ok(con)
    }

    pub fn read_n_bytes(&mut self, n: usize) -> Result<Vec<u8>, PeerError> {
        let mut buffer = vec![0; n];
        match self.stream.read_exact(buffer.as_mut()) {
            Ok(_) => Ok(buffer),
            Err(_) => Err(PeerError::PeerDidNotRespond(format!(
                "Error reading {} bytes from peer {}",
                n, self.peer.id
            ))),
        }
    }

    fn check_handshake(&mut self, info_hash: Vec<u8>) -> Result<(), PeerError> {
        // Must receive <pstrlen><pstr><reserved><info_hash><peer_id>
        let pstr_len = match self.read_n_bytes(PSTR_LEN_LEN) {
            Ok(i) => {
                if i[0] != PSTR.len() as u8 {
                    return Err(PeerError::HandshakeReadingError(format!(
                        "Error reading handshake from peer {}:{}",
                        &self.peer.ip, self.peer.port
                    )));
                } else {
                    i[0]
                }
            }
            Err(e) => {
                println!("{:?}", e);
                return Err(PeerError::HandshakeReadingError(format!(
                    "Error reading handshake from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )));
            }
        };

        match self.read_n_bytes((pstr_len + RESERVED_SPACE_LEN) as usize) {
            Ok(_) => {}
            Err(_) => {
                return Err(PeerError::HandshakeReadingError(format!(
                    "Error reading handshake from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };

        match self.read_n_bytes(INFO_HASH_LEN) {
            Ok(info) => {
                if info_hash != info {
                    return Err(PeerError::HandshakeReadingError(format!(
                        "Wrong info_hash in handshake from peer {}:{}",
                        &self.peer.ip, self.peer.port
                    )));
                }
            }
            Err(_) => {
                return Err(PeerError::HandshakeReadingError(format!(
                    "Error reading info_hash in handshake from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };

        match self.read_n_bytes(PEER_ID_LEN) {
            Ok(id) => {
                if id != self.peer.id.as_bytes() {
                    return Err(PeerError::HandshakeReadingError(format!(
                        "Wrong peer_id in handshake from peer {}:{}",
                        &self.peer.ip, self.peer.port
                    )));
                }
            }
            Err(_) => {
                return Err(PeerError::HandshakeReadingError(format!(
                    "Error reading handshake from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };
        Ok(())
    }

    pub fn handshake(&mut self, info_hash: Vec<u8>, peer_id: String) -> Result<(), PeerError> {
        // Must send <pstrlen><pstr><reserved><info_hash><peer_id>
        let mut data = vec![PSTR.len() as u8];
        data.extend(PSTR.as_bytes());
        data.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        data.extend(&info_hash);
        data.extend(peer_id.as_bytes());

        let _i = match self.stream.write_all(&data) {
            Ok(i) => i,
            Err(_) => {
                return Err(PeerError::HandshakeWritingError(format!(
                    "Error writing handshake to peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };

        match self.check_handshake(info_hash) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn read_bitfield(&mut self) -> Result<(), PeerError> {
        let len = match self.read_n_bytes(BITFIELD_LEN_LEN) {
            Ok(i) => vecu8_to_u32(&i) - 1,
            Err(_) => {
                return Err(PeerError::PeerDidNotRespond(format!(
                    "Error reading bitfield from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };
        match self.read_n_bytes(BITFIELD_ID_LEN) {
            Ok(i) => {
                if i[0] != BITFIELD_ID {
                    return Err(PeerError::PeerDidNotRespond(format!(
                        "Error reading bitfield from peer {}:{}",
                        &self.peer.ip, self.peer.port
                    )));
                }
            }
            Err(_) => {
                return Err(PeerError::PeerDidNotRespond(format!(
                    "Error reading bitfield from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };
        match self.read_n_bytes(len as usize) {
            Ok(bitfield) => {
                self.peer.bitfield = bitfield;
            }
            Err(_) => {
                return Err(PeerError::PeerDidNotRespond(format!(
                    "Error reading bitfield from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };
        Ok(())
    }
    // las comento porque nose si esto de iniciar una coneccion con el peer es necesario o se le puede pasar un stream ya conectado
    /*
    pub fn _choke(&mut self) -> Result<(), PeerError> {
        println!("SENDING CHOKE");
        let mut stream = match TcpStream::connect(format!("{}:{}", &self.ip, self.port)) {
            Ok(stream) => stream,
            Err(_) => return Err(PeerError::ConnectionError),
        };

        match stream.write_all(CHOKE_MESSAGE) {
            Ok(i) => i,
            Err(_) => return Err(PeerError::ChokeError),
        };
        self.is_choked = true;

        Ok(())
    }
    */
    pub fn unchoke(&mut self) -> Result<(), PeerError> {
        match self.stream.write_all(UNCHOKE_MESSAGE) {
            Ok(_) => self.peer.is_choked = false,
            Err(_) => return Err(PeerError::UnchokeError("Error writing unchoke".to_string())),
        };
        Ok(())
    }

    pub fn interested(&mut self) -> Result<(), PeerError> {
        match self.stream.write_all(INTERESTED_MESSAGE) {
            Ok(i) => i,
            Err(_) => {
                return Err(PeerError::InterestedError(format!(
                    "Error writing interested to peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };
        Ok(())
    }

    pub fn read_unchoke(&mut self) -> Result<(), PeerError> {
        match self.read_n_bytes(UNCHOKE_MESSAGE_LEN) {
            Ok(response) => {
                println!("{:?}", response);
                if response == UNCHOKE_MESSAGE {
                    self.peer.choked_me = false;
                } else {
                    return Err(PeerError::PeerDidNotUnchokeUs(
                        "Peer didn't unchoke us".to_string(),
                    ));
                }
            }
            Err(_) => {
                return Err(PeerError::UnchokeError(format!(
                    "Error reading unchoke from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };
        Ok(())
    }

    pub fn request(&mut self, piece_idx: u32, offset: u32, length: &u32) -> Result<(), PeerError> {
        let mut vec_message = REQUEST_MESSAGE.to_vec();
        vec_message.extend(&piece_idx.to_be_bytes());
        vec_message.extend(&offset.to_be_bytes());
        vec_message.extend(&length.to_be_bytes());
        println!("sending request message: {:?}", &vec_message);
        match self.stream.write_all(vec_message.as_slice()) {
            Ok(i) => i,
            Err(_) => {
                return Err(PeerError::RequestError(format!(
                    "Error writing request to peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };
        Ok(())
    }

    pub fn read_chunk(
        &mut self,
        piece_idx: u32,
        offset: u32,
        length: &u32,
    ) -> Result<Vec<u8>, PeerError> {
        let block_len = match self.read_n_bytes(CHUNK_LEN_LEN) {
            Ok(len_vec) => vecu8_to_u32(&len_vec) - CHUNK_INITIAL_LEN,
            Err(_) => {
                return Err(PeerError::RequestError(format!(
                    "Error reading block length from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )));
            }
        };
        if block_len != *length {
            return Err(PeerError::RequestError(format!(
                "Wrong chunk received, mismatched length  {}:{}",
                &self.peer.ip, self.peer.port
            )));
        }
        match self.read_n_bytes(PIECE_ID_LEN) {
            Ok(id) => {
                if id[0] != PIECE_ID {
                    return Err(PeerError::PeerDidNotRespond(format!(
                        "Peer {}:{} didn't respond with a piece",
                        &self.peer.ip, self.peer.port
                    )));
                }
            }
            Err(_) => {
                return Err(PeerError::RequestError(format!(
                    "Error reading chunk from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };

        match self.read_n_bytes(PIECE_INDEX_LEN) {
            Ok(idx) => {
                if vecu8_to_u32(&idx) != piece_idx {
                    return Err(PeerError::PeerDidNotRespond(format!(
                        "Peer {}:{} didn't respond with the right piece",
                        &self.peer.ip, self.peer.port
                    )));
                }
            }
            Err(_) => {
                return Err(PeerError::RequestError(format!(
                    "Error reading chunk from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };

        match self.read_n_bytes(PIECE_OFFSET_LEN) {
            Ok(begin) => {
                if vecu8_to_u32(&begin) != offset {
                    return Err(PeerError::PeerDidNotRespond(format!(
                        "Peer {}:{} didn't respond with the right chunk",
                        &self.peer.ip, self.peer.port
                    )));
                }
            }
            Err(_) => {
                return Err(PeerError::RequestError(format!(
                    "Error reading chunk from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };

        let chunk = match self.read_n_bytes(block_len as usize) {
            Ok(chunk) => chunk,
            Err(_) => {
                return Err(PeerError::RequestError(format!(
                    "Error reading chunk from peer {}:{}",
                    &self.peer.ip, self.peer.port
                )))
            }
        };

        println!("RECEIVED=[piece_idx={}, offset={}]", &piece_idx, &offset);
        Ok(chunk)
    }
}

impl Peer {
    pub fn new(id: String, ip: String, port: u16) -> Peer {
        Peer {
            id,
            ip,
            port,
            is_choked: true,
            //interested_in_me: false,
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
}
