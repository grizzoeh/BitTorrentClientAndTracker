use crate::{
    errors::{
        data_manager_error::DataManagerError, listener_error::ListenerError,
        logger_error::LoggerError,
    },
    logger::LogMsg,
};
use std::{fmt::Display, io::Error, sync::mpsc::SendError};

#[derive(Debug)]
pub struct AppError {
    msg: String,
}

impl AppError {
    pub fn new(message: String) -> AppError {
        AppError { msg: message }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for AppError {
    fn from(error: Error) -> AppError {
        AppError {
            msg: format!("AppError: ({})", error),
        }
    }
}

impl From<DataManagerError> for AppError {
    fn from(error: DataManagerError) -> AppError {
        AppError {
            msg: format!("AppError: ({})", error),
        }
    }
}

impl From<LoggerError> for AppError {
    fn from(error: LoggerError) -> AppError {
        AppError {
            msg: format!("AppError: ({})", error),
        }
    }
}

impl From<ListenerError> for AppError {
    fn from(error: ListenerError) -> AppError {
        AppError {
            msg: format!("AppError: ({})", error),
        }
    }
}

impl From<SendError<LogMsg>> for AppError {
    fn from(error: SendError<LogMsg>) -> AppError {
        AppError {
            msg: format!("AppError: ({})", error),
        }
    }
}

impl Default for AppError {
    fn default() -> Self {
        Self::new("AppError: error during tracker initialization".to_string())
    }
}
