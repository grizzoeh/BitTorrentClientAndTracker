use crate::peer::{Peer, PeerConnection};
use crate::torrentparser::torrent_parse;
use crate::tracker::Tracker;
use std::collections::HashMap;
extern crate chrono;
use crate::utils::create_id;
use crate::utils::vecu8_to_u64;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Write;

const CHUNK_SIZE: u32 = 16384;
const MAX_PEERS: usize = 50;
const INITIAL_OFFSET: u32 = 0;
const PIECE_HASH_LEN: usize = 20;

pub struct Client {
    pub peers: Vec<Peer>,
    pub torrent_path: String,
    pub download_path: String,
    pub tracker: Tracker,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub event: String,
    pub id: String,
    pub port: u16,
    pub pieces: Vec<u8>,
    pub pieces_length: u64,
    pub length: u64,
}

#[derive(Debug)]
pub enum ClientError {
    TrackerError(String),
    TorrentError(String),
    DownloadError(String),
    TorrentParseFileNotFound(String),
    ParserError(String),
    PeerConnectionError(String),
    VerifyError(String),
}

impl Client {
    pub fn new(config: HashMap<String, String>) -> Result<Client, ClientError> {
        let id = create_id();
        //let log_path = config["log_path"].clone();
        let download_path = config["download_path"].clone();
        let torrent_path = config["torrent_path"].clone();
        let port = match config["port"].clone().parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                return Err(ClientError::ParserError(
                    "error during port parse".to_string(),
                ))
            }
        };

        let uploaded = 0;
        let downloaded = 0;
        let left = 0;
        let event = "started".to_string();

        let torrent_data = match torrent_parse(&config["torrent_path"]) {
            Ok(dic) => dic,
            Err(_) => {
                return Err(ClientError::TorrentParseFileNotFound(
                    "error during torrent parse".to_string(),
                ))
            }
        };

        let announce_url = match String::from_utf8(torrent_data["url"].clone()) {
            Ok(url) => url,
            Err(_) => {
                return Err(ClientError::ParserError(
                    "error during url parse".to_string(),
                ))
            }
        };

        let mut info = HashMap::new();
        info.insert(String::from("URL"), announce_url);
        info.insert(String::from("peer_id"), id.clone());
        info.insert(String::from("port"), format!("{}", port));
        info.insert(String::from("uploaded"), format!("{}", uploaded));
        info.insert(String::from("downloaded"), format!("{}", downloaded));
        info.insert(String::from("left"), format!("{}", left));
        info.insert(String::from("event"), event.clone());

        let tracker = match Tracker::new(info, torrent_data["info_hash"].clone()) {
            Ok(tracker) => tracker,
            Err(_) => {
                return Err(ClientError::TrackerError(
                    "error during tracker init".to_string(),
                ))
            }
        };

        Ok(Client {
            torrent_path,
            download_path,
            id,
            uploaded,
            downloaded,
            left,
            event,
            port,
            tracker,
            peers: vec![],
            pieces: torrent_data["pieces"].clone(),
            pieces_length: vecu8_to_u64(&torrent_data["piece length"]),
            length: vecu8_to_u64(&torrent_data["length"]) as u64,
        })
    }

    pub fn get_peer_by_id(&self, id: String) -> Option<&Peer> {
        self.peers.iter().find(|&p| p.id == id)
    }

    pub fn get_peer_by_ip(&self, ip: &str) -> Option<&Peer> {
        self.peers.iter().find(|&p| p.ip == ip)
    }

    pub fn start(&self) {
        for i in 0..MAX_PEERS {
            let peer = match self.tracker.initialize_peer(i) {
                Ok(peer) => peer,
                Err(_) => {
                    println!("error during peer initialization");
                    continue;
                }
            };

            let mut peer_connection = match self.try_peer_connection(&peer) {
                Ok(peer) => peer,
                Err(_) => {
                    println!("error during peer connection");
                    continue;
                }
            };
            println!("Peer connected");
            match peer_connection.read_bitfield() {
                Ok(_) => {
                    println!("READ BITFIELD SUCCESS");
                }
                Err(_) => {
                    println!("READ BITFIELD ERROR");
                }
            }

            match peer_connection.unchoke() {
                Ok(_) => {
                    println!("UNCHOKE OK");
                }
                Err(err) => {
                    println!("UNCHOKE ERROR: {:?}", err);
                    continue;
                }
            }

            match peer_connection.interested() {
                Ok(_) => {
                    println!("INTERESTED OK");
                }
                Err(err) => {
                    println!("INTERESTED ERROR: {:?}", err);
                    continue;
                }
            }

            match peer_connection.read_unchoke() {
                Ok(_) => {
                    println!("UNCHOKED OK");
                }
                Err(err) => {
                    println!("UNCHOKE ERROR: {:?}", err);
                    continue;
                }
            }
            let mut piece_idx: u32 = 0;
            for i in 0..(self.length / self.pieces_length) as u32 {
                if peer_connection.peer.has_piece(i) {
                    piece_idx = i;
                    break;
                }
            }
            let piece = match self.request_piece(piece_idx, &mut peer_connection) {
                Ok(data) => data,
                Err(e) => {
                    println!("ERROR: {:?}", e);
                    continue;
                }
            };

            // Save piece on file
            let path = format!("{}piece_{}.txt", &self.download_path, piece_idx);
            let mut file = match File::create(path) {
                Ok(file) => file,
                Err(e) => {
                    println!("ERROR: {:?}", e);
                    continue;
                }
            };

            match file.write(&piece) {
                Ok(_) => println!("PIECE SAVED SUCCESSFULLY"),
                Err(e) => {
                    println!("PIECE SAVE ERROR: {:?}", e);
                    continue;
                }
            }

            println!("--------------------------------------");
            return;
        }
    }

    fn try_peer_connection(&self, peer: &Peer) -> Result<PeerConnection, ClientError> {
        let peer = peer.clone();
        let peer_connection =
            match PeerConnection::new(peer, self.tracker.info_hash.clone(), self.id.clone()) {
                Ok(peer_connection) => peer_connection,
                Err(e) => {
                    println!("error during peer connection: {:?}", e);
                    return Err(ClientError::PeerConnectionError(
                        "error during peer connection".to_string(),
                    ));
                }
            };
        Ok(peer_connection)
    }

    fn request_piece(
        &self,
        piece_idx: u32,
        peer: &mut PeerConnection,
    ) -> Result<Vec<u8>, ClientError> {
        let piece_length = self.pieces_length as u32;
        let mut offset: u32 = INITIAL_OFFSET;
        let mut piece_data: Vec<u8> = vec![];

        while offset < piece_length {
            match peer.request(piece_idx, offset, &CHUNK_SIZE) {
                Ok(_) => {}
                Err(e) => {
                    return Err(ClientError::PeerConnectionError(format!(
                        "error sending chunk, ERROR: {:?}",
                        e
                    )));
                }
            }

            let chunk = match peer.read_chunk(piece_idx, offset, &CHUNK_SIZE) {
                Ok(data) => data,
                Err(e) => {
                    return Err(ClientError::PeerConnectionError(format!(
                        "error reading chunk, ERROR: {:?}",
                        e
                    )));
                }
            };

            piece_data.extend(chunk);
            offset += CHUNK_SIZE;
        }

        self.verify_piece(&piece_data, &piece_idx)?;
        Ok(piece_data)
    }

    fn verify_piece(&self, piece_data: &[u8], piece_idx: &u32) -> Result<(), ClientError> {
        let piece_idx = *piece_idx as usize;
        let mut hasher = Sha1::new();
        hasher.update(piece_data);
        let piece_hash = hasher.finalize().to_vec();
        if piece_hash
            != self.pieces[(piece_idx * PIECE_HASH_LEN)..((piece_idx + 1) * PIECE_HASH_LEN)]
                .to_vec()
        {
            return Err(ClientError::VerifyError(format!(
                "piece hash does not match, piece_idx: {}\nleft: {:?}\nright: {:?}",
                piece_idx,
                piece_hash,
                self.pieces[(piece_idx * PIECE_HASH_LEN)..((piece_idx + 1) * PIECE_HASH_LEN)]
                    .to_vec()
            )));
        }
        Ok(())
    }
}
