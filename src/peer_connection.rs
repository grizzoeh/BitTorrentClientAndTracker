use crate::peer::{Peer, PeerError};
use crate::utils::vecu8_to_u32;
use std::io::prelude::*;

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

pub struct PeerConnection<T: Read + Write> {
    pub peer: Peer,
    pub stream: T,
}

impl<T: Read + Write> PeerConnection<T> {
    pub fn new(
        peer: Peer,
        info_hash: Vec<u8>,
        client_id: String,
        stream: T,
    ) -> Result<PeerConnection<T>, PeerError> {
        let mut connection = PeerConnection { peer, stream };
        let peer_con = match connection.handshake(info_hash, client_id) {
            Ok(_) => connection,
            Err(e) => return Err(e),
        };
        Ok(peer_con)
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

        match self.read_handshake(info_hash) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn read_handshake(&mut self, info_hash: Vec<u8>) -> Result<(), PeerError> {
        // Must receive <pstrlen><pstr><reserved><info_hash><peer_id>
        let pstr_len = match self.read_n_bytes(PSTR_LEN_LEN) {
            Ok(i) => {
                if i[0] != PSTR.len() as u8 {
                    return Err(PeerError::HandshakeReadingError(format!(
                        "Error reading handshake from peer {}:{}, wron pstrlen",
                        &self.peer.ip, self.peer.port
                    )));
                } else {
                    i[0]
                }
            }
            Err(_) => {
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

    pub fn request_chunk(
        &mut self,
        piece_idx: u32,
        offset: u32,
        length: &u32,
    ) -> Result<(), PeerError> {
        let mut vec_message = REQUEST_MESSAGE.to_vec();
        vec_message.extend(&piece_idx.to_be_bytes());
        vec_message.extend(&offset.to_be_bytes());
        vec_message.extend(&length.to_be_bytes());
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    struct MockTcpStream {
        cursor_r: Cursor<Vec<u8>>,
        cursor_w: Cursor<Vec<u8>>,
    }

    impl MockTcpStream {
        pub fn new(vec_r: Vec<u8>) -> MockTcpStream {
            MockTcpStream {
                cursor_r: Cursor::new(vec_r),
                cursor_w: Cursor::new(Vec::new()),
            }
        }
    }
    impl Read for MockTcpStream {
        fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), io::Error> {
            self.cursor_r.read_exact(buf)
        }
        fn read(&mut self, _buf: &mut [u8]) -> Result<usize, io::Error> {
            self.cursor_r.read(_buf)
        }
    }
    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
            self.cursor_w.write(buf)
        }
        fn flush(&mut self) -> Result<(), io::Error> {
            self.cursor_w.flush()
        }
    }

    #[test]
    fn test_new_with_wrong_pstrlen() {
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        // peer side
        let mut handshake = vec![5 as u8]; // wrong pstrlen
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        // end peer side
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        assert!(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream
        )
        .is_err());
    }

    #[test]
    fn test_new_with_wrong_pstr() {
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        // peer side
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(b"wrong pstr");
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        // end peer side
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        assert!(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream
        )
        .is_err());
    }

    #[test]
    fn test_new_with_wrong_infohash() {
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        // peer side
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&"wrong_infohash".as_bytes().to_vec());
        handshake.extend("peer_id_123456789012".as_bytes());
        // end peer side
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        assert!(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream
        )
        .is_err());
    }

    #[test]
    fn test_new_with_wrong_peer_id() {
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        // peer side
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_wrong".as_bytes()); // wrong peer_id
                                                      // end peer side
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        assert!(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream
        )
        .is_err());
    }

    #[test]
    fn test_new_ok_handshake() {
        // this tests new(), handshake(), and read_handshake()
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        // peer side
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        // end peer side
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        assert!(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream
        )
        .is_ok());
    }

    #[test]
    fn test_read_n_bytes() {
        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        handshake.extend("testing_read_n_bytes_123".as_bytes()); // to be read
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert_eq!(
            peer_connection.read_n_bytes(24).unwrap(),
            "testing_read_n_bytes_123".as_bytes()
        );
    }

    #[test]
    fn test_read_bitfield_ok() {
        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        handshake.extend([0, 0, 0, 5, 5, 255, 255, 255, 128]); // bitfield to be read
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.read_bitfield().is_ok());
        assert_eq!(peer_connection.peer.bitfield, vec![255, 255, 255, 128]);
    }

    #[test]
    fn test_read_bitfield_wrong_bitfield_id() {
        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        handshake.extend([0, 0, 0, 5, 2, 255, 255, 255, 128]); // bitfield to be read
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.read_bitfield().is_err());
    }

    #[test]
    fn test_unchoke() {
        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.unchoke().is_ok());
        // stream.cursorW has len 73
        assert_eq!(&stream.cursor_w.get_ref()[(73 - 5)..], UNCHOKE_MESSAGE);
    }

    #[test]
    fn test_interested() {
        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.interested().is_ok());
        // stream.cursorW has len 73
        assert_eq!(&stream.cursor_w.get_ref()[(73 - 5)..], INTERESTED_MESSAGE);
    }

    #[test]
    fn test_read_unchoke_ok() {
        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        handshake.extend(UNCHOKE_MESSAGE); // to be read
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.read_unchoke().is_ok());
        assert!(peer_connection.peer.choked_me == false);
    }

    #[test]
    fn test_request_chunk() {
        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        let mut request_message = REQUEST_MESSAGE.to_vec();
        request_message.extend([0, 0, 0, 0]); // idx
        request_message.extend([0, 0, 0, 0]); // offset
        request_message.extend([0, 0, 64, 0]); // lenght

        assert!(peer_connection.request_chunk(0, 0, &(16384 as u32)).is_ok());
        // stream.cursorW has len 68+17=85
        assert_eq!(&stream.cursor_w.get_ref()[(85 - 17)..], request_message);
    }

    #[test]
    fn test_read_chunk_ok() {
        let mut piece_message: Vec<u8> = [0, 0, 0, 12].to_vec(); // lenght = 9 + 3
        piece_message.extend([7]); // piece message id
        piece_message.extend([0, 0, 0, 0]); // index
        piece_message.extend([0, 0, 0, 0]); // offset
        piece_message.extend([34, 58, 12]); // block

        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        handshake.extend(piece_message); // to be read
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert_eq!(
            peer_connection.read_chunk(0, 0, &(3 as u32)).unwrap(),
            [34, 58, 12]
        );
    }

    #[test]
    fn test_read_chunk_wrong_message_id() {
        let mut piece_message: Vec<u8> = [0, 0, 0, 12].to_vec(); // lenght = 9 + 3
        piece_message.extend([2]); // wrong piece message id
        piece_message.extend([0, 0, 0, 0]); // index
        piece_message.extend([0, 0, 0, 0]); // offset
        piece_message.extend([34, 58, 12]); // block

        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        handshake.extend(piece_message); // to be read
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.read_chunk(0, 0, &(3 as u32)).is_err());
    }

    #[test]
    fn test_read_chunk_wrong_idx() {
        let mut piece_message: Vec<u8> = [0, 0, 0, 12].to_vec(); // lenght = 9 + 3
        piece_message.extend([7]); // piece message id
        piece_message.extend([0, 0, 0, 1]); // wrong index
        piece_message.extend([0, 0, 0, 0]); // offset
        piece_message.extend([34, 58, 12]); // block

        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        handshake.extend(piece_message); // to be read
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.read_chunk(0, 0, &(3 as u32)).is_err());
    }

    #[test]
    fn test_read_chunk_wrong_lenght() {
        let mut piece_message: Vec<u8> = [0, 0, 0, 11].to_vec(); // wrong lenght = 9 + 2
        piece_message.extend([7]); // piece message id
        piece_message.extend([0, 0, 0, 0]); // index
        piece_message.extend([0, 0, 0, 0]); // offset
        piece_message.extend([34, 58, 12]); // block

        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        handshake.extend(piece_message); // to be read
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.read_chunk(0, 0, &(3 as u32)).is_err());
    }

    #[test]
    fn test_read_chunk_wrong_offset() {
        let mut piece_message: Vec<u8> = [0, 0, 0, 12].to_vec(); // lenght = 9 + 3
        piece_message.extend([7]); // piece message id
        piece_message.extend([0, 0, 0, 0]); // index
        piece_message.extend([0, 0, 0, 1]); // offset
        piece_message.extend([34, 58, 12]); // block

        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let mut handshake = vec![PSTR.len() as u8];
        handshake.extend(PSTR.as_bytes());
        handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        handshake.extend(&info_hash);
        handshake.extend("peer_id_123456789012".as_bytes());
        handshake.extend(piece_message); // to be read
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let mut stream = MockTcpStream::new(handshake.clone());
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            &mut stream,
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.read_chunk(0, 0, &(3 as u32)).is_err());
    }
}
