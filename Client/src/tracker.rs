use crate::{
    errors::tracker_error::TrackerError,
    logger::LogMsg,
    parsing::bdecoder::{bdecode, from_string_to_vec, from_vec_to_string, Decodification},
    peer_entities::peer::Peer,
    utilities::constants::NUMBER_OF_PEERS_TO_ORDER,
    utilities::utils::to_urlencoded,
};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    str,
    sync::mpsc::Sender,
    sync::Arc,
};

/// This struct is used to initialize connection with the tracker and store its information.
#[derive(Debug, PartialEq)]
pub struct Tracker {
    pub peers: Decodification,
    pub info_hash: Vec<u8>,
    pub interval: Decodification,
}

pub trait TrackerInterface {
    fn create(
        info: HashMap<String, String>,
        info_hash: Vec<u8>,
        sender_logger: Sender<LogMsg>,
    ) -> Result<Arc<(dyn TrackerInterface + Send + 'static)>, TrackerError>
    where
        Self: Sized;
    fn get_peers(&self) -> Result<Vec<Peer>, TrackerError>;
    fn get_info_hash(&self) -> Vec<u8>;
}

impl TrackerInterface for Tracker {
    /// Creates the tracker object and initializes the connection with the tracker.
    fn create(
        info: HashMap<String, String>,
        info_hash: Vec<u8>,
        sender_logger: Sender<LogMsg>,
    ) -> Result<Arc<(dyn TrackerInterface + Send + 'static)>, TrackerError> {
        sender_logger.send(LogMsg::Info("CONNECTING WITH THE TRACKER".to_string()))?;
        let response = request_tracker(info, &info_hash)?;
        sender_logger.send(LogMsg::Info("RESPONSE OBTAINED SUCCESSFULLY".to_string()))?;

        if let Decodification::Dic(dic_aux) = response {
            let tracker = Tracker {
                interval: dic_aux[&from_string_to_vec("interval")].clone(),

                peers: dic_aux[&from_string_to_vec("peers")].clone(),
                info_hash,
            };
            Ok(Arc::new(tracker))
        } else {
            Err(TrackerError::new("Expected Dic not found".to_string()))
        }
    }

    /// Returns the info hash,
    fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.clone()
    }

    /// Takes a vector of bencoded peers and returns a vector of bdecoded peers.
    fn get_peers(&self) -> Result<Vec<Peer>, TrackerError> {
        if let Decodification::List(peer_list) = &self.peers {
            let mut peers = Vec::new();
            for peer in peer_list.iter() {
                if let Decodification::Dic(peer_dict) = peer {
                    let peer_ip = match peer_dict.get(&from_string_to_vec("ip")) {
                        Some(Decodification::String(ip)) => ip,
                        _ => return Err(TrackerError::new("missing ip".to_string())),
                    };

                    let peer_port = match peer_dict.get(&from_string_to_vec("port")) {
                        Some(Decodification::Int(port)) => *port as u16,
                        _ => return Err(TrackerError::new("missing port".to_string())),
                    };

                    let peer = Peer::new(
                        from_vec_to_string(&from_string_to_vec("default_id")),
                        from_vec_to_string(peer_ip),
                        peer_port,
                    );
                    peers.push(peer);
                }
            }
            return Ok(peers);
        }
        Err(TrackerError::new("Expected List not found".to_string()))
    }
}

/// This function is used to request the tracker with the given info and info_hash.
fn request_tracker(
    info: HashMap<String, String>,
    info_hash: &[u8],
) -> Result<Decodification, TrackerError> {
    let mut stream = start_connection(info["URL"].clone(), info["port"].clone())?;

    let url = info["URL"].split("//").collect::<Vec<&str>>()[1]
        .split('/')
        .collect::<Vec<&str>>()[0];

    let request = format_request(info.clone(), info_hash, url);
    let response = write_and_read_stream(&mut stream, request)?;
    let d: &[u8] = response_splitter(response.as_ref());

    let response = bdecode(d)?;

    Ok(response)
}

/// Creates the TCP connection with the tracker.
fn start_connection(initial_url: String, _port: String) -> Result<TcpStream, TrackerError> {
    if initial_url.is_empty() {
        return Err(TrackerError::new("URL not found".to_string()));
    }
    let url = initial_url.split("//").collect::<Vec<&str>>()[1]
        .split('/')
        .collect::<Vec<&str>>()[0];

    let stream = TcpStream::connect(url)?;
    Ok(stream)
}

/// Returns a String with the formatted tracker request given the info data.
fn format_request(info: HashMap<String, String>, info_hash: &[u8], url: &str) -> String {
    let url_with_port = format!("{}:{}", url, info["port"]);
    let request = format!("GET /announce?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={}&numwant={} HTTP/1.1\r\nHost: {}\r\n\r\n",
                              to_urlencoded(info_hash),
                              info["peer_id"],
                              info["port"],
                              info["uploaded"],
                              info["downloaded"],
                              info["left"],
                              info["event"],
                              NUMBER_OF_PEERS_TO_ORDER,
                              url_with_port);
    request
}

// VERSION PARA PROBAR TRACKER:
// fn request_tracker(
//     info: HashMap<String, String>,
//     info_hash: &[u8],
// ) -> Result<Decodification, TrackerError> {
//     let mut stream = start_connection("http://localhost:8088/announce".to_string(), "8088".to_string())?;

//     let url = "http://localhost:8088/announce".split("//").collect::<Vec<&str>>()[1]
//         .split('/')
//         .collect::<Vec<&str>>()[0];

//     let request = format_request(info.clone(), info_hash, url);
//     let response = write_and_read_stream(&mut stream, request)?;
//     let d: &[u8] = response_splitter(response.as_ref());

//     let response = bdecode(d)?;

//     Ok(response)
// }

// // VERSION PARA PROBAR TRACKER:
// fn format_request(info: HashMap<String, String>, info_hash: &[u8], url: &str) -> String {
//     let url_with_port = format!("{}:{}", url, info["port"]);
//     let request = format!("GET /announce?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={}&numwant={} HTTP/1.1\r\nHost: {}\r\n\r\n",
//                               to_urlencoded(info_hash),
//                               info["peer_id"],
//                               8088,
//                               info["uploaded"],
//                               info["downloaded"],
//                               info["left"],
//                               info["event"],
//                               NUMBER_OF_PEERS_TO_ORDER,
//                               "localhost:8088");
//     request
// }

/// Writes the request on the stream and then reads and return the response.
fn write_and_read_stream(stream: &mut TcpStream, request: String) -> Result<Vec<u8>, TrackerError> {
    stream.write_all(request.as_bytes())?;
    let mut response = vec![];

    stream.read_to_end(&mut response)?;
    Ok(response)
}

/// Splits the response to avoid the html header.
fn response_splitter(response: &[u8]) -> &[u8] {
    let mut pos = 0;
    for i in 0..response.len() {
        if response[i] == b'\r'
            && response[i + 1] == b'\n'
            && response[i + 2] == b'\r'
            && response[i + 3] == b'\n'
        {
            break;
        }
        pos += 1;
    }
    &response[pos + 4..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::Logger;
    use std::sync::mpsc::channel;
    use std::thread::spawn;

    #[test]
    fn connection_successful() {
        let mut info = HashMap::new();
        info.insert(
            String::from("URL"),
            "http://bttracker.debian.org:6969/announce".to_string(),
        );
        info.insert(String::from("peer_id"), "12187165419728154321".to_string());
        info.insert(String::from("port"), format!("{}", 443));
        info.insert(String::from("uploaded"), format!("{}", 0));
        info.insert(String::from("downloaded"), format!("{}", 0));
        info.insert(String::from("left"), format!("{}", 0));
        info.insert(String::from("event"), "started".to_string());

        let info_hash = [
            177, 17, 129, 60, 230, 15, 66, 145, 151, 52, 130, 61, 245, 236, 32, 189, 30, 4, 231,
            247,
        ]
        .to_vec();

        let (sender, receiver) = channel();
        let mut logger = Logger::new(
            "src/test_files/logger_test_files/logs_tracker1.txt".to_string(),
            receiver,
        )
        .unwrap();
        spawn(move || logger.start());
        let tracker = Tracker::create(info, info_hash, sender).unwrap();

        let peers = tracker.get_peers().unwrap();
        assert_ne!(peers.len(), 0);
    }

    #[test]
    fn wrong_url() {
        let mut info = HashMap::new();
        info.insert(String::from("URL"), "".to_string());
        info.insert(String::from("peer_id"), "12187165419728154321".to_string());
        info.insert(String::from("port"), format!("{}", 443));
        info.insert(String::from("uploaded"), format!("{}", 0));
        info.insert(String::from("downloaded"), format!("{}", 0));
        info.insert(String::from("left"), format!("{}", 0));
        info.insert(String::from("event"), "started".to_string());

        let info_hash = [
            177, 17, 129, 60, 230, 15, 66, 145, 151, 52, 130, 61, 245, 236, 32, 189, 30, 4, 231,
            247,
        ]
        .to_vec();

        let (sender, _) = channel();
        assert!(Tracker::create(info, info_hash, sender).is_err());
    }

    #[test]
    fn empty_info_hash() {
        let mut info = HashMap::new();
        info.insert(
            String::from("URL"),
            "http://bttracker.debian.org:6969/announce".to_string(),
        );
        info.insert(String::from("peer_id"), "12187165419728154321".to_string());
        info.insert(String::from("port"), format!("{}", 443));
        info.insert(String::from("uploaded"), format!("{}", 0));
        info.insert(String::from("downloaded"), format!("{}", 0));
        info.insert(String::from("left"), format!("{}", 0));
        info.insert(String::from("event"), "started".to_string());

        let info_hash = [].to_vec();

        let (sender, _) = channel();
        assert!(Tracker::create(info, info_hash, sender).is_err());
    }

    #[test]
    fn start_connection_successful() {
        assert!(!start_connection(
            "http://bttracker.debian.org:6969/announce".to_string(),
            "443".to_string(),
        )
        .is_err());
    }

    #[test]

    fn write_and_read_stream_correctly() {
        //let mut stream22 = TlsStream::connect("https://torrent.ubuntu.com/announce").unwrap();

        let stream = start_connection(
            "http://bttracker.debian.org:6969/announce".to_string(),
            "443".to_string(),
        );

        assert!(!write_and_read_stream(&mut stream.unwrap(), format!("GET /announce?info_hash=%2ckhX%d6%1d%a9T%3dB1%a7%1d%b4%b1%c9%26K%06%85&peer_id=12187165419728154321&port=443&uploaded=0&downloaded=0&left=0&event=started HTTP/1.1\r\nHost: torrent.ubuntu.com:433:443\r\n\r\n")).is_err());
    }

    #[test]

    fn request_has_correct_format() {
        let mut info = HashMap::new();
        info.insert(
            String::from("URL"),
            "http://bttracker.debian.org:6969/announce".to_string(),
        );
        info.insert(String::from("peer_id"), "12187165419728154321".to_string());
        info.insert(String::from("port"), format!("{}", 443));
        info.insert(String::from("uploaded"), format!("{}", 0));
        info.insert(String::from("downloaded"), format!("{}", 0));
        info.insert(String::from("left"), format!("{}", 0));
        info.insert(String::from("event"), "started".to_string());

        let info_hash = [
            177, 17, 129, 60, 230, 15, 66, 145, 151, 52, 130, 61, 245, 236, 32, 189, 30, 4, 231,
            247,
        ]
        .to_vec();

        assert_eq!(format_request(info, &info_hash,"torrent.ubuntu.com:433"), format!("GET /announce?info_hash=%b1%11%81%3c%e6%0fB%91%974%82%3d%f5%ec%20%bd%1e%04%e7%f7&peer_id=12187165419728154321&port=443&uploaded=0&downloaded=0&left=0&event=started&numwant=100 HTTP/1.1\r\nHost: torrent.ubuntu.com:433:443\r\n\r\n"));
    }

    #[test]
    fn correct_info_hash_urlencoded() {
        let info_hash = [
            177, 17, 129, 60, 230, 15, 66, 145, 151, 52, 130, 61, 245, 236, 32, 189, 30, 4, 231,
            247,
        ]
        .to_vec();

        assert_eq!(
            to_urlencoded(&info_hash),
            "%b1%11%81%3c%e6%0fB%91%974%82%3d%f5%ec%20%bd%1e%04%e7%f7"
        );
    }
}
