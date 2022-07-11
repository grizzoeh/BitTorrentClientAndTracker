use crate::errors::config_parser_error::ConfigParserError;
use std::{
    collections::HashMap,
    fs::File,
    io::{prelude::*, BufReader, Error},
};

/// Returns a Hashmap of Strings with the configuration fields given a configuration file.
pub fn config_parse(filename: String) -> Result<HashMap<String, String>, ConfigParserError> {
    let cfgfile = File::open(&filename);
    let cfgfile = cfgfile?;
    let cfgfile = BufReader::new(cfgfile);
    let mut cfg = HashMap::new();
    for line in cfgfile.lines() {
        process_line(line, &mut cfg)?;
    }

    if cfg.keys().len() == 0 {
        return Err(ConfigParserError::new());
    }

    Ok(cfg)
}

/// Reads a line and if there is a key and value, it adds them to the HashMap.
fn process_line(
    line: Result<String, Error>,
    cfg: &mut HashMap<String, String>,
) -> Result<(), ConfigParserError> {
    let line = line?.replace(' ', "").replace('"', "");
    let index = line.find(':').unwrap_or(0);

    if index == 0 {
        return Ok(());
    }

    let key = line.get(0..index);
    let key = key.unwrap_or("");

    let val = line.get(index + 1..);
    let val = match val {
        Some(val) => val,
        None => return Err(ConfigParserError::new()),
    };

    cfg.insert(key.to_string(), val.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrong_path() {
        let filename = String::from("src/test_files/config_test_files/this_does_not_exist.yml");
        let cfg = config_parse(filename);
        assert!(cfg.is_err());
    }

    #[test]
    fn test_empty_parser() {
        let filename = String::from("src/test_files/config_test_files/config_empty.yml");
        let cfg = config_parse(filename);
        assert!(cfg.is_err());
    }

    #[test]
    fn test_incomplete_file() {
        let filename = String::from("src/test_files/config_test_files/incomplete_file.yml");
        let cfg = config_parse(filename);
        assert!(cfg.is_err());
    }

    #[test]
    fn test_right_parser() {
        let filename = String::from("src/config.yml");
        let cfg = config_parse(filename);
        let right = HashMap::from([
            ("port".to_string(), "443".to_string()),
            ("log_path".to_string(), "src/reports".to_string()),
            ("download_path".to_string(), "src/downloads".to_string()),
            ("log_level".to_string(), "5".to_string()),
        ]);
        assert_eq!(cfg.unwrap(), right);
    }
}
