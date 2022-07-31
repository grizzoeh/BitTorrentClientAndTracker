use std::{fmt::Display, io::Error, num::ParseIntError};

#[derive(Debug)]
pub struct AnnounceError {
    msg: String,
}

impl AnnounceError {
    pub fn new(message: String) -> AnnounceError {
        AnnounceError { msg: message }
    }
}

impl Display for AnnounceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for AnnounceError {
    fn from(error: Error) -> AnnounceError {
        AnnounceError {
            msg: format!("AnnounceError: ({})", error),
        }
    }
}

impl From<ParseIntError> for AnnounceError {
    fn from(error: ParseIntError) -> AnnounceError {
        AnnounceError {
            msg: format!("AnnounceError: ({})", error),
        }
    }
}

impl Default for AnnounceError {
    fn default() -> Self {
        Self::new("AnnounceError: error during tracker initialization".to_string())
    }
}
