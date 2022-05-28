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
    let mut cfg = HashMap::from([
        ("port".to_string(), "".to_string()),
        ("log_path".to_string(), "".to_string()),
        ("download_path".to_string(), "".to_string()),
        ("log_level".to_string(), "1".to_string()),
    ]);

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

        if !cfg.contains_key(key) {
            continue;
        }
        cfg.insert(key.to_string(), val.to_string());
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
        let filename = String::from("src/config_test_files/config.yml");
        let cfg = config_parse(filename);
        let right = HashMap::from([
            ("port".to_string(), "7001".to_string()),
            ("log_path".to_string(), "reports/logs".to_string()),
            ("download_path".to_string(), "reports/docs".to_string()),
            ("log_level".to_string(), "5".to_string()),
        ]);
        assert_eq!(cfg.unwrap(), right);
    }

    #[test]
    fn test_duplicate_parser() {
        let filename = String::from("src/config_test_files/config_duplicate.yml");
        let cfg = config_parse(filename);
        let right = HashMap::from([
            ("port".to_string(), "8008".to_string()),
            ("log_path".to_string(), "reports/logs".to_string()),
            ("download_path".to_string(), "reports/docs".to_string()),
            ("log_level".to_string(), "1".to_string()),
        ]);
        assert_eq!(cfg.unwrap(), right);
    }

    #[test]
    fn test_missing_parser() {
        let filename = String::from("src/config_test_files/config_missing.yml");
        let cfg = config_parse(filename);
        assert!(cfg.is_err());
    }

    #[test]
    fn test_missing_duplicate_parser() {
        let filename = String::from("src/config_test_files/config_missing_duplicate.yml");
        let cfg = config_parse(filename);
        assert!(cfg.is_err());
    }
    #[test]
    fn test_extra_parser() {
        let filename = String::from("src/config_test_files/config_extra.yml");
        let cfg = config_parse(filename);
        let right = HashMap::from([
            ("port".to_string(), "7001".to_string()),
            ("log_path".to_string(), "reports/logs".to_string()),
            ("download_path".to_string(), "reports/docs".to_string()),
            ("log_level".to_string(), "5".to_string()),
        ]);
        assert_eq!(cfg.unwrap(), right);
    }
}
