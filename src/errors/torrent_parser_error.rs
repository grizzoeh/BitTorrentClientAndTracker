use crate::errors::bdecoder_error::BDecoderError;
use std::fmt::Display;
use std::io::Error;

#[derive(Debug)]
pub struct TorrentParserError {
    msg: String,
}

impl TorrentParserError {
    pub fn new(message: String) -> TorrentParserError {
        TorrentParserError { msg: message }
    }
}

impl Display for TorrentParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for TorrentParserError {
    fn from(error: Error) -> TorrentParserError {
        TorrentParserError {
            msg: format!("TorrentParserError: ({})", error),
        }
    }
}

impl From<BDecoderError> for TorrentParserError {
    fn from(error: BDecoderError) -> TorrentParserError {
        TorrentParserError {
            msg: format!("TorrentParserError: ({})", error),
        }
    }
}

impl Default for TorrentParserError {
    fn default() -> Self {
        Self::new("TorrentParserError: error on client initialization".to_string())
    }
}
