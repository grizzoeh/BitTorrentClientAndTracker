use super::{
    communication_method_error::CommunicationMethodError,
    peer_connection_error::PeerConnectionError,
};
use crate::{
    download_manager::PieceStatus, logger::LogMsg,
    peer_entities::communication_method::CommunicationMethod, peer_entities::peer::IncomingPeer,
    utilities::utils::UiParams,
};
use glib::Sender as UISender;
use std::{
    fmt::Display,
    io::Error,
    net::TcpStream,
    string::FromUtf8Error,
    sync::mpsc::{Receiver, SendError, Sender},
    sync::{MutexGuard, PoisonError, RwLockReadGuard},
};

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

impl From<PoisonError<MutexGuard<'_, UISender<Vec<(usize, UiParams, String)>>>>> for ListenerError {
    fn from(
        error: PoisonError<MutexGuard<'_, UISender<Vec<(usize, UiParams, String)>>>>,
    ) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}

impl From<SendError<Vec<(usize, UiParams, String)>>> for ListenerError {
    fn from(error: SendError<Vec<(usize, UiParams, String)>>) -> ListenerError {
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
