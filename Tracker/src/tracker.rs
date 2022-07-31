use crate::{
    announce_utils::URLParams,
    bdecoder::Decodification,
    bencoder::{bencode, BencoderTypes},
    errors::tracker_error::TrackerError,
    logger::LogMsg,
    torrent::Torrent,
};
use chrono::{offset::Utc, DateTime};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, Mutex},
    time::SystemTime,
};

#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Tracker {
    #[serde_as(as = "Vec<(_, _)>")]
    pub torrents: HashMap<Vec<u8>, Torrent>,
    pub historical_torrents: Vec<i64>, // Vec<timestamp>
    #[serde_as(as = "Vec<(_, Vec<(_, _)>)>")]
    pub historical_peers: HashMap<Vec<u8>, HashMap<String, Vec<i64>>>, // (info_hash, hashMap<"connected"/..., Vec<timestamp>>)
    pub new_changes: bool, // To know if we have to store the changes in the file
}

pub trait TrackerInterface {
    fn add_torrent(&mut self, info_hash: Vec<u8>, timestamp: i64);
    fn delete_torrent(&mut self, torrent: Torrent);
    fn handle_announce(
        &mut self,
        announce_dic: HashMap<String, URLParams>,
        logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    ) -> Result<Vec<u8>, TrackerError>;

    fn get_stats_data(&self) -> Result<String, TrackerError>;
    fn get_announce_response(
        &self,
        info_hash: Vec<u8>,
        compact: u16,
        numwant: u32,
    ) -> Result<Vec<u8>, TrackerError>;
    fn get_announce_peer_list(
        &self,
        info_hash: Vec<u8>,
        compact: u16,
        numwant: u32,
    ) -> Result<Decodification, TrackerError>;
}

impl TrackerInterface for Tracker {
    /// Returns the statistics of the tracker.
    fn get_stats_data(&self) -> Result<String, TrackerError> {
        let tracker_copy = self.clone();
        let json = serde_json::to_string(&tracker_copy)?;
        Ok(json)
    }

    /// Given a infohash, adds a new torrent to the tracker.
    fn add_torrent(&mut self, info_hash: Vec<u8>, timestamp: i64) {
        self.torrents
            .insert(info_hash.clone(), Torrent::new(info_hash.clone()));
        self.historical_torrents.push(timestamp);

        if self.historical_peers.get(&info_hash).is_none() {
            self.historical_peers.insert(info_hash, HashMap::new());
        }
        self.new_changes = true;
    }

    fn delete_torrent(&mut self, torrent: Torrent) {
        self.torrents.remove(&torrent.info_hash);
        self.new_changes = true;
    }

    /// Given a announce dic, returns the announce response.
    fn handle_announce(
        &mut self,
        announce_dic: HashMap<String, URLParams>,
        logger_sender: Arc<Mutex<Sender<LogMsg>>>,
    ) -> Result<Vec<u8>, TrackerError> {
        let info_hash = match announce_dic.get("info_hash") {
            Some(URLParams::Vector(vec_aux)) => vec_aux.clone(),
            _ => {
                logger_sender.lock()?.send(LogMsg::Info(
                    "Info Hash not found in announce URL".to_string(),
                ))?;
                return Err(TrackerError::new("info_hash not found".to_string()));
            }
        };

        let system_time = SystemTime::now();
        let datetime: DateTime<Utc> = system_time.into();
        let timestamp = datetime.timestamp();

        if self.torrents.get(&info_hash).is_none() {
            self.add_torrent(info_hash.clone(), timestamp);
        }
        let compact = match announce_dic.get("compact") {
            Some(URLParams::String(i)) => i.parse::<u16>().unwrap(),
            _ => 0,
        };
        let numwant = match announce_dic.get("numwant") {
            Some(URLParams::String(i)) => i.parse::<u32>().unwrap(),
            _ => 0,
        };
        let response = self.get_announce_response(info_hash.clone(), compact, numwant);

        let peer = self
            .torrents
            .get_mut(&info_hash)
            .unwrap()
            .update_peer(announce_dic, logger_sender)?;

        let torrent_historical_peers = &mut self.historical_peers.get_mut(&peer.info_hash).unwrap();

        let state = if peer.completed {
            "completed"
        } else if peer.connected {
            "connected"
        } else {
            "stopped"
        };

        let torrent_historical_peers_aux = &mut torrent_historical_peers.clone();
        let timestamps = torrent_historical_peers_aux
            .entry(state.to_string())
            .or_insert(vec![])
            .clone();

        let mut timestamps_aux = timestamps.to_vec();
        timestamps_aux.push(timestamp);

        torrent_historical_peers.insert(state.to_string(), timestamps_aux);

        self.new_changes = true;
        response
    }

    fn get_announce_response(
        &self,
        info_hash: Vec<u8>,
        compact: u16,
        numwant: u32,
    ) -> Result<Vec<u8>, TrackerError> {
        let complete = self.torrents[&info_hash]
            .peers
            .iter()
            .filter(|p| p.1.completed)
            .count();

        let total_peers_torrent = self.torrents[&info_hash].peers.len();

        let mut response = HashMap::new();
        response.insert("interval".as_bytes().to_vec(), Decodification::Int(1));
        response.insert(
            "tracker id".as_bytes().to_vec(),
            Decodification::Int(222320),
        );
        response.insert(
            "complete".as_bytes().to_vec(),
            Decodification::Int(complete as i64),
        );
        response.insert(
            "incomplete".as_bytes().to_vec(),
            Decodification::Int((total_peers_torrent - complete) as i64),
        );
        response.insert(
            "peers".as_bytes().to_vec(),
            self.get_announce_peer_list(info_hash, compact, numwant)?,
        );
        Ok(bencode(&BencoderTypes::Decodification(
            Decodification::Dic(response),
        )))
    }

    fn get_announce_peer_list(
        &self,
        info_hash: Vec<u8>,
        compact: u16,
        numwant: u32,
    ) -> Result<Decodification, TrackerError> {
        if self.torrents.get(&info_hash).is_none() {
            return Ok(Decodification::List(Vec::new()));
        }
        let torrent = self.torrents.get(&info_hash).unwrap();

        if compact == 1 {
            Ok(torrent.get_peers_deco_compacted(numwant)?)
        } else {
            Ok(torrent.get_peers_deco(numwant)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{announce_utils::decode_peer_list, bdecoder::bdecode, peer::Peer};

    use super::*;
    use std::{
        sync::mpsc::{channel, Receiver},
        vec,
    };

    #[test]
    fn test_announce_response() {
        // Initialization
        let mut torrents = HashMap::new();
        let historical_torrents = vec![];
        let historical_peers = HashMap::new();
        let new_changes = false;

        let mut torrent1 = Torrent::new(vec![1, 2, 3]);
        let mut torrent2 = Torrent::new(vec![4, 5, 6]);

        torrent1.peers.insert(
            "peer1".to_string(),
            Peer::new(
                "1".to_string(),
                "peer1".to_string(),
                "1:1:1".to_string(),
                vec![1, 2, 3],
                "0".to_string(),
                "0".to_string(),
                "0".to_string(),
                true,
                true,
            ),
        );

        torrent2.peers.insert(
            "peer2".to_string(),
            Peer::new(
                "2".to_string(),
                "peer2".to_string(),
                "1:1:1".to_string(),
                vec![4, 5, 6],
                "0".to_string(),
                "0".to_string(),
                "0".to_string(),
                true,
                true,
            ),
        );

        torrents.insert(vec![1, 2, 3], torrent1);
        torrents.insert(vec![4, 5, 6], torrent2);

        let tracker = Tracker {
            torrents,
            historical_torrents,
            historical_peers,
            new_changes,
        };

        // Execution
        let response = tracker.get_announce_response(vec![1, 2, 3], 0, 50).unwrap();

        let peers_decoded = bdecode(&response).unwrap();
        // Decode response
        let peers = decode_peer_list(vec![1, 2, 3], peers_decoded);
        // Assert that the response is correct
        assert_eq!(
            peers,
            vec![Peer::new(
                "1".to_string(),
                "peer1".to_string(),
                "1:1:1".to_string(),
                vec![1, 2, 3],
                "0".to_string(),
                "0".to_string(),
                "0".to_string(),
                true,
                true,
            )]
        );
    }

    #[test]
    fn test_add_one_torrent() {
        let mut tracker = Tracker {
            torrents: HashMap::new(),
            historical_torrents: vec![],
            historical_peers: HashMap::new(),
            new_changes: false,
        };
        tracker.add_torrent("test".to_string().into_bytes(), 0);
        assert_eq!(tracker.torrents.len(), 1);
    }

    #[test]
    fn test_add_some_torrent() {
        let mut tracker = Tracker {
            torrents: HashMap::new(),
            historical_torrents: vec![],
            historical_peers: HashMap::new(),
            new_changes: false,
        };
        tracker.add_torrent("test".to_string().into_bytes(), 0);
        tracker.add_torrent("test2".to_string().into_bytes(), 3);
        tracker.add_torrent("test3".to_string().into_bytes(), 423);
        assert_eq!(tracker.torrents.len(), 3);
    }

    #[test]
    fn test_delete_one_torrent() {
        let mut tracker = Tracker {
            torrents: HashMap::new(),
            historical_torrents: vec![],
            historical_peers: HashMap::new(),
            new_changes: false,
        };
        let infohash = "test".to_string().into_bytes();
        tracker.add_torrent(infohash.clone(), 0);
        tracker.delete_torrent(tracker.torrents[&infohash].clone());
        assert_eq!(tracker.torrents.len(), 0);
    }

    #[test]
    fn test_delete_some_torrent() {
        let mut tracker = Tracker {
            torrents: HashMap::new(),
            historical_torrents: vec![],
            historical_peers: HashMap::new(),
            new_changes: false,
        };
        let infohash = "test".to_string().into_bytes();
        tracker.add_torrent(infohash.clone(), 0);
        tracker.add_torrent("test2".to_string().into_bytes(), 3);
        tracker.add_torrent("test3".to_string().into_bytes(), 423);
        tracker.delete_torrent(tracker.torrents[&infohash].clone());
        assert_eq!(tracker.torrents.len(), 2);
    }

    #[test]
    fn test_handle_announce() {
        let (logger_sender, _r): (Sender<LogMsg>, Receiver<LogMsg>) = channel();
        let logger_sender = Arc::new(Mutex::new(logger_sender));

        let mut tracker = Tracker {
            torrents: HashMap::new(),
            historical_torrents: vec![],
            historical_peers: HashMap::new(),
            new_changes: false,
        };
        let info_hash = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let peer_id = "peer_id".to_string();
        let peer_ip = "ip".to_string();
        let port = "port".to_string();
        let uploaded = "uploaded".to_string();
        let downloaded = "downloaded".to_string();
        let left = "left".to_string();

        let mut announce_dic: HashMap<String, URLParams> = HashMap::new();
        announce_dic.insert("peer_id".to_string(), URLParams::String(peer_id.clone()));
        announce_dic.insert("ip".to_string(), URLParams::String(peer_ip.clone()));
        // Insert info_hash
        announce_dic.insert(
            "info_hash".to_string(),
            URLParams::Vector(info_hash.clone()),
        );
        announce_dic.insert("port".to_string(), URLParams::String(port.clone()));
        announce_dic.insert(
            "uploaded".to_string(),
            URLParams::String(uploaded.to_string()),
        );
        announce_dic.insert(
            "downloaded".to_string(),
            URLParams::String(downloaded.to_string()),
        );
        announce_dic.insert("left".to_string(), URLParams::String(left.to_string()));
        announce_dic.insert(
            "event".to_string(),
            URLParams::String("stopped".to_string()),
        );

        tracker.add_torrent(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 0);

        let _result = tracker.handle_announce(announce_dic, logger_sender);

        assert_eq!(tracker.new_changes, true)
    }
}
