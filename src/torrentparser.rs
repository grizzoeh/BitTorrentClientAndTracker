use crate::bdecoder::{bdecode, from_string_to_vec, from_vec_to_string, Decodification};
use crate::bencoder::{bencode, BencoderTypes};
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{prelude::*, BufReader};

#[derive(Debug)]
pub enum TorrentError {
    FileNotFound,
    FileNotReadable,
    ExpectedFieldNotFound,
    DecodeError,
    HashError,
    ExpectedInfoHashmapNotFound,
}

pub fn torrent_parse(filename: &str) -> Result<(String, Vec<u8>), TorrentError> {
    let torrentfile = File::open(&filename);
    let torrentfile = match torrentfile {
        Ok(torrentfile) => torrentfile,
        Err(_) => return Err(TorrentError::FileNotFound),
    };

    let mut data = (String::new(), Vec::new()); // atado con alambre
    let mut torrentfile = BufReader::new(torrentfile);
    let mut torrent_vec = Vec::new();

    let _i = match torrentfile.read_to_end(&mut torrent_vec) {
        Ok(i) => i,
        Err(_) => return Err(TorrentError::FileNotReadable),
    };

    let torrent = &torrent_vec;
    let decoded = bdecode(torrent);
    let decoded = match decoded {
        Ok(decoded) => decoded,
        Err(_) => {
            return Err(TorrentError::DecodeError);
        }
    };
    if let Decodification::Dic(hashmap_aux) = decoded {
        if let Decodification::String(str_aux) = &hashmap_aux[&from_string_to_vec("announce")] {
            data.0 = from_vec_to_string(str_aux);
        }

        if let Decodification::Dic(_) = &hashmap_aux[&from_string_to_vec("info")] {
            let encoded = bencode(&BencoderTypes::Decodification(
                hashmap_aux[&from_string_to_vec("info")].clone(),
            ));
            let mut hasher = Sha1::new();
            hasher.update(encoded);
            let hash = hasher.finalize()[..].to_vec();
            data.1 = hash;
        } else {
            return Err(TorrentError::ExpectedInfoHashmapNotFound);
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
            decoded.unwrap().0,
            "http://torrent.ubuntu.com:6969/announce"
        );
    }

    #[test]
    fn test_vector_to_string() {
        let vec = vec![b'a', b'b', b'c'];
        let str = from_vec_to_string(&vec);
        assert_eq!(str, "abc");
    }
    /*
    #[test]
    fn test_info_ubuntu_torrent() {
        let torrentfile = File::open(String::from(
            "src/torrent_test_files/ubuntu-14.04.6-server-ppc64el.iso.torrent",
        ));
        let torrentfile = match torrentfile {
            Ok(torrentfile) => torrentfile,
            Err(_) => panic!("File not found"),
        };
        let mut torrentfile = BufReader::new(torrentfile);
        let mut torrent_vec = Vec::new();
        torrentfile.read_to_end(&mut torrent_vec).unwrap();
        let torrent = &torrent_vec;
        let decoded = bdecode(torrent);
        let decoded = match decoded {
            Ok(decoded) => decoded,
            Err(_) => panic!("File not found"),
        };
        if let Decodification::Dic(hashmap_aux) = decoded {
            if let Decodification::Dic(_) = &hashmap_aux["info"] {
                let encoded = bencode(&BencoderTypes::Decodification(hashmap_aux["info"].clone()));
                //le aplico SHA1 al valor de "info"
                let mut hasher = Sha1::new();
                hasher.update(encoded);
                //let hash = hasher.finalize()[..];
                assert_eq!(hasher.finalize()[..], hex!("11"))
            }
        }
    }
    */
}
