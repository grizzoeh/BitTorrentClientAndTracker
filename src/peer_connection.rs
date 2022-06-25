use crate::client::CHUNK_SIZE;
use crate::errors::peer_connection_error::PeerConnectionError;
use crate::peer::Peer;
use crate::utils::vecu8_to_u32;
use std::io::prelude::*;
use std::str;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

const PSTR: &str = "BitTorrent protocol";
const CHOKE_MESSAGE: &[u8] = &[0, 0, 0, 1, 0]; //len = 1, id = 0
const UNCHOKE_MESSAGE: &[u8] = &[0, 0, 0, 1, 1]; //len = 1, id = 1
const INTERESTED_MESSAGE: &[u8] = &[0, 0, 0, 1, 2]; //len = 1, id = 2
const NOT_INTERESTED_MESSAGE: &[u8] = &[0, 0, 0, 1, 3]; //len = 1, id = 3
const HAVE_MESSAGE: &[u8] = &[0, 0, 0, 5, 4]; //len = 5, id = 4
const INFO_HASH_LEN: usize = 20;
const PEER_ID_LEN: usize = 20;
const RESERVED_SPACE_LEN: u8 = 8;
const HAVE_LEN: usize = 4;
const REQUEST_MESSAGE: &[u8] = &[0, 0, 0, 13, 6];
const PIECE_INDEX_LEN: usize = 4;
const PIECE_OFFSET_LEN: usize = 4;
const CHUNK_LEN_LEN: usize = 4;
const CHUNK_INITIAL_LEN: u32 = 9;
const PSTR_LEN_LEN: usize = 1;
const CANCEL_LEN: usize = 12;
const CANCEL_MESSAGE: &[u8] = &[0, 0, 0, 13, 8];
const MESSAGE_ID_LEN: usize = 1;
const CHOKE_ID: u8 = 0;
const UNCHOKE_ID: u8 = 1;
const INTERESTED_ID: u8 = 2;
const NOT_INTERESTED_ID: u8 = 3;
const HAVE_ID: u8 = 4;
const BITFIELD_ID: u8 = 5;
const REQUEST_ID: u8 = 6;
const PIECE_ID: u8 = 7;
const CANCEL_ID: u8 = 8;
const ERROR_ID: u8 = 10;

pub struct PeerConnection<T: Read + Write> {
    pub peer: Peer,
    pub stream: Arc<Mutex<T>>,
    pub sender_logger: Arc<Mutex<Sender<String>>>,
}

impl<T: Read + Write> PeerConnection<T> {
    pub fn new(
        peer: Peer,
        info_hash: Vec<u8>,
        client_id: String,
        stream: Arc<Mutex<T>>,
        sender_logger: Arc<Mutex<Sender<String>>>,
    ) -> Result<PeerConnection<T>, PeerConnectionError> {
        let mut connection = PeerConnection {
            peer,
            stream,
            sender_logger,
        };
        connection.handshake(info_hash, client_id)?;
        Ok(connection)
    }

    pub fn read_n_bytes(&mut self, n: usize) -> Result<Vec<u8>, PeerConnectionError> {
        let mut buffer = vec![0; n];
        self.stream.lock()?.read_exact(buffer.as_mut())?;
        Ok(buffer)
    }

    pub fn read_detect_message(&mut self) -> Result<u8, PeerConnectionError> {
        let msg_len_vec = self.read_n_bytes(CHUNK_LEN_LEN)?;
        let msg_len = vecu8_to_u32(&msg_len_vec);
        let msg_id_vec = self.read_n_bytes(MESSAGE_ID_LEN)?;
        let msg_id = msg_id_vec[0];

        match msg_id {
            CHOKE_ID => {
                self.read_choke();
                return Ok(CHOKE_ID);
            }
            UNCHOKE_ID => {
                self.read_unchoke();
                return Ok(UNCHOKE_ID);
            }
            INTERESTED_ID => {
                self.read_interested()?;
                return Ok(INTERESTED_ID);
            }
            NOT_INTERESTED_ID => {
                self.read_not_interested()?;
                return Ok(NOT_INTERESTED_ID);
            }
            HAVE_ID => {
                self.read_have()?;
                return Ok(HAVE_ID);
            }
            BITFIELD_ID => {
                self.read_bitfield((msg_len - 1) as usize)?;
                return Ok(BITFIELD_ID);
            }
            REQUEST_ID => {
                self.read_request()?;
                return Ok(REQUEST_ID);
            }
            PIECE_ID => {
                let right_len = CHUNK_SIZE + CHUNK_INITIAL_LEN;
                if msg_len != right_len {
                    return Err(PeerConnectionError::new(format!(
                        "Wrong chunk received, expected:{}, received{}, peer: {}:{}",
                        right_len, msg_len, &self.peer.ip, self.peer.port
                    )));
                }
                return Ok(PIECE_ID);
            }
            CANCEL_ID => {
                self.read_cancel()?;
                println!("Cancel message received");
                self.sender_logger
                    .lock()?
                    .send("Cancal message received".to_string())?;
            }
            _ => {
                return Err(PeerConnectionError::new(format!(
                    "unexpected character: {}",
                    msg_id
                )))
            }
        }
        Ok(ERROR_ID)
    }

    // This function sends the handshake, reads the bitfield and saves it in the peer.
    pub fn handshake(
        &mut self,
        info_hash: Vec<u8>,
        peer_id: String,
    ) -> Result<(), PeerConnectionError> {
        // Must send <pstrlen><pstr><reserved><info_hash><peer_id>
        let mut data = vec![PSTR.len() as u8];
        data.extend(PSTR.as_bytes());
        data.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        data.extend(&info_hash);
        data.extend(peer_id.as_bytes());

        let _ = self.stream.lock()?.write_all(&data)?;

        let _ = self.read_handshake(info_hash)?;
        Ok(())
    }

    fn read_handshake(&mut self, info_hash: Vec<u8>) -> Result<(), PeerConnectionError> {
        // Must receive <pstrlen><pstr><reserved><info_hash><peer_id>
        let i = self.read_n_bytes(PSTR_LEN_LEN)?;
        if i[0] != PSTR.len() as u8 {
            return Err(PeerConnectionError::new(format!(
                "Error reading handshake from peer {}:{}, wrong pstrlen",
                &self.peer.ip, self.peer.port
            )));
        }
        let pstr_len = i[0];

        self.read_n_bytes((pstr_len + RESERVED_SPACE_LEN) as usize)?;

        let info = self.read_n_bytes(INFO_HASH_LEN)?;
        if info_hash != info {
            return Err(PeerConnectionError::new(format!(
                "Wrong info_hash in handshake from peer {}:{}",
                &self.peer.ip, self.peer.port
            )));
        }

        let id = self.read_n_bytes(PEER_ID_LEN)?;
        if id != self.peer.id.as_bytes() {
            return Err(PeerConnectionError::new(format!(
                "Wrong peer_id in handshake from peer {}:{}",
                &self.peer.ip, self.peer.port
            )));
        }

        Ok(())
    }

    pub fn read_bitfield(&mut self, msg_len: usize) -> Result<(), PeerConnectionError> {
        let bitfield = self.read_n_bytes(msg_len)?;
        self.peer.bitfield = bitfield;
        Ok(())
    }

    pub fn choke(&mut self) -> Result<(), PeerConnectionError> {
        self.stream.lock()?.write_all(CHOKE_MESSAGE)?;
        self.peer.is_choked = true;
        Ok(())
    }

    pub fn read_choke(&mut self) {
        self.peer.choked_me = true;
    }

    pub fn unchoke(&mut self) -> Result<(), PeerConnectionError> {
        self.stream.lock()?.write_all(UNCHOKE_MESSAGE)?;
        self.peer.is_choked = false;
        Ok(())
    }

    pub fn read_unchoke(&mut self) {
        self.peer.choked_me = false;
    }

    pub fn interested(&mut self) -> Result<(), PeerConnectionError> {
        self.stream.lock()?.write_all(INTERESTED_MESSAGE)?;
        Ok(())
    }

    pub fn read_interested(&mut self) -> Result<(), PeerConnectionError> {
        self.peer.interested_in_me = true;
        Ok(())
    }

    pub fn not_interested(&mut self) -> Result<(), PeerConnectionError> {
        self.stream.lock()?.write_all(NOT_INTERESTED_MESSAGE)?;
        Ok(())
    }

    pub fn read_not_interested(&mut self) -> Result<(), PeerConnectionError> {
        self.peer.interested_in_me = false;
        Ok(())
    }

    pub fn have(&mut self, piece_index: u32) -> Result<(), PeerConnectionError> {
        let mut data = HAVE_MESSAGE.to_vec();
        data.extend(&piece_index.to_be_bytes());
        self.stream.lock()?.write_all(data.as_slice())?;
        Ok(())
    }

    pub fn read_have(&mut self) -> Result<u32, PeerConnectionError> {
        let have_vector = self.read_n_bytes(HAVE_LEN)?;
        let piece_idx = vecu8_to_u32(&have_vector);
        self.peer.add_piece(piece_idx); // FIXME HAY Q TESTEAR ESTA FUNCION
        Ok(piece_idx)
    }

    pub fn cancel(
        &mut self,
        piece_index: u32,
        begin: u32,
        length: u32,
    ) -> Result<(), PeerConnectionError> {
        let mut data = CANCEL_MESSAGE.to_vec();
        data.extend(&piece_index.to_be_bytes());
        data.extend(&begin.to_be_bytes());
        data.extend(&length.to_be_bytes());
        self.stream.lock()?.write_all(data.as_slice())?;
        Ok(())
    }

    pub fn read_cancel(&mut self) -> Result<(u32, u32, u32), PeerConnectionError> {
        let cancel_vec = self.read_n_bytes(CANCEL_LEN)?;
        Ok((
            vecu8_to_u32(&cancel_vec[..3]), //FIX, hay que declarar buffer y ver que hacer
            vecu8_to_u32(&cancel_vec[4..7]),
            vecu8_to_u32(&cancel_vec[8..11]),
        ))
    }

    pub fn request_chunk(
        &mut self,
        piece_idx: u32,
        offset: u32,
        length: &u32,
    ) -> Result<(), PeerConnectionError> {
        let mut vec_message = REQUEST_MESSAGE.to_vec();
        vec_message.extend(&piece_idx.to_be_bytes());
        vec_message.extend(&offset.to_be_bytes());
        vec_message.extend(&length.to_be_bytes());
        self.stream.lock()?.write_all(vec_message.as_slice())?;
        Ok(())
    }

    pub fn read_request(&mut self) -> Result<(), PeerConnectionError> {
        let request_vec = self.read_n_bytes(PIECE_INDEX_LEN)?;
        let piece_idx = vecu8_to_u32(&request_vec);
        let offset_vec = self.read_n_bytes(PIECE_OFFSET_LEN)?;
        let offset = vecu8_to_u32(&offset_vec);
        let length_vec = self.read_n_bytes(CHUNK_LEN_LEN)?;
        let length = vecu8_to_u32(&length_vec);
        println!(
            "piece index:{}, offset:{}, length:{}",
            piece_idx, offset, length
        );
        self.sender_logger.lock()?.send(format!(
            "piece index:{}, offset:{}, length:{}",
            piece_idx, offset, length
        ))?;

        //TODO, send pieces requested.
        Ok(())
    }

    pub fn read_chunk(
        &mut self,
        piece_idx: u32,
        offset: u32,
    ) -> Result<Vec<u8>, PeerConnectionError> {
        let index = self.read_n_bytes(PIECE_INDEX_LEN)?;
        if vecu8_to_u32(&index) != piece_idx {
            return Err(PeerConnectionError::new(format!(
                "Peer {}:{} didn't respond with the right piece",
                &self.peer.ip, self.peer.port
            )));
        }

        let begin = self.read_n_bytes(PIECE_OFFSET_LEN)?;
        if vecu8_to_u32(&begin) != offset {
            return Err(PeerConnectionError::new(format!(
                "Peer {}:{} didn't respond with the right chunk",
                &self.peer.ip, self.peer.port
            )));
        }

        let chunk = self.read_n_bytes(CHUNK_SIZE as usize)?;

        Ok(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};
    use std::sync::mpsc::channel;

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
        let (sender, _) = channel();
        assert!(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
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
        let (sender, _) = channel();
        assert!(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
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
        let (sender, _) = channel();
        assert!(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
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
        let (sender, _) = channel();
        assert!(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
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
        let (sender, _) = channel();
        assert!(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
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
        let (sender, _) = channel();
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
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
        let (sender, _) = channel();
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.read_detect_message().is_ok());
        assert_eq!(peer_connection.peer.bitfield, vec![255, 255, 255, 128]);
    }

    /*
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
            Arc::new(Mutex::new(&mut stream)),
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.read_detect_message().is_err());
    }
     */

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
        let (sender, _) = channel();
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
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
        let (sender, _) = channel();
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.interested().is_ok());
        // stream.cursorW has len 73
        assert_eq!(&stream.cursor_w.get_ref()[(73 - 5)..], INTERESTED_MESSAGE);
    }

    #[test]
    fn test_have() {
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
        let (sender, _) = channel();
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.have(22).is_ok());
        let mut have_message = HAVE_MESSAGE.to_vec();
        have_message.extend([0, 0, 0, 22]); //idx
                                            // stream.cursorW has len 73
        assert_eq!(&stream.cursor_w.get_ref()[(73 - 5)..], have_message);
    }

    #[test]
    fn test_cancel() {
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
        let (sender, _) = channel();
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.cancel(2, 1, 33).is_ok());
        let mut cancel_message = CANCEL_MESSAGE.to_vec();
        cancel_message.extend([0, 0, 0, 2]); //idx
        cancel_message.extend([0, 0, 0, 1]); //begin
        cancel_message.extend([0, 0, 0, 33]); //length

        // stream.cursorW has len 73
        assert_eq!(&stream.cursor_w.get_ref()[(73 - 5)..], cancel_message);
    }

    // #[test]
    // fn test_read_unchoke_ok() {
    //     // init peer connection
    //     let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
    //     let mut handshake = vec![PSTR.len() as u8];
    //     handshake.extend(PSTR.as_bytes());
    //     handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
    //     handshake.extend(&info_hash);
    //     handshake.extend("peer_id_123456789012".as_bytes());
    //     handshake.extend(UNCHOKE_MESSAGE); // to be read
    //     let peer = Peer::new(
    //         "peer_id_123456789012".to_string(),
    //         "1".to_string(),
    //         433 as u16,
    //     );
    //     let mut stream = MockTcpStream::new(handshake.clone());
    //     let mut peer_connection = PeerConnection::new(
    //         peer,
    //         info_hash,
    //         "client_id_1234567890".to_string(),
    //         Arc::new(Mutex::new(&mut stream)),
    //     )
    //     .unwrap();
    //     // end init peer connection
    //     assert!(peer_connection.read_unchoke().is_ok());
    //     assert!(peer_connection.peer.choked_me == false);
    // }

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
        let (sender, _) = channel();
        let mut peer_connection = PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(&mut stream)),
            Arc::new(Mutex::new(sender)),
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
    //FIXME AREGLAR PRUEBAS
    /*
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
            Arc::new(Mutex::new(&mut stream)),
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
            Arc::new(Mutex::new(&mut stream)),
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
            Arc::new(Mutex::new(&mut stream)),
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
            Arc::new(Mutex::new(&mut stream)),
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
            Arc::new(Mutex::new(&mut stream)),
        )
        .unwrap();
        // end init peer connection
        assert!(peer_connection.read_chunk(0, 0, &(3 as u32)).is_err());
    }
    */
}
