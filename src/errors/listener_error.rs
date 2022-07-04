use std::fmt::Display;
use std::io::Error;
use std::net::TcpStream;
use std::string::FromUtf8Error;
use std::sync::mpsc::{Receiver, SendError, Sender};
use std::sync::{MutexGuard, PoisonError, RwLockReadGuard};

use crate::communication_method::CommunicationMethod;
use crate::download_manager::PieceStatus;
use crate::logger::LogMsg;
use crate::peer::IncomingPeer;

use super::communication_method_error::CommunicationMethodError;
use super::peer_connection_error::PeerConnectionError;

#[derive(Debug)]
pub struct ListenerError {
    msg: String,
}

impl ListenerError {
    pub fn new(message: String) -> ListenerError {
        ListenerError { msg: message }
    }
}

impl Display for ListenerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for ListenerError {
    fn from(error: Error) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}
impl From<PoisonError<MutexGuard<'_, Sender<LogMsg>>>> for ListenerError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<LogMsg>>>) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: poisoned thread ({})", error),
        }
    }
}
impl From<PoisonError<MutexGuard<'_, Receiver<String>>>> for ListenerError {
    fn from(error: PoisonError<MutexGuard<'_, Receiver<String>>>) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: poisoned thread ({})", error),
        }
    }
}
impl From<SendError<String>> for ListenerError {
    fn from(error: SendError<String>) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: error logging ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Box<(dyn CommunicationMethod + Send + 'static)>>>>
    for ListenerError
{
    fn from(
        error: PoisonError<MutexGuard<'_, Box<(dyn CommunicationMethod + Send + 'static)>>>,
    ) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: poison error ({})", error),
        }
    }
}

impl From<SendError<LogMsg>> for ListenerError {
    fn from(error: SendError<LogMsg>) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: send error ({})", error),
        }
    }
}

impl From<FromUtf8Error> for ListenerError {
    fn from(error: FromUtf8Error) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: error reading peer id ({})", error),
        }
    }
}

impl From<PoisonError<RwLockReadGuard<'_, IncomingPeer>>> for ListenerError {
    fn from(error: PoisonError<RwLockReadGuard<'_, IncomingPeer>>) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: error reading peer id ({})", error),
        }
    }
}

impl From<PeerConnectionError> for ListenerError {
    fn from(error: PeerConnectionError) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: error on peer connection ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, TcpStream>>> for ListenerError {
    fn from(error: PoisonError<MutexGuard<'_, TcpStream>>) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}
impl From<PoisonError<RwLockReadGuard<'_, Vec<PieceStatus>>>> for ListenerError {
    fn from(error: PoisonError<RwLockReadGuard<'_, Vec<PieceStatus>>>) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}

impl
    From<
        PoisonError<
            MutexGuard<'_, PoisonError<MutexGuard<'_, Box<(dyn CommunicationMethod + Send)>>>>,
        >,
    > for ListenerError
{
    fn from(
        error: PoisonError<
            MutexGuard<'_, PoisonError<MutexGuard<'_, Box<(dyn CommunicationMethod + Send)>>>>,
        >,
    ) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: poisoned thread ({})", error),
        }
    }
}

impl From<CommunicationMethodError> for ListenerError {
    fn from(error: CommunicationMethodError) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}
