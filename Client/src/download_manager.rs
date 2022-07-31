use crate::{
    errors::download_manager_error::DownloadManagerError,
    errors::peer_connection_error::PeerConnectionError,
    logger::LogMsg,
    peer_entities::peer::{Peer, PeerInterface},
    peer_entities::peer_connection::PeerConnection,
    ui::ui_codes::*,
    upload_manager::PieceRequest,
    utilities::constants::*,
    utilities::file_assembler::assemble,
    utilities::utils::UiParams,
};
use chrono::{offset::Utc, DateTime};
use core::hash::Hash;
use glib::Sender as UISender;
use sha1::{Digest, Sha1};
use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::mpsc::{channel, Receiver, Sender},
    sync::{Arc, Mutex, RwLock},
    thread::{self, spawn},
    time::Duration,
    time::SystemTime,
};

/// The DownloadManager is responsible for downloading pieces from other peers.
#[allow(clippy::type_complexity)]
pub struct DownloadManager {
    info: Arc<RwLock<DownloaderInfo>>,
    pub bitfield: Arc<Vec<Mutex<PieceStatus>>>,
    pieces_quantity: usize,
    pieces_receiver: Arc<Mutex<Receiver<PieceInfo>>>,
    pieces_sender: Arc<Mutex<Sender<PieceInfo>>>,
    logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    sender_client: Arc<Mutex<UISender<Vec<(usize, UiParams, String)>>>>,
    active_threads_quantity: Arc<Mutex<usize>>,
    threads_handles: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
}

/// The enum PieceStatus represents the status of a piece that we want to download.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceStatus {
    NotDownloaded,
    Downloading,
    Downloaded,
    End,
}

/// The struct PieceInfo contains the information needed to download the corresponding piece.
#[derive(Debug, Clone, Eq)]
pub struct PieceInfo {
    pub piece_index: usize,
    pub piece_status: PieceStatus,
    pub piece_data: Vec<u8>,
}

impl PartialEq for PieceInfo {
    fn eq(&self, other: &Self) -> bool {
        self.piece_index == other.piece_index
    }
}

impl Hash for PieceInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.piece_index.hash(state);
    }
}

/// This struct is used to give information to the download manager.
#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub struct DownloaderInfo {
    pub piece_length: u64,
    pub download_path: String,
    pub download_pieces_path: String,
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
    /// Creates a download manager and the corresponding bitfield.
    pub fn new(info: DownloaderInfo) -> Result<Arc<DownloadManager>, DownloadManagerError> {
        let pieces_quantity = (info.file_length / info.piece_length) as usize;
        info.ui_sender.lock()?.send(vec![(
            GET_PIECES_QUANTITY,
            UiParams::Integer(pieces_quantity as i64),
            info.torrent_name.clone(),
        )])?;
        info.logger_sender
            .lock()?
            .send(LogMsg::Info("Reading disk, please wait...".to_string()))?;
        let bitfield = build_bitfield(pieces_quantity, info.download_pieces_path.clone());

        let current_downloaded_pieces = bitfield
            .iter()
            .filter(|x| PieceStatus::Downloaded == x.lock().unwrap().to_owned())
            .count();
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
            pieces_quantity,
            pieces_receiver: Arc::new(Mutex::new(pieces_receiver)),
            pieces_sender: Arc::new(Mutex::new(pieces_sender)),
            logger_sender: info.logger_sender.clone(),
            sender_client: info.ui_sender,
            active_threads_quantity: Arc::new(Mutex::new(0)),
            threads_handles: Arc::new(Mutex::new(Vec::new())),
        }))
    }

    /// Starts the download process. Once the download is finished, assembles the downloaded pieces into a file.
    pub fn start_download(self: Arc<Self>) -> Result<(), DownloadManagerError> {
        let mut pretty_torrent_name = self.info.read()?.torrent_name.clone();
        pretty_torrent_name = pretty_torrent_name.split('/').last().unwrap().to_string();
        pretty_torrent_name = pretty_torrent_name.rsplit_once('.').unwrap().0.to_string();
        // checks if file exists, if it does exits
        let assembled_file_path = format!(
            "{}/{}",
            self.info.read()?.download_path,
            pretty_torrent_name
        );
        if Path::new(&assembled_file_path).exists() {
            self.logger_sender.lock()?.send(LogMsg::Info(format!(
                "File {} is already downloaded, exiting...",
                assembled_file_path
            )))?;
            self.info.read()?.upload_sender.lock()?.send(None)?;
            self.logger_sender.lock()?.send(LogMsg::End)?;
            return Ok(());
        }

        if self
            .bitfield
            .iter()
            .all(|x| PieceStatus::Downloaded == x.lock().unwrap().to_owned())
        {
            self.info.read()?.upload_sender.lock()?.send(None)?;
            self.logger_sender.lock()?.send(LogMsg::End)?;
            self.logger_sender.lock()?.send(LogMsg::Info(format!(
                "All pieces downloaded, assembling file {}...",
                pretty_torrent_name
            )))?;
            assemble(
                self.info.read()?.download_pieces_path.clone(),
                assembled_file_path.clone(),
                self.pieces_quantity,
                self.info.read()?.piece_length as usize,
            )?;
            self.logger_sender.lock()?.send(LogMsg::Info(format!(
                "File {} assembled, exiting...",
                assembled_file_path
            )))?;
            self.info.read()?.upload_sender.lock()?.send(None)?;
            self.logger_sender.lock()?.send(LogMsg::End)?;
            return Ok(());
        }
        let mut current_time = chrono::Utc::now();
        self.logger_sender.lock()?.send(LogMsg::Info(format!(
            "DOWNLOADING STARTED: {}",
            current_time
        )))?;

        let self_copy = self.clone();
        self.threads_handles.lock()?.push(spawn(move || {
            let _r = self_copy.listen_for_pieces();
        }));

        // Init the threads to download pieces
        let _r = self.clone().init_peers_connnections(50);
        thread::sleep(Duration::from_secs(3));
        self.logger_sender.lock()?.send(LogMsg::Info(
            "Jobs spawned, waiting for downloads to end...".to_string(),
        ))?;
        self.logger_sender.lock()?.send(LogMsg::Info(format!(
            "PIECES LEFT: {}",
            self.pieces_quantity
                - self
                    .bitfield
                    .iter()
                    .filter(|x| PieceStatus::Downloaded == x.lock().unwrap().to_owned())
                    .count()
        )))?;
        while !self
            .bitfield
            .iter()
            .all(|x| PieceStatus::Downloaded == x.lock().unwrap().to_owned())
        {
            if self.active_threads_quantity.lock()?.to_owned() == 0 {
                let _r = self.clone().init_peers_connnections(1);
            }
            thread::sleep(Duration::from_secs(10));
        }
        self.logger_sender.lock()?.send(LogMsg::Info(
            "Finished waiting for downloads to end :)".to_string(),
        ))?;
        self.info.read()?.upload_sender.lock()?.send(None)?;
        self.pieces_sender.lock()?.send(PieceInfo {
            // Exit the thread when all pieces are downloaded
            piece_index: 0,
            piece_status: PieceStatus::End,
            piece_data: vec![],
        })?;
        current_time = chrono::Utc::now();
        self.logger_sender.lock()?.send(LogMsg::Info(format!(
            "DOWNLOADING FINISHED: {}",
            current_time
        )))?;

        // joins all threads
        let mut handles = self.threads_handles.lock()?;
        while handles.len() > 0 {
            match handles.pop() {
                Some(handle) => handle.join()?,
                None => break,
            };
        }

        self.logger_sender
            .lock()?
            .send(LogMsg::Info("Assembling file...".to_string()))?;
        match assemble(
            self.info.read()?.download_pieces_path.clone(),
            assembled_file_path.clone(),
            self.pieces_quantity,
            self.info.read()?.piece_length as usize,
        ) {
            Ok(_) => {
                self.logger_sender
                    .lock()?
                    .send(LogMsg::Info("File assembled successfully...".to_string()))?;
            }
            Err(e) => {
                self.logger_sender
                    .lock()?
                    .send(LogMsg::Info("Error assembling the file".to_string()))?;
                self.logger_sender.lock()?.send(LogMsg::End)?;
                return Err(DownloadManagerError::new(format!(
                    "Error assembling the file:{}",
                    e
                )));
            }
        }
        self.logger_sender
            .lock()?
            .send(LogMsg::Info("Verifying file integrity...".to_string()))?;
        // check if the file is valid
        match verify_assembled_file(
            &self.info.read()?.pieces_hash,
            assembled_file_path,
            self.info.read()?.piece_length as usize,
            self.pieces_quantity,
        ) {
            Ok(_) => {
                self.logger_sender
                    .lock()?
                    .send(LogMsg::Info("File verified successfully...".to_string()))?;
                self.logger_sender.lock()?.send(LogMsg::End)?;
                Ok(())
            }
            Err(e) => {
                self.logger_sender.lock()?.send(LogMsg::Info(format!(
                    "Error verifying the file, error:{}",
                    e
                )))?;
                self.logger_sender.lock()?.send(LogMsg::End)?;
                Err(DownloadManagerError::new(
                    "Error verifying the file".to_string(),
                ))
            }
        }
    }

    /// Initializes the struct peer connection for each peer.
    fn init_peers_connnections(
        self: Arc<Self>,
        job_quantity: usize,
    ) -> Result<usize, DownloadManagerError> {
        let mut job_counter: usize = 0;
        let mut job_quantity = job_quantity;
        let peers_quantity = self.info.read()?.peers.read()?.iter().count();
        if peers_quantity < job_quantity {
            job_quantity = peers_quantity;
        }
        let info = self.info.read()?;
        let peers = info.peers.clone();
        for peer in peers.read()?.iter() {
            let peer_copy = peer.clone();
            let self_copy = self.clone();
            // exits if there are no more pieces to download or if there are enough jobs spawned
            if job_counter >= job_quantity
                || !self
                    .bitfield
                    .iter()
                    .any(|x| PieceStatus::NotDownloaded == x.lock().unwrap().to_owned())
            {
                break;
            }

            if self.clone().try_peer_connection(peer.clone()).is_ok() {
                self.threads_handles.lock()?.push(spawn(move || {
                    let _r = self_copy.download_pieces(peer_copy);
                }));
                job_counter += 1;
            } else {
                self.logger_sender.lock()?.send(LogMsg::Info(format!(
                    "try peer connection failed, peer:{}",
                    peer.peer.read()?.ip
                )))?;
            }
        }
        Ok(job_counter)
    }

    /// It waits for pieces and stores them in the disk, it is intended to run in a separate thread.
    fn listen_for_pieces(self: Arc<Self>) -> Result<(), DownloadManagerError> {
        let pieces_receiver = self.pieces_receiver.lock()?;
        loop {
            let piece = pieces_receiver.recv()?;
            if piece.piece_status == PieceStatus::End {
                return Ok(());
            }
            let mut file = File::create(format!(
                "{}/piece_{}.txt",
                self.info.read()?.download_pieces_path,
                piece.piece_index
            ))?;
            file.write_all(&piece.piece_data)?;
            file.flush()?;
        }
    }

    /// Starts with the exchange of messagges until unchoke is detected, then start to request pieces by checking the bitfield.
    /// Also send messages to UI to update data in real time.
    fn download_pieces(
        self: Arc<Self>,
        peer_connection: Arc<PeerConnection<Peer>>,
    ) -> Result<(), DownloadManagerError> {
        let mut active_threads = self.active_threads_quantity.lock()?;
        *active_threads += 1;
        drop(active_threads);

        self.info
            .read()?
            .logger_sender
            .lock()?
            .send(LogMsg::Info(format!(
                "PEER CONNECTION ESTABLISHED WITH PEER {:?}",
                peer_connection.peer.read()?.ip
            )))?;
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

        // Start iterating until unchoked received
        let mut message_type = ERROR_ID;

        while message_type != UNCHOKE_ID {
            // If we receive a "have" message instead of a piece message
            message_type = match peer_connection.clone().read_detect_message() {
                Ok(message) => message,
                Err(e) => {
                    self.sender_client.lock()?.send(vec![(
                        DELETE_ONE_ACTIVE_CONNECTION,
                        UiParams::Vector(vec![
                            peer_connection.peer.read()?.id.clone(),
                            "Disconnected".to_string(),
                        ]),
                        self.info.read()?.torrent_name.clone(),
                    )])?;
                    self.logger_sender.lock()?.send(LogMsg::Info(format!(
                        "SUBSTRACTING THREAD, PEER:{}, ERROR:{}",
                        peer_connection.peer.read()?.ip,
                        e
                    )))?;
                    let mut active_threads = self.active_threads_quantity.lock()?;
                    *active_threads -= 1;
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

            if message_type == CHOKE_ID {
                self.logger_sender.lock()?.send(LogMsg::Info(format!(
                    "PEER {} CHOKED US, BEFORE SENDING PIECE REQUEST",
                    peer_connection.peer.read()?.ip
                )))?;
                self.sender_client.lock()?.send(vec![(
                    DELETE_ONE_ACTIVE_CONNECTION,
                    UiParams::Vector(vec![
                        peer_connection.peer.read()?.id.clone(),
                        "Disconnected".to_string(),
                    ]),
                    self.info.read()?.torrent_name.clone(),
                )])?;
                let mut active_threads = self.active_threads_quantity.lock()?;
                *active_threads -= 1;
                return Err(DownloadManagerError::new("Peer choked us".to_string()));
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

        // Start downloading pieces
        loop {
            let (pieces_indexes, mut pieces_to_download) = self
                .clone()
                .select_pieces_to_download(peer_connection.clone())?;

            if pieces_indexes.is_empty() || pieces_to_download.is_empty() {
                let mut active_threads = self.active_threads_quantity.lock()?;
                *active_threads -= 1;
                return Ok(());
            }

            self.clone().attempt_download_pieces(
                pieces_indexes,
                &mut pieces_to_download,
                peer_connection.clone(),
            )?;
        }
    }

    /// Selects the pieces to download from the bitfield of the peer.
    fn select_pieces_to_download(
        self: Arc<Self>,
        peer_connection: Arc<PeerConnection<Peer>>,
    ) -> Result<(Vec<usize>, Vec<PieceInfo>), DownloadManagerError> {
        let mut pieces_indexes: Vec<usize> = Vec::new();
        let mut pieces_to_download: Vec<PieceInfo> = Vec::new();
        let quantity_not_downloaded = self
            .bitfield
            .iter()
            .filter(|x| PieceStatus::NotDownloaded == x.lock().unwrap().to_owned())
            .count();
        let quantity_to_download = if quantity_not_downloaded < MAX_PIECES_TO_DOWNLOAD {
            quantity_not_downloaded
        } else {
            MAX_PIECES_TO_DOWNLOAD
        };

        if quantity_not_downloaded == 0 {
            self.sender_client.lock()?.send(vec![(
                DELETE_ONE_ACTIVE_CONNECTION,
                UiParams::Vector(vec![
                    peer_connection.peer.read()?.id.clone(),
                    "Disconnected".to_string(),
                ]),
                self.info.read()?.torrent_name.clone(),
            )])?;
            return Ok((pieces_indexes, pieces_to_download));
        }

        let peer_bitfield = peer_connection.peer.read()?.get_bitfield().clone();
        for (i, piece) in self.bitfield.iter().enumerate() {
            if !peer_bitfield.contains(&(i as u32)) {
                continue;
            }
            match piece.try_lock() {
                Ok(mut piece_lock) => {
                    if let PieceStatus::NotDownloaded = piece_lock.to_owned() {
                        pieces_indexes.push(i);
                        pieces_to_download.push(PieceInfo {
                            piece_index: i,
                            piece_status: PieceStatus::Downloading,
                            piece_data: vec![],
                        });
                        *piece_lock = PieceStatus::Downloading;
                    }
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    self.logger_sender
                        .lock()?
                        .send(LogMsg::Info(format!("WouldBlock PIECE: {}", i)))?;
                    continue;
                }
                Err(_) => {
                    self.logger_sender.lock()?.send(LogMsg::Info(format!(
                        "UNKNOWN ERROR trying to lock PIECE: {}",
                        i
                    )))?;
                    self.logger_sender.lock()?.send(LogMsg::Info(format!(
                        "SUBSTRACTING THREAD, PEER:{}",
                        peer_connection.peer.read()?.ip
                    )))?;
                    let mut active_threads = self.active_threads_quantity.lock()?;
                    *active_threads -= 1;
                    return Err(DownloadManagerError::new(
                        "Error trying to lock piece mutex".to_string(),
                    ));
                }
            }

            if pieces_to_download.len() >= quantity_to_download {
                break;
            }
        }

        if pieces_to_download.is_empty() {
            self.info
                .read()?
                .logger_sender
                .lock()?
                .send(LogMsg::Info("Ended idle job".to_string()))?;
            self.sender_client.lock()?.send(vec![(
                DELETE_ONE_ACTIVE_CONNECTION,
                UiParams::Vector(vec![
                    peer_connection.peer.read()?.id.clone(),
                    "Disconnected".to_string(),
                ]),
                self.info.read()?.torrent_name.clone(),
            )])?;

            let mut active_threads = self.active_threads_quantity.lock()?;
            *active_threads -= 1;
            return Err(DownloadManagerError::new(
                "Ended idle job, pieces to download with len 0".to_string(),
            ));
        }
        Ok((pieces_indexes, pieces_to_download))
    }

    /// Attempts to download the pieces from the peer. If there is an error, the piece status will be restored to not downloaded.
    fn attempt_download_pieces(
        self: Arc<Self>,
        pieces_indexes: Vec<usize>,
        pieces_to_download: &mut [PieceInfo],
        peer_connection: Arc<PeerConnection<Peer>>,
    ) -> Result<(), DownloadManagerError> {
        self.info
            .read()?
            .logger_sender
            .lock()?
            .send(LogMsg::Info(format!(
                "PIECES TO DOWNLOAD: {:?}",
                pieces_indexes
            )))?;

        let pieces_downloading = self
            .bitfield
            .iter()
            .filter(|x| PieceStatus::Downloading == x.lock().unwrap().to_owned())
            .count();

        self.logger_sender.lock()?.send(LogMsg::Info(format!(
            "PIECES DOWNLOADING: {}",
            pieces_downloading
        )))?;

        for (iteration, piece) in pieces_to_download.iter_mut().enumerate() {
            let index = piece.piece_index;
            let system_time = SystemTime::now();
            let datetime: DateTime<Utc> = system_time.into();
            let timestamp = datetime.timestamp();
            match self
                .clone()
                .request_piece(index as u32, peer_connection.clone())
            {
                Ok(piece_data) => {
                    let system_time2 = SystemTime::now();
                    let datetime2: DateTime<Utc> = system_time2.into();
                    let timestamp2 = datetime2.timestamp();
                    let mut time_difference = timestamp2 - timestamp;
                    if time_difference == 0 {
                        time_difference = 8;
                    }
                    let download_speed = 16384 / time_difference;
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
                    let mut piece_lock = self.bitfield[piece.piece_index].lock()?;
                    *piece_lock = PieceStatus::Downloaded;

                    self.info.read()?.logger_sender.lock()?.send(LogMsg::Info(
                        format!(
                            "Piece {} downloaded from {}",
                            index,
                            peer_connection.clone().peer.read()?.ip
                        )
                        .to_string(),
                    ))?;

                    self.sender_client.lock()?.send(vec![(
                        UPDATE_DOWNLOADED_PIECES,
                        UiParams::U64(1),
                        self.info.read()?.clone().torrent_name,
                    )])?
                }
                Err(e) => {
                    self.logger_sender.lock()?.send(LogMsg::Info(format!(
                        "FAILED PIECE {} request_piece(), peer {}, error: {}",
                        piece.piece_index,
                        peer_connection.peer.read()?.ip,
                        e
                    )))?;
                    self.logger_sender.lock()?.send(LogMsg::Info(format!(
                        "(peer request failed) CLEAN BITFIELD RETURNED={:?}",
                        self.clone().clean_bitfield_at(&pieces_indexes[iteration..])
                    )))?;
                    self.logger_sender.lock()?.send(LogMsg::Info(format!(
                        "SUBSTRACTING THREAD, PEER:{}",
                        peer_connection.peer.read()?.ip
                    )))?;
                    self.sender_client.lock()?.send(vec![(
                        DELETE_ONE_ACTIVE_CONNECTION,
                        UiParams::Vector(vec![
                            peer_connection.peer.read()?.id.clone(),
                            "Disconnected".to_string(),
                        ]),
                        self.info.read()?.torrent_name.clone(),
                    )])?;
                    let mut active_threads = self.active_threads_quantity.lock()?;
                    *active_threads -= 1;
                    return Err(DownloadManagerError::new(
                        "Error downloading piece".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Returns the entire piece given the piece index and the peer connection.
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

                //if they choke us, we have to stop the download returning error
                if message_type == CHOKE_ID {
                    self.logger_sender.lock()?.send(LogMsg::Info(format!(
                        " PEER {} CHOKED US, STOPPING DOWNLOAD OF THIS PIECE",
                        peer.peer.read()?.ip
                    )))?;
                    return Err(DownloadManagerError::new("Peer choked us".to_string()));
                }
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

    /// This function starts the connection with the peer using the handshake message.
    fn try_peer_connection(
        self: Arc<Self>,
        peer: Arc<PeerConnection<Peer>>,
    ) -> Result<(), PeerConnectionError> {
        let peer_id = self.info.read()?.client_id.clone();
        peer.handshake(peer_id)
    }

    /// It change status of given pieces in the bitfield to NotDownloaded.
    fn clean_bitfield_at(self: Arc<Self>, indexes: &[usize]) -> Result<(), DownloadManagerError> {
        for index in indexes.iter() {
            let mut piece_lock = self.bitfield[*index].lock()?;
            *piece_lock = PieceStatus::NotDownloaded;
        }
        Ok(())
    }
}

/// Checks if the assembled file is the same as the original file comparing the sha1 of each piece.
fn verify_assembled_file(
    pieces: &[u8],
    assembled_file_path: String,
    piece_length: usize,
    pieces_quantity: usize,
) -> Result<(), DownloadManagerError> {
    let mut assembled_file = File::open(assembled_file_path)?;
    let mut buffer = vec![0u8; piece_length];
    for i in 0..pieces_quantity {
        assembled_file.read_exact(&mut buffer)?;
        if let Err(e) = verify_piece(pieces, &buffer, &(i as u32)) {
            return Err(DownloadManagerError::new(e.to_string()));
        }
    }
    Ok(())
}

/// Verify the given piece with the real piece using sha1.
fn verify_piece(
    pieces: &[u8],
    piece_data: &[u8],
    piece_idx: &u32,
) -> Result<(), DownloadManagerError> {
    let piece_idx = *piece_idx as usize;
    let mut hasher = Sha1::new();
    hasher.update(piece_data);
    let hash = hasher.finalize().to_vec();

    if hash != pieces[(piece_idx * PIECE_HASH_LEN)..((piece_idx + 1) * PIECE_HASH_LEN)].to_vec() {
        return Err(DownloadManagerError::new(format!(
            "piece hash does not match, piece_idx: {}\nleft: {:?}\nright: {:?}",
            piece_idx,
            hash,
            pieces[(piece_idx * PIECE_HASH_LEN)..((piece_idx + 1) * PIECE_HASH_LEN)].to_vec()
        )));
    }

    Ok(())
}

/// Returns a Vec of PieceStatus acting as Bitfield. It scans the download path and build the bitfield, only works if pieces names are in the right format ["piece_{idx}.txt"]
fn build_bitfield(
    pieces_quantity: usize,
    download_pieces_path: String,
) -> Arc<Vec<Mutex<PieceStatus>>> {
    let mut stored_indexes = HashSet::new();
    if !std::path::Path::new(&download_pieces_path).exists() {
        std::fs::create_dir_all(&download_pieces_path).unwrap();
    }

    for entry in std::fs::read_dir(download_pieces_path).unwrap().flatten() {
        if !entry.path().is_dir() {
            let piece_idx: usize = entry
                .file_name()
                .to_str()
                .unwrap()
                .split('_')
                .collect::<Vec<&str>>()[1]
                .split('.')
                .collect::<Vec<&str>>()[0]
                .parse()
                .unwrap();
            stored_indexes.insert(piece_idx);
        }
    }
    let mut bitfield = Vec::with_capacity(pieces_quantity);
    for i in 0..pieces_quantity {
        if stored_indexes.contains(&i) {
            bitfield.push(Mutex::new(PieceStatus::Downloaded));
        } else {
            bitfield.push(Mutex::new(PieceStatus::NotDownloaded));
        }
    }
    Arc::new(bitfield)
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

    #[test]
    fn test_piece_info_eq() {
        let piece1 = PieceInfo {
            piece_index: 1,
            piece_status: PieceStatus::NotDownloaded,
            piece_data: vec![],
        };
        let piece2 = PieceInfo {
            piece_index: 1,
            piece_status: PieceStatus::Downloaded,
            piece_data: vec![],
        };
        assert_eq!(piece1, piece2);

        let piece1 = PieceInfo {
            piece_index: 1,
            piece_status: PieceStatus::NotDownloaded,
            piece_data: vec![],
        };
        let piece2 = PieceInfo {
            piece_index: 2,
            piece_status: PieceStatus::NotDownloaded,
            piece_data: vec![],
        };
        assert_ne!(piece1, piece2);
    }

    #[test]
    fn test_piece_info_hash_set() {
        let piece1 = PieceInfo {
            piece_index: 1,
            piece_status: PieceStatus::NotDownloaded,
            piece_data: vec![],
        };
        let piece2 = PieceInfo {
            piece_index: 2,
            piece_status: PieceStatus::NotDownloaded,
            piece_data: vec![],
        };
        let piece2_copy = PieceInfo {
            piece_index: 2,
            piece_status: PieceStatus::Downloaded,
            piece_data: vec![],
        };
        let piece3 = PieceInfo {
            piece_index: 3,
            piece_status: PieceStatus::Downloaded,
            piece_data: vec![],
        };
        let mut set = HashSet::new();
        set.insert(piece1.clone());
        set.insert(piece2);
        assert!(set.contains(&piece1)); // same status
        assert!(set.contains(&piece2_copy)); // diff status
        assert!(!set.contains(&piece3)); // absent piece
    }

    #[test]
    fn test_verify_assembled_file() {
        let mut hasher = Sha1::new();
        hasher.update("abcde".as_bytes());
        let hash1 = hasher.finalize()[..].to_vec();
        let mut hasher2 = Sha1::new();
        hasher2.update("fghij".as_bytes());
        let hash2 = hasher2.finalize()[..].to_vec();
        let mut hasher3 = Sha1::new();
        hasher3.update("kabcd".as_bytes());
        let hash3 = hasher3.finalize()[..].to_vec();
        let mut pieces = Vec::new();
        pieces.extend(hash1);
        pieces.extend(hash2);
        pieces.extend(hash3);
        let assembled_file_path =
            "src/test_files/test_verify_assembled_files/test_1.txt".to_string();
        assert!(verify_assembled_file(&pieces, assembled_file_path, 5, 3).is_ok());
    }
}
