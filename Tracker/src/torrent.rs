use crate::{
    announce_utils::{int_to_hex, URLParams},
    bdecoder::Decodification,
    errors::torrent_error::TorrentError,
    logger::LogMsg,
    peer::Peer,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, Mutex},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Torrent {
    pub info_hash: Vec<u8>,
    pub peers: HashMap<String, Peer>,
}

impl Torrent {
    pub fn new(info_hash: Vec<u8>) -> Torrent {
        Torrent {
            info_hash,
            peers: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, peer: Peer) {
        self.peers.insert(peer.id.clone(), peer);
    }

    pub fn delete_peer(&mut self, peer_id: String) {
        self.peers.remove(&peer_id);
    }

    /// Returns a Vec of peers with ip, id & port.
    pub fn get_peers_deco(&self, limit: u32) -> Result<Decodification, TorrentError> {
        //Vec<HashMap<String, String>>
        let mut peers: Vec<Decodification> = Vec::new();
        for (count, (id, peer)) in self.peers.iter().enumerate() {
            if count == limit as usize {
                break;
            }
            let mut peer_info: HashMap<Vec<u8>, Decodification> = HashMap::new();
            peer_info.insert(
                "ip".as_bytes().to_vec(),
                Decodification::String(peer.ip.clone().as_bytes().to_vec()),
            );
            peer_info.insert(
                "id".as_bytes().to_vec(),
                Decodification::String(id.clone().as_bytes().to_vec()),
            );
            peer_info.insert(
                "port".as_bytes().to_vec(),
                Decodification::Int(peer.port.parse::<i64>()?),
            );
            peers.push(Decodification::Dic(peer_info));
        }
        Ok(Decodification::List(peers))
    }

    /// Returns a compacted Vec of peers with ip & port.
    pub fn get_peers_deco_compacted(&self, limit: u32) -> Result<Decodification, TorrentError> {
        //Vec<HashMap<String, String>>
        let mut peers: Vec<Decodification> = Vec::new();
        for (count, peer) in self.peers.values().enumerate() {
            if count == limit as usize {
                break;
            }
            let mut peer_info_compacted: String = "".to_string();

            let ip_aux = peer.ip.clone();
            let ip_vec = ip_aux.split('.').collect::<Vec<&str>>();
            for octect in ip_vec.iter() {
                let octect = *octect;
                peer_info_compacted.push_str(&*int_to_hex(octect.parse::<u32>()?));
            }

            peers.push(Decodification::String(
                peer_info_compacted.as_bytes().to_vec(),
            ));
        }
        Ok(Decodification::List(peers))
    }

    /// Given a HashMap with the params of the announce, updates the Peer values.
    pub fn update_peer(
        &mut self,
        announce_dic: HashMap<String, URLParams>,
        logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    ) -> Result<Peer, TorrentError> {
        let peer_id = match announce_dic.get("peer_id") {
            Some(URLParams::String(value)) => value.clone(),
            e => return Err(TorrentError::from(e)),
        };
        let peer_ip = match announce_dic.get("ip") {
            Some(URLParams::String(value)) => value.clone(),
            e => return Err(TorrentError::from(e)),
        };
        let info_hash = match announce_dic.get("info_hash") {
            Some(URLParams::Vector(value)) => value.clone(),
            e => return Err(TorrentError::from(e)),
        };
        let port = match announce_dic.get("port") {
            Some(URLParams::String(value)) => value.clone(),
            e => return Err(TorrentError::from(e)),
        };
        let uploaded = match announce_dic.get("uploaded") {
            Some(URLParams::String(value)) => value.clone(),
            e => return Err(TorrentError::from(e)),
        };
        let downloaded = match announce_dic.get("downloaded") {
            Some(URLParams::String(value)) => value.clone(),
            e => return Err(TorrentError::from(e)),
        };
        let left = match announce_dic.get("left") {
            Some(URLParams::String(value)) => value.clone(),
            e => return Err(TorrentError::from(e)),
        };
        let event = match announce_dic.get("event") {
            Some(URLParams::String(value)) => value.clone(),
            _ => "started".to_string(),
        };

        let mut connected = true;
        let mut completed = false;
        if event == "stopped" {
            connected = false;
            logger_sender
                .lock()?
                .send(LogMsg::Info(format!("Peer {} download stopped", peer_ip)))?;
        }
        if event == "completed" {
            completed = true;
            logger_sender
                .lock()?
                .send(LogMsg::Info(format!("Peer {} download completed", peer_ip)))?;
        }

        if !self.peers.contains_key(&peer_id) {
            if !connected {
                return Err(TorrentError::new("Peer not connected".to_string()));
            }
            let peer = Peer::new(
                port,
                peer_id.clone(),
                peer_ip,
                info_hash,
                uploaded,
                downloaded,
                left,
                connected,
                completed,
            );

            self.add_peer(peer);
        } else if let Some(peer) = self.peers.get_mut(&peer_id) {
            // Update peer
            peer.uploaded = uploaded;
            peer.downloaded = downloaded;
            peer.left = left;
            peer.connected = connected;
            peer.completed = completed;
        }

        if let Some(peer) = self.peers.get_mut(&peer_id) {
            if announce_dic.contains_key("numwant") {
                if let URLParams::String(numwant_aux) = announce_dic["numwant"].clone() {
                    peer.set_numwant(numwant_aux.parse::<u32>()?);
                }
            }

            if announce_dic.contains_key("compact") {
                if let URLParams::String(compact_aux) = announce_dic["compact"].clone() {
                    peer.set_compact(compact_aux.parse::<u16>()?);
                }
            }

            if announce_dic.contains_key("no_peer_id") {
                if let URLParams::String(no_peer_id_aux) = announce_dic["no_peer_id"].clone() {
                    peer.set_no_peer_id(no_peer_id_aux.parse::<u16>()?);
                }
            }

            if announce_dic.contains_key("key") {
                if let URLParams::String(key_aux) = announce_dic["key"].clone() {
                    peer.set_key(key_aux.parse::<String>()?);
                }
            }

            if announce_dic.contains_key("trackerid") {
                if let URLParams::String(trackerid_aux) = announce_dic["trackerid"].clone() {
                    peer.set_trackerid(trackerid_aux.parse::<String>()?);
                }
            }
        }
        Ok(self.peers[&peer_id].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{channel, Receiver};

    #[test]
    fn test_torrent_new() {
        let info_hash = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let torrent = Torrent::new(info_hash.clone());
        assert_eq!(torrent.info_hash, info_hash);
        assert_eq!(torrent.peers.len(), 0);
    }

    #[test]
    fn test_torrent_add_and_delete_peer() {
        let info_hash = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut torrent = Torrent::new(info_hash.clone());
        let peer_id = "peer_id".to_string();
        let peer_ip = "ip".to_string();
        let port = "port".to_string();
        let uploaded = "uploaded".to_string();
        let downloaded = "downloaded".to_string();
        let left = "left".to_string();
        let connected = true;
        let completed = false;
        let peer = Peer::new(
            port,
            peer_id.clone(),
            peer_ip,
            info_hash,
            uploaded,
            downloaded,
            left,
            connected,
            completed,
        );
        torrent.add_peer(peer);
        assert_eq!(torrent.peers.len(), 1);
        assert_eq!(torrent.peers[&peer_id].id, peer_id);

        // Now delete peer
        torrent.delete_peer(peer_id);
        assert_eq!(torrent.peers.len(), 0);
    }

    #[test]
    fn test_torrent_update_peer() {
        // Create logger sender

        let (logger_sender, _r): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let logger_sender = Arc::new(Mutex::new(logger_sender));
        /*  let mut logger = Logger::new(path.clone(), receiver).unwrap();
        let logger_handler = spawn(move || logger.start()); */

        let info_hash = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut torrent = Torrent::new(info_hash.clone());
        let peer_id = "peer_id".to_string();
        let peer_ip = "ip".to_string();
        let port = "port".to_string();
        let uploaded = "uploaded".to_string();
        let downloaded = "downloaded".to_string();
        let left = "left".to_string();
        let connected = true;
        let completed = false;
        let peer = Peer::new(
            port.clone(),
            peer_id.clone(),
            peer_ip.clone(),
            info_hash.clone(),
            uploaded.clone(),
            downloaded.clone(),
            left.clone(),
            connected,
            completed,
        );
        torrent.add_peer(peer);
        assert_eq!(torrent.peers.len(), 1);
        assert_eq!(torrent.peers[&peer_id].id, peer_id);
        let mut announce_dic: HashMap<String, URLParams> = HashMap::new();
        announce_dic.insert("peer_id".to_string(), URLParams::String(peer_id.clone()));
        announce_dic.insert("ip".to_string(), URLParams::String(peer_ip.clone()));
        // Insert info_hash
        announce_dic.insert(
            "info_hash".to_string(),
            URLParams::Vector(info_hash.clone()),
        );
        announce_dic.insert("port".to_string(), URLParams::String(port.clone()));
        announce_dic.insert("uploaded".to_string(), URLParams::String("7".to_string()));
        announce_dic.insert(
            "downloaded".to_string(),
            URLParams::String("12".to_string()),
        );
        announce_dic.insert("left".to_string(), URLParams::String("142".to_string()));
        announce_dic.insert(
            "event".to_string(),
            URLParams::String("stopped".to_string()),
        );
        let peer_updated = torrent.update_peer(announce_dic, logger_sender).unwrap();

        assert_eq!(peer_updated.id, peer_id);
        assert_eq!(peer_updated.ip, peer_ip.clone());
        assert_eq!(peer_updated.port, port);
        assert_eq!(peer_updated.uploaded, "7".to_string());
        assert_eq!(peer_updated.downloaded, "12".to_string());
        assert_eq!(peer_updated.left, "142".to_string());
        assert_eq!(peer_updated.connected, false);
        assert_eq!(peer_updated.completed, false);
    }

    #[test]
    fn test_get_peers_deco_ok() {
        let mut torrent = Torrent::new(vec![1, 2, 1, 2]);
        let peer1 = Peer::new(
            "4041".to_string(),
            "id1".to_string(),
            "ip1".to_string(),
            vec![1, 2, 1, 2],
            "0".to_string(),
            "0".to_string(),
            "123".to_string(),
            true,
            false,
        );
        let peer2 = Peer::new(
            "4042".to_string(),
            "id2".to_string(),
            "ip2".to_string(),
            vec![1, 2, 1, 2],
            "0".to_string(),
            "0".to_string(),
            "123".to_string(),
            true,
            false,
        );

        torrent.add_peer(peer1);
        torrent.add_peer(peer2);

        let first_possible_result = Decodification::List(vec![
            Decodification::Dic(HashMap::from([
                (vec![105, 112], Decodification::String(vec![105, 112, 50])),
                (vec![112, 111, 114, 116], Decodification::Int(4042)),
                (vec![105, 100], Decodification::String(vec![105, 100, 50])),
            ])),
            Decodification::Dic(HashMap::from([
                (vec![112, 111, 114, 116], Decodification::Int(4041)),
                (vec![105, 112], Decodification::String(vec![105, 112, 49])),
                (vec![105, 100], Decodification::String(vec![105, 100, 49])),
            ])),
        ]);

        let second_possible_result = Decodification::List(vec![
            Decodification::Dic(HashMap::from([
                (vec![112, 111, 114, 116], Decodification::Int(4041)),
                (vec![105, 112], Decodification::String(vec![105, 112, 49])),
                (vec![105, 100], Decodification::String(vec![105, 100, 49])),
            ])),
            Decodification::Dic(HashMap::from([
                (vec![105, 112], Decodification::String(vec![105, 112, 50])),
                (vec![112, 111, 114, 116], Decodification::Int(4042)),
                (vec![105, 100], Decodification::String(vec![105, 100, 50])),
            ])),
        ]);

        let test_result = torrent.get_peers_deco(50).unwrap();

        if first_possible_result == test_result || second_possible_result == test_result {
            assert_eq!(true, true); // All OK
        } else {
            assert_eq!(true, false); // ERROR
        }
    }

    #[test]
    fn test_get_peers_deco_compacted_ok() {
        let mut torrent = Torrent::new(vec![1, 2, 1, 2]);
        let peer1 = Peer::new(
            "4041".to_string(),
            "id1".to_string(),
            "192.0.21.4".to_string(),
            vec![1, 2, 1, 2],
            "0".to_string(),
            "0".to_string(),
            "123".to_string(),
            true,
            false,
        );
        let peer2 = Peer::new(
            "4042".to_string(),
            "id2".to_string(),
            "191.2.22.42".to_string(),
            vec![1, 2, 1, 2],
            "0".to_string(),
            "0".to_string(),
            "123".to_string(),
            true,
            false,
        );

        torrent.add_peer(peer1);
        torrent.add_peer(peer2);

        let first_possible_result = Decodification::List(vec![
            Decodification::String(vec![99, 48, 48, 49, 53, 52]),
            Decodification::String(vec![98, 102, 50, 49, 54, 50, 97]),
        ]);

        let second_possible_result = Decodification::List(vec![
            Decodification::String(vec![98, 102, 50, 49, 54, 50, 97]),
            Decodification::String(vec![99, 48, 48, 49, 53, 52]),
        ]);

        let test_result = torrent.get_peers_deco_compacted(50).unwrap();

        if first_possible_result == test_result || second_possible_result == test_result {
            assert_eq!(true, true); // All OK
        } else {
            assert_eq!(true, false); // ERROR
        }
    }
}
