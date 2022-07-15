use crate::{
    download_manager::PieceStatus, errors::communication_method_error::CommunicationMethodError,
    errors::peer_connection_error::PeerConnectionError, logger::LogMsg, peer_entities::peer::Peer,
    utilities::utils::UiParams,
};
use std::{
    fmt::Display,
    io::Error,
    sync::mpsc::{RecvError, SendError},
    sync::{MutexGuard, PoisonError, RwLockReadGuard, RwLockWriteGuard},
};

use super::client_error::ClientError;

#[derive(Debug)]
pub struct UploadManagerError {
    msg: String,
}

impl UploadManagerError {
    pub fn new(message: String) -> UploadManagerError {
        UploadManagerError { msg: message }
    }
}

impl Display for UploadManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for UploadManagerError {
    fn from(error: Error) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: ({})", error),
        }
    }
}

impl From<PeerConnectionError> for UploadManagerError {
    fn from(error: PeerConnectionError) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: ({})", error),
        }
    }
}
impl From<ClientError> for UploadManagerError {
    fn from(error: ClientError) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: ({})", error),
        }
    }
}

impl From<PoisonError<RwLockReadGuard<'_, Vec<PieceStatus>>>> for UploadManagerError {
    fn from(error: PoisonError<RwLockReadGuard<'_, Vec<PieceStatus>>>) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<SendError<Vec<(usize, UiParams, String)>>> for UploadManagerError {
    fn from(error: SendError<Vec<(usize, UiParams, String)>>) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: send error ({})", error),
        }
    }
}

impl From<PoisonError<RwLockReadGuard<'_, Vec<Peer>>>> for UploadManagerError {
    fn from(error: PoisonError<RwLockReadGuard<'_, Vec<Peer>>>) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<CommunicationMethodError> for UploadManagerError {
    fn from(error: CommunicationMethodError) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: communication method error ({})", error),
        }
    }
}

impl From<PoisonError<RwLockWriteGuard<'_, Vec<PieceStatus>>>> for UploadManagerError {
    fn from(error: PoisonError<RwLockWriteGuard<'_, Vec<PieceStatus>>>) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: poisoned thread ({})", error),
        }
    }
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for UploadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, T>>) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<RecvError> for UploadManagerError {
    fn from(error: RecvError) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: ({})", error),
        }
    }
}

impl From<SendError<String>> for UploadManagerError {
    fn from(error: SendError<String>) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: error logging ({})", error),
        }
    }
}
impl From<SendError<LogMsg>> for UploadManagerError {
    fn from(error: SendError<LogMsg>) -> UploadManagerError {
        UploadManagerError {
            msg: format!("UploadManagerError: error logging ({})", error),
        }
    }
}
impl Default for UploadManagerError {
    fn default() -> Self {
        Self::new("UploadManagerError: default error".to_string())
    }
}
