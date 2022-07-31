use crate::{bdecoder::Decodification, errors::announce_error::AnnounceError, peer::Peer};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum URLParams {
    String(String),
    Vector(Vec<u8>),
}

/// Returns a Hashmap of Strings with the announce URL fields as URLParams.
pub fn parse_announce(announce_url: String) -> Result<HashMap<String, URLParams>, AnnounceError> {
    let mut hashmap_valores_announce: HashMap<String, URLParams> = HashMap::new();

    let url_split = announce_url.split('?').collect::<Vec<&str>>();
    if url_split.len() == 1 {
        return Err(AnnounceError::new(
            "There must be params in the announce url".to_string(),
        ));
    }

    let url_split_aux = url_split[1].split(" HTTP").collect::<Vec<&str>>();

    let relevant_parameters = url_split_aux[0];
    let parameters_split = relevant_parameters.split('&').collect::<Vec<&str>>();

    for parameter in parameters_split {
        let parameter_split = parameter.split('=').collect::<Vec<&str>>();

        let key = parameter_split[0];
        let value = parameter_split[1];

        if key == "info_hash" {
            hashmap_valores_announce
                .insert(key.to_string(), URLParams::Vector(decode(value.as_bytes())));
        } else {
            hashmap_valores_announce.insert(key.to_string(), URLParams::String(value.to_string()));
        }
    }

    if !hashmap_valores_announce.contains_key("event") {
        hashmap_valores_announce.insert(
            "event".to_string(),
            URLParams::String("started".to_string()),
        );
    }
    check_contains_mandatory_key(hashmap_valores_announce.clone())?;

    Ok(hashmap_valores_announce)
}

/// Given the HashMap of the URL announce fields, returns an error if a mandatory key is not there.Returns a Hashmap of Strings with the announce URL fields as URLParams.
fn check_contains_mandatory_key(hash_map: HashMap<String, URLParams>) -> Result<(), AnnounceError> {
    if !hash_map.contains_key("info_hash")
        || !hash_map.contains_key("peer_id")
        || !hash_map.contains_key("port")
        || !hash_map.contains_key("uploaded")
        || !hash_map.contains_key("downloaded")
        || !hash_map.contains_key("left")
        || !hash_map.contains_key("event")
    {
        return Err(AnnounceError::new(
            "The announce url must contain info_hash, peer_id, port, uploaded, downloaded, left and event".to_string(),
        ));
    }
    Ok(())
}

/// Returns a Vec<u8> decoded.
fn decode(bytes: &[u8]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();
    let mut bytes = bytes.iter();

    loop {
        match bytes.next() {
            None => break,
            Some(b) => match *b {
                b'%' => {
                    let mut new_byte: u8 = 0;

                    match bytes.next() {
                        None => continue,
                        Some(x) => {
                            let x = *x as char;
                            if x.is_ascii_hexdigit() {
                                let x = match x.to_digit(16) {
                                    None => {
                                        continue;
                                    }
                                    Some(x) => x,
                                };
                                new_byte += (x * 0x10) as u8;
                            } else {
                                continue;
                            }
                        }
                    }

                    match bytes.next() {
                        None => {
                            continue;
                        }
                        Some(x) => {
                            let x = *x as char;
                            if x.is_ascii_hexdigit() {
                                let x = match x.to_digit(16) {
                                    None => {
                                        continue;
                                    }
                                    Some(x) => x,
                                };
                                new_byte += x as u8;
                            } else {
                                continue;
                            }
                        }
                    }

                    output.push(new_byte);
                }
                b'+' => output.push(b' '),
                _ => output.push(*b),
            },
        }
    }
    output
}

/// Converts hexadecimal string to a vector of u8.
pub fn hex_to_vecu8(hex: &str) -> Result<Vec<u8>, AnnounceError> {
    let mut bytes = Vec::new();
    for i in 0..hex.len() / 2 {
        let s = &hex[i * 2..i * 2 + 2];
        let n = u8::from_str_radix(s, 16)?;
        bytes.push(n);
    }
    Ok(bytes)
}

/// Converts integer to hexadecimal string.
pub fn int_to_hex(i: u32) -> String {
    format!("{:x}", i)
}

/// Converts a Decodification hashmap to a vector of peers ( only used in tests ).
pub fn decode_peer_list(current_info_hash: Vec<u8>, peers_decoded: Decodification) -> Vec<Peer> {
    let mut peers = Vec::new();
    if let Decodification::Dic(mut peer_dic) = peers_decoded {
        if let Decodification::List(peer_list) =
            peer_dic.get_mut(&"peers".as_bytes().to_vec()).unwrap()
        {
            for peer in peer_list.iter() {
                if let Decodification::Dic(peer_dict) = peer {
                    let peer_ip = match peer_dict.get(&"ip".as_bytes().to_vec()) {
                        Some(Decodification::String(ip)) => ip,
                        _ => panic!("ip not found"),
                    };

                    let peer_id = match peer_dict.get(&"id".as_bytes().to_vec()) {
                        Some(Decodification::String(id)) => id,
                        _ => panic!("id not found"),
                    };

                    let peer_port = match peer_dict.get(&"port".as_bytes().to_vec()) {
                        Some(Decodification::Int(port)) => *port as u16,
                        _ => panic!("port not found"),
                    };

                    peers.push(Peer::new(
                        peer_port.to_string(),
                        String::from_utf8(peer_id.to_vec()).unwrap(),
                        String::from_utf8(peer_ip.to_vec()).unwrap(),
                        current_info_hash.clone(),
                        "0".to_string(),
                        "0".to_string(),
                        "0".to_string(),
                        true,
                        true,
                    ));
                }
            }
        }
    }
    peers
}

/// Splits the response to avoid the html header ( only used in tests ).
pub fn response_splitter(response: &[u8]) -> &[u8] {
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
    fn test_parser_port() {
        let result = parse_announce("GET /announce?info_hash=%b1%11%81%3c%e6%0fB%91%974%82%3d%f5%ec%20%bd%1e%04%e7%f7&peer_id=55681657836697543210&port=443&uploaded=0&downloaded=0&left=0&event=started&numwant=100%20HTTP/1.1\r\n".to_string());

        assert_eq!(
            result.unwrap()["port"],
            URLParams::String("443".to_string())
        );
    }

    #[test]
    fn test_parser_info_hash() {
        let result = parse_announce("GET /announce?info_hash=%b1%11%81%3c%e6%0fB%91%974%82%3d%f5%ec%20%bd%1e%04%e7%f7&peer_id=55681657836697543210&port=443&uploaded=0&downloaded=0&left=0&event=started&numwant=100%20HTTP/1.1\r\n".to_string());
        assert_eq!(
            result.unwrap()["info_hash"],
            URLParams::Vector(vec![
                177, 17, 129, 60, 230, 15, 66, 145, 151, 52, 130, 61, 245, 236, 32, 189, 30, 4,
                231, 247
            ])
        );
    }

    #[test]
    fn test_parser_last_parameter_ok() {
        let result = parse_announce("GET /announce?info_hash=%b1%11%81%3c%e6%0fB%91%974%82%3d%f5%ec%20%bd%1e%04%e7%f7&peer_id=55681657836697543210&port=443&uploaded=0&downloaded=0&left=0&event=started&numwant=100 HTTP/1.1\r\n".to_string());

        assert_eq!(
            result.unwrap()["numwant"],
            URLParams::String("100".to_string())
        );
    }

    #[test]
    fn test_contains_mandatory_key_error() {
        let hash_map = HashMap::from([
            ("info_hash".to_string(), URLParams::String("".to_string())),
            ("peer_id".to_string(), URLParams::String("".to_string())),
            ("port".to_string(), URLParams::String("".to_string())),
            ("uploaded".to_string(), URLParams::String("".to_string())),
            ("downloaded".to_string(), URLParams::String("".to_string())),
            ("left".to_string(), URLParams::String("".to_string())),
        ]);

        assert!(check_contains_mandatory_key(hash_map).is_err());
    }

    #[test]
    fn test_contains_mandatory_key_ok() {
        let hash_map = HashMap::from([
            ("info_hash".to_string(), URLParams::String("".to_string())),
            ("peer_id".to_string(), URLParams::String("".to_string())),
            ("port".to_string(), URLParams::String("".to_string())),
            ("uploaded".to_string(), URLParams::String("".to_string())),
            ("downloaded".to_string(), URLParams::String("".to_string())),
            ("left".to_string(), URLParams::String("".to_string())),
            ("event".to_string(), URLParams::String("".to_string())),
        ]);

        assert!(check_contains_mandatory_key(hash_map).is_ok());
    }

    #[test]
    fn test_int_to_hex() {
        assert_eq!(int_to_hex(0), "0");
        assert_eq!(int_to_hex(1), "1");
        assert_eq!(int_to_hex(16), "10");
        assert_eq!(int_to_hex(255), "ff");
        assert_eq!(int_to_hex(256), "100");
        assert_eq!(int_to_hex(65535), "ffff");
        assert_eq!(int_to_hex(65536), "10000");
        assert_eq!(int_to_hex(16777215), "ffffff");
        assert_eq!(int_to_hex(16777216), "1000000");
        assert_eq!(int_to_hex(4294967295), "ffffffff");
    }
}
