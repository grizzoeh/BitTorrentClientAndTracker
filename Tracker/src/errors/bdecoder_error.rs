use std::{fmt::Display, io::Error};

#[derive(Debug)]
pub struct BDecoderError {
    msg: String,
}

impl BDecoderError {
    pub fn new(message: String) -> BDecoderError {
        BDecoderError { msg: message }
    }
}

impl Display for BDecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for BDecoderError {
    fn from(error: Error) -> BDecoderError {
        BDecoderError {
            msg: format!("BDecoderError: unexpected character ({})", error),
        }
    }
}

impl Default for BDecoderError {
    fn default() -> Self {
        Self::new("BDecoderError: unexpected character".to_string())
    }
}
