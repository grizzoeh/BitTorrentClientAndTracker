use crate::errors::bdecoder_error::BDecoderError;
use native_tls::HandshakeError;
use std::fmt::Display;
use std::io::Error;
use std::net::TcpStream;

#[derive(Debug)]
pub struct TrackerError {
    msg: String,
}

impl TrackerError {
    pub fn new(message: String) -> TrackerError {
        TrackerError { msg: message }
    }
}

impl Display for TrackerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for TrackerError {
    fn from(error: Error) -> TrackerError {
        TrackerError {
            msg: format!("TrackerError: ({})", error),
        }
    }
}

impl From<BDecoderError> for TrackerError {
    fn from(error: BDecoderError) -> TrackerError {
        TrackerError {
            msg: format!("TrackerError: ({})", error),
        }
    }
}

impl From<native_tls::Error> for TrackerError {
    fn from(error: native_tls::Error) -> TrackerError {
        TrackerError {
            msg: format!("TrackerError: ({})", error),
        }
    }
}

impl From<HandshakeError<TcpStream>> for TrackerError {
    fn from(error: HandshakeError<TcpStream>) -> TrackerError {
        TrackerError {
            msg: format!("TrackerError: ({})", error),
        }
    }
}

impl Default for TrackerError {
    fn default() -> Self {
        Self::new("TrackerError: error during tracker initialization".to_string())
    }
}
