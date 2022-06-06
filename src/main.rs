extern crate native_tls;
use crabrave::args::get_torrents_paths;
use crabrave::client::Client;
use crabrave::config_parser::config_parse;
use std::env;

const CONFIG_PATH: &str = "src/config.yml";

#[derive(Debug)]
pub enum ArgsError {
    IncorrectNumberOfArgs,
    TorrentFileError,
    ConfigParseError,
    ClientError,
}

fn main() -> Result<(), ArgsError> {
    initialize_app()
}

fn initialize_app() -> Result<(), ArgsError> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if (args.len() < 2) || (args.len() > 2) {
        return Err(ArgsError::IncorrectNumberOfArgs);
    }

    let torrent_dir = match args.get(1) {
        Some(dir) => dir,
        None => return Err(ArgsError::IncorrectNumberOfArgs),
    };

    // read file from directory
    let torrent_paths: Vec<String> = match get_torrents_paths(torrent_dir) {
        Ok(paths) => paths,
        Err(_) => {
            return Err(ArgsError::TorrentFileError);
        }
    };

    let torrent_path = &torrent_paths[0];

    let mut config = match config_parse(CONFIG_PATH.to_string()) {
        Ok(config) => config,
        Err(_) => return Err(ArgsError::ConfigParseError),
    };
    config.insert("torrent_path".to_string(), torrent_path.clone());
    let client: Client = match Client::new(config) {
        Ok(client) => client,
        Err(_) => return Err(ArgsError::ClientError),
    };
    client.start();
    Ok(())
}
