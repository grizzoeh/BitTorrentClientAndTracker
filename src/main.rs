extern crate native_tls;
use crabrave::args::get_torrents_paths;
use crabrave::client::Client;
use crabrave::client::ClientError;
use crabrave::parser::config_parse;
use std::env;
const CONFIG_PATH: &str = "src/config.yml";

fn main() -> Result<(), ClientError> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let torrent_dir = args.get(1).unwrap();
    // read file from directory
    let torrent_paths: Vec<String> = match get_torrents_paths(torrent_dir) {
        Ok(paths) => paths,
        Err(_) => {
            return Err(ClientError::TorrentParseFileNotFound); // FIXME agregarle mensajes de error
        }
    };

    let torrent_path = &torrent_paths[0];

    let mut config = match config_parse(CONFIG_PATH.to_string()) {
        Ok(config) => config,
        Err(_) => return Err(ClientError::ParserError),
    };
    config.insert("torrent_path".to_string(), torrent_path.clone());
    let client: Client = Client::new(config).unwrap();
    client.start();
    Ok(())
}
