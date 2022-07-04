use crate::bdecoder::{bdecode, from_string_to_vec, Decodification};
use crate::bencoder::{bencode, BencoderTypes};
use crate::errors::torrent_parser_error::TorrentParserError;
use crate::utils::i64_to_vecu8;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};

pub fn torrent_parse(filename: &str) -> Result<HashMap<String, Vec<u8>>, TorrentParserError> {
    let torrentfile = File::open(&filename);
    let torrentfile = torrentfile?;

    let mut torrentfile = BufReader::new(torrentfile);
    let mut torrent_vec = Vec::new();

    torrentfile.read_to_end(&mut torrent_vec)?;
    let torrent = &torrent_vec;
    let decoded = bdecode(torrent)?;

    get_torrent_info(&decoded)
}

fn get_torrent_info(
    decoded: &Decodification,
) -> Result<HashMap<String, Vec<u8>>, TorrentParserError> {
    let mut data: HashMap<String, Vec<u8>> = HashMap::new();
    if let Decodification::Dic(hashmap_aux) = decoded {
        if let Decodification::String(str_aux) = &hashmap_aux[&from_string_to_vec("announce")] {
            data.insert("url".to_string(), str_aux.clone());
        }

        if let Decodification::Dic(info_hashmap) = &hashmap_aux[&from_string_to_vec("info")] {
            let encoded = bencode(&BencoderTypes::Decodification(
                hashmap_aux[&from_string_to_vec("info")].clone(),
            ));
            let mut hasher = Sha1::new();
            hasher.update(encoded);
            let hash = hasher.finalize()[..].to_vec();
            data.insert("info_hash".to_string(), hash);

            if let Decodification::Int(piece_length) =
                &info_hashmap[&from_string_to_vec("piece length")]
            {
                data.insert(
                    "piece length".to_string(),
                    i64_to_vecu8(piece_length).to_vec(),
                );
            }
            if let Decodification::String(str_aux2) = &info_hashmap[&from_string_to_vec("pieces")] {
                data.insert("pieces".to_string(), str_aux2.clone());
            }
            if let Decodification::String(str_aux3) = &info_hashmap[&from_string_to_vec("name")] {
                data.insert("name".to_string(), str_aux3.clone());
            }
            if let Decodification::Int(lenght) = &info_hashmap[&from_string_to_vec("length")] {
                data.insert("length".to_string(), i64_to_vecu8(lenght).to_vec());
            }
        } else {
            return Err(TorrentParserError::new(
                "Info hash_map is not of type Decodification".to_string(),
            ));
        }
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_announce_ubuntu_torrent() {
        let filename =
            String::from("src/torrent_test_files/ubuntu-14.04.6-server-ppc64el.iso.torrent");
        let decoded = torrent_parse(&filename);

        assert_eq!(
            String::from_utf8(decoded.unwrap()["url"].clone()).unwrap(),
            "http://torrent.ubuntu.com:6969/announce"
        );
    }

    #[test]
    fn test_announce_ubuntu_torrent2() {
        let filename =
            String::from("src/torrent_test_files/ubuntu-21.10-desktop-amd64.iso.torrent");
        let decoded = torrent_parse(&filename);

        assert_eq!(
            String::from_utf8(decoded.unwrap()["url"].clone()).unwrap(),
            "https://torrent.ubuntu.com/announce"
        );
    }
}
