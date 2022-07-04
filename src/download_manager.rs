use crate::constants::*;
use crate::errors::download_manager_error::DownloadManagerError;
use crate::errors::peer_connection_error::PeerConnectionError;
use crate::logger::LogMsg;
use crate::peer::{Peer, PeerInterface};
use crate::peer_connection::PeerConnection;
use crate::threadpool::ThreadPool;
use crate::ui_codes::*;
use crate::upload_manager::PieceRequest;
use crate::utils::UiParams;
use chrono::offset::Utc;
use chrono::DateTime;
use glib::Sender as UISender;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::spawn;
use std::time::SystemTime;

#[allow(clippy::type_complexity)]
pub struct DownloadManager {
    info: Arc<RwLock<DownloaderInfo>>,
    pub bitfield: Arc<RwLock<Vec<PieceStatus>>>,
    pieces_left: Arc<RwLock<Vec<PieceInfo>>>,
    _pieces_quantity: usize,
    pieces_receiver: Arc<Mutex<Receiver<PieceInfo>>>,
    pieces_sender: Arc<Mutex<Sender<PieceInfo>>>,
    logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    sender_client: Arc<Mutex<UISender<Vec<(usize, UiParams, String)>>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceStatus {
    NotDownloaded,
    Downloading,
    Downloaded,
    End,
}

#[derive(Debug, Clone)]
pub(crate) struct PieceInfo {
    pub piece_index: usize,
    pub piece_status: PieceStatus,
    pub piece_data: Vec<u8>,
}

#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub struct DownloaderInfo {
    pub length: u64,
    pub piece_length: u64,
    pub download_path: String,
    pub logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    pub pieces_hash: Vec<u8>,
    pub peers: Arc<RwLock<Vec<Arc<PeerConnection<Peer>>>>>,
    pub info_hash: Vec<u8>,
    pub client_id: String,
    pub upload_sender: Arc<Mutex<Sender<Option<PieceRequest>>>>,
    pub torrent_name: String,
    pub file_length: u64,
    pub ui_sender: Arc<Mutex<glib::Sender<Vec<(usize, UiParams, String)>>>>,
}

impl DownloadManager {
    pub fn new(info: DownloaderInfo) -> Result<Arc<DownloadManager>, DownloadManagerError> {
        let pieces_quantity = (info.file_length / info.piece_length) as usize;
        info.ui_sender.lock()?.send(vec![(
            GET_PIECES_QUANTITY,
            UiParams::Integer(pieces_quantity as i64),
            info.torrent_name.clone(),
        )])?;
        println!("Reading disk, please wait...");
        info.logger_sender
            .lock()?
            .send(LogMsg::Info("Reading disk, please wait...".to_string()))?;
        let bitfield = build_bitfield(pieces_quantity, info.download_path.clone());
        let pieces_left = get_pieces_left(bitfield.clone());

        let current_downloaded_pieces = pieces_quantity - pieces_left.len();
        info.ui_sender.lock()?.send(vec![(
            UPDATE_INITIAL_DOWNLOADED_PIECES,
            UiParams::Integer(current_downloaded_pieces as i64),
            info.torrent_name.clone(),
        )])?;

        let downloader_info = Arc::new(RwLock::new(info.clone()));
        let (pieces_sender, pieces_receiver) = channel();
        Ok(Arc::new(DownloadManager {
            info: downloader_info,
            bitfield,
            pieces_left: Arc::new(RwLock::new(pieces_left)),
            _pieces_quantity: pieces_quantity,
            pieces_receiver: Arc::new(Mutex::new(pieces_receiver)),
            pieces_sender: Arc::new(Mutex::new(pieces_sender)),
            logger_sender: info.logger_sender.clone(),
            sender_client: info.ui_sender,
        }))
    }

    pub fn start_download(self: Arc<Self>) -> Result<(), DownloadManagerError> {
        if self.pieces_left.read().unwrap().is_empty() {
            self.logger_sender.lock()?.send(LogMsg::End)?;
            self.info.read()?.upload_sender.lock()?.send(None)?;
            println!("No pieces left to download, exiting...");
            return Ok(());
        }
        println!("DOWNLOADING STARTED");
        let mut current_time = chrono::Utc::now();
        self.logger_sender.lock()?.send(LogMsg::Info(format!(
            "DOWNLOADING STARTED: {}",
            current_time
        )))?;
        let peers_quantity = self.info.read()?.peers.read()?.len();
        let threadpool = ThreadPool::new(peers_quantity);
        let self_copy = self.clone();
        let _piece_listener_handler = spawn(move || self_copy.listen_for_pieces());

        for peer in self.info.read()?.peers.read()?.iter() {
            let peer_copy = peer.clone();
            let self_copy = self.clone();
            if self.clone().try_peer_connection(peer.clone()).is_ok() {
                let _rt = threadpool.execute(move || {
                    let _r = self_copy.download_pieces(peer_copy);
                });
            };
        }

        self.pieces_sender.lock()?.send(PieceInfo {
            // exit the thread when all pieces are downloaded
            piece_index: 0,
            piece_status: PieceStatus::End,
            piece_data: vec![],
        })?;
        if self.pieces_left.read().unwrap().is_empty() {
            self.logger_sender.lock()?.send(LogMsg::End)?;
            self.info.read()?.upload_sender.lock()?.send(None)?;
            println!("No pieces left to download, exiting...");
            return Ok(());
        }
        current_time = chrono::Utc::now();
        self.logger_sender.lock()?.send(LogMsg::Info(format!(
            "DOWNLOADING FINISHED: {}",
            current_time
        )))?;
        match _piece_listener_handler.join() {
            Ok(_) => Ok(()),
            Err(e) => Err(DownloadManagerError::from(e)),
        }
    }

    // this function waits for pieces and stores them in the disk, it is intended to run in a separate thread
    fn listen_for_pieces(self: Arc<Self>) -> Result<(), DownloadManagerError> {
        let pieces_receiver = self.pieces_receiver.lock()?;
        loop {
            let piece = pieces_receiver.recv()?;
            if piece.piece_status == PieceStatus::End {
                return Ok(());
            }
            let mut file = File::create(format!(
                "{}/piece_{}.txt",
                self.info.read()?.download_path,
                piece.piece_index
            ))?;
            file.write_all(&piece.piece_data)?;
            file.flush()?;
        }
    }

    fn download_pieces(
        self: Arc<Self>,
        peer_connection: Arc<PeerConnection<Peer>>,
    ) -> Result<(), DownloadManagerError> {
        println!("PEER CONNECTION ESTABLISHED");
        self.info
            .read()?
            .logger_sender
            .lock()?
            .send(LogMsg::Info("PEER CONNECTION ESTABLISHED".to_string()))?;
        self.sender_client.lock()?.send(vec![(
            UPDATE_ACTIVE_CONNS,
            UiParams::Usize(1),
            self.info.read()?.torrent_name.clone(),
        )])?;
        self.sender_client.lock()?.send(vec![(
            UPDATE_PEER_ID_IP_PORT,
            UiParams::Vector(vec![
                peer_connection.peer.read()?.id.clone(),
                peer_connection.peer.read()?.ip.clone(),
                format!("{}", peer_connection.peer.read()?.port.clone()),
            ]),
            self.info.read()?.torrent_name.clone(),
        )])?;

        //start iterating until unchoked received
        let mut message_type = ERROR_ID;
        while message_type != UNCHOKE_ID {
            // If we receive a "have" message instead of a piece message
            message_type = match peer_connection.clone().read_detect_message() {
                Ok(message) => message,
                Err(e) => {
                    self.sender_client.lock()?.send(vec![(
                        DELETE_ONE_ACTIVE_CONNECTION,
                        UiParams::Usize(1),
                        self.info.read()?.torrent_name.clone(),
                    )])?;
                    return Err(DownloadManagerError::from(e));
                }
            };
            if message_type == BITFIELD_ID {
                peer_connection.clone().unchoke()?;
                peer_connection.clone().interested()?;
                self.sender_client.lock()?.send(vec![(
                    UPDATE_INTERESTED,
                    UiParams::Vector(vec![
                        peer_connection.peer.read()?.id.clone(),
                        "Interested".to_string(),
                    ]),
                    self.info.read()?.torrent_name.clone(),
                )])?;
            }
        }

        self.sender_client.lock()?.send(vec![(
            UPDATE_UNCHOKE,
            UiParams::Vector(vec![
                peer_connection.peer.read()?.id.clone(),
                "Unchoked".to_string(),
            ]),
            self.info.read()?.torrent_name.clone(),
        )])?;

        // decide which pieces to download

        loop {
            let mut piece_indices = Vec::new();
            let mut pieces_to_download: Vec<PieceInfo> = Vec::new();
            while piece_indices.len() < 10 {
                let length = self.pieces_left.read()?.len();
                if length == 0 {
                    println!("Ended Job Successfully");
                    self.info
                        .read()?
                        .logger_sender
                        .lock()?
                        .send(LogMsg::Info("Ended Job Successfully".to_string()))?;
                    self.sender_client.lock()?.send(vec![(
                        DELETE_ONE_ACTIVE_CONNECTION,
                        UiParams::Usize(1),
                        self.info.read()?.torrent_name.clone(),
                    )])?;
                    return Ok(());
                }

                let index = self.pieces_left.read()?[length - 2].piece_index;

                if peer_connection.clone().peer.read()?.has_piece(index as u32) {
                    match self.pieces_left.write()?.pop() {
                        Some(piece) => {
                            piece_indices.push(piece.piece_index);
                            pieces_to_download.push(piece);
                        }
                        None => {
                            println!("Ended Job Successfully");
                            self.info
                                .read()?
                                .logger_sender
                                .lock()?
                                .send(LogMsg::Info("Ended Job Successfully".to_string()))?;
                            self.sender_client.lock()?.send(vec![(
                                DELETE_ONE_ACTIVE_CONNECTION,
                                UiParams::Usize(1),
                                self.info.read()?.torrent_name.clone(),
                            )])?;
                            return Ok(());
                        }
                    }
                }
            }
            println!("PIECES TO DOWNLOAD: {:?}", piece_indices);
            self.info.read()?.logger_sender.lock()?.send(LogMsg::Info(
                format!("PIECES TO DOWNLOAD: {:?}", piece_indices).to_string(),
            ))?;

            let system_time = SystemTime::now();
            let datetime: DateTime<Utc> = system_time.into();
            let timestamp = datetime.timestamp();

            for piece in pieces_to_download {
                let index = piece.piece_index;
                if let Ok(piece_data) = self
                    .clone()
                    .request_piece(index as u32, peer_connection.clone())
                {
                    //ui

                    let system_time2 = SystemTime::now();
                    let datetime2: DateTime<Utc> = system_time2.into();
                    let timestamp2 = datetime2.timestamp();
                    let download_speed = 16384 / (timestamp2 - timestamp);
                    self.sender_client.lock()?.send(vec![(
                        UPDATE_DOWNSPEED,
                        UiParams::Vector(vec![
                            peer_connection.peer.read()?.id.clone(),
                            format!("{} bytes / sec", download_speed),
                        ]),
                        self.info.read()?.torrent_name.clone(),
                    )])?;
                    self.pieces_sender.lock()?.send(PieceInfo {
                        piece_index: index,
                        piece_status: PieceStatus::Downloaded,
                        piece_data,
                    })?;
                    // drop(piece_data); // FIXME sirve esto pa bajar la ram?
                    self.bitfield.write()?[index as usize] = PieceStatus::Downloaded;
                    println!("piece {} downloaded", index);
                    let _ = self
                        .info
                        .read()?
                        .logger_sender
                        .lock()?
                        .send(LogMsg::Info(format!("piece {} downloaded", index,)));
                    self.info.read()?.logger_sender.lock()?.send(LogMsg::Info(
                        format!("piece {} downloaded ", index).to_string(),
                    ))?;

                    self.sender_client.lock()?.send(vec![(
                        UPDATE_DOWNLOADED_PIECES,
                        UiParams::U64(1),
                        self.info.read()?.clone().torrent_name,
                    )])?
                } else {
                    self.clone().clean_bitfield_at(index)?;
                    let piece = PieceInfo {
                        piece_index: index,
                        piece_status: PieceStatus::NotDownloaded,
                        piece_data: Vec::new(),
                    };
                    self.pieces_left.write()?.push(piece);
                    return Err(DownloadManagerError::new(
                        "Error requesting piece".to_string(),
                    ));
                }
            }
        }
    }

    fn request_piece(
        self: Arc<Self>,
        piece_idx: u32,
        peer: Arc<PeerConnection<Peer>>,
    ) -> Result<Vec<u8>, DownloadManagerError> {
        let piece_length = self.info.read()?.piece_length as u32;
        let mut offset: u32 = INITIAL_OFFSET;
        let mut piece_data: Vec<u8> = vec![];

        while offset < piece_length {
            peer.clone().request_chunk(piece_idx, offset, &CHUNK_SIZE)?;
            let mut message_type = ERROR_ID;

            while message_type != PIECE_ID {
                // in case we receive a "have" message instead of a piece message
                message_type = peer.clone().read_detect_message()?;
            }
            let chunk = peer.clone().read_chunk(piece_idx, offset)?;
            piece_data.extend(chunk);
            offset += CHUNK_SIZE;
        }
        verify_piece(&self.info.read()?.pieces_hash, &piece_data, &piece_idx)?;
        self.sender_client.lock()?.send(vec![(
            UPDATE_VERIFIED_PIECES,
            UiParams::Usize(1),
            self.info.read()?.torrent_name.clone(),
        )])?;
        Ok(piece_data)
    }

    pub fn try_peer_connection(
        self: Arc<Self>,
        peer: Arc<PeerConnection<Peer>>,
    ) -> Result<(), PeerConnectionError> {
        let peer_id = self.info.read()?.client_id.clone();
        peer.handshake(peer_id)
    }

    fn clean_bitfield_at(self: Arc<Self>, piece_index: usize) -> Result<(), DownloadManagerError> {
        self.bitfield.write()?[piece_index as usize] = PieceStatus::NotDownloaded;
        Ok(())
    }

    pub fn get_bitfield(self: Arc<Self>) -> Arc<RwLock<Vec<PieceStatus>>> {
        self.bitfield.clone()
    }
}

fn verify_piece(
    pieces: &[u8],
    piece_data: &[u8],
    piece_idx: &u32,
) -> Result<(), DownloadManagerError> {
    let piece_idx = *piece_idx as usize;
    let mut hasher = Sha1::new();
    hasher.update(piece_data);
    let piece_hash = hasher.finalize().to_vec();
    if piece_hash
        != pieces[(piece_idx * PIECE_HASH_LEN)..((piece_idx + 1) * PIECE_HASH_LEN)].to_vec()
    {
        return Err(DownloadManagerError::new(format!(
            "piece hash does not match, piece_idx: {}\nleft: {:?}\nright: {:?}",
            piece_idx,
            piece_hash,
            pieces[(piece_idx * PIECE_HASH_LEN)..((piece_idx + 1) * PIECE_HASH_LEN)].to_vec()
        )));
    }
    Ok(())
}

fn build_bitfield(pieces_quantity: usize, download_path: String) -> Arc<RwLock<Vec<PieceStatus>>> {
    let mut bitfield = vec![PieceStatus::NotDownloaded; pieces_quantity];
    if !std::path::Path::new(&download_path).exists() {
        std::fs::create_dir_all(&download_path).unwrap(); // para que no rompa, porlas
    }
    for entry in std::fs::read_dir(download_path).unwrap().flatten() {
        if !entry.path().is_dir() {
            let piece_id: usize = entry
                .file_name()
                .to_str()
                .unwrap()
                .split('_')
                .collect::<Vec<&str>>()[1]
                .split('.')
                .collect::<Vec<&str>>()[0]
                .parse()
                .unwrap();
            bitfield[piece_id] = PieceStatus::Downloaded;
        }
    }
    Arc::new(RwLock::new(bitfield))
}

fn get_pieces_left(bitfield: Arc<RwLock<Vec<PieceStatus>>>) -> Vec<PieceInfo> {
    let mut pieces_left = vec![];
    let bitfield = bitfield.read().unwrap();
    for (i, piece_status) in bitfield.iter().enumerate() {
        if *piece_status == PieceStatus::NotDownloaded {
            pieces_left.push(PieceInfo {
                piece_index: i as usize,
                piece_status: PieceStatus::NotDownloaded,
                piece_data: vec![],
            });
        }
    }
    pieces_left
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_verify_piece_with_wrong_piece() {
        let mut hasher = Sha1::new();
        hasher.update(
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
            .as_ref(),
        );
        let hash1 = hasher.finalize()[..].to_vec();

        let mut hasher = Sha1::new();
        hasher.update([
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        ]);
        let hash2 = hasher.finalize()[..].to_vec();

        let mut pieces = hash1.clone();
        pieces.extend(hash2.clone());

        let piece_data = [
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 200,
        ]; // wrong piece
        let piece_idx = 1;
        assert!(verify_piece(&pieces, &piece_data, &piece_idx).is_err());
    }

    #[test]
    fn test_verify_right_piece() {
        let mut hasher = Sha1::new();
        hasher.update(
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
            .as_ref(),
        );
        let hash1 = hasher.finalize()[..].to_vec();

        let mut hasher = Sha1::new();
        hasher.update([
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        ]);
        let hash2 = hasher.finalize()[..].to_vec();

        let mut pieces = hash1.clone();
        pieces.extend(hash2.clone());

        let piece_data = [
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        ];
        let piece_idx = 1;
        assert!(verify_piece(&pieces, &piece_data, &piece_idx).is_ok());
    }
}
