use super::communication_method_error::CommunicationMethodError;
use crate::{
    logger::LogMsg, peer_entities::communication_method::CommunicationMethod,
    upload_manager::PieceRequest,
};
use std::{
    fmt::Display,
    io::Error,
    string::FromUtf8Error,
    sync::mpsc::SendError,
    sync::TryLockError,
    sync::{MutexGuard, PoisonError, RwLockWriteGuard},
};

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

impl From<TryLockError<MutexGuard<'_, Box<(dyn CommunicationMethod + Send + 'static)>>>>
    for PeerConnectionError
{
    fn from(
        error: TryLockError<MutexGuard<'_, Box<(dyn CommunicationMethod + Send + 'static)>>>,
    ) -> PeerConnectionError {
        PeerConnectionError {
            msg: format!("PeerConnectionError: poisoned thread ({})", error),
        }
    }
}

impl<P> From<PoisonError<std::sync::RwLockReadGuard<'_, P>>> for PeerConnectionError {
    fn from(error: PoisonError<std::sync::RwLockReadGuard<'_, P>>) -> PeerConnectionError {
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

impl From<SendError<LogMsg>> for PeerConnectionError {
    fn from(error: SendError<LogMsg>) -> PeerConnectionError {
        PeerConnectionError {
            msg: format!("PeerConnectionError: poisoned thread ({})", error),
        }
    }
}

impl From<SendError<Option<PieceRequest>>> for PeerConnectionError {
    fn from(error: SendError<Option<PieceRequest>>) -> PeerConnectionError {
        PeerConnectionError {
            msg: format!("PeerConnectionError: poisoned thread ({})", error),
        }
    }
}

impl<P> From<PoisonError<RwLockWriteGuard<'_, P>>> for PeerConnectionError {
    fn from(error: PoisonError<RwLockWriteGuard<'_, P>>) -> PeerConnectionError {
        PeerConnectionError {
            msg: format!("PeerConnectionError: poisoned thread ({})", error),
        }
    }
}

impl From<FromUtf8Error> for PeerConnectionError {
    fn from(error: FromUtf8Error) -> PeerConnectionError {
        PeerConnectionError {
            msg: format!("PeerConnectionError: error reading peer id ({})", error),
        }
    }
}

impl From<CommunicationMethodError> for PeerConnectionError {
    fn from(error: CommunicationMethodError) -> PeerConnectionError {
        PeerConnectionError {
            msg: format!(
                "PeerConnectionError: error communicating with peer ({})",
                error
            ),
        }
    }
}

impl Default for PeerConnectionError {
    fn default() -> Self {
        Self::new("PeerConnectionError: error connecting with peer".to_string())
    }
}
