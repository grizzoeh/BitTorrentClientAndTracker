use crate::{
    errors::{announce_error::AnnounceError, tracker_error::TrackerError},
    logger::LogMsg,
    tracker::Tracker,
};
use std::{
    fmt::Display,
    io::Error,
    string::FromUtf8Error,
    sync::{
        mpsc::{SendError, Sender},
        MutexGuard, PoisonError,
    },
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

impl From<AnnounceError> for ListenerError {
    fn from(error: AnnounceError) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Tracker>>> for ListenerError {
    fn from(error: PoisonError<MutexGuard<'_, Tracker>>) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Sender<LogMsg>>>> for ListenerError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<LogMsg>>>) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}

impl From<SendError<LogMsg>> for ListenerError {
    fn from(error: SendError<LogMsg>) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}

impl From<TrackerError> for ListenerError {
    fn from(error: TrackerError) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}

impl From<FromUtf8Error> for ListenerError {
    fn from(error: FromUtf8Error) -> ListenerError {
        ListenerError {
            msg: format!("ListenerError: ({})", error),
        }
    }
}

impl Default for ListenerError {
    fn default() -> Self {
        Self::new("ListenerError: error during tracker initialization".to_string())
    }
}
