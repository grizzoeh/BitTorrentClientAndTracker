use crate::bdecoder::{from_string_to_vec, from_vec_to_string, Decodification};
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
    pub tracker: Tracker,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub event: String,
    pub id: String,
    pub port: u16,
}

#[derive(Debug)]
pub enum ClientError {
    TrackerError,
    TorrentError,
    DownloadError,
    TorrentParseFileNotFound,
    ParserError,
}

impl Client {
    pub fn new(config: HashMap<String, String>) -> Result<Client, ClientError> {
        let system_time = SystemTime::now();
        let datetime: DateTime<Utc> = system_time.into();
        let mut client_id = format!("{}{}54321", process::id(), (datetime.timestamp() as u64));
        if client_id.len() != 20 {
            client_id.push('0');
        }

        //let log_path = config["log_path"].clone();
        let download_path = config["download_path"].clone();
        let torrent_path = config["torrent_path"].clone();
        let port = match config["port"].clone().parse::<u16>() {
            Ok(port) => port,
            Err(_) => return Err(ClientError::ParserError),
        };

        // Define vars to use it in tracker
        let uploaded = 0;
        let downloaded = 0;
        let left = 0;
        let event = "started".to_string();

        let (url, info_hash) = match torrent_parse(&config["torrent_path"]) {
            Ok(info) => (info.0, info.1),
            Err(_) => return Err(ClientError::TorrentParseFileNotFound),
        };

        let mut info = HashMap::new();
        info.insert(String::from("URL"), url);
        info.insert(String::from("peer_id"), client_id.clone());
        info.insert(String::from("port"), format!("{}", port));
        info.insert(String::from("uploaded"), format!("{}", uploaded));
        info.insert(String::from("downloaded"), format!("{}", downloaded));
        info.insert(String::from("left"), format!("{}", left));
        info.insert(String::from("event"), event.clone());

        let tracker = match Tracker::new(info, info_hash) {
            Ok(tracker) => tracker,
            Err(_) => return Err(ClientError::TrackerError),
        };

        let peers = initialize_peers(&tracker)?;
        Ok(Client {
            torrent_path,
            download_path,
            id: client_id,
            uploaded,
            downloaded,
            left,
            event,
            port,
            tracker,
            peers,
        })
    }

    pub fn get_peer_by_id(&self, id: String) -> Option<&Peer> {
        self.peers.iter().find(|&p| p.id == id)
    }

    pub fn get_peer_by_ip(&self, ip: &str) -> Option<&Peer> {
        self.peers.iter().find(|&p| p.ip == ip)
    }

    pub fn start(&self) {
        println!("STARTING CONNECTIONS WITH PEERS");
        //let mut first_peer = self.peers[0].clone();
        //first_peer.handshake(self.tracker.info_hash.clone(), self.id.clone());
        // despues lo haremos con varios peers

        for i in 0..self.peers.len() {
            println!("PEER NUMBER: {}", i);
            let mut first_peer = self.peers[i].clone();
            first_peer.handshake(self.tracker.info_hash.clone(), self.id.clone());
            println!("--------------------------------------");
        }
    }
}

fn initialize_peers(tracker: &Tracker) -> Result<Vec<Peer>, ClientError> {
    let peers = tracker.peers.clone();
    let mut peers_vec = Vec::new();

    if let Decodification::List(peer_list) = peers {
        for peer in peer_list {
            if let Decodification::Dic(peer_dict) = peer {
                let peer_ip = match peer_dict.get(&from_string_to_vec("ip")) {
                    Some(Decodification::String(ip)) => ip,
                    _ => return Err(ClientError::TrackerError),
                };
                let peer_port = match peer_dict.get(&from_string_to_vec("port")) {
                    Some(Decodification::Int(port)) => *port as u16,
                    _ => return Err(ClientError::TrackerError),
                };
                let peer_id = match peer_dict.get(&from_string_to_vec("peer id")) {
                    Some(Decodification::String(id)) => id,
                    _ => return Err(ClientError::TrackerError),
                };
                let peer = Peer::new(
                    from_vec_to_string(peer_id),
                    from_vec_to_string(peer_ip),
                    peer_port,
                );
                peers_vec.push(peer);
            }
        }
    }
    Ok(peers_vec)
}
