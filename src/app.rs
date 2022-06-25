use crate::args::get_torrents_paths;
use crate::client::Client;
use crate::config_parser::config_parse;
use crate::errors::app_error::AppError;
use std::env;
use std::sync::Arc;
const CONFIG_PATH: &str = "src/config.yml";

pub fn initialize_app() -> Result<(), AppError> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if (args.len() < 2) || (args.len() > 2) {
        return Err(AppError::new("Incorrect number of arguments".to_string()));
    }

    let torrent_dir = match args.get(1) {
        Some(dir) => dir,
        None => return Err(AppError::new("Incorrect number of arguments".to_string())),
    };

    // read file from directory
    let torrent_paths: Vec<String> = get_torrents_paths(torrent_dir)?;

    let torrent_path = &torrent_paths[0];

    let mut config = config_parse(CONFIG_PATH.to_string())?;
    config.insert("torrent_path".to_string(), torrent_path.clone());
    let client: Arc<Client> = Client::new(config)?;
    let _ = client.start();
    Ok(())
}
