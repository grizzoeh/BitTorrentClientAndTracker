use crate::communication_method::CommunicationMethod;
use crate::communication_method::TCP;
use crate::download_manager::DownloadManager;
use crate::download_manager::DownloaderInfo;
use crate::errors::client_error::ClientError;
use crate::errors::download_manager_error::DownloadManagerError;
use crate::errors::listener_error::ListenerError;
use crate::errors::logger_error::LoggerError;
use crate::errors::upload_manager_error::UploadManagerError;
use crate::listener::Listener;
use crate::logger::LogMsg;
use crate::logger::Logger;
use crate::peer::Peer;
use crate::peer_connection::PeerConnection;
use crate::tracker::Tracker;
use crate::tracker::TrackerInterface;
use crate::ui_codes::*;
use crate::upload_manager::PieceRequest;
use crate::upload_manager::UploadManager;
use crate::utils::create_id;
use crate::utils::vecu8_to_string;
use crate::utils::vecu8_to_u64;
use crate::utils::UiParams;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::{channel, Sender};
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::thread::JoinHandle;
extern crate chrono;
use glib::Sender as UISender;

#[allow(clippy::type_complexity)]
pub struct Client {
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
    ) -> Result<
        (
            Arc<dyn ClientInterface>,
            JoinHandle<Result<(), LoggerError>>,
        ),
        ClientError,
    >
    where
        Self: Sized;

    #[allow(clippy::type_complexity)]
    fn start(
        self: Arc<Self>,
    ) -> Result<
        (
            JoinHandle<Result<(), DownloadManagerError>>,
            JoinHandle<Result<(), ListenerError>>,
            JoinHandle<Result<(), UploadManagerError>>,
        ),
        ClientError,
    >;

    fn get_info_hash(&self) -> Vec<u8>;
}

#[allow(clippy::type_complexity)]
impl ClientInterface for Client {
    fn create(
        config: HashMap<String, String>,
        sender_client: Arc<Mutex<UISender<Vec<(usize, UiParams, String)>>>>,
        torrent_name: String,
        port_listener: u16,
        torrent_data: HashMap<String, Vec<u8>>,
    ) -> Result<
        (
            Arc<(dyn ClientInterface + 'static)>,
            JoinHandle<Result<(), LoggerError>>,
        ),
        ClientError,
    > {
        let id = create_id();
        let log_path = config["log_path"].clone();
        let mut download_path = config["download_path"].clone();
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
        download_path.push('/');

        download_path.push_str(real_name);
        if !std::path::Path::new(&download_path).exists() {
            std::fs::create_dir_all(&download_path)?;
        }

        println!("DOWNLOAD PATH {}", &download_path);
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
        let _logger_handler = spawn(move || logger.start());

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
            download_path: Mutex::new(download_path.clone()),
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

    fn start(
        self: Arc<Self>,
    ) -> Result<
        (
            JoinHandle<Result<(), DownloadManagerError>>,
            JoinHandle<Result<(), ListenerError>>,
            JoinHandle<Result<(), UploadManagerError>>,
        ),
        ClientError,
    > {
        let downloader_info = DownloaderInfo {
            length: *self.file_length.read()?,
            piece_length: *self.pieces_length.read()?,
            download_path: self.download_path.lock()?.clone(),
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
        let bitfield = download_manager.clone().get_bitfield();
        let listener = Listener::new(
            format!("127.0.0.1:{}", self.port_listener).as_str(),
            bitfield.clone(),
            Arc::new(Mutex::new(listener_channel.1)),
            self.logger_sender.clone(),
            self.upload_sender.clone(),
            self.id.clone(),
            self.get_info_hash(),
        )?;
        let upload_manager = UploadManager::new(
            self.logger_sender.clone().lock()?.clone(),
            self.download_path.lock()?.clone(),
            bitfield,
            self.upload_receiver.clone(),
            Arc::new(Mutex::new(listener_channel.0)),
        );

        let _download_handle = spawn(move || download_manager.start_download());
        let _listener_handle = spawn(move || listener.listen());

        let sender_client_cp = self.sender_client.clone();
        let torrent_name_cp = self.torrent_name.clone();

        let _upload_handle =
            spawn(move || upload_manager.start_uploader(sender_client_cp, torrent_name_cp));

        Ok((_download_handle, _listener_handle, _upload_handle))
    }

    fn get_info_hash(&self) -> Vec<u8> {
        self.tracker.get_info_hash()
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_new_ok_config() {
        let config = HashMap::from([
            ("port".to_string(), "443".to_string()),
            (
                "torrent_path".to_string(),
                "src/torrent_files/ubuntu-22.04-desktop-amd64.iso.torrent".to_string(),
            ),
            ("log_level".to_string(), "5".to_string()),
            ("download_path".to_string(), "src/downloads/".to_string()),
            ("log_path".to_string(), "src/reports/logs".to_string()),
        ]);
        assert!(Client::init(config).is_ok());
    }

    #[test]
    fn test_client_new_wrong_torrent_path() {
        let config = HashMap::from([
            ("port".to_string(), "443".to_string()),
            (
                "torrent_path".to_string(),
                "src/torrent_files/wrong_path.iso.torrent".to_string(),
            ),
            ("log_level".to_string(), "5".to_string()),
            ("download_path".to_string(), "src/downloads/".to_string()),
            ("log_path".to_string(), "reports/logs".to_string()),
        ]);
        assert!(Client::init(config).is_err());
    }
}
*/
