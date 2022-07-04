use std::{fmt::Display, io::Error};

#[derive(Debug, Default)]
pub struct CommunicationMethodError {
    msg: String,
}

impl CommunicationMethodError {
    pub fn new(message: String) -> CommunicationMethodError {
        CommunicationMethodError { msg: message }
    }
}

impl Display for CommunicationMethodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for CommunicationMethodError {
    fn from(error: Error) -> CommunicationMethodError {
        CommunicationMethodError {
            msg: format!("CommunicationMethodError: ({})", error),
        }
    }
}
