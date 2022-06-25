use std::fmt::Display;
use std::io::Error;

#[derive(Debug)]
pub struct ConfigParserError {
    msg: String,
}

impl ConfigParserError {
    pub fn new() -> ConfigParserError {
        ConfigParserError {
            msg: "ConfigParserError: Expected field not found".to_string(),
        }
    }
}

impl Display for ConfigParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for ConfigParserError {
    fn from(error: Error) -> ConfigParserError {
        ConfigParserError {
            msg: format!("ConfigParserError: Invalid argument ({})", error),
        }
    }
}

impl Default for ConfigParserError {
    fn default() -> Self {
        Self::new()
    }
}
