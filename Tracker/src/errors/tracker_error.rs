use crate::{errors::torrent_error::TorrentError, logger::LogMsg};
use std::{
    fmt::Display,
    io::Error,
    sync::{
        mpsc::{SendError, Sender},
        MutexGuard, PoisonError,
    },
};

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

impl From<TorrentError> for TrackerError {
    fn from(error: TorrentError) -> TrackerError {
        TrackerError {
            msg: format!("TrackerError: ({:?})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Sender<LogMsg>>>> for TrackerError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<LogMsg>>>) -> TrackerError {
        TrackerError {
            msg: format!("TrackerError: ({:?})", error),
        }
    }
}

impl From<SendError<LogMsg>> for TrackerError {
    fn from(error: SendError<LogMsg>) -> TrackerError {
        TrackerError {
            msg: format!("TrackerError: ({:?})", error),
        }
    }
}

impl From<serde_json::Error> for TrackerError {
    fn from(error: serde_json::Error) -> TrackerError {
        TrackerError {
            msg: format!("TrackerError: ({:?})", error),
        }
    }
}

impl Default for TrackerError {
    fn default() -> Self {
        Self::new("TrackerError: error during tracker initialization".to_string())
    }
}
