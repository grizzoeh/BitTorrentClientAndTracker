use std::{fmt::Display, io::Error, sync::mpsc::RecvError};

#[derive(Debug)]
pub struct LoggerError {
    msg: String,
}

impl LoggerError {
    pub fn new(message: String) -> LoggerError {
        LoggerError { msg: message }
    }
}

impl Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for LoggerError {
    fn from(error: Error) -> LoggerError {
        LoggerError {
            msg: format!("LoggerError: error during logging ({})", error),
        }
    }
}

impl From<RecvError> for LoggerError {
    fn from(error: RecvError) -> LoggerError {
        LoggerError {
            msg: format!("LoggerError: ({})", error),
        }
    }
}

impl Default for LoggerError {
    fn default() -> Self {
        Self::new("LoggerError: error during logging".to_string())
    }
}
