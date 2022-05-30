use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound,
    FileNotReadable,
    ExpectedFieldNotFound,
}

pub fn config_parse(filename: String) -> Result<HashMap<String, String>, ConfigError> {
    let cfgfile = File::open(&filename);
    let cfgfile = match cfgfile {
        Ok(cfgfile) => cfgfile,
        Err(_) => return Err(ConfigError::FileNotFound),
    };
    let mut cfg = HashMap::new();

    let cfgfile = BufReader::new(cfgfile);
    for line in cfgfile.lines() {
        let line = match line {
            Ok(line) => line.replace(' ', "").replace('"', ""),
            Err(_) => {
                return Err(ConfigError::FileNotReadable);
            }
        };
        let index = line.find(':').unwrap_or(0);
        if index == 0 {
            continue;
        }
        let key = line.get(0..index);
        let key: &str = match key {
            Some(key) => key,
            None => "",
        };

        let val = line.get(index + 1..);
        let val: &str = match val {
            Some(val) => val,
            None => "",
        };

        cfg.insert(key.to_string(), val.to_string());
    }

    if cfg.keys().len() == 0 {
        return Err(ConfigError::ExpectedFieldNotFound);
    }

    for v in cfg.values() {
        if v.is_empty() {
            return Err(ConfigError::ExpectedFieldNotFound);
        }
    }
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_parser() {
        let filename = String::from("src/config_test_files/config_empty.yml");
        let cfg = config_parse(filename);
        assert!(cfg.is_err());
    }

    #[test]
    fn test_right_parser() {
        let filename = String::from("src/config.yml");
        let cfg = config_parse(filename);
        let right = HashMap::from([
            ("port".to_string(), "443".to_string()),
            ("log_path".to_string(), "reports/logs".to_string()),
            (
                "download_path".to_string(),
                "src/downloads/download.txt".to_string(),
            ),
            (
                "torrent_path".to_string(),
                "src/torrent_test_files/ubuntu-22.04-desktop-amd64.iso.torrent".to_string(),
            ),
            ("log_level".to_string(), "5".to_string()),
        ]);
        assert_eq!(cfg.unwrap(), right);
    }
}
