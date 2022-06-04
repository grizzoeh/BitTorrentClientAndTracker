use crate::bdecoder::{bdecode, from_string_to_vec, from_vec_to_string, Decodification};
use crate::peer::Peer;
use crate::utils::to_urlencoded;
use native_tls::{TlsConnector, TlsStream};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;

#[derive(Debug, PartialEq)]
pub struct Tracker {
    pub interval: Decodification,
    pub complete: Decodification,
    pub incomplete: Decodification,
    pub peers: Decodification,
    pub info_hash: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum TrackerError {
    ConnectionError(String),
    ResponseDecodeError(String),
    ExpectedDicNotFound(String),
    FileNotReadable(String),
    FileNotWritable(String),
    IncompletePeer(String),
    InvalidPeerIndex(String),
}

impl Tracker {
    pub fn new(info: HashMap<String, String>, info_hash: Vec<u8>) -> Result<Tracker, TrackerError> {
        println!("CONNECTING WITH THE TRACKER");
        let response = match request_tracker(info, &info_hash) {
            Ok(response) => response,
            Err(err) => {
                return Err(TrackerError::ConnectionError(format!(
                    "Tracker Request Error. With Error: {:?}",
                    err
                )))
            }
        };
        println!("RESPONSE OBTAINED SUCCESSFULLY");

        if let Decodification::Dic(dic_aux) = response {
            let tracker = Tracker {
                interval: dic_aux[&from_string_to_vec("interval")].clone(),
                complete: dic_aux[&from_string_to_vec("complete")].clone(),
                incomplete: dic_aux[&from_string_to_vec("incomplete")].clone(),
                peers: dic_aux[&from_string_to_vec("peers")].clone(),
                info_hash,
            };
            Ok(tracker)
        } else {
            Err(TrackerError::ExpectedDicNotFound(
                "Expected Dic not found".to_string(),
            ))
        }
    }

    pub fn initialize_peer(&self, index: usize) -> Result<Peer, TrackerError> {
        if let Decodification::List(peer_list) = &self.peers {
            if index >= peer_list.len() {
                return Err(TrackerError::InvalidPeerIndex(
                    "Peer index out of range".to_string(),
                ));
            }
            if let Decodification::Dic(peer_dict) = &peer_list[index] {
                let peer_ip = match peer_dict.get(&from_string_to_vec("ip")) {
                    Some(Decodification::String(ip)) => ip,
                    _ => return Err(TrackerError::IncompletePeer("missing ip".to_string())),
                };
                let peer_port = match peer_dict.get(&from_string_to_vec("port")) {
                    Some(Decodification::Int(port)) => *port as u16,
                    _ => return Err(TrackerError::IncompletePeer("missing port".to_string())),
                };
                let peer_id = match peer_dict.get(&from_string_to_vec("peer id")) {
                    Some(Decodification::String(id)) => id,
                    _ => return Err(TrackerError::IncompletePeer("missing id".to_string())),
                };
                let peer = Peer::new(
                    from_vec_to_string(peer_id),
                    from_vec_to_string(peer_ip),
                    peer_port,
                );
                return Ok(peer);
            }
        }
        Err(TrackerError::IncompletePeer("Unexpected error".to_string()))
    }
}

fn request_tracker(
    info: HashMap<String, String>,
    info_hash: &[u8],
) -> Result<Decodification, TrackerError> {
    // Request tracker info
    let mut stream = match start_connection(info["URL"].clone(), info["port"].clone()) {
        Ok(stream) => stream,
        Err(err) => return Err(err),
    };

    let url = info["URL"].split("//").collect::<Vec<&str>>()[1] // Obtengo la URL sin el protocolo
        .split('/')
        .collect::<Vec<&str>>()[0];

    let request = format_request(info.clone(), info_hash, url);

    let response = match write_and_read_stream(&mut stream, request) {
        Ok(response) => response,
        Err(err) => return Err(err),
    };

    let d: &[u8] = response_splitter(response.as_ref());

    let response = match bdecode(d) {
        Ok(response) => response,
        Err(err) => {
            return Err(TrackerError::ResponseDecodeError(format!(
                "Response Decode Error. With Error: {:?}",
                err
            )));
        }
    };

    Ok(response)
}

fn start_connection(
    initial_url: String,
    port: String,
) -> Result<TlsStream<TcpStream>, TrackerError> {
    if initial_url.is_empty() {
        return Err(TrackerError::ConnectionError("URL not found".to_string()));
    }

    let connector = match TlsConnector::new() {
        Ok(connector) => connector,
        Err(_) => {
            return Err(TrackerError::ConnectionError(
                "TlsConnector Error".to_string(),
            ))
        }
    };
    let url = initial_url.split("//").collect::<Vec<&str>>()[1] // Obtengo la URL sin el protocolo
        .split('/')
        .collect::<Vec<&str>>()[0]; // Le saco el /announce
    let url_with_port = format!("{}:{}", url, port);

    let stream = match TcpStream::connect(url_with_port) {
        Ok(stream) => stream,
        Err(_) => return Err(TrackerError::ConnectionError("TcpStream Error".to_string())),
    };
    let stream = match connector.connect(url, stream) {
        Ok(stream) => stream,
        Err(_) => return Err(TrackerError::ConnectionError("TlsStream Error".to_string())),
    };

    Ok(stream)
}

fn format_request(info: HashMap<String, String>, info_hash: &[u8], url: &str) -> String {
    let url_with_port = format!("{}:{}", url, info["port"]);
    let request = format!("GET /announce?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={} HTTP/1.1\r\nHost: {}\r\n\r\n",
                              to_urlencoded(info_hash),
                              info["peer_id"],
                              info["port"],
                              info["uploaded"],
                              info["downloaded"],
                              info["left"],
                              info["event"],
                              url_with_port);
    request
}

fn write_and_read_stream(
    stream: &mut TlsStream<TcpStream>,
    request: String,
) -> Result<Vec<u8>, TrackerError> {
    let _i = match stream.write_all(request.as_bytes()) {
        Ok(i) => i,
        Err(_) => {
            return Err(TrackerError::FileNotWritable(
                "Stream Write Error".to_string(),
            ))
        }
    };
    let mut response = vec![];

    let _i = match stream.read_to_end(&mut response) {
        Ok(i) => i,
        Err(_) => {
            return Err(TrackerError::FileNotReadable(
                "Stream Read Error".to_string(),
            ))
        }
    };
    Ok(response)
}

fn response_splitter(response: &[u8]) -> &[u8] {
    // Return tracker response from the first \r\n\r\n
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

    #[test]
    fn connection_successful() {
        let mut info = HashMap::new();
        info.insert(
            String::from("URL"),
            "https://torrent.ubuntu.com/announce".to_string(),
        );
        info.insert(String::from("peer_id"), "12187165419728154321".to_string());
        info.insert(String::from("port"), format!("{}", 443));
        info.insert(String::from("uploaded"), format!("{}", 0));
        info.insert(String::from("downloaded"), format!("{}", 0));
        info.insert(String::from("left"), format!("{}", 0));
        info.insert(String::from("event"), "started".to_string());

        let info_hash = [
            44, 107, 104, 88, 214, 29, 169, 84, 61, 66, 49, 167, 29, 180, 177, 201, 38, 75, 6, 133,
        ]
        .to_vec();

        let tracker = Tracker::new(info, info_hash).unwrap();

        println!("{:?}", tracker);
        let peers = tracker.peers.clone();
        if let Decodification::List(peer_list) = peers {
            assert_ne!(peer_list.len(), 0);
        }
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
            44, 107, 104, 88, 214, 29, 169, 84, 61, 66, 49, 167, 29, 180, 177, 201, 38, 75, 6, 133,
        ]
        .to_vec();

        assert!(Tracker::new(info, info_hash).is_err());
    }

    #[test]
    fn empty_info_hash() {
        let mut info = HashMap::new();
        info.insert(
            String::from("URL"),
            "https://torrent.ubuntu.com/announce".to_string(),
        );
        info.insert(String::from("peer_id"), "12187165419728154321".to_string());
        info.insert(String::from("port"), format!("{}", 443));
        info.insert(String::from("uploaded"), format!("{}", 0));
        info.insert(String::from("downloaded"), format!("{}", 0));
        info.insert(String::from("left"), format!("{}", 0));
        info.insert(String::from("event"), "started".to_string());

        let info_hash = [].to_vec();

        assert!(Tracker::new(info, info_hash).is_err());
    }

    #[test]
    fn start_connection_successful() {
        assert!(!start_connection(
            "https://torrent.ubuntu.com/announce".to_string(),
            "443".to_string(),
        )
        .is_err());
    }

    #[test]

    fn write_and_read_stream_correctly() {
        //let mut stream22 = TlsStream::connect("https://torrent.ubuntu.com/announce").unwrap();

        let stream = start_connection(
            "https://torrent.ubuntu.com/announce".to_string(),
            "443".to_string(),
        );

        assert!(!write_and_read_stream(&mut stream.unwrap(), format!("GET /announce?info_hash=%2ckhX%d6%1d%a9T%3dB1%a7%1d%b4%b1%c9%26K%06%85&peer_id=12187165419728154321&port=443&uploaded=0&downloaded=0&left=0&event=started HTTP/1.1\r\nHost: torrent.ubuntu.com:433:443\r\n\r\n")).is_err());
    }

    #[test]

    fn request_has_correct_format() {
        let mut info = HashMap::new();
        info.insert(
            String::from("URL"),
            "https://torrent.ubuntu.com/announce".to_string(),
        );
        info.insert(String::from("peer_id"), "12187165419728154321".to_string());
        info.insert(String::from("port"), format!("{}", 443));
        info.insert(String::from("uploaded"), format!("{}", 0));
        info.insert(String::from("downloaded"), format!("{}", 0));
        info.insert(String::from("left"), format!("{}", 0));
        info.insert(String::from("event"), "started".to_string());

        let info_hash = [
            44, 107, 104, 88, 214, 29, 169, 84, 61, 66, 49, 167, 29, 180, 177, 201, 38, 75, 6, 133,
        ]
        .to_vec();

        assert_eq!(format_request(info, &info_hash,"torrent.ubuntu.com:433"), format!("GET /announce?info_hash=%2ckhX%d6%1d%a9T%3dB1%a7%1d%b4%b1%c9%26K%06%85&peer_id=12187165419728154321&port=443&uploaded=0&downloaded=0&left=0&event=started HTTP/1.1\r\nHost: torrent.ubuntu.com:433:443\r\n\r\n"));
    }

    #[test]
    fn correct_info_hash_urlencoded() {
        let info_hash = [
            44, 107, 104, 88, 214, 29, 169, 84, 61, 66, 49, 167, 29, 180, 177, 201, 38, 75, 6, 133,
        ]
        .to_vec();

        assert_eq!(
            to_urlencoded(&info_hash),
            "%2ckhX%d6%1d%a9T%3dB1%a7%1d%b4%b1%c9%26K%06%85"
        );
    }
}
