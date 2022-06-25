use crate::errors::args_error::ArgsError;
use crate::errors::client_error::ClientError;
use crate::errors::config_parser_error::ConfigParserError;
use std::fmt::Display;
use std::io::Error;

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
            msg: format!("AppError: error during app initialization ({})", error),
        }
    }
}

impl From<ArgsError> for AppError {
    fn from(error: ArgsError) -> AppError {
        AppError {
            msg: format!("AppError: error during app initialization ({})", error),
        }
    }
}

impl From<ConfigParserError> for AppError {
    fn from(error: ConfigParserError) -> AppError {
        AppError {
            msg: format!("AppError: error with config file ({})", error),
        }
    }
}

impl From<ClientError> for AppError {
    // FIXME: Cuando se implemente el Client error se deberá mostrar ese msg.
    fn from(_: ClientError) -> AppError {
        AppError {
            msg: "AppError: error during client initialization".to_string(),
        }
    }
}

impl Default for AppError {
    fn default() -> Self {
        Self::new("AppError: error during app initialization".to_string())
    }
}
