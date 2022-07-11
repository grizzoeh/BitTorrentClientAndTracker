use crate::{
    download_manager::DownloadManager,
    download_manager::DownloaderInfo,
    errors::client_error::ClientError,
    listener::Listener,
    logger::LogMsg,
    logger::Logger,
    peer_entities::communication_method::CommunicationMethod,
    peer_entities::communication_method::TCP,
    peer_entities::peer::Peer,
    peer_entities::peer_connection::PeerConnection,
    tracker::Tracker,
    tracker::TrackerInterface,
    ui::ui_codes::*,
    upload_manager::PieceRequest,
    upload_manager::UploadManager,
    utilities::utils::{create_id, vecu8_to_string, vecu8_to_u64, UiParams},
};
use glib::Sender as UISender;
use std::{
    collections::HashMap,
    sync::mpsc,
    sync::mpsc::Receiver,
    sync::mpsc::{channel, Sender},
    sync::RwLock,
    sync::{Arc, Mutex},
    thread::spawn,
    thread::JoinHandle,
};

/// This struct is the responsible of creating the different parts of the application, such as the logger, listener, tracker, upload manager and download manager.
#[allow(clippy::type_complexity)]
pub struct Client {
    pub peers: Arc<RwLock<Vec<Arc<PeerConnection<Peer>>>>>,
    pub torrent_path: String,
    pub download_path: Mutex<String>,
    pub download_pieces_path: Mutex<String>,
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
    upload_receiver: Arc<Mutex<Receiver<Option<PieceRequest>>>>,
    pub port_listener: u16,
}

#[allow(clippy::type_complexity)]
pub trait ClientInterface {
    fn create(
        config: HashMap<String, String>,
        sender_client: Arc<Mutex<UISender<Vec<(usize, UiParams, String)>>>>,
        torrent_name: String,
        port_listener: u16,
        torrent_data: HashMap<String, Vec<u8>>,
    ) -> Result<(Arc<dyn ClientInterface>, JoinHandle<()>), ClientError>
    where
        Self: Sized;

    #[allow(clippy::type_complexity)]
    fn start(
        self: Arc<Self>,
    ) -> Result<(JoinHandle<()>, JoinHandle<()>, JoinHandle<()>), ClientError>;

    fn get_info_hash(&self) -> Vec<u8>;
}

#[allow(clippy::type_complexity)]
impl ClientInterface for Client {
    /// This function is responsible for creating the client and connect with the tracker.
    fn create(
        config: HashMap<String, String>,
        sender_client: Arc<Mutex<UISender<Vec<(usize, UiParams, String)>>>>,
        torrent_name: String,
        port_listener: u16,
        torrent_data: HashMap<String, Vec<u8>>,
    ) -> Result<(Arc<(dyn ClientInterface + 'static)>, JoinHandle<()>), ClientError> {
        let id = create_id();
        let log_path = config["log_path"].clone();
        let download_path = config["download_path"].clone();
        let mut download_pieces_path = "src/downloaded_pieces".to_string();
        let torrent_path = config["torrent_path"].clone();
        let port = config["port"].clone().parse::<u16>()?;

        let uploaded = 0;
        let downloaded = 0;
        let left = 0;
        let event = "started".to_string();

        let torrent_name_aux1 = torrent_name.clone();
        let torrent_name_aux2 = torrent_name.clone();

        // create directory for the torrent if it doesn't exist
        let mut split_filename = torrent_path.split('/');
        let real_name = split_filename
            .next_back()
            .ok_or_else(|| ClientError::new("Error spliting name of filename".to_string()))?;

        download_pieces_path.push('/');

        download_pieces_path.push_str(real_name);
        if !std::path::Path::new(&download_pieces_path).exists() {
            std::fs::create_dir_all(&download_pieces_path)?;
        }

        sender_client.lock()?.send(vec![(
            UPDATE_TOTAL_SIZE,
            UiParams::U64(vecu8_to_u64(&torrent_data["length"])),
            torrent_name,
        )])?;

        sender_client.lock()?.send(vec![(
            UPDATE_FILENAME,
            UiParams::String(vecu8_to_string(&torrent_data["name"])),
            torrent_name_aux1,
        )])?;

        let announce_url = String::from_utf8(torrent_data["url"].clone())?;

        let mut info = HashMap::new();
        info.insert(String::from("URL"), announce_url);
        info.insert(String::from("peer_id"), id.clone());
        info.insert(String::from("port"), format!("{}", port));
        info.insert(String::from("uploaded"), format!("{}", uploaded));
        info.insert(String::from("downloaded"), format!("{}", downloaded));
        info.insert(String::from("left"), format!("{}", left));
        info.insert(String::from("event"), event.clone());

        let log_path_aux = format!("{}/{}_log.txt", log_path, real_name);

        let (logger_sender, logger_receiver) = channel();
        let mut logger = Logger::new(log_path_aux, logger_receiver)?;
        let _logger_handler = spawn(move || {
            let _r = logger.start();
        });

        let tracker = Tracker::create(
            info,
            torrent_data["info_hash"].clone(),
            logger_sender.clone(),
        )?;

        let (upload_sender, upload_receiver) = channel();

        let peers = tracker.get_peers()?;

        sender_client.lock()?.send(vec![(
            UPDATE_PEERS_NUMBER,
            UiParams::Usize(peers.len()),
            torrent_name_aux2.clone(),
        )])?;

        let mut peers_conn = Vec::new();
        for peer in peers {
            let peer_conn = Arc::new(PeerConnection::new(
                peer.clone(),
                tracker.get_info_hash().clone(),
                id.clone(),
                Arc::new(Mutex::new(TCP::create())),
                Arc::new(Mutex::new(logger_sender.clone())),
                Arc::new(Mutex::new(upload_sender.clone())),
            ));
            peers_conn.push(peer_conn);
        }

        let client = Arc::new(Client {
            torrent_path,
            download_path: Mutex::new(download_path),
            download_pieces_path: Mutex::new(download_pieces_path.clone()),
            id,
            uploaded,
            downloaded,
            left,
            logger_sender: Arc::new(Mutex::new(logger_sender)),
            torrent_name: torrent_name_aux2,
            event,
            port,
            tracker,
            peers: Arc::new(RwLock::new(peers_conn)),
            pieces: torrent_data["pieces"].clone(),
            pieces_length: RwLock::new(vecu8_to_u64(&torrent_data["piece length"])),
            file_length: RwLock::new(vecu8_to_u64(&torrent_data["length"]) as u64),
            sender_client,
            upload_sender: Arc::new(Mutex::new(upload_sender)),
            upload_receiver: Arc::new(Mutex::new(upload_receiver)),
            port_listener,
        });
        Ok((client, _logger_handler))
    }

    /// This function starts the application and all the different parts of the application in differents threads
    /// Returns JoinHandlers for DownloadManager, Listener and UploadManager.
    fn start(
        self: Arc<Self>,
    ) -> Result<(JoinHandle<()>, JoinHandle<()>, JoinHandle<()>), ClientError> {
        let downloader_info = DownloaderInfo {
            piece_length: *self.pieces_length.read()?,
            download_path: self.download_path.lock()?.clone(),
            download_pieces_path: self.download_pieces_path.lock()?.clone(),
            logger_sender: self.logger_sender.clone(),
            pieces_hash: self.pieces.clone(),
            peers: self.peers.clone(),
            info_hash: self.tracker.get_info_hash(),
            client_id: self.id.clone(),
            upload_sender: self.upload_sender.clone(),
            torrent_name: self.torrent_name.clone(),
            file_length: *self.file_length.read()?,
            ui_sender: self.sender_client.clone(),
        };
        let download_manager = DownloadManager::new(downloader_info)?;
        let listener_channel = mpsc::channel();
        let bitfield = download_manager.bitfield.clone();
        let listener = Listener::new(
            format!("127.0.0.1:{}", self.port_listener).as_str(),
            bitfield.clone(),
            Arc::new(Mutex::new(listener_channel.1)),
            self.logger_sender.clone(),
            self.upload_sender.clone(),
            self.id.clone(),
            self.get_info_hash(),
            self.sender_client.clone(),
            self.torrent_name.clone(),
        )?;
        let upload_manager = UploadManager::new(
            self.logger_sender.clone().lock()?.clone(),
            self.download_pieces_path.lock()?.clone(),
            bitfield,
            self.upload_receiver.clone(),
            Arc::new(Mutex::new(listener_channel.0)),
        );

        let download_handle = spawn(move || {
            let _r = download_manager.start_download();
        });
        let listener_handle = spawn(move || {
            let _r = listener.listen();
        });

        let sender_client_cp = self.sender_client.clone();
        let torrent_name_cp = self.torrent_name.clone();

        let upload_handle = spawn(move || {
            let _r = upload_manager.start_uploader(sender_client_cp, torrent_name_cp);
        });

        Ok((download_handle, listener_handle, upload_handle))
    }

    fn get_info_hash(&self) -> Vec<u8> {
        self.tracker.get_info_hash()
    }
}
