use crate::announce_utils::URLParams;
use crate::logger::LogMsg;
use std::{
    convert::Infallible,
    fmt::Display,
    io::Error,
    num::ParseIntError,
    sync::{
        mpsc::{SendError, Sender},
        MutexGuard, PoisonError,
    },
};

#[derive(Debug)]
pub struct TorrentError {
    msg: String,
}

impl TorrentError {
    pub fn new(message: String) -> TorrentError {
        TorrentError { msg: message }
    }
}

impl Display for TorrentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for TorrentError {
    fn from(error: Error) -> TorrentError {
        TorrentError {
            msg: format!("TorrentError: ({})", error),
        }
    }
}

impl From<Option<&URLParams>> for TorrentError {
    fn from(error: Option<&URLParams>) -> TorrentError {
        TorrentError {
            msg: format!("TorrentError: ({:?})", error),
        }
    }
}

impl From<SendError<LogMsg>> for TorrentError {
    fn from(error: SendError<LogMsg>) -> TorrentError {
        TorrentError {
            msg: format!("TorrentError: ({:?})", error),
        }
    }
}

impl From<Infallible> for TorrentError {
    fn from(error: Infallible) -> TorrentError {
        TorrentError {
            msg: format!("TorrentError: ({:?})", error),
        }
    }
}

impl From<ParseIntError> for TorrentError {
    fn from(error: ParseIntError) -> TorrentError {
        TorrentError {
            msg: format!("TorrentError: ({:?})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Sender<LogMsg>>>> for TorrentError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<LogMsg>>>) -> TorrentError {
        TorrentError {
            msg: format!("TorrentError: ({:?})", error),
        }
    }
}

impl Default for TorrentError {
    fn default() -> Self {
        Self::new("TorrentError: error during tracker initialization".to_string())
    }
}
