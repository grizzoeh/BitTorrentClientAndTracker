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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_random_url() {
        assert!(get_torrents_paths("asdasdasdasd").is_err());
    }

    #[test]
    fn test_correct_url_returns_correct_number_of_paths() {
        let paths = get_torrents_paths("src/torrent_test_files").unwrap();
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_correct_url_returns_correct_paths() {
        let paths = get_torrents_paths("src/torrent_test_files").unwrap();
        let correct_paths = [
            "src/torrent_test_files/ubuntu-14.04.6-server-ppc64el.iso.torrent",
            "src/torrent_test_files/ubuntu-21.10-desktop-amd64.iso.torrent",
            "src/torrent_test_files/ubuntu-22.04-desktop-amd64.iso.torrent",
        ];

        for path in paths {
            assert!(correct_paths.contains(&&path.as_str()));
        }
    }
}
