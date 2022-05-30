#[derive(Debug)]
pub enum ArgsError {
    NoArgument,
    TooManyArguments,
    InvalidArgument(String),
}

pub fn get_torrents_paths(torrent_dir: &str) -> Result<Vec<String>, ArgsError> {
    let mut torrent_paths: Vec<String> = Vec::new();
    let torrents_paths = match std::fs::read_dir(torrent_dir) {
        Ok(paths) => paths,
        Err(paths) => return Err(ArgsError::InvalidArgument(paths.to_string())),
    };
    for file in torrents_paths {
        let file = match file {
            // unwrap file
            Ok(file) => file.path(),
            Err(file) => return Err(ArgsError::InvalidArgument(file.to_string())),
        };
        let file = match file.to_str() {
            // unwrap file to string
            Some(file) => file.to_string(),
            None => return Err(ArgsError::InvalidArgument("Invalid path".to_string())),
        };
        torrent_paths.push(file);
    }
    Ok(torrent_paths)
}
