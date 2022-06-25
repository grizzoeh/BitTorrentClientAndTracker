use crate::download_manager::PieceStatus;
use crate::errors::peer_connection_error::PeerConnectionError;
use crate::peer::Peer;
use crate::peer_connection::PeerConnection;
use std::fmt::Display;
use std::io::Error;
use std::net::TcpStream;
use std::sync::mpsc::{SendError, Sender};
use std::sync::{MutexGuard, PoisonError, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct DownloadManagerError {
    msg: String,
}

impl DownloadManagerError {
    pub fn new(message: String) -> DownloadManagerError {
        DownloadManagerError { msg: message }
    }
}

impl Display for DownloadManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for DownloadManagerError {
    fn from(error: Error) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: ({})", error),
        }
    }
}

impl From<PeerConnectionError> for DownloadManagerError {
    fn from(error: PeerConnectionError) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, u64>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, u64>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<PoisonError<RwLockReadGuard<'_, Vec<PieceStatus>>>> for DownloadManagerError {
    fn from(error: PoisonError<RwLockReadGuard<'_, Vec<PieceStatus>>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<PoisonError<RwLockReadGuard<'_, Vec<Peer>>>> for DownloadManagerError {
    fn from(error: PoisonError<RwLockReadGuard<'_, Vec<Peer>>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<PoisonError<RwLockWriteGuard<'_, Vec<PieceStatus>>>> for DownloadManagerError {
    fn from(error: PoisonError<RwLockWriteGuard<'_, Vec<PieceStatus>>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, String>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, String>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Vec<u8>>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, Vec<u8>>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Vec<PeerConnection<TcpStream>>>>> for DownloadManagerError {
    fn from(
        error: PoisonError<MutexGuard<'_, Vec<PeerConnection<TcpStream>>>>,
    ) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<SendError<String>> for DownloadManagerError {
    fn from(error: SendError<String>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: error logging ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Sender<String>>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<String>>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: error logging ({})", error),
        }
    }
}

impl Default for DownloadManagerError {
    fn default() -> Self {
        Self::new("DownloadManagerError: error during tracker initialization".to_string())
    }
}
