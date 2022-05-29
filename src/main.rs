extern crate native_tls;
pub use crate::client::Client;
mod client;

pub use crate::peer::Peer;
mod peer;

pub use crate::tracker::Tracker;
mod tracker;

pub use crate::bdecoder::bdecode;
mod bdecoder;

pub use crate::bencoder::bencode;
mod bencoder;

pub use crate::torrentparser::torrent_parse;
mod torrentparser;

fn main() {
    let torrent_path: String =
        String::from("src/torrent_test_files/ubuntu-22.04-desktop-amd64.iso.torrent");
    let download_path: String = String::from("src/downloads/download.txt");

    let _: Client = Client::new(torrent_path, download_path).unwrap();
}
