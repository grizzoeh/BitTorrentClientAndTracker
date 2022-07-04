use crabrave::logger::LogMsg;
use crabrave::peer::Peer;
use crabrave::peer_connection::PeerConnection;
use crabrave::tracker::TrackerInterface;
use crabrave::upload_manager::PieceRequest;
use crabrave::utils::UiParams;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
extern crate chrono;
use glib::Sender as UISender;

pub struct ClientMock {
    pub peers: Arc<RwLock<Vec<Arc<PeerConnection<Peer>>>>>,
    pub torrent_path: String,
    pub download_path: Mutex<String>,
    pub logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    pub torrent_name: String,
    pub tracker: Arc<dyn TrackerInterface>,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub event: String,
    pub id: String,
    pub port: u16,
    pub pieces: Vec<u8>,
    pub pieces_length: RwLock<u64>,
    pub file_length: RwLock<u64>,
    pub sender_client: Arc<Mutex<UISender<Vec<(usize, UiParams, String)>>>>,
    pub upload_sender: Arc<Mutex<Sender<Option<PieceRequest>>>>,
    _upload_receiver: Arc<Mutex<Receiver<Option<PieceRequest>>>>,
    pub port_listener: u16,
}

// impl ClientInterface for ClientMock {
//     fn create(
//         config: HashMap<String, String>,
//         sender_client: Arc<Mutex<UISender<Vec<(usize, UiParams, String)>>>>,
//         torrent_name: String,
//         port_listener: u16,
//         torrent_data: HashMap<String, Vec<u8>>,
//     ) -> Result<
//         (
//             Arc<dyn ClientInterface>,
//             JoinHandle<Result<(), LoggerError>>,
//         ),
//         ClientError,
//     > {
//         let piece1 = vec![0; CHUNK_SIZE as usize];
//         let piece2 = vec![1; CHUNK_SIZE as usize];
//         let piece3 = vec![2; CHUNK_SIZE as usize];

//         let mut hasher = Sha1::new();
//         hasher.update(piece1);
//         let hash1 = hasher.finalize()[..].to_vec();

//         let mut hasher2 = Sha1::new();
//         hasher2.update(piece2);
//         let hash2 = hasher2.finalize()[..].to_vec();

//         let mut hasher3 = Sha1::new();
//         hasher3.update(piece3);
//         let hash3 = hasher3.finalize()[..].to_vec();

//         let mut piece_hash = Vec::new();
//         piece_hash.extend(hash1);
//         piece_hash.extend(hash2);
//         piece_hash.extend(hash3);

//         let info_hash: Vec<u8> = vec![5; 20];
//         let client_id = "client_id11111111111".to_string();

//         let peer1 = Peer::new("default".to_string(), "localhost".to_string(), 8080);
//         let peer2 = Peer::new("default".to_string(), "localhost".to_string(), 8080);
//         let peer3 = Peer::new("default".to_string(), "localhost".to_string(), 8080);

//         let (sender_logger, receiver_logger): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
//         let (sender_upload, receiver_upload): (
//             Sender<Option<PieceRequest>>,
//             Receiver<Option<PieceRequest>>,
//         ) = channel();

//         let peer_connection1 = PeerConnection::new(
//             peer1,
//             info_hash.clone(),
//             client_id.clone(),
//             Arc::new(Mutex::new(test_helper::CommunicationMock1::create())),
//             Arc::new(Mutex::new(sender_logger.clone())),
//             Arc::new(Mutex::new(sender_upload.clone())),
//         );
//         let peer_connection2 = PeerConnection::new(
//             peer2,
//             info_hash.clone(),
//             client_id.clone(),
//             Arc::new(Mutex::new(test_helper::CommunicationMock2::create())),
//             Arc::new(Mutex::new(sender_logger.clone())),
//             Arc::new(Mutex::new(sender_upload.clone())),
//         );
//         let peer_connection3 = PeerConnection::new(
//             peer3,
//             info_hash.clone(),
//             client_id.clone(),
//             Arc::new(Mutex::new(test_helper::CommunicationMock3::create())),
//             Arc::new(Mutex::new(sender_logger.clone())),
//             Arc::new(Mutex::new(sender_upload.clone())),
//         );

//         let peers_vec = vec![
//             Arc::new(peer_connection1),
//             Arc::new(peer_connection2),
//             Arc::new(peer_connection3),
//         ];

//         let (sender_client, _): (
//             glib::Sender<Vec<(usize, UiParams, String)>>,
//             glib::Receiver<Vec<(usize, UiParams, String)>>,
//         ) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

//         let length = 3 * CHUNK_SIZE as u64;
//         let piece_length = CHUNK_SIZE as u64;
//         let download_path = "tests/test_files/downloads/".to_string();
//         let logger_sender = Arc::new(Mutex::new(sender_logger.clone()));
//         let pieces_hash = piece_hash;
//         let peers = Arc::new(RwLock::new(peers_vec));
//         let info_hash = info_hash;
//         let upload_sender = Arc::new(Mutex::new(sender_upload.clone()));
//         let torrent_name = "archivotorrent.txt".to_string();
//         let file_length = 3 * CHUNK_SIZE as u64;
//         let ui_sender = Arc::new(Mutex::new(sender_client));

//         let client = Arc::new(ClientMock {
//             torrent_path: "asdasd".to_string(),
//             download_path: Mutex::new(download_path.clone()),
//             id: client_id,
//             uploaded: 0,
//             downloaded: 0,
//             left: 0,
//             logger_sender: Arc::new(Mutex::new(sender_logger.clone())),
//             torrent_name: torrent_name.clone(),
//             event: "started".to_string(),
//             port: port_listener,
//             tracker: Arc::new(TrackerMock::create(
//                 HashMap::new(),
//                 info_hash.clone(),
//                 sender_logger.clone(),
//             )),
//             peers: Arc::new(RwLock::new(peers_conn)),
//             pieces: torrent_data["pieces"].clone(),
//             pieces_length: RwLock::new(vecu8_to_u64(&torrent_data["piece length"])),
//             file_length: RwLock::new(vecu8_to_u64(&torrent_data["length"]) as u64),
//             sender_client,
//             upload_sender: Arc::new(Mutex::new(upload_sender)),
//             upload_receiver: Arc::new(Mutex::new(upload_receiver)),
//             port_listener,
//         });
//         // info: HashMap<String, String>,
//         // info_hash: Vec<u8>,
//         // sender_logger: Sender<LogMsg>,
//         Ok((client, _logger_handler))
//     }

//     fn start(
//         self: Arc<Self>,
//     ) -> Result<
//         (
//             JoinHandle<Result<(), DownloadManagerError>>,
//             JoinHandle<Result<(), ListenerError>>,
//             JoinHandle<Result<(), UploadManagerError>>,
//         ),
//         ClientError,
//     > {
//         // armo el download manager
//         let downloader_info = DownloaderInfo {
//             length: self.file_length.read()?.clone(),
//             piece_length: self.pieces_length.read()?.clone(),
//             download_path: self.download_path.lock()?.clone(),
//             logger_sender: self.logger_sender.clone(),
//             pieces_hash: self.pieces.clone(),
//             peers: self.peers.clone(),
//             info_hash: self.tracker.get_info_hash(),
//             client_id: self.id.clone(),
//             upload_sender: self.upload_sender.clone(),
//             torrent_name: self.torrent_name.clone(),
//             file_length: self.file_length.read()?.clone(),
//             ui_sender: self.sender_client.clone(),
//         };
//         let download_manager = DownloadManager::new(downloader_info)?;
//         let listener_channel = mpsc::channel();
//         let bitfield = download_manager.clone().get_bitfield();
//         let listener = Listener::new(
//             format!("127.0.0.1:{}", self.port_listener).as_str(),
//             bitfield.clone(),
//             Arc::new(Mutex::new(listener_channel.1)),
//             self.logger_sender.clone(),
//             self.upload_sender.clone(),
//             self.id.clone(),
//             self.get_info_hash(),
//         )?;
//         let upload_manager = UploadManager::new(
//             self.logger_sender.clone().lock()?.clone(),
//             self.download_path.lock()?.clone(),
//             bitfield,
//             self.upload_receiver.clone(),
//             Arc::new(Mutex::new(listener_channel.0)),
//         );

//         let _download_handle = spawn(move || download_manager.clone().start_download());
//         let _listener_handle = spawn(move || listener.listen());

//         let sender_client_cp = self.sender_client.clone();
//         let torrent_name_cp = self.torrent_name.clone();

//         let _upload_handle =
//             spawn(move || upload_manager.start_uploader(sender_client_cp, torrent_name_cp));

//         Ok((_download_handle, _listener_handle, _upload_handle))
//     }

//     fn get_info_hash(&self) -> Vec<u8> {
//         self.tracker.get_info_hash()
//     }
// }

// pub struct TrackerMock {
//     info_hash: Vec<u8>,
//     port: u16,
// }
// impl TrackerInterface for TrackerMock {
//     fn get_info_hash(&self) -> Vec<u8> {
//         vec![
//             1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
//         ]
//     }

//     fn create(
//         info: HashMap<String, String>,
//         info_hash: Vec<u8>,
//         sender_logger: Sender<LogMsg>,
//     ) -> Result<
//         Arc<(dyn TrackerInterface + Send + 'static)>,
//         crabrave::errors::tracker_error::TrackerError,
//     >
//     where
//         Self: Sized,
//     {
//         Ok(Arc::new(TrackerMock {
//             info_hash,
//             port: info["port"].parse::<u16>().unwrap(),
//         }))
//     }

//     fn get_peers(&self) -> Result<Vec<Peer>, crabrave::errors::tracker_error::TrackerError> {
//         let peer1 = Peer::new("default".to_string(), "localhost".to_string(), 8080);
//         let peer2 = Peer::new("default".to_string(), "localhost".to_string(), 8080);
//         let peer3 = Peer::new("default".to_string(), "localhost".to_string(), 8080);
//         return Ok(vec![peer1, peer2, peer3]);
//     }
// }
