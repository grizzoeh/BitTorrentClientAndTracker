use std::fmt::Display;
use std::io::Error;
use std::sync::mpsc::SendError;
use std::sync::{MutexGuard, PoisonError};

#[derive(Debug)]
pub struct PeerConnectionError {
    msg: String,
}

impl PeerConnectionError {
    pub fn new(message: String) -> PeerConnectionError {
        PeerConnectionError { msg: message }
    }
}

impl Display for PeerConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for PeerConnectionError {
    fn from(error: Error) -> PeerConnectionError {
        PeerConnectionError {
            msg: format!("PeerConnectionError: ({})", error),
        }
    }
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for PeerConnectionError {
    fn from(error: PoisonError<MutexGuard<'_, T>>) -> PeerConnectionError {
        PeerConnectionError {
            msg: format!("PeerConnectionError: poisoned thread ({})", error),
        }
    }
}

impl From<SendError<String>> for PeerConnectionError {
    fn from(error: SendError<String>) -> PeerConnectionError {
        PeerConnectionError {
            msg: format!("PeerConnectionError: poisoned thread ({})", error),
        }
    }
}

impl Default for PeerConnectionError {
    fn default() -> Self {
        Self::new("PeerConnectionError: error connecting with peer".to_string())
    }
}
