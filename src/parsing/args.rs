use crate::errors::args_error::ArgsError;
use std::fs::read_dir;

pub fn get_torrents_paths(torrent_dir: &str) -> Result<Vec<String>, ArgsError> {
    //! Returns a String Vector with the filenames into given directory.
    //! If there is any issue it returns an specific Error.
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_random_url() {
        assert!(get_torrents_paths("asdasdasdasd").is_err());
    }

    #[test]
    fn test_correct_url_returns_correct_number_of_paths() {
        let paths = get_torrents_paths("src/test_files/torrent_test_files").unwrap();
        assert!(paths.len() >= 1);
    }
}
