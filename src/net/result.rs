use std::result;

use async_std::io;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

pub type Result<T> = result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error {
            message: format!("Connection error: {}", value),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error {
            message: format!("Parsing error: {}", value),
        }
    }
}
