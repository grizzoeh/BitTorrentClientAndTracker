use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader, Error};

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
    let cfgfile = BufReader::new(cfgfile);
    let mut cfg = HashMap::new();
    for line in cfgfile.lines() {
        cfg = match process_line(line, &mut cfg) {
            Ok(cfg) => cfg,
            Err(_) => return Err(ConfigError::FileNotReadable),
        };
    }

    if cfg.keys().len() == 0 {
        return Err(ConfigError::ExpectedFieldNotFound);
    }

    Ok(cfg)
}

fn process_line(
    line: Result<String, Error>,
    cfg: &mut HashMap<String, String>,
) -> Result<HashMap<String, String>, ConfigError> {
    let line = match line {
        Ok(line) => line.replace(' ', "").replace('"', ""),
        Err(_) => {
            return Err(ConfigError::FileNotReadable);
        }
    };
    let index = line.find(':').unwrap_or(0);

    if index == 0 {
        return Ok(cfg.clone());
    }

    let key = line.get(0..index);
    let key: &str = match key {
        Some(key) => key,
        None => "",
    };

    let val = line.get(index + 1..);
    let val: &str = match val {
        Some(val) => val,
        None => return Err(ConfigError::ExpectedFieldNotFound),
    };

    cfg.insert(key.to_string(), val.to_string());
    Ok(cfg.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrong_path() {
        let filename = String::from("src/config_test_files/this_does_not_exist.yml");
        let cfg = config_parse(filename);
        assert!(cfg.is_err());
    }

    #[test]
    fn test_empty_parser() {
        let filename = String::from("src/config_test_files/config_empty.yml");
        let cfg = config_parse(filename);
        assert!(cfg.is_err());
    }

    #[test]
    fn test_incomplete_file() {
        let filename = String::from("src/config_test_files/incomplete_file.yml");
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
            ("download_path".to_string(), "src/downloads/".to_string()),
            ("log_level".to_string(), "5".to_string()),
        ]);
        assert_eq!(cfg.unwrap(), right);
    }
}
