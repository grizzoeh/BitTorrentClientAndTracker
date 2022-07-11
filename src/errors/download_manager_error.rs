use super::client_error::ClientError;
use crate::{
    download_manager::{DownloaderInfo, PieceInfo, PieceStatus},
    errors::peer_connection_error::PeerConnectionError,
    logger::LogMsg,
    peer_entities::peer::Peer,
    peer_entities::peer_connection::PeerConnection,
    upload_manager::PieceRequest,
    utilities::utils::UiParams,
};
use glib::Sender as UISender;
use std::{
    any::Any,
    fmt::Display,
    io::Error,
    sync::mpsc::{Receiver, RecvError},
    sync::mpsc::{SendError, Sender},
    sync::Arc,
    sync::{MutexGuard, PoisonError, RwLockReadGuard, RwLockWriteGuard},
    thread::JoinHandle,
};

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

impl From<ClientError> for DownloadManagerError {
    fn from(error: ClientError) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, usize>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, usize>>) -> DownloadManagerError {
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

impl From<PoisonError<MutexGuard<'_, PieceStatus>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, PieceStatus>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Vec<JoinHandle<()>>>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, Vec<JoinHandle<()>>>>) -> DownloadManagerError {
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

impl From<PoisonError<RwLockReadGuard<'_, Vec<Arc<PeerConnection<Peer>>>>>>
    for DownloadManagerError
{
    fn from(
        error: PoisonError<RwLockReadGuard<Vec<Arc<PeerConnection<Peer>>>>>,
    ) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<SendError<LogMsg>> for DownloadManagerError {
    fn from(error: SendError<LogMsg>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: error logging ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Receiver<PieceInfo>>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, Receiver<PieceInfo>>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: error logging ({})", error),
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

impl From<RecvError> for DownloadManagerError {
    fn from(error: RecvError) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<PoisonError<RwLockReadGuard<'_, DownloaderInfo>>> for DownloadManagerError {
    fn from(error: PoisonError<RwLockReadGuard<'_, DownloaderInfo>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<PoisonError<RwLockReadGuard<'_, Peer>>> for DownloadManagerError {
    fn from(error: PoisonError<RwLockReadGuard<'_, Peer>>) -> DownloadManagerError {
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

impl From<PoisonError<MutexGuard<'_, Sender<LogMsg>>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<LogMsg>>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: logger error ({})", error),
        }
    }
}

impl From<PoisonError<RwLockReadGuard<'_, Vec<PieceInfo>>>> for DownloadManagerError {
    fn from(error: PoisonError<RwLockReadGuard<'_, Vec<PieceInfo>>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned error ({})", error),
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

impl From<PoisonError<MutexGuard<'_, Vec<PieceInfo>>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, Vec<PieceInfo>>>) -> DownloadManagerError {
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

impl From<PoisonError<RwLockWriteGuard<'_, Vec<PieceInfo>>>> for DownloadManagerError {
    fn from(error: PoisonError<RwLockWriteGuard<'_, Vec<PieceInfo>>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Vec<PeerConnection<Peer>>>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, Vec<PeerConnection<Peer>>>>) -> DownloadManagerError {
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
impl From<SendError<PieceInfo>> for DownloadManagerError {
    fn from(error: SendError<PieceInfo>) -> DownloadManagerError {
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
impl From<PoisonError<MutexGuard<'_, Sender<PieceInfo>>>> for DownloadManagerError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<PieceInfo>>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: error logging ({})", error),
        }
    }
}

impl From<Box<dyn Any + Send>> for DownloadManagerError {
    fn from(error: Box<dyn Any + Send>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: error logging ({:#?})", error),
        }
    }
}

impl From<SendError<Vec<(usize, UiParams)>>> for DownloadManagerError {
    fn from(error: SendError<Vec<(usize, UiParams)>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: error sending to ui ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, UISender<Vec<(usize, UiParams, String)>>>>>
    for DownloadManagerError
{
    fn from(
        error: PoisonError<MutexGuard<'_, UISender<Vec<(usize, UiParams, String)>>>>,
    ) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: error sending to ui ({})", error),
        }
    }
}
impl
    From<
        PoisonError<
            MutexGuard<'_, std::sync::mpsc::Sender<Vec<(usize, UiParams, std::string::String)>>>,
        >,
    > for DownloadManagerError
{
    fn from(
        error: PoisonError<
            MutexGuard<'_, std::sync::mpsc::Sender<Vec<(usize, UiParams, std::string::String)>>>,
        >,
    ) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: error sending to ui ({})", error),
        }
    }
}

impl From<SendError<Vec<(usize, UiParams, String)>>> for DownloadManagerError {
    fn from(error: SendError<Vec<(usize, UiParams, String)>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: error sending to ui ({})", error),
        }
    }
}
impl From<PoisonError<MutexGuard<'_, std::sync::mpsc::Sender<Option<PieceRequest>>>>>
    for DownloadManagerError
{
    fn from(
        error: PoisonError<MutexGuard<'_, std::sync::mpsc::Sender<Option<PieceRequest>>>>,
    ) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}
impl From<SendError<Option<PieceRequest>>> for DownloadManagerError {
    fn from(error: SendError<Option<PieceRequest>>) -> DownloadManagerError {
        DownloadManagerError {
            msg: format!("DownloadManagerError: poisoned thread ({})", error),
        }
    }
}

impl Default for DownloadManagerError {
    fn default() -> Self {
        Self::new("DownloadManagerError: error during tracker initialization".to_string())
    }
}
