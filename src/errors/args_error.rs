use std::{fmt::Display, io::Error};

#[derive(Debug)]
pub struct ArgsError {
    msg: String,
}

impl ArgsError {
    pub fn new() -> ArgsError {
        ArgsError {
            msg: "ArgsError: Invalid argument".to_string(),
        }
    }
}

impl Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Error> for ArgsError {
    fn from(error: Error) -> ArgsError {
        ArgsError {
            msg: format!("ArgsError: Invalid argument ({})", error),
        }
    }
}

impl Default for ArgsError {
    fn default() -> Self {
        Self::new()
    }
}
