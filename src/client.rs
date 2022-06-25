use crate::download_manager::DownloadManager;
use crate::errors::client_error::ClientError;
use crate::peer::Peer;
use crate::peer_connection::PeerConnection;
use crate::torrent_parser::torrent_parse;
use crate::tracker::Tracker;
use crate::utils::create_id;
use crate::utils::vecu8_to_u64;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
extern crate chrono;

pub const CHUNK_SIZE: u32 = 16384;
pub const INITIAL_OFFSET: u32 = 0;
pub const PIECE_HASH_LEN: usize = 20;

pub struct Client {
    pub peers: RwLock<Vec<Peer>>,
    pub torrent_path: String,
    pub download_path: Mutex<String>,
    pub tracker: Tracker,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub event: String,
    pub id: String,
    pub port: u16,
    pub pieces: Mutex<Vec<u8>>,
    pub pieces_length: Mutex<u64>,
    pub length: Mutex<u64>,
}

impl Client {
    pub fn new(config: HashMap<String, String>) -> Result<Arc<Client>, ClientError> {
        let id = create_id();
        //let log_path = config["log_path"].clone();
        let download_path = config["download_path"].clone();
        let torrent_path = config["torrent_path"].clone();
        let port = config["port"].clone().parse::<u16>()?;

        let uploaded = 0;
        let downloaded = 0;
        let left = 0;
        let event = "started".to_string();

        let torrent_data = torrent_parse(&config["torrent_path"])?;

        let announce_url = String::from_utf8(torrent_data["url"].clone())?;

        let mut info = HashMap::new();
        info.insert(String::from("URL"), announce_url);
        info.insert(String::from("peer_id"), id.clone());
        info.insert(String::from("port"), format!("{}", port));
        info.insert(String::from("uploaded"), format!("{}", uploaded));
        info.insert(String::from("downloaded"), format!("{}", downloaded));
        info.insert(String::from("left"), format!("{}", left));
        info.insert(String::from("event"), event.clone());

        let tracker = Tracker::new(info, torrent_data["info_hash"].clone())?;
        Ok(Arc::new(Client {
            torrent_path,
            download_path: Mutex::new(download_path),
            id,
            uploaded,
            downloaded,
            left,
            event,
            port,
            tracker,
            peers: RwLock::new(Vec::new()),
            pieces: Mutex::new(torrent_data["pieces"].clone()),
            pieces_length: Mutex::new(vecu8_to_u64(&torrent_data["piece length"])),
            length: Mutex::new(vecu8_to_u64(&torrent_data["length"]) as u64),
        }))
    }

    pub fn start(self: Arc<Self>) -> Result<(), ClientError> {
        let mut peers = self.tracker.get_peers()?;
        self.peers.write()?.append(&mut peers);

        let download_manager =
            DownloadManager::new(self.download_path.lock()?.clone(), self.clone())?;

        let _ = download_manager.start_download();
        Ok(())
    }

    pub fn try_peer_connection(
        &self,
        peer: &Peer,
    ) -> Result<PeerConnection<TcpStream>, ClientError> {
        let peer = peer.clone();
        let stream = TcpStream::connect(format!("{}:{}", &peer.ip, &peer.port))?;
        let _r = stream.set_read_timeout(Some(std::time::Duration::from_secs(15)));
        let peer_connection = PeerConnection::new(
            peer,
            self.tracker.info_hash.clone(),
            self.id.clone(),
            Arc::new(Mutex::new(stream)),
        )?;
        Ok(peer_connection)
    }
}

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
            ("log_path".to_string(), "reports/logs".to_string()),
        ]);
        assert!(Client::new(config).is_ok());
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
        assert!(Client::new(config).is_err());
    }
}
