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

pub use crate::parser::config_parse;
mod parser;

pub use crate::client::ClientError;

const CONFIG_PATH: &str = "src/config.yml";

fn main() -> Result<(), ClientError> {
    let config = match config_parse(CONFIG_PATH.to_string()) {
        Ok(config) => config,
        Err(_) => return Err(ClientError::ParserError),
    };

    let client: Client = Client::new(config).unwrap();
    client.start();
    Ok(())
}
