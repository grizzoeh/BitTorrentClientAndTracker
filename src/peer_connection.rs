use crate::communication_method::CommunicationMethod;
use crate::constants::*;
use crate::errors::peer_connection_error::PeerConnectionError;
use crate::logger::LogMsg;
use crate::peer::PeerInterface;
use crate::upload_manager::PieceRequest;
use crate::utils::{u32_to_vecu8, vecu8_to_u32};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, RwLock};
pub struct PeerConnection<P: PeerInterface> {
    pub peer: RwLock<P>,
    info_hash: Vec<u8>,
    _client_id: String,
    pub stream: Arc<Mutex<Box<dyn CommunicationMethod + Send>>>,
    sender_logger: Arc<Mutex<Sender<LogMsg>>>,
    sender_upload_manager: Arc<Mutex<Sender<Option<PieceRequest>>>>,
}

pub struct Chunk {
    pub data: Vec<u8>,
    pub piece_index: u32,
    pub chunk_index: u32,
}

impl<P: PeerInterface> PeerConnection<P> {
    pub fn new(
        peer: P,
        info_hash: Vec<u8>,
        client_id: String,
        stream: Arc<Mutex<Box<dyn CommunicationMethod + Send>>>,
        sender_logger: Arc<Mutex<Sender<LogMsg>>>,
        sender_upload_manager: Arc<Mutex<Sender<Option<PieceRequest>>>>,
    ) -> PeerConnection<P> {
        PeerConnection {
            peer: RwLock::new(peer),
            info_hash,
            _client_id: client_id,
            stream,
            sender_logger,
            sender_upload_manager,
        }
    }

    pub fn read_n_bytes(self: Arc<Self>, n: usize) -> Result<Vec<u8>, PeerConnectionError> {
        let mut buffer = vec![0; n];
        self.stream.lock()?.read_exact(buffer.as_mut())?;
        Ok(buffer)
    }

    pub fn read_detect_message(self: Arc<Self>) -> Result<u8, PeerConnectionError> {
        let msg_len_vec = self.clone().read_n_bytes(CHUNK_LEN_LEN)?;
        let msg_len = vecu8_to_u32(&msg_len_vec);
        let msg_id_vec = self.clone().read_n_bytes(MESSAGE_ID_LEN)?;
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
                self.clone().read_interested()?;
                self.unchoke()?;
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
                        right_len,
                        msg_len,
                        &self.peer.read()?.get_ip(),
                        self.peer.read()?.get_port()
                    )));
                }
                return Ok(PIECE_ID);
            }
            CANCEL_ID => {
                self.clone().read_cancel()?;
                self.sender_logger
                    .lock()?
                    .send(LogMsg::Info("Cancal message received".to_string()))?;
            }
            PORT_ID => {
                self.clone().read_port()?;
                self.sender_logger
                    .lock()?
                    .send(LogMsg::Info("Port message received".to_string()))?;
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
    pub fn handshake(self: Arc<Self>, peer_id: String) -> Result<(), PeerConnectionError> {
        if !self.stream.lock()?.is_connected() {
            self.stream
                .lock()?
                .connect(&self.peer.read()?.get_ip(), self.peer.read()?.get_port())?;
        }

        let _r = self
            .stream
            .lock()?
            .set_read_timeout(Some(std::time::Duration::from_secs(15)));
        // Must send <pstrlen><pstr><reserved><info_hash><peer_id>
        let mut data = vec![PSTR.len() as u8];
        data.extend(PSTR.as_bytes());
        data.extend(vec![0; RESERVED_SPACE_LEN as usize]);
        data.extend(&self.info_hash);
        data.extend(peer_id.as_bytes());

        let _ = self.clone().stream.lock()?.write_all(&data)?;

        let _ = self.clone().read_handshake(self.info_hash.clone())?;
        Ok(())
    }

    fn read_handshake(self: Arc<Self>, info_hash: Vec<u8>) -> Result<(), PeerConnectionError> {
        // Must receive <pstrlen><pstr><reserved><info_hash><peer_id>
        let i = self.clone().read_n_bytes(PSTR_LEN_LEN)?;

        if i[0] != PSTR.len() as u8 {
            return Err(PeerConnectionError::new(format!(
                "Error reading handshake from peer {}:{}, wrong pstrlen",
                &self.peer.read()?.get_ip(),
                self.peer.read()?.get_port()
            )));
        }
        let pstr_len = i[0];

        let _r = self
            .clone()
            .read_n_bytes((pstr_len + RESERVED_SPACE_LEN) as usize)?;

        let info = self.clone().read_n_bytes(INFO_HASH_LEN)?;

        if info_hash != info {
            self.sender_logger.lock()?.send(LogMsg::Info(format!(
                "ta mal INFO_HASH_LEN: se espera:{:?} y se recibe:{:?} ",
                info_hash, info
            )))?;
            return Err(PeerConnectionError::new(format!(
                "Wrong info_hash in handshake from peer {}:{}",
                &self.peer.read()?.get_ip(),
                self.clone().peer.read()?.get_port()
            )));
        }
        let id = self.clone().read_n_bytes(PEER_ID_LEN)?;

        if self.peer.read()?.get_id() != "default_id" {
            if id != self.peer.read()?.get_id().as_bytes() {
                return Err(PeerConnectionError::new(format!(
                    "Wrong peer_id in handshake from peer {}:{}",
                    &self.peer.read()?.get_ip(),
                    self.peer.read()?.get_port()
                )));
            }
        } else {
            self.peer.write()?.set_id(String::from_utf8(id.to_vec())?);
            self.sender_logger
                .lock()?
                .send(LogMsg::Info("escribo al final".to_string()))?;
        }

        Ok(())
    }

    pub fn bitfield(self: Arc<Self>, bitfield: Vec<u8>) -> Result<(), PeerConnectionError> {
        let mut data = Vec::new();
        data.extend(u32_to_vecu8(&(bitfield.len() as u32 + 1)));
        data.extend(u32_to_vecu8(&(BITFIELD_ID as u32)));
        data.extend(bitfield);
        let _ = self.stream.lock()?.write_all(&data)?;
        Ok(())
    }

    pub fn read_bitfield(self: Arc<Self>, msg_len: usize) -> Result<(), PeerConnectionError> {
        let bitfield = self.clone().read_n_bytes(msg_len)?;
        self.peer.write()?.set_bitfield(bitfield);

        Ok(())
    }

    pub fn choke(self: Arc<Self>) -> Result<(), PeerConnectionError> {
        self.stream.lock()?.write_all(CHOKE_MESSAGE)?;
        self.peer.write()?.set_is_choked(true);
        Ok(())
    }

    pub fn read_choke(self: Arc<Self>) {
        self.peer.write().unwrap().set_choked_me(true);
    }

    pub fn unchoke(self: Arc<Self>) -> Result<(), PeerConnectionError> {
        self.stream.lock()?.write_all(UNCHOKE_MESSAGE)?;
        self.peer.write()?.set_is_choked(false);
        Ok(())
    }

    pub fn read_unchoke(self: Arc<Self>) {
        self.peer.write().unwrap().set_choked_me(false);
    }

    pub fn interested(self: Arc<Self>) -> Result<(), PeerConnectionError> {
        self.stream.lock()?.write_all(INTERESTED_MESSAGE)?;
        Ok(())
    }

    pub fn read_interested(self: Arc<Self>) -> Result<(), PeerConnectionError> {
        self.peer.write()?.set_interested_in_me(true);
        Ok(())
    }

    pub fn not_interested(self: Arc<Self>) -> Result<(), PeerConnectionError> {
        self.stream.lock()?.write_all(NOT_INTERESTED_MESSAGE)?;
        Ok(())
    }

    pub fn read_not_interested(self: Arc<Self>) -> Result<(), PeerConnectionError> {
        self.peer.write()?.set_interested_in_me(false);
        Ok(())
    }

    pub fn have(self: Arc<Self>, piece_index: u32) -> Result<(), PeerConnectionError> {
        let mut data = HAVE_MESSAGE.to_vec();
        data.extend(&piece_index.to_be_bytes());
        self.stream.lock()?.write_all(data.as_slice())?;
        Ok(())
    }

    pub fn read_have(self: Arc<Self>) -> Result<u32, PeerConnectionError> {
        let have_vector = self.clone().read_n_bytes(HAVE_LEN)?;
        let piece_idx = vecu8_to_u32(&have_vector);
        self.peer.write()?.add_piece(piece_idx);
        Ok(piece_idx)
    }

    pub fn cancel(
        self: Arc<Self>,
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

    pub fn read_cancel(self: Arc<Self>) -> Result<(u32, u32, u32), PeerConnectionError> {
        let cancel_vec = self.read_n_bytes(CANCEL_LEN)?;
        Ok((
            vecu8_to_u32(&cancel_vec[..3]),
            vecu8_to_u32(&cancel_vec[4..7]),
            vecu8_to_u32(&cancel_vec[8..11]),
        ))
    }

    pub fn request_chunk(
        self: Arc<Self>,
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

    pub fn read_request(self: Arc<Self>) -> Result<(), PeerConnectionError> {
        let request_vec = self.clone().read_n_bytes(PIECE_INDEX_LEN)?;
        let piece_idx = vecu8_to_u32(&request_vec);
        let offset_vec = self.clone().read_n_bytes(PIECE_OFFSET_LEN)?;
        let offset = vecu8_to_u32(&offset_vec);
        let length_vec = self.clone().read_n_bytes(CHUNK_LEN_LEN)?;
        let length = vecu8_to_u32(&length_vec);

        self.sender_logger.lock()?.send(LogMsg::Info(format!(
            "A peer requested piece index:{}, offset:{}, length:{}",
            piece_idx, offset, length
        )))?;

        self.sender_upload_manager.lock()?.send(Some(PieceRequest {
            piece_index: piece_idx,
            offset,
            length,
            stream: self.stream.clone(),
            peer_id: self.peer.read()?.get_id(),
        }))?;
        Ok(())
    }

    pub fn read_chunk(
        self: Arc<Self>,
        piece_idx: u32,
        offset: u32,
    ) -> Result<Vec<u8>, PeerConnectionError> {
        let index = self.clone().read_n_bytes(PIECE_INDEX_LEN)?;
        if vecu8_to_u32(&index) != piece_idx {
            return Err(PeerConnectionError::new(format!(
                "Peer {}:{} didn't respond with the right piece",
                &self.peer.read()?.get_ip(),
                self.peer.read()?.get_port()
            )));
        }

        let begin = self.clone().read_n_bytes(PIECE_OFFSET_LEN)?;
        if vecu8_to_u32(&begin) != offset {
            return Err(PeerConnectionError::new(format!(
                "Peer {}:{} didn't respond with the right chunk",
                &self.peer.read()?.get_ip(),
                self.peer.read()?.get_port()
            )));
        }

        let chunk = self.read_n_bytes(CHUNK_SIZE as usize)?;

        Ok(chunk)
    }

    pub fn read_port(self: Arc<Self>) -> Result<(), PeerConnectionError> {
        let _port_vec = self.read_n_bytes(PORT_LEN)?;
        Ok(())
    }
}

pub fn fmt_chunk(
    // format the chunk to be sent to the peer as a piece message
    piece_idx: u32,
    offset: u32,
    chunk: &Vec<u8>,
) -> Vec<u8> {
    let msg_len = (9 + chunk.len() as u32).to_be_bytes();
    let mut data: Vec<u8> = Vec::new();
    data.extend(msg_len);
    data.extend(PIECE_ID.to_be_bytes());
    data.extend(&piece_idx.to_be_bytes());
    data.extend(&offset.to_be_bytes());
    data.extend(chunk);
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::*;
    use crate::test_helper::MockTcpStream;
    use std::sync::mpsc::channel;

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
        let stream = MockTcpStream::new(handshake.clone());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection
            .handshake("esto_es_un_id_de_20_".to_string())
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
        let stream = MockTcpStream::new(handshake.clone());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));
        assert!(peer_connection
            .handshake("esto_es_un_id_de_20_".to_string())
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
        let stream = MockTcpStream::new(handshake.clone());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection
            .handshake("esto_es_un_id_de_20_".to_string())
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
        let stream = MockTcpStream::new(handshake.clone());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection
            .handshake("esto_es_un_id_de_20_".to_string())
            .is_err());
    }

    #[test]
    fn test_new_ok_handshake() {
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
        let stream = MockTcpStream::new(handshake.clone());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection
            .handshake("esto_es_un_id_de_20_".to_string())
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
        let stream = MockTcpStream::new(handshake.clone());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert_eq!(
            peer_connection.read_n_bytes(24).unwrap(),
            "\u{13}BitTorrent protocol\0\0\0\0".as_bytes()
        );
    }

    #[test]
    fn test_read_bitfield_ok() {
        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let bitfield = [0, 0, 0, 5, 5, 255, 255, 255, 128];

        let stream = MockTcpStream::new(bitfield.to_vec());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection.clone().read_detect_message().is_ok());
        assert_eq!(
            peer_connection.peer.read().unwrap().get_bitfield(),
            vec![255, 255, 255, 128]
        );
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
        handshake.extend([0, 0, 0, 5, 11, 255, 255, 255, 128]); // bitfield to be read
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let stream = MockTcpStream::new(handshake.clone());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection.read_detect_message().is_err());
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
        let stream = MockTcpStream::new(handshake.clone());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection.clone().unchoke().is_ok());
        assert_eq!(peer_connection.peer.read().unwrap().get_is_choked(), false);
    }

    // #[test]
    // fn test_interested() {
    //     // init peer connection
    //     let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
    //     let mut handshake = vec![PSTR.len() as u8];
    //     handshake.extend(PSTR.as_bytes());
    //     handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
    //     handshake.extend(&info_hash);
    //     handshake.extend("peer_id_123456789012".as_bytes());
    //     let peer = Peer::new(
    //         "peer_id_123456789012".to_string(),
    //         "1".to_string(),
    //         433 as u16,
    //     );
    //     let stream = MockTcpStream::new(handshake.clone());
    //     let (sender1, _) = channel();
    //     let (sender3, _) = channel();
    //     let peer_connection = Arc::new(PeerConnection::new(
    //         peer,
    //         info_hash,
    //         "client_id_1234567890".to_string(),
    //         Arc::new(Mutex::new(Box::new(stream.clone()))),
    //         Arc::new(Mutex::new(sender1)),
    //         Arc::new(Mutex::new(sender3)),
    //     ));

    //     assert!(peer_connection.interested().is_ok());
    //     assert_eq!(&stream.cursor_w.get_ref()[(73 - 5)..], INTERESTED_MESSAGE);
    // }

    // #[test]
    // fn test_have() {
    //     // init peer connection
    //     let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
    //     let mut handshake = vec![PSTR.len() as u8];
    //     handshake.extend(PSTR.as_bytes());
    //     handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
    //     handshake.extend(&info_hash);
    //     handshake.extend("peer_id_123456789012".as_bytes());
    //     let peer = Peer::new(
    //         "peer_id_123456789012".to_string(),
    //         "1".to_string(),
    //         433 as u16,
    //     );
    //     let stream = MockTcpStream::new(handshake.clone());
    //     let (sender1, _) = channel();
    //     let (sender3, _) = channel();
    //     let peer_connection = Arc::new(PeerConnection::new(
    //         peer,
    //         info_hash,
    //         "client_id_1234567890".to_string(),
    //         Arc::new(Mutex::new(Box::new(stream.clone()))),
    //         Arc::new(Mutex::new(sender1)),
    //         Arc::new(Mutex::new(sender3)),
    //     ));

    //     assert!(peer_connection.have(22).is_ok());

    //     let mut have_message = HAVE_MESSAGE.to_vec();
    //     have_message.extend([0, 0, 0, 22]); //idx

    //     assert_eq!(&stream.cursor_w.get_ref()[(73 - 5)..], have_message);
    // }

    // #[test]
    // fn test_cancel() {
    //     // init peer connection
    //     let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
    //     let mut cancel_message = CANCEL_MESSAGE.to_vec();
    //     cancel_message.extend([0, 0, 0, 2]); //idx
    //     cancel_message.extend([0, 0, 0, 1]); //begin
    //     cancel_message.extend([0, 0, 0, 33]); //length

    //     let mut handshake = vec![PSTR.len() as u8];
    //     handshake.extend(PSTR.as_bytes());
    //     handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
    //     handshake.extend(&info_hash);
    //     handshake.extend("peer_id_123456789012".as_bytes());

    //     let peer = Peer::new(
    //         "peer_id_123456789012".to_string(),
    //         "1".to_string(),
    //         433 as u16,
    //     );
    //     let stream = MockTcpStream::new(vec![]); // no quiero leerle nada
    //     let (sender1, _) = channel();
    //     let (sender3, _) = channel();

    //     let peer_connection = Arc::new(PeerConnection::new(
    //         peer,
    //         info_hash,
    //         "client_id_1234567890".to_string(),
    //         Arc::new(Mutex::new(Box::new(stream.clone()))),
    //         Arc::new(Mutex::new(sender1)),
    //         Arc::new(Mutex::new(sender3)),
    //     ));

    //     assert!(peer_connection.cancel(2, 1, 33).is_ok());
    //     assert_eq!(&stream.cursor_w.get_ref()[(73 - 5)..], cancel_message);
    // }

    #[test]
    fn test_read_unchoke_ok() {
        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let stream = MockTcpStream::new(UNCHOKE_MESSAGE.to_vec());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection.clone().read_detect_message().is_ok());
        assert_eq!(peer_connection.peer.read().unwrap().get_choked_me(), false);
    }

    // #[test]
    // fn test_request_chunk() {
    //     // init peer connection
    //     let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
    //     let mut handshake = vec![PSTR.len() as u8];
    //     handshake.extend(PSTR.as_bytes());
    //     handshake.extend(vec![0; RESERVED_SPACE_LEN as usize]);
    //     handshake.extend(&info_hash);
    //     handshake.extend("peer_id_123456789012".as_bytes());
    //     let peer = Peer::new(
    //         "peer_id_123456789012".to_string(),
    //         "1".to_string(),
    //         433 as u16,
    //     );
    //     let stream = MockTcpStream::new(handshake.clone());
    //     let (sender1, _) = channel();
    //     let (sender3, _) = channel();
    //     let peer_connection = Arc::new(PeerConnection::new(
    //         peer,
    //         info_hash,
    //         "client_id_1234567890".to_string(),
    //         Arc::new(Mutex::new(Box::new(stream.clone()))),
    //         Arc::new(Mutex::new(sender1)),
    //         Arc::new(Mutex::new(sender3)),
    //     ));

    //     let mut request_message = REQUEST_MESSAGE.to_vec();
    //     request_message.extend([0, 0, 0, 0]); // idx
    //     request_message.extend([0, 0, 0, 0]); // offset
    //     request_message.extend([0, 0, 64, 0]); // lenght

    //     assert!(peer_connection.request_chunk(0, 0, &(16384 as u32)).is_ok());

    //     assert_eq!(&stream.cursor_w.get_ref()[(85 - 17)..], request_message);
    // }

    // #[test]
    // fn test_read_chunk_ok() {
    //     let mut piece_message: Vec<u8> = [0, 0, 64, 9].to_vec(); // lenght = 9 + 3
    //     piece_message.extend([7]); // piece message id
    //     piece_message.extend([0, 0, 0, 0]); // index
    //     piece_message.extend([0, 0, 0, 0]); // offset
    //     piece_message.extend(vec![0; CHUNK_SIZE as usize - 9 - 3]); // chunk_size
    //     piece_message.extend([34, 58, 12]); // block

    //     // init peer connection
    //     let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
    //     let peer = Peer::new(
    //         "peer_id_123456789012".to_string(),
    //         "1".to_string(),
    //         433 as u16,
    //     );
    //     let stream = MockTcpStream::new(piece_message);
    //     let (sender1, _) = channel();
    //     let (sender3, _) = channel();
    //     let peer_connection = Arc::new(PeerConnection::new(
    //         peer,
    //         info_hash,
    //         "client_id_1234567890".to_string(),
    //         Arc::new(Mutex::new(Box::new(stream.clone()))),
    //         Arc::new(Mutex::new(sender1)),
    //         Arc::new(Mutex::new(sender3)),
    //     ));

    //     assert!(peer_connection.clone().read_detect_message().is_ok());
    //     let mut piece: Vec<u8> = vec![0; CHUNK_SIZE as usize - 3];
    //     piece.extend([34, 58, 12]);

    //     assert_eq!(peer_connection.read_chunk(0, 0).unwrap(), piece);
    // }

    #[test]
    fn test_read_chunk_wrong_message_id() {
        let mut piece_message: Vec<u8> = [0, 0, 64, 9].to_vec(); // lenght = 9 + 3
        piece_message.extend([12]); // wrong piece message id
        piece_message.extend([0, 0, 0, 0]); // index
        piece_message.extend([0, 0, 0, 0]); // offset
        piece_message.extend(vec![0; CHUNK_SIZE as usize - 9 - 3]); // chunk_size
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
        let stream = MockTcpStream::new(handshake.clone());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection.read_detect_message().is_err());
    }

    #[test]
    fn test_read_chunk_wrong_idx() {
        let mut piece_message: Vec<u8> = [0, 0, 64, 9].to_vec(); // lenght = 9 + 3
        piece_message.extend([7]); // piece message id
        piece_message.extend([0, 0, 0, 1]); // wrong index
        piece_message.extend([0, 0, 0, 0]); // offset
        piece_message.extend(vec![0; CHUNK_SIZE as usize - 9 - 3]); // chunk_size
        piece_message.extend([34, 58, 12]); // block

        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let stream = MockTcpStream::new(piece_message);
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection.clone().read_detect_message().is_ok());
        assert!(peer_connection.read_chunk(0, 0).is_err());
    }

    #[test]
    fn test_read_chunk_wrong_lenght() {
        let mut piece_message: Vec<u8> = [0, 0, 64, 8].to_vec(); // lenght = 9 + 3
        piece_message.extend([7]); // piece message id
        piece_message.extend([0, 0, 0, 0]); // index
        piece_message.extend([0, 0, 0, 0]); // offset
        piece_message.extend(vec![0; CHUNK_SIZE as usize - 9 - 3]); // chunk_size
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
        let stream = MockTcpStream::new(handshake.clone());
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection.read_detect_message().is_err());
    }

    #[test]
    fn test_read_chunk_wrong_offset() {
        let mut piece_message: Vec<u8> = [0, 0, 64, 9].to_vec(); // lenght = 9 + 3
        piece_message.extend([7]); // piece message id
        piece_message.extend([0, 0, 0, 0]); // index
        piece_message.extend([0, 0, 0, 1]); // offset
        piece_message.extend(vec![0; CHUNK_SIZE as usize - 9 - 3]); // chunk_size
        piece_message.extend([34, 58, 12]); // block

        // init peer connection
        let info_hash = "1abcabcaabcabcacbac1".as_bytes().to_vec();
        let peer = Peer::new(
            "peer_id_123456789012".to_string(),
            "1".to_string(),
            433 as u16,
        );
        let stream = MockTcpStream::new(piece_message);
        let (sender1, _) = channel();
        let (sender3, _) = channel();
        let peer_connection = Arc::new(PeerConnection::new(
            peer,
            info_hash,
            "client_id_1234567890".to_string(),
            Arc::new(Mutex::new(Box::new(stream.clone()))),
            Arc::new(Mutex::new(sender1)),
            Arc::new(Mutex::new(sender3)),
        ));

        assert!(peer_connection.clone().read_detect_message().is_ok());
        assert!(peer_connection.read_chunk(0, 0).is_err());
    }
}
