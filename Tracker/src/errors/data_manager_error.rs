use crate::{logger::LogMsg, tracker::Tracker};
use std::{
    fmt::Display,
    io::Error,
    sync::{
        mpsc::{RecvError, SendError, Sender},
        MutexGuard, PoisonError,
    },
};

#[derive(Debug)]
pub struct DataManagerError {
    msg: String,
}

impl DataManagerError {
    pub fn new(message: String) -> DataManagerError {
        DataManagerError { msg: message }
    }
}

impl Display for DataManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for DataManagerError {
    fn from(error: Error) -> DataManagerError {
        DataManagerError {
            msg: format!("DataManagerError: error during logging ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Tracker>>> for DataManagerError {
    fn from(error: PoisonError<MutexGuard<'_, Tracker>>) -> DataManagerError {
        DataManagerError {
            msg: format!("DataManagerError: error during logging ({})", error),
        }
    }
}

impl From<PoisonError<MutexGuard<'_, Sender<LogMsg>>>> for DataManagerError {
    fn from(error: PoisonError<MutexGuard<'_, Sender<LogMsg>>>) -> DataManagerError {
        DataManagerError {
            msg: format!("DataManagerError: error during logging ({})", error),
        }
    }
}

impl From<RecvError> for DataManagerError {
    fn from(error: RecvError) -> DataManagerError {
        DataManagerError {
            msg: format!("DataManagerError: ({})", error),
        }
    }
}

impl From<SendError<LogMsg>> for DataManagerError {
    fn from(error: SendError<LogMsg>) -> DataManagerError {
        DataManagerError {
            msg: format!("DataManagerError: ({})", error),
        }
    }
}

impl From<serde_json::Error> for DataManagerError {
    fn from(error: serde_json::Error) -> DataManagerError {
        DataManagerError {
            msg: format!("DataManagerError: ({})", error),
        }
    }
}

impl Default for DataManagerError {
    fn default() -> Self {
        Self::new("DataManagerError: error during logging".to_string())
    }
}
