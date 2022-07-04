use super::listener_error::ListenerError;
use crate::errors::download_manager_error::DownloadManagerError;
use crate::errors::logger_error::LoggerError;
use crate::errors::peer_connection_error::PeerConnectionError;
use crate::errors::torrent_parser_error::TorrentParserError;
use crate::errors::tracker_error::TrackerError;
use crate::logger::LogMsg;
use crate::peer::Peer;
use crate::upload_manager::PieceRequest;
use crate::utils::UiParams;
use glib::Sender as UISender;
use std::fmt::Display;
use std::io::Error;
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use std::sync::mpsc::{SendError, Sender};
use std::sync::{MutexGuard, PoisonError, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug, Default)]
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
impl From<ListenerError> for ClientError {
    fn from(error: ListenerError) -> ClientError {
        ClientError {
            msg: format!("ClientError: error listening on port ({})", error),
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

impl From<PoisonError<MutexGuard<'_, UISender<Vec<(usize, UiParams, String)>>>>> for ClientError {
    fn from(
        error: PoisonError<MutexGuard<'_, UISender<Vec<(usize, UiParams, String)>>>>,
    ) -> ClientError {
        ClientError {
            msg: format!("ClientError: poisoned error ({})", error),
        }
    }
}

impl From<SendError<Vec<(usize, UiParams, String)>>> for ClientError {
    fn from(error: SendError<Vec<(usize, UiParams, String)>>) -> ClientError {
        ClientError {
            msg: format!("ClientError: send error ({})", error),
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
impl From<PoisonError<MutexGuard<'_, Vec<u8>>>> for ClientError {
    fn from(error: PoisonError<MutexGuard<'_, Vec<u8>>>) -> ClientError {
        ClientError {
            msg: format!("ClientError: poisoned thread ({})", error),
        }
    }
}
impl From<PoisonError<RwLockReadGuard<'_, u64>>> for ClientError {
    fn from(error: PoisonError<RwLockReadGuard<'_, u64>>) -> ClientError {
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

impl From<PoisonError<MutexGuard<'_, Sender<LogMsg>>>> for ClientError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<LogMsg>>>) -> ClientError {
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

impl From<PoisonError<MutexGuard<'_, Sender<Option<PieceRequest>>>>> for ClientError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<Option<PieceRequest>>>>) -> ClientError {
        ClientError {
            msg: format!("ClientError: ({})", error),
        }
    }
}
impl From<SendError<Vec<(usize, UiParams)>>> for ClientError {
    fn from(error: SendError<Vec<(usize, UiParams)>>) -> ClientError {
        ClientError {
            msg: format!("ClientError: error sending to ui ({})", error),
        }
    }
}

impl From<SendError<String>> for ClientError {
    fn from(error: SendError<String>) -> ClientError {
        ClientError {
            msg: format!("ClientError: ({})", error),
        }
    }
}

impl From<SendError<LogMsg>> for ClientError {
    fn from(error: SendError<LogMsg>) -> ClientError {
        ClientError {
            msg: format!("ClientError: ({})", error),
        }
    }
}

impl From<SendError<Option<PieceRequest>>> for ClientError {
    fn from(error: SendError<Option<PieceRequest>>) -> ClientError {
        ClientError {
            msg: format!("ClientError: ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Sender<Vec<(usize, UiParams)>>>>> for ClientError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<Vec<(usize, UiParams)>>>>) -> ClientError {
        ClientError {
            msg: format!("ClientError: error creating ui channel ({})", error),
        }
    }
}
