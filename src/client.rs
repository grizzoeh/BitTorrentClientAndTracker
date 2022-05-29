use crate::bdecoder::Decodification;
use crate::peer::Peer;

use crate::torrentparser::torrent_parse;
use crate::tracker::Tracker;
use std::collections::HashMap;
use std::process;
use std::time::SystemTime;
extern crate chrono;
use chrono::offset::Utc;
use chrono::DateTime;
pub struct Client {
    pub peers: Vec<Peer>,
    pub torrent_path: String,
    pub download_path: String,
    tracker: Tracker,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    event: String,
    id: String,
    port: u16,
}

#[derive(Debug)]
pub enum ClientError {
    TrackerError,
    TorrentError,
    DownloadError,
    TorrentParseFileNotFound,
}

impl Client {
    pub fn new(torrent_path: String, download_path: String) -> Result<Client, ClientError> {
        let system_time = SystemTime::now();
        let datetime: DateTime<Utc> = system_time.into();
        // client id
        let mut id = format!("{}{}54321", process::id(), (datetime.timestamp() as u64));
        if id.len() != 20 {
            id.push('0');
        }

        // define vars to use it in tracker

        let uploaded = 0;
        let downloaded = 0;
        let left = 0;
        let event = "started".to_string();
        let port = 443; // FIXME get from config

        // info hashmap for tracker
        let (url, info_hash) = match torrent_parse(&torrent_path) {
            Ok(info) => (info.0, info.1),
            Err(_) => return Err(ClientError::TorrentParseFileNotFound),
        };

        let mut info = HashMap::new();
        info.insert(String::from("URL"), url);
        info.insert(String::from("peer_id"), id.clone());
        info.insert(String::from("port"), format!("{}", port));
        info.insert(String::from("uploaded"), format!("{}", uploaded));
        info.insert(String::from("downloaded"), format!("{}", downloaded));
        info.insert(String::from("left"), format!("{}", left));
        info.insert(String::from("event"), event.clone());

        let tracker = Tracker::new(info, info_hash).unwrap();
        let peers = initialize_peers(&tracker)?;
        Ok(Client {
            torrent_path,
            download_path,
            id,
            uploaded,
            downloaded,
            left,
            event,
            port, // FIXME get from config
            tracker,
            peers,
        })
    }

    pub fn get_peers(&self) -> &Vec<Peer> {
        &self.peers
    }

    pub fn get_torrent_path(&self) -> &String {
        &self.torrent_path
    }

    pub fn get_download_path(&self) -> &String {
        &self.download_path
    }

    pub fn get_peer_by_id(&self, id: String) -> Option<&Peer> {
        self.peers.iter().find(|&p| p.get_id() == id)
    }

    pub fn get_peer_by_ip(&self, ip: &str) -> Option<&Peer> {
        self.peers.iter().find(|&p| p.get_ip() == ip) // habrá drama con esta comparación entre &String ?? FIXME
    }

    pub fn read_params(&self) {
        // solo sirve para satisfacer clippy antes de implementar el resto de las funciones
        println!(
            "{:?}{}{}{}{}{}{}",
            self.tracker, self.uploaded, self.downloaded, self.left, self.event, self.id, self.port
        );
    }
}

fn initialize_peers(tracker: &Tracker) -> Result<Vec<Peer>, ClientError> {
    // request peers from tracker
    let peers = tracker.peers.clone();
    let mut peers_vec = Vec::new();
    if let Decodification::List(peer_list) = peers {
        for peer in peer_list {
            if let Decodification::Dic(peer_dict) = peer {
                let peer_ip = match peer_dict.get("ip") {
                    Some(Decodification::String(ip)) => ip,
                    _ => return Err(ClientError::TrackerError),
                };
                let peer_port = match peer_dict.get("port") {
                    Some(Decodification::Int(port)) => *port as u16,
                    _ => return Err(ClientError::TrackerError),
                };
                let peer_id = match peer_dict.get("peer id") {
                    Some(Decodification::String(id)) => id,
                    _ => return Err(ClientError::TrackerError),
                };
                let peer = Peer::new(peer_id.to_string(), peer_ip.to_string(), peer_port);
                peers_vec.push(peer);
            }
        }
    }
    Ok(peers_vec)
}
