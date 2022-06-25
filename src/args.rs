use crate::errors::args_error::ArgsError;
use std::fs::read_dir;

pub fn get_torrents_paths(torrent_dir: &str) -> Result<Vec<String>, ArgsError> {
    let mut torrent_paths: Vec<String> = Vec::new();
    let torrents_paths = read_dir(torrent_dir)?;
    for file in torrents_paths {
        let file = file?.path();
        let file = match file.to_str() {
            Some(file) => file.to_string(),
            None => continue,
        };
        torrent_paths.push(file);
    }
    Ok(torrent_paths)
}
