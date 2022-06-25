use crate::errors::download_manager_error::DownloadManagerError;
use crate::errors::logger_error::LoggerError;
use crate::errors::peer_connection_error::PeerConnectionError;
use crate::errors::torrent_parser_error::TorrentParserError;
use crate::errors::tracker_error::TrackerError;
use crate::peer::Peer;
use std::fmt::Display;
use std::io::Error;
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use std::sync::mpsc::Sender;
use std::sync::{MutexGuard, PoisonError, RwLockWriteGuard};

#[derive(Debug)]
pub struct ClientError {
    msg: String,
}

impl ClientError {
    pub fn new(message: String) -> ClientError {
        ClientError { msg: message }
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for ClientError {
    fn from(error: Error) -> ClientError {
        ClientError {
            msg: format!("ClientError: ({})", error),
        }
    }
}

impl From<ParseIntError> for ClientError {
    fn from(error: ParseIntError) -> ClientError {
        ClientError {
            msg: format!("ClientError: error parsing port number ({})", error),
        }
    }
}

impl From<TorrentParserError> for ClientError {
    fn from(error: TorrentParserError) -> ClientError {
        ClientError {
            msg: format!("ClientError: error parsing torrent file ({})", error),
        }
    }
}

impl From<FromUtf8Error> for ClientError {
    fn from(error: FromUtf8Error) -> ClientError {
        ClientError {
            msg: format!("ClientError: error parsing tracker url ({})", error),
        }
    }
}

impl From<TrackerError> for ClientError {
    fn from(error: TrackerError) -> ClientError {
        ClientError {
            msg: format!("ClientError: error initializing tracker ({})", error),
        }
    }
}

impl From<PeerConnectionError> for ClientError {
    fn from(error: PeerConnectionError) -> ClientError {
        ClientError {
            msg: format!("ClientError: error initializing peer ({})", error),
        }
    }
}

impl From<DownloadManagerError> for ClientError {
    fn from(error: DownloadManagerError) -> ClientError {
        ClientError {
            msg: format!("ClientError: error in downloadManager ({})", error),
        }
    }
}

impl From<PoisonError<RwLockWriteGuard<'_, Vec<Peer>>>> for ClientError {
    fn from(error: PoisonError<RwLockWriteGuard<'_, Vec<Peer>>>) -> ClientError {
        ClientError {
            msg: format!("ClientError: poisoned thread ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, String>>> for ClientError {
    fn from(error: PoisonError<MutexGuard<'_, String>>) -> ClientError {
        ClientError {
            msg: format!("ClientError: poisoned thread ({})", error),
        }
    }
}

impl From<LoggerError> for ClientError {
    fn from(error: LoggerError) -> ClientError {
        ClientError {
            msg: format!("ClientError: logger error ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Sender<String>>>> for ClientError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<String>>>) -> ClientError {
        ClientError {
            msg: format!("ClientError: logger error ({})", error),
        }
    }
}

impl Default for ClientError {
    fn default() -> Self {
        Self::new("ClientError: error during client initialization".to_string())
    }
}
