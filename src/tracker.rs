use crate::bdecoder::{bdecode, from_string_to_vec, Decodification};
use native_tls::TlsConnector;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
#[derive(Debug)]
pub struct Tracker {
    pub interval: Decodification,
    pub complete: Decodification,
    pub incomplete: Decodification,
    pub peers: Decodification,
    pub info_hash: Vec<u8>,
}
// enum TrackerError
#[derive(Debug)]
pub enum TrackerError {
    ConnectionError,
    ResponseDecodeError,
    ExpectedDicNotFound,
    FileNotReadable,
    FileNotWritable,
}

impl Tracker {
    pub fn new(info: HashMap<String, String>, info_hash: Vec<u8>) -> Result<Tracker, TrackerError> {
        println!("CONNECTING WITH THE TRACKER");
        let response = match request_tracker(info, &info_hash) {
            Ok(response) => response,
            Err(_) => return Err(TrackerError::ConnectionError),
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
            Err(TrackerError::ExpectedDicNotFound)
        }
    }
}

fn request_tracker(
    info: HashMap<String, String>,
    info_hash: &[u8],
) -> Result<Decodification, TrackerError> {
    // Request tracker info
    let connector = match TlsConnector::new() {
        Ok(connector) => connector,
        Err(_) => return Err(TrackerError::ConnectionError),
    };

    let url = info["URL"].split("//").collect::<Vec<&str>>()[1] // Obtengo la URL sin el protocolo
        .split('/')
        .collect::<Vec<&str>>()[0]; // Le saco el /announce
    let url_with_port = format!("{}:{}", url, info["port"]);

    let stream = match TcpStream::connect(url_with_port.to_string()) {
        Ok(stream) => stream,
        Err(_) => return Err(TrackerError::ConnectionError),
    };
    let mut stream = match connector.connect(url, stream) {
        Ok(stream) => stream,
        Err(_) => return Err(TrackerError::ConnectionError),
    };
    let request = format!("GET /announce?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={} HTTP/1.1\r\nHost: {}\r\n\r\n",
                              to_urlencoded(info_hash),
                              info["peer_id"],
                              info["port"],
                              info["uploaded"],
                              info["downloaded"],
                              info["left"],
                              info["event"],
                              url_with_port);

    let _i = match stream.write_all(request.as_bytes()) {
        Ok(i) => i,
        Err(_) => return Err(TrackerError::FileNotWritable),
    };
    let mut response = vec![];

    let _i = match stream.read_to_end(&mut response) {
        Ok(i) => i,
        Err(_) => return Err(TrackerError::FileNotReadable),
    };
    let d: &[u8] = response_splitter(response.as_ref());

    //FAILS
    let response = match bdecode(d) {
        Ok(response) => response,
        Err(_) => return Err(TrackerError::ResponseDecodeError),
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

// Transforms a slice of bytes into an url-encoded string
fn to_urlencoded(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| {
            if b.is_ascii_alphanumeric() || *b == b'.' || *b == b'-' || *b == b'_' || *b == b'~' {
                String::from(*b as char)
            } else {
                format!("%{:02x}", *b)
            }
        })
        .collect()
}
