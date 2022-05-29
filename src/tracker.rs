use crate::bdecoder::{bdecode, Decodification};
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
}
// enum TrackerError
#[derive(Debug)]
pub enum TrackerError {
    ConnectionError,
    ResponseDecodeError,
    ExpectedDicNotFound,
}

impl Tracker {
    pub fn new(info: HashMap<String, String>, info_hash: Vec<u8>) -> Result<Tracker, TrackerError> {
        let response = match request_tracker(info, info_hash) {
            Ok(response) => response,
            Err(_) => return Err(TrackerError::ConnectionError),
        };
        if let Decodification::Dic(dic_aux) = response {
            let tracker = Tracker {
                interval: (*dic_aux.get("interval").unwrap()).clone(),
                complete: (*dic_aux.get("complete").unwrap()).clone(),
                incomplete: (*dic_aux.get("incomplete").unwrap()).clone(),
                peers: (*dic_aux.get("peers").unwrap()).clone(),
            };
            Ok(tracker)
        } else {
            Err(TrackerError::ExpectedDicNotFound)
        }
    }
}

fn request_tracker(
    info: HashMap<String, String>,
    info_hash: Vec<u8>,
) -> Result<Decodification, TrackerError> {
    // request tracker info
    let connector = TlsConnector::new().unwrap(); // FIXME

    // la url de info es: http://torrent.ubuntu.com/announce
    println!("URL:{:?}", info["URL"]);
    let url = info["URL"].split("//").collect::<Vec<&str>>()[1] //le saco el http
        .split('/')
        .collect::<Vec<&str>>()[0]; // le saco el announce
    println!("URL:{:?}", &url);
    let url_with_port = format!("{}:{}", url, info["port"]);

    println!("INFOHASH={:?}", info_hash);
    /* no tiene mucho sentido porq en string tira cualquiera
    print!("INFOHASH=");
    for i in &info_hash {
        // Imprimo la response
        print!("{}", *i as char);
    }
    println!();
    */
    let stream = TcpStream::connect(url_with_port.to_string()).unwrap(); // FIXME
    let mut stream = connector.connect(url, stream).unwrap(); // FIXME get domain from url
    let urlencoded_infohash = to_urlencoded(&info_hash);
    println!("INFO_ENCODED={}", &urlencoded_infohash);
    let request = format!("GET /announce?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={} HTTP/1.1\r\nHost: {}\r\n\r\n",
                            //   "%05%9b%89%a0%44%17%83%fe%34%32%39%0e%0a%7f%3f%78%44%5e%22%2e",
                            "%2c%6b%68%58%d6%1d%a9%54%3d%42%31%a7%1d%b4%b1%c9%26%4b%06%85",
                              info["peer_id"],
                              info["port"],
                              info["uploaded"],
                              info["downloaded"],
                              info["left"],
                              info["event"],
                              url_with_port);

    stream.write_all(request.as_bytes()).unwrap(); // FIXME
    let mut response = vec![];

    stream.read_to_end(&mut response).unwrap(); // FIXME
    let d: &[u8] = response_splitter(response.as_ref());

    println!("RESPONSE CRUDA");
    // println!("{:?}", d);
    for i in d {
        // Imprimo la response
        print!("{}", *i as char);
    }
    println!("\n --------------------------------------------------");
    let response = match bdecode(d) {
        Ok(response) => response,
        Err(_) => return Err(TrackerError::ResponseDecodeError),
    };
    println!("RESPONSE DECODIFICADA");
    println!("{:?}", response);

    Ok(response)
}

fn response_splitter(response: &[u8]) -> &[u8] {
    // return tracker response from the first \r\n\r\n
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
// transforms a slice of bytes into an url-encoded string
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
